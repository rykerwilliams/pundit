//! The decode side: the source through the player's GL bin into a pull
//! `appsink`, and the lookup "last frame at or before this source time".

use std::path::{Path, PathBuf};

use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;

use super::{stamp_buffer, CompositeError, Gl, Stopper, Watch, POLL};
use crate::mailbox::stream_time;
use crate::player::{diagnostics, gl_bin, gl_caps, Diagnostics};
use pundit_core::export::OUTPUT_FPS;

/// A target at most this far ahead of the current frame is reached by
/// pulling forward (~1.3 ms a frame) rather than a seek, which decodes from
/// the keyframe before it (12 ms on camera footage, up to ~100 ms on a 2 s
/// GOP). Measured.
const PULL_AHEAD: gst::ClockTime = gst::ClockTime::from_mseconds(500);

/// How far after a target a frame may start and still count as "at" it:
/// nanosecond rounding, which a strict comparison can't tell from a real gap.
///
/// Frame times reach us through integer conversions that each floor, so a
/// frame meant to start exactly on a target can read back a few ns after it.
/// Measured: an MP4 at timescale 3000 with a two-frame edit list (B-frames;
/// what `mp4mux` writes at 30 fps, and what Trace's HLS downloads are) puts
/// frame `i` at `floor((i+2)/30 s) - floor(2/30 s)` in stream time, 1 ns after
/// `i/30 s`, so a round anchor showed the previous frame every third frame.
///
/// 1 µs is a thousand times that rounding and a four-thousandth of the
/// shortest frame there is (4.2 ms at 240 fps), so it can only change the
/// answer for a frame that starts within 1 µs of the target, which is that
/// frame to any precision a video has. It is **not** meant to absorb a
/// container's own coarser rounding (Matroska's millisecond timecodes store
/// frame 2 at 30 fps at 67 ms): that is where the file says the frame
/// starts, and absorbing it would pick a different frame than a strict
/// reading of the file does.
const SLACK: gst::ClockTime = gst::ClockTime::from_useconds(1);

/// One decoded frame and its source time.
struct Decoded {
    /// Holds a buffer with a PTS.
    sample: gst::Sample,
    /// [`stream_time`], not PTS, which is what the schedule's times are.
    time: gst::ClockTime,
}

pub(super) struct Decoder {
    pipeline: Stopper,
    appsink: gst_app::AppSink,
    glupload: gst::Element,
    /// The frame `frame_at` last answered with.
    current: Option<Decoded>,
    /// The frame after `current`, pulled to learn that `current` is still the
    /// last one at or before a target.
    next: Option<Decoded>,
    /// The stream ended after `current`.
    eos: bool,
}

impl Decoder {
    /// Builds the pipeline, prerolls it, and sets it PLAYING. Its errors
    /// reach `watch`.
    pub(super) fn start(source: &Path, gl: &Gl, watch: &Watch) -> Result<Decoder, CompositeError> {
        let pipeline = gst::Pipeline::new();
        let make = |factory: &str| {
            gst::ElementFactory::make(factory)
                .build()
                .map_err(|e| CompositeError::Failed(format!("{factory} is missing: {e}")))
        };
        let filesrc = make("filesrc")?;
        filesrc.set_property("location", source);
        let decodebin = make("decodebin3")?;
        let appsink = gst_app::AppSink::builder()
            .caps(&gl_caps())
            .sync(false)
            .max_buffers(2u32)
            .enable_last_sample(false)
            .build();
        let (gl_sink, glupload) = gl_bin(&appsink);
        pipeline
            .add_many([&filesrc, &decodebin, &gl_sink])
            .expect("add decode elements");
        filesrc
            .link(&decodebin)
            .expect("link filesrc to decodebin3");
        // The video stream only. Other pads (audio) stay unlinked, which
        // `decodebin3` tolerates, across seeks too (measured).
        decodebin.connect_pad_added(move |_, pad| {
            let sink = gl_sink.static_pad("sink").expect("the GL bin has a sink");
            if pad.name().starts_with("video_") && !sink.is_linked() {
                let _ = pad.link(&sink);
            }
        });
        // The appsink reports the end of the stream.
        gl.install(&pipeline, watch, |_| {});
        let pipeline = Stopper(pipeline);

        // Preroll first: a seek before the stream is up is dropped.
        if pipeline.set_state(gst::State::Paused).is_err() {
            return Err(watch.failure("could not open the source"));
        }
        loop {
            watch.check()?;
            match pipeline.state(POLL) {
                (Ok(_), gst::State::Paused, gst::State::VoidPending) => break,
                (Err(_), ..) => {
                    watch.check()?;
                    return Err(CompositeError::Failed("could not read the source".into()));
                }
                _ => {}
            }
        }
        if pipeline.set_state(gst::State::Playing).is_err() {
            return Err(watch.failure("could not play the source"));
        }
        Ok(Decoder {
            pipeline,
            appsink,
            glupload,
            current: None,
            next: None,
            eos: false,
        })
    }

    /// The last frame with stream time at or before `target` give or take
    /// [`SLACK`] (or the first frame, for a target before it; the last, for
    /// one past the end).
    ///
    /// Reuses the current frame while it still answers, pulls forward to a
    /// target up to [`PULL_AHEAD`] ahead, and seeks otherwise. It never seeks
    /// backwards to a target at or after the current frame: a freeze, or a
    /// 60 → 30 fps drop, costs nothing.
    pub(super) fn frame_at(
        &mut self,
        target: gst::ClockTime,
        watch: &Watch,
    ) -> Result<&gst::Sample, CompositeError> {
        // The latest frame time that answers `target`. Every comparison uses
        // it, so a frame that starts inside the slack is reused rather than
        // seeked for again.
        let reach = target + SLACK;
        // Past the end, the last frame answers every later target.
        let far = self
            .current
            .as_ref()
            .is_none_or(|c| reach < c.time || (!self.eos && reach - c.time > PULL_AHEAD));
        if far {
            self.seek(reach)?;
        }
        while !self.eos {
            if self.next.is_none() {
                self.next = self.pull(watch)?;
                if self.next.is_none() {
                    self.eos = true;
                    break;
                }
            }
            let next = self.next.as_ref().expect("pulled above");
            if self.current.is_some() && next.time > reach {
                break;
            }
            self.current = self.next.take();
        }
        self.current
            .as_ref()
            .map(|c| &c.sample)
            .ok_or_else(|| CompositeError::Failed(format!("the source has no frame at {target}")))
    }

    pub(super) fn diagnostics(&self) -> Diagnostics {
        diagnostics(&self.pipeline, Some(&self.glupload))
    }

    /// A flushing seek to the keyframe at or before `target`, from which
    /// `frame_at` pulls forward. Not an accurate seek: it drops a frame
    /// whose duration ends before `target` although it is the last one
    /// before it (a gap, or VFR), and past the video's end it finds nothing.
    /// The frames held are from before it.
    fn seek(&mut self, target: gst::ClockTime) -> Result<(), CompositeError> {
        self.current = None;
        self.next = None;
        self.eos = false;
        self.pipeline
            .seek_simple(
                gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT | gst::SeekFlags::SNAP_BEFORE,
                target,
            )
            .map_err(|_| CompositeError::Failed(format!("the source refused a seek to {target}")))
    }

    /// The next decoded frame, or `None` at the end of the stream. Waits in
    /// [`POLL`] steps, checking `watch` between them.
    fn pull(&mut self, watch: &Watch) -> Result<Option<Decoded>, CompositeError> {
        loop {
            if let Some(sample) = self.appsink.try_pull_sample(POLL) {
                let time = stream_time(&sample).ok_or_else(|| {
                    CompositeError::Failed(
                        "a decoded frame has no timestamp or time segment".into(),
                    )
                })?;
                return Ok(Some(Decoded { sample, time }));
            }
            if self.appsink.is_eos() {
                return Ok(None);
            }
            watch.check()?;
        }
    }
}

/// Where one entry's picture comes from (BACKLOG #138).
///
/// **One value rather than a path plus a flag**, which is
/// `preview::InsetSource`'s rule for its reason: the two cases are exclusive,
/// and the decoder, the letterbox, the board and the highlights are all driven
/// from this one answer. A `PathBuf` that "means the image for a whiteboard"
/// with the clip's `backdrop` separately saying *whether* it is one would be
/// two values free to disagree — and the way they disagree is a `Decoder`
/// opened on a PNG, or a [`Texture`] built from an MP4.
///
/// **The bus builds it once**, where it has the project and the clip, and both
/// tails take it from there.
///
/// **What this does *not* buy is a decode failure, which is worth knowing.**
/// Sabotaged — a `Backdrop` handed to the decoder arm — the export is
/// pixel-identical and every test passes: `decodebin3` reads a PNG as a
/// one-frame video, so `Decoder` is perfectly happy with one. The arms differ
/// in **cost** (a decoder and a seek per frame against one upload held for the
/// run) and in what the callers above must do with the two cases — the preview
/// has to skip the game-video refusals for a backdrop, and the board and the
/// highlights have to be omitted — not in what reaches the file. So this enum
/// earns its place by removing a pairing nobody can get wrong rather than by
/// turning a silent error into a loud one, and a test that claimed otherwise
/// would be claiming a proof there is none of.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Picture {
    /// The game video this entry's frames are decoded from.
    Footage(PathBuf),
    /// The still this entry was recorded over: no decoder, no match clock, no
    /// player highlights. The file is under the project's `backdrops/`.
    Backdrop(PathBuf),
}

impl Picture {
    /// The file either way, for the `stat` that refuses a missing one and for
    /// the diagnostics line.
    pub fn path(&self) -> &Path {
        match self {
            Picture::Footage(path) | Picture::Backdrop(path) => path,
        }
    }

    /// The backdrop's file, or `None` for footage — the **one** reading of
    /// "is this a whiteboard entry", so no site re-derives it.
    pub fn backdrop(&self) -> Option<&Path> {
        match self {
            Picture::Backdrop(path) => Some(path),
            Picture::Footage(_) => None,
        }
    }
}

/// One still RGBA image **in GL memory**, uploaded once and re-stamped for
/// every frame that shows it: the inset pad's 1×1 transparent filler
/// ([`Texture::filler`]), and the project's avatar ([`avatar_texture`]). The pad
/// scales it to whatever rect it has — a transparent pixel is invisible
/// however big (measured), and the avatar's rect is the pulse.
///
/// **It has to be GL memory, because the pad's caps feature may not change.**
/// The recording decodes to GL, so a system-memory filler made the branch's
/// `glupload` take GL frames and then a system-memory one, which it refuses
/// ("Failed to upload buffer"): any target whose first entry has no inset and
/// whose second has one died there (reproduced). Uploaded here, the pad
/// carries GL memory from the first frame to the last, and only the caps
/// change — which GL to GL takes. A mixed avatar-and-camera compilation is
/// that same case (spec I2).
///
/// **Whether its pixels are premultiplied is the *pad's* business, not
/// this type's**, and the two callers differ (BACKLOG #138). The inset pad is
/// told `premultiplied_over` and is fed a tiny-skia pixmap, which is
/// premultiplied storage. The **picture** pad is not told it, so a whiteboard
/// backdrop is uploaded straight from `Still`'s own straight-alpha RGBA with no
/// preparation at all — and premultiplying it would be the mirror image of the
/// halo bug `composite::avatar`'s module comment records.
///
/// **It lives here, beside [`Decoder`], because the picture pad takes one or
/// the other.** `head`'s branch is `appsrc ! gltransformation ! glvideomixer`
/// with **no `glupload`**, so whatever feeds that pad must already be GL
/// memory — which is why a whiteboard still cannot take `AvatarInset`'s
/// system-memory route. That route is right for the *inset* pad, whose branch
/// has a `glupload` of its own: two patterns, because two pads differ.
#[derive(Clone)]
pub(super) struct Texture {
    buffer: gst::Buffer,
    caps: gst::Caps,
}

impl Texture {
    /// The inset pad's stand-in: one transparent pixel.
    pub(super) fn filler(gl: &Gl, watch: &Watch) -> Result<Texture, CompositeError> {
        Texture::upload(gl, watch, 1, 1, vec![0; 4])
    }

    /// This texture as output frame `n`, with the caps it must be pushed
    /// under: a new buffer header over the **same texture**, never a copy of
    /// the pixels.
    pub(super) fn stamped(&self, n: u64) -> (gst::Buffer, gst::Caps) {
        let mut buffer = self.buffer.copy();
        stamp_buffer(&mut buffer, n);
        (buffer, self.caps.clone())
    }

    /// Uploads `rgba` (`w`×`h`, tightly packed) on `gl`, through a pipeline of
    /// its own that is gone by the time this returns. The texture outlives it:
    /// the buffer holds it, and `gl`'s context is the process's
    /// ([`Gl::shared`]).
    pub(super) fn upload(
        gl: &Gl,
        watch: &Watch,
        w: u32,
        h: u32,
        rgba: Vec<u8>,
    ) -> Result<Texture, CompositeError> {
        debug_assert_eq!(rgba.len(), (w * h * 4) as usize, "tightly packed RGBA");
        let pipeline = gst::parse::launch(&format!(
            "appsrc name=src format=time is-live=false block=false \
               caps=video/x-raw,format=RGBA,width={w},height={h},framerate={OUTPUT_FPS}/1 \
             ! glupload ! glcolorconvert ! appsink name=out sync=false"
        ))
        .map_err(|e| CompositeError::Failed(format!("could not build the upload graph: {e}")))?
        .downcast::<gst::Pipeline>()
        .expect("a multi-element launch string yields a pipeline");
        let by_name = |n: &str| pipeline.by_name(n).expect("named in the launch string");
        let src = by_name("src")
            .downcast::<gst_app::AppSrc>()
            .expect("named as an appsrc in the launch string");
        let sink = by_name("out")
            .downcast::<gst_app::AppSink>()
            .expect("named as an appsink in the launch string");
        sink.set_caps(Some(&gl_caps()));
        gl.install(&pipeline, watch, |_| {});
        let pipeline = Stopper(pipeline);
        if pipeline.set_state(gst::State::Playing).is_err() {
            return Err(watch.failure("could not start the upload graph"));
        }

        let mut buffer = gst::Buffer::from_mut_slice(rgba);
        stamp_buffer(&mut buffer, 0);
        src.push_buffer(buffer)
            .map_err(|e| watch.failure(format!("uploading a {w}x{h} still: {e:?}")))?;
        let _ = src.end_of_stream();
        loop {
            watch.check()?;
            if let Some(sample) = sink.try_pull_sample(POLL) {
                let (Some(buffer), Some(caps)) = (sample.buffer(), sample.caps()) else {
                    return Err(CompositeError::Failed(
                        "an uploaded still came back bare".into(),
                    ));
                };
                return Ok(Texture {
                    buffer: buffer.copy(),
                    caps: caps.to_owned(),
                });
            }
            if sink.is_eos() {
                return Err(watch.failure(format!("a {w}x{h} still did not upload")));
            }
        }
    }
}
