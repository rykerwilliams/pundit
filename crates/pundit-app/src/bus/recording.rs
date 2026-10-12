//! Recording (Phase 4 spec R6): the one Active state, from Start to a clip.
//!
//! Active is flagged *starting* until the recorder's first buffer reaches the
//! muxer — the camera's in a camera project, the microphone's in an avatar
//! one. Stopping before then (StopRecording, a recorder error, or the start
//! timeout) aborts: no clip, and the file is deleted. Stopping after it always
//! keeps the clip, even if finalizing didn't go cleanly.
//!
//! The recorder's messages arrive as their own input, tagged here with the
//! generation of the recording that produced them, and never reach the
//! player: its EOS would advance the source and its ERROR reset the player.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use gstreamer::glib;
use pundit_core::project::Preferences;
use pundit_core::recording::{PendingClip, RecordingLog};
use pundit_core::store::{BACKDROPS_DIRNAME, RECORDINGS_DIRNAME};
use pundit_core::stroke::Stroke;
use pundit_core::zoom::Zoom;
use pundit_media::{
    decode_still, list_devices, resolve_camera, resolve_mic, CaptureSources, Origin, Recorder,
    RecorderMessage,
};
use uuid::Uuid;

use super::{Armed, Bus, Event, Input, Take, UserError};

/// How long a recording may wait for its first buffer.
const START_TIMEOUT: Duration = Duration::from_secs(5);
/// How long a stop waits for the file to finalize.
const STOP_TIMEOUT: Duration = Duration::from_secs(5);

/// Where recordings come from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CaptureKind {
    /// The project's preferred camera and microphone, or the defaults.
    Devices,
    /// Live test sources, with video starting `video_delay` in, as a camera
    /// warming up. For tests: no camera, microphone or display.
    Test { video_delay: Duration },
}

/// What the recording is doing, as the UI shows it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecordingStatus {
    Idle,
    /// Recording, but nothing has reached the muxer yet: stopping now
    /// aborts.
    ///
    /// **`whiteboard` is whether the take is over the armed diagram** (BACKLOG
    /// #138), and it is the window's only way to know: a W pressed during an R
    /// take's start-up *stops* that take, so the window cannot infer it from
    /// the key it sent.
    Starting {
        whiteboard: bool,
    },
    /// `t0_ns` is the recording's time 0 on `now_ns()`'s clock, for the
    /// elapsed-time readout.
    Recording {
        t0_ns: u64,
    },
}

/// The recording in progress.
pub(super) struct Active {
    pub(super) pending: PendingClip,
    /// The slate this take was shot from, if it was shot from one. The clip
    /// it produces inherits that slate's name and tags, and carries its id.
    pub(super) slate: Option<Uuid>,
    /// The diagram this take is recorded over (BACKLOG #138), as the clip will
    /// name it — copied in already, under `backdrops/`. **`Some` is a
    /// whiteboard take**, which is what narrows the recording allow-list.
    pub(super) backdrop: Option<String>,
    recorder: Recorder,
    pub(super) log: RecordingLog,
    /// The file, deleted on an abort.
    path: PathBuf,
    /// When it started, for [`START_TIMEOUT`].
    started: Instant,
    /// The first buffer reached the muxer: stopping keeps the clip.
    media_seen: bool,
}

impl Bus {
    /// R and W: starts `take` while idle, else stops (or aborts) the take
    /// running, whichever it is. The UI's status can lag the bus's, so the bus
    /// decides: a second R during start-up cancels, as the user means.
    ///
    /// **`take` carries the selected range, and the same lag is why it arrives
    /// here rather than being branched on in the window** (BACKLOG #120). With
    /// one, R is that range's take, so the themed pass is two keys and the clip
    /// carries the `slate_id` that marks the range shot. A window that chose
    /// between two commands from its own phase would send a `ShootSlate` into
    /// a take the recording guard refuses it in, and the R meant as a cancel
    /// would do nothing.
    pub(super) fn toggle_recording(&mut self, zoom: Zoom, take: Take) {
        match self.recording.is_some() {
            true => self.stop_recording(),
            false => self.start_recording(zoom, take),
        }
    }

    /// Starts `take` from where the player is heading (R6), or — shooting a
    /// slate — from the range's video and in point. Refused with an error
    /// unless a project with sources, none missing, is open and its current
    /// source is loaded or loading.
    ///
    /// **A slate's seek happens here, not in the caller**, and after
    /// `capture_sources`. `can_record` does not see every refusal:
    /// `UserError::NoCamera` comes from `resolve_camera` inside
    /// `capture_sources`, so a caller that seeked first would move the game
    /// video to the in point and *then* refuse on a machine with no camera.
    /// Every refusal but one leaves the player exactly as it was — the
    /// recorder failing to start leaves it at the in point, which is harmless
    /// and where the coach was going anyway.
    ///
    /// **A whiteboard's diagram is copied with the refusals, above the pause**
    /// (#138), so a failed copy leaves the player as it was; the clip's id is
    /// minted there because the copy is named by it. Copied in
    /// `finish_recording` instead, a failure would land after the coach had
    /// spoken, as a clip silently recorded over footage it never showed.
    pub(super) fn start_recording(&mut self, zoom: Zoom, take: Take) {
        if let Err(e) = self.can_record() {
            return self.emit(Event::Error(e));
        }
        let Some(open) = &self.open else {
            return;
        };
        // What the take starts on: a range's video and in point, or the
        // armed diagram.
        let (shot, armed) = match take {
            Take::Footage => (None, None),
            Take::Slate(id) => match open.project.slates.iter().find(|s| s.id == id) {
                Some(slate) => (Some((id, slate.source_index, slate.in_seconds)), None),
                None => return eprintln!("bus: a take of slate {id}, which isn't there"),
            },
            // A notice: it refuses one key, and the answer is a pick in a
            // popover, not a dialog.
            Take::Whiteboard => match &self.backdrop {
                Some(armed) => (None, Some(armed.path.clone())),
                None => {
                    return self.emit(Event::Error(UserError::Whiteboard(
                        "no diagram is armed; pick one first".into(),
                    )))
                }
            },
        };
        let folder = open.folder.join(RECORDINGS_DIRNAME);
        // Before anything changes: a refusal leaves the player as it was.
        let sources = match self.capture_sources(&open.project.preferences) {
            Ok(sources) => sources,
            Err(e) => return self.emit(Event::Error(e)),
        };
        if let Err(e) = std::fs::create_dir_all(&folder) {
            return self.emit(Event::Error(UserError::Io(format!(
                "{}: {e}",
                folder.display()
            ))));
        }
        let id = Uuid::new_v4();
        let backdrops = open.folder.join(BACKDROPS_DIRNAME);
        let backdrop = match armed {
            None => None,
            Some(path) => match copy_into(&path, &backdrops, &id.to_string()) {
                Ok(name) => Some(name),
                Err(e) => {
                    let why = match e.kind() {
                        std::io::ErrorKind::NotFound => {
                            "the armed diagram is gone; pick it again".to_owned()
                        }
                        _ => format!("the diagram couldn't be copied into the project: {e}"),
                    };
                    return self.emit(Event::Error(UserError::Whiteboard(why)));
                }
            },
        };

        // Every clip starts on a still frame.
        if self.playing {
            self.set_playing(false);
        }
        // Now that nothing can refuse: go to the slate's in point. The skip
        // coordinator is reset first because `heading` prefers its pending
        // target over the player's, so a skip burst still in the air would
        // otherwise stamp this clip where the arrows were heading rather than
        // where the range starts.
        if let Some((_, source_index, in_seconds)) = shot {
            self.reset_skip();
            self.load(source_index, in_seconds, true, Origin::Scrub);
        }
        let (source_index, start_source_seconds) = self.heading(None);
        let pending = PendingClip {
            id,
            source_index,
            start_source_seconds,
        };
        let path = folder.join(format!("{}.mkv", pending.id));

        self.generation += 1;
        let (tx, generation) = (self.tx.clone(), self.generation);
        // The last take's final frame must not open this one.
        let _ = self.self_view.take();
        let started = Recorder::start(sources, &path, self.self_view.clone(), move |msg| {
            // Fails only once the bus thread has exited.
            let _ = tx.send(Input::Recorder(generation, msg));
        });
        let recorder = match started {
            Ok(recorder) => recorder,
            Err(e) => {
                // filesink has already created it — and the diagram, copied
                // above, would be left with no clip to name it.
                remove_file(&path);
                if let Some(name) = &backdrop {
                    remove_file(&backdrops.join(name));
                }
                return self.emit(Event::Error(UserError::RecordingFailed(e)));
            }
        };
        let slate = shot.map(|(id, ..)| id);
        let whiteboard = backdrop.is_some();
        self.recording = Some(Active {
            pending,
            slate,
            backdrop,
            log: RecordingLog::new(recorder.t0_ns(), zoom, start_source_seconds),
            recorder,
            path,
            started: Instant::now(),
            media_seen: false,
        });
        // What the footage stops at, if this take was shot from a range
        // (BACKLOG #114). **Assigned unconditionally, so a plain `R` take
        // clears whatever was armed:** without that a take started while a
        // slate was armed from elsewhere would inherit its out point and pause
        // at a range it has nothing to do with. **Here**, where nothing can
        // refuse any more — every refusal above must leave the arm it found
        // alone, exactly as it leaves the player.
        self.armed_slate = slate;
        self.emit(Event::Recording(RecordingStatus::Starting { whiteboard }));
        // Recording always wins (Phase 10 spec S5): the transcript running
        // gives way and goes back to the front of the queue. **Here**, once
        // the recording exists — not at the top of `toggle_recording`, which
        // bails at five points above, where a refused record would have
        // killed a transcript for nothing. After the status, since this joins
        // the transcription thread and the UI is waiting to say Recording.
        self.preempt_transcription();
    }

    /// Stops the recording, keeping its clip, or aborts it if nothing has
    /// reached the muxer. Nothing to do while idle.
    pub(super) fn stop_recording(&mut self) {
        match &self.recording {
            None => {}
            Some(active) if active.media_seen => self.finish_recording(),
            Some(_) => self.abort_recording(),
        }
    }

    pub(super) fn recorder_message(&mut self, generation: u64, msg: RecorderMessage) {
        if generation != self.generation {
            return;
        }
        let Some(active) = &mut self.recording else {
            return;
        };
        match msg {
            RecorderMessage::FirstBuffer => {
                active.media_seen = true;
                let t0_ns = active.recorder.t0_ns();
                self.emit(Event::Recording(RecordingStatus::Recording { t0_ns }));
            }
            RecorderMessage::Level { peak_db, rms_db } => {
                self.emit(Event::Level { peak_db, rms_db })
            }
            RecorderMessage::Error(e) => {
                eprintln!("bus: recorder error: {e}");
                self.emit(Event::Error(UserError::RecordingFailed(e)));
                self.stop_recording();
            }
        }
    }

    /// When a recording that has had no buffer gives up, if one is starting.
    pub(super) fn start_deadline(&self) -> Option<Instant> {
        self.recording
            .as_ref()
            .filter(|active| !active.media_seen)
            .map(|active| active.started + START_TIMEOUT)
    }

    /// Nothing reached the muxer by [`Bus::start_deadline`]. A microphone
    /// that never delivers is exactly as fatal as a camera that never does.
    pub(super) fn start_timed_out(&mut self) {
        let silent = match self.avatar_mode() {
            true => "no sound from the microphone",
            false => "no video from the camera",
        };
        self.emit(Event::Error(UserError::RecordingFailed(format!(
            "{silent} within {} seconds",
            START_TIMEOUT.as_secs()
        ))));
        self.abort_recording();
    }

    /// Logs the play state just entered at `host_ns`, while recording.
    pub(super) fn log_playing(&mut self, host_ns: u64, ui_secs: Option<f64>) {
        let (_, anchor) = self.heading(ui_secs);
        let playing = self.playing;
        if let Some(active) = &mut self.recording {
            if playing {
                active.log.play(host_ns, anchor);
            } else {
                active.log.pause(host_ns, anchor);
            }
        }
    }

    /// Logs a zoom change while recording. The UI sends every change, so one
    /// made before its status caught up isn't lost; while idle it's ignored.
    pub(super) fn log_zoom(&mut self, host_ns: u64, zoom: Zoom) {
        if let Some(active) = &mut self.recording {
            active.log.zoom(host_ns, zoom);
        }
    }

    /// Logs a finished drawing while recording, ignored while idle (Phase 6
    /// spec D4). `host_ns` is its pen-up, the moment of its last point. The UI
    /// is what confines drawing to the Recording phase.
    pub(super) fn log_stroke(&mut self, host_ns: u64, stroke: Stroke) {
        if let Some(active) = &mut self.recording {
            active.log.stroke(host_ns, stroke);
        }
    }

    /// Logs a Clear while recording, ignored while idle.
    pub(super) fn log_clear_all(&mut self, host_ns: u64) {
        if let Some(active) = &mut self.recording {
            active.log.clear_all(host_ns);
        }
    }

    pub(super) fn set_camera(&mut self, camera: Option<String>) {
        if let Some(open) = &mut self.open {
            open.project.preferences.preferred_camera_id = camera;
            self.project_changed();
        }
    }

    pub(super) fn set_mic(&mut self, mic: Option<String>) {
        if let Some(open) = &mut self.open {
            open.project.preferences.preferred_mic_id = mic;
            self.project_changed();
        }
    }

    /// Arms `path` as the diagram W records over (BACKLOG #138), decoded first
    /// as `set_avatar` decodes: pixels out of [`decode_still`] are proof the
    /// export will get pixels too, and a refused pick leaves the arm it found.
    ///
    /// **Not the guarantee it looks like**, and worth saying so: the take
    /// copies the file minutes later, so one *deleted* since is refused at the
    /// copy, but one *replaced* with something undecodable records fine and
    /// fails at export. `set_avatar` decodes and copies adjacently; this
    /// splits them, and that is where the resemblance stops.
    pub(super) fn arm_backdrop(&mut self, path: PathBuf) {
        let still = match decode_still(&path) {
            Ok(still) => still,
            Err(e) => {
                return self.emit(Event::Error(UserError::Whiteboard(format!(
                    "that image couldn't be read, so it wasn't armed: {e}"
                ))))
            }
        };
        self.files.set_backdrop(Some(&path));
        self.backdrop = Some(Armed {
            path,
            still: Arc::new(still),
        });
        self.emit(Event::Backdrop(self.backdrop.clone()));
    }

    /// Forgets the armed diagram. The coach's file is theirs.
    pub(super) fn disarm_backdrop(&mut self) {
        self.files.set_backdrop(None);
        self.backdrop = None;
        self.emit(Event::Backdrop(None));
    }

    /// Decodes the diagram `state.json` says is armed, once, as the bus
    /// starts — **the bus is its one decoder**, and the window draws the
    /// pixels this sends. A file that won't decode any more (gone, replaced,
    /// on a drive not mounted) arms nothing for this run and leaves the file's
    /// choice alone, so the next run tries again.
    pub(super) fn load_backdrop(&mut self) {
        self.backdrop = self
            .files
            .backdrop()
            .and_then(|path| match decode_still(&path) {
                Ok(still) => Some(Armed {
                    path,
                    still: Arc::new(still),
                }),
                Err(e) => {
                    eprintln!(
                        "bus: the armed diagram {} won't decode: {e}",
                        path.display()
                    );
                    None
                }
            });
        self.emit(Event::Backdrop(self.backdrop.clone()));
    }

    /// Start's preconditions (R6), and no export or preview running. A skip or
    /// scrub in flight is not a reason to refuse: the recording starts where
    /// the player is heading.
    fn can_record(&self) -> Result<(), UserError> {
        let refused = |why| Err(UserError::CantRecord(why));
        match &self.open {
            None => refused("no project is open"),
            // They never overlap: sharing the video engine costs the
            // recording frames (Phase 5 spec X4).
            Some(_) if self.export.is_some() => refused("an export is running"),
            // The preview holds the picture and the audio sink, and a
            // recording is made over the game video (spec P5).
            Some(_) if self.preview.is_some() => refused("a preview is open; close it first"),
            Some(open) if open.project.source_videos.is_empty() => {
                refused("add a game video to record over first")
            }
            Some(_) if self.any_missing() => refused("a game video is missing; relink it first"),
            Some(_) if !self.loaded() => refused("the game video isn't loaded"),
            Some(_) => Ok(()),
        }
    }

    /// The project's avatar image **is** avatar mode (spec B1): with one, a
    /// take opens no camera.
    fn avatar_mode(&self) -> bool {
        self.open
            .as_ref()
            .is_some_and(|open| open.project.avatar.is_some())
    }

    /// The recorder's sources. For devices: the preferred camera and mic if
    /// connected, else the defaults with a notice, keeping the preference
    /// (R2). In avatar mode, the mic alone.
    fn capture_sources(&self, preferences: &Preferences) -> Result<CaptureSources, UserError> {
        let avatar = self.avatar_mode();
        if let CaptureKind::Test { video_delay } = self.capture {
            // Avatar mode is the project's, not the device path's: a test take
            // must record the same shape of file the coach's would.
            return Ok(CaptureSources::Test {
                video: (!avatar).then_some(video_delay),
            });
        }
        let devices = list_devices();
        // In avatar mode `resolve_camera` is never called, so `NoCamera`
        // cannot be raised and a machine with no camera records fine (C6).
        let camera = match avatar {
            true => None,
            false => {
                let (camera, fell_back) =
                    resolve_camera(&devices.cameras, preferences.preferred_camera_id.as_deref())
                        .ok_or(UserError::NoCamera)?;
                if fell_back {
                    self.emit(Event::Error(UserError::DeviceFallback { what: "camera" }));
                }
                Some(camera.clone())
            }
        };
        let (mic, mic_fell_back) =
            resolve_mic(&devices.mics, preferences.preferred_mic_id.as_deref());
        if mic_fell_back {
            self.emit(Event::Error(UserError::DeviceFallback {
                what: "microphone",
            }));
        }
        Ok(CaptureSources::Devices {
            camera,
            mic: mic.map(str::to_owned),
        })
    }

    /// Stops the recorder and adds the clip (R6's stop): the log is closed
    /// first, so no event outlasts the file.
    fn finish_recording(&mut self) {
        let Some(active) = self.recording.take() else {
            return;
        };
        // The arm goes with the take (BACKLOG #114): the range has had its
        // commentary, and the log below is closed, so nothing is left to write
        // a pause into. Cleared here and in `abort_recording` rather than in
        // `stop_recording`, which is not the only way to either — the start
        // timeout aborts directly.
        self.armed_slate = None;
        let events = active.log.finish();
        let clip_id = active.pending.id;
        let outcome = active.recorder.stop(STOP_TIMEOUT);
        if let Some(open) = &mut self.open {
            // Read out before the mutable borrow: what a slate-shot take
            // inherits. A slate deleted while the take ran inherits nothing,
            // which is the right answer and needs no special case.
            let inherited = active.slate.and_then(|id| {
                open.project
                    .slates
                    .iter()
                    .find(|s| s.id == id)
                    .map(|s| (s.name.clone(), s.tags.clone()))
            });
            let clip = open.project.add_recorded_clip(
                active.pending,
                outcome.duration,
                events,
                created_at(),
            );
            clip.slate_id = active.slate;
            clip.backdrop = active.backdrop;
            if let Some((name, tags)) = inherited {
                // An unnamed slate leaves the generated "2-01:02:05" alone: it
                // says more than an empty name would.
                if !name.is_empty() {
                    clip.name = name;
                }
                clip.tags = tags;
            }
        }
        self.project_changed();
        self.emit(Event::Recording(RecordingStatus::Idle));
        if !outcome.clean {
            self.emit(Event::Error(UserError::StopNotClean));
        }
        // The clip exists and is saved, so it can be transcribed (Phase 10
        // spec S6). Whatever this recording preempted resumes on its own:
        // `Bus::run`'s tail starts the queue once nothing is in its way.
        self.transcribe_after_recording(clip_id);
    }

    /// Drops a recording that never got a buffer: no clip, no file.
    fn abort_recording(&mut self) {
        let Some(active) = self.recording.take() else {
            return;
        };
        // As in `finish_recording`: the arm goes with the take.
        self.armed_slate = None;
        // NULL first, so nothing still writes the file.
        drop(active.recorder);
        remove_file(&active.path);
        // The diagram goes with the recording: nothing will ever name it.
        if let (Some(name), Some(open)) = (&active.backdrop, &self.open) {
            remove_file(&open.folder.join(BACKDROPS_DIRNAME).join(name));
        }
        self.emit(Event::Recording(RecordingStatus::Idle));
    }
}

/// Copies `src` into `dir` as `<stem>.<ext>` and returns that file name — a
/// project's avatar and a whiteboard take's diagram alike. The extension is
/// kept, lowercased, so the file opens in a file manager as what it is (the
/// decoders sniff regardless), and the bytes go through `.<stem>.tmp` and a
/// rename in the same directory, as `store::write` writes `project.json`: a
/// copy cut short leaves nothing.
///
/// `dir` is made if it isn't there, with **`create_dir`**: it is one leaf under
/// a project folder that exists, and a project folder deleted from under the
/// app should refuse the copy rather than be re-created around it.
pub(super) fn copy_into(src: &Path, dir: &Path, stem: &str) -> std::io::Result<String> {
    match std::fs::create_dir(dir) {
        Err(e) if e.kind() != std::io::ErrorKind::AlreadyExists => return Err(e),
        _ => {}
    }
    let name = match src.extension().and_then(|e| e.to_str()) {
        Some(ext) => format!("{stem}.{}", ext.to_ascii_lowercase()),
        None => stem.to_owned(),
    };
    let temp = dir.join(format!(".{stem}.tmp"));
    if let Err(e) = std::fs::copy(src, &temp).and_then(|_| std::fs::rename(&temp, dir.join(&name)))
    {
        let _ = std::fs::remove_file(&temp);
        return Err(e);
    }
    Ok(name)
}

/// Deletes a take's file. One that was never created is already gone.
fn remove_file(path: &Path) {
    match std::fs::remove_file(path) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            eprintln!("bus: could not delete {}: {e}", path.display());
        }
        _ => {}
    }
}

/// Now as RFC3339 in whole seconds, matching the fixtures and macOS. Empty
/// if the system time can't be read: nothing reads it.
fn created_at() -> String {
    glib::DateTime::now_utc()
        .and_then(|now| now.format("%Y-%m-%dT%H:%M:%SZ"))
        .map(Into::into)
        .unwrap_or_default()
}
