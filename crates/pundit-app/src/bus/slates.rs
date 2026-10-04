//! Slates: a range marked while watching, waiting for its commentary.
//!
//! **The times are the caller's**, like every logged position: the displayed
//! frame's source time, captured on the UI thread at the key press. The bus
//! asking the player where it is would mark whatever frame the queue delay had
//! reached.
//!
//! **The out-point stop ([`Bus::stop_at_slate_out`]) is the one place the bus
//! mints a timestamp of its own, and the exception is narrow.** `CLAUDE.md`'s
//! contract is about the queue delay between an input event and the handler
//! that stamps it; a crossing has no input event — the poll *is* the event, so
//! nothing was queued, and the time wanted is when the picture stopped, which
//! is now. What keeps it honest is that the *position* is still not a reading:
//! the pause is anchored at the stored `out_seconds`, exactly as
//! [`Bus::shoot_slate`] seeks to a stored `in_seconds`. `transport.rs` declines
//! to log a pause at a player error and at EOS, and the reason there is not a
//! bus-side time either — it is that neither has anything to anchor to.
//!
//! **A refusal here is a notice, never a modal.** Marking is on the recording
//! allow-list — a coach spots the next moment while talking over this one — so
//! a modal could land over a live take and swallow the transport keys. That is
//! `UserError::Scoreboard`'s reasoning, and it applies for the same reason.
//!
//! Each command is one `EditSlates` step holding the **whole** list, as a
//! match-event or highlight edit is: the list is small, a snapshot needs no
//! per-command inverse, and a source move or removal purges it from both
//! stacks because a snapshot holds source indices.

use std::time::{Duration, Instant};

use pundit_core::project::{Project, SlateEdit, SlateError};
use pundit_core::undo::UndoAction;
use pundit_core::zoom::Zoom;
use pundit_media::LEVEL_INTERVAL_NS;
use uuid::Uuid;

use super::recording::Shot;
use super::{Bus, Event, UserError};

/// How often the bus wakes to see whether the armed slate's out point has
/// passed, when nothing else would wake it.
///
/// It is the recorder's level interval because that is what a take already
/// wakes at — level messages arrive 10 times a second, so during a take this
/// poll costs no wake-ups at all. It is here for the quiet path, where plain
/// playback posts nothing periodic. Worst case the stop is one poll late,
/// which is three frames of footage and costs the log nothing: the pause is
/// anchored at the stored `out`, never at the position read here.
const OUT_POLL: Duration = Duration::from_nanos(LEVEL_INTERVAL_NS);

impl Bus {
    /// `i`: open a range at the displayed frame.
    pub(super) fn mark_slate_in(&mut self, source_index: usize, source_seconds: f64) {
        let Some(open) = &self.open else {
            return;
        };
        // The same guard `tag_match_event` and `set_highlight_key` carry: a
        // record stored on a source that isn't there would be a row that
        // panics the moment it tries to name its video.
        if source_index >= open.project.source_videos.len() {
            return eprintln!("bus: MarkSlateIn on source {source_index}, which isn't there");
        }
        self.edit_slates(|project| {
            project.mark_slate_in(source_index, source_seconds);
        });
    }

    /// `o`: close the range most recently opened on this video, or say why
    /// not.
    pub(super) fn mark_slate_out(&mut self, source_index: usize, source_seconds: f64) {
        let mut refused = None;
        self.edit_slates(|project| {
            refused = project.mark_slate_out(source_index, source_seconds).err();
        });
        if let Some(e) = refused {
            self.emit(Event::Error(UserError::Slate(e.to_string())));
        }
    }

    /// Record the commentary for slate `id`: go to its in point and arm a
    /// take, which is the whole point of the feature.
    ///
    /// **This carries no captured position**, unlike the marks: the in point
    /// is a stored field, not a reading of the playhead. `EditMatchEvent`
    /// records the same reasoning for a typed time. The `zoom` is the one the
    /// recording log opens with, which the window owns.
    ///
    /// The seek itself is `start_recording`'s, because the refusals are:
    /// see its doc comment.
    pub(super) fn shoot_slate(&mut self, id: Uuid, zoom: Zoom) {
        let Some(open) = &self.open else {
            return;
        };
        let Some(slate) = open.project.slates.iter().find(|s| s.id == id) else {
            return eprintln!("bus: ShootSlate on slate {id}, which isn't there");
        };
        let shot = Shot {
            slate: id,
            source_index: slate.source_index,
            in_seconds: slate.in_seconds,
        };
        self.start_recording(zoom, Some(shot));
    }

    /// When to wake to check the armed slate's out point, if one is armed and
    /// the footage is playing.
    ///
    /// **Recomputed every iteration rather than stored**, which is what makes
    /// this a poll and not a deadline: there is nothing to arm, and so no
    /// re-arm site to miss when a play, a pause, a skip landing or a rate
    /// change moves when — or whether — the out point will be reached. An
    /// estimate of *when* it falls cannot replace it, and the plan's §R has
    /// the arithmetic: armed at `out = 100` with the playhead at 20, the
    /// estimate is 80 s out; skip to 95 and the out point passes 5 s later,
    /// but the first check lands 75 s late, after the take ended, which is
    /// indistinguishable from never.
    pub(super) fn out_poll(&self) -> Option<Instant> {
        (self.armed_slate.is_some() && self.playing).then(|| Instant::now() + OUT_POLL)
    }

    /// Pauses the game video where the armed slate's range ends (spec S1),
    /// once, leaving the recording running: the picture holds on the range's
    /// last frame while the coach finishes the sentence, and Stop ends the
    /// take as before. Called from the bus loop's tail.
    ///
    /// **The pause is logged directly, not through `log_playing`**, which
    /// computes its own anchor from `heading` — the skip coordinator's pending
    /// target, then the seek in flight, then the caller's value — so it
    /// reaches `out` only when both are idle. `set_playing(false)` below
    /// issues a seek of its own when the rate is not 1x, making
    /// `target_secs()` `Some` before the log line runs, so the anchor would be
    /// non-deterministic; and `EventKind`'s own doc says an anchor overrides
    /// the wall-clock cursor on replay. There is nothing to compute here
    /// anyway: `out` is a stored field.
    ///
    /// **Disarming is also the state-change guard, deliberately.**
    /// `slate_out_reached` requires `self.playing`, so the `set_playing` below
    /// always changes state and no second pause can be written — the same
    /// `if self.playing != was_playing` rule `toggle_play` applies, by
    /// construction rather than by coincidence.
    pub(super) fn stop_at_slate_out(&mut self) {
        let Some(out) = self.slate_out_reached() else {
            return;
        };
        self.armed_slate = None;
        // The bus's own clock: the module header says why that is sound here
        // and nowhere else, and "when the picture stopped" is now.
        let host_ns = pundit_media::now_ns();
        self.set_playing(false);
        if let Some(active) = &mut self.recording {
            active.log.pause(host_ns, out);
        }
    }

    /// The armed slate's out point, if the footage has played up to it.
    ///
    /// **`preview.is_none()` is not optional.** `set_playing` acts on
    /// whichever pipeline is on screen while `query_position` always reads the
    /// **game** pipeline, so without it a stale arm would stop a clip preview
    /// dead, with no notice and no log.
    ///
    /// **`query_position` is safe to compare against a stored source time,
    /// measured:** after a flushing seek it returns `None` for 0–15 ms and
    /// then the seek's *target* — never the pre-seek value — and the same
    /// across a source change. So there is no stale read to defend against,
    /// which is why no previous position is kept. What it cannot do is tell a
    /// deliberate move past the out point from playback reaching it, which is
    /// [`Bus::disarm_if_past_slate_out`]'s job.
    ///
    /// The source check is the slate's own rule — one source per slate — and
    /// the position is read in source seconds, so comparing without it would
    /// pause the footage in the *next* video wherever its time passed the out
    /// point.
    fn slate_out_reached(&self) -> Option<f64> {
        let id = self.armed_slate?;
        if !self.playing || self.preview.is_some() {
            return None;
        }
        let slate = self
            .open
            .as_ref()?
            .project
            .slates
            .iter()
            .find(|s| s.id == id)?;
        let out = slate.out_seconds?;
        if slate.source_index != self.current {
            return None;
        }
        (self.position.query_position()? >= out).then_some(out)
    }

    /// Disarms, **without pausing**, when a request takes the footage to or
    /// past the armed slate's out point (BACKLOG #114).
    ///
    /// **The distinguisher is the event, not the position**, and that is
    /// measured: a forward skip makes `query_position` read the skip's target
    /// within 5 ms, so the check above cannot tell the two apart on its own.
    /// Skip +6 s from 59.5 with `out = 60` reads ~65.5, fires, and logs
    /// `Pause { source_time: 60.0 }` while the picture is at 65.5 — replay
    /// then freezes 5.5 s behind the footage the coach is talking over. #114 is
    /// explicit that a deliberate move past the out point must not trigger the
    /// stop. With this, the poll only ever sees a position reached by
    /// **playback**, so anchoring at `out` is honest to the frame, and no
    /// previous-tick state machine is needed. A position tolerance cannot do
    /// this job: a missed poll and a deliberate skip look identical in
    /// position alone.
    ///
    /// **One site, because `load` is the one path every request to the player
    /// takes** — a skip landing, a scrub release, a frame step, a jump to a
    /// clip. It is given the request rather than a reading, so it decides
    /// before the seek is even issued, and after `load`'s clamp, so it sees
    /// where the seek will actually land. A move *back* before the out point
    /// keeps the arm: the coach went there, and playing on reaches the end of
    /// the range again.
    pub(super) fn disarm_if_past_slate_out(&mut self, index: usize, secs: f64) {
        let Some(id) = self.armed_slate else {
            return;
        };
        let Some(open) = &self.open else {
            return;
        };
        let past = open.project.slates.iter().any(|s| {
            s.id == id && s.source_index == index && s.out_seconds.is_some_and(|out| secs >= out)
        });
        if past {
            self.armed_slate = None;
        }
    }

    /// Applies `edit` to slate `id`, **refusing one that would invert the
    /// range** (BACKLOG #119).
    ///
    /// **The check is here and not in core** because this is where a refusal
    /// can be seen: `edit_slates` below returns silently when nothing changed —
    /// by design, so a command naming a slate that is gone costs nothing — so a
    /// core-side refusal would be indistinguishable from a key that did nothing.
    /// `Project::edit_slate` stays infallible and `Slate::would_invert` is the
    /// question.
    ///
    /// The notice is `mark_slate_out`'s own `SlateError::OutBeforeIn`, reused
    /// rather than reworded: it is the same rule, and a coach who meets it from
    /// `o` and from this button should read the same sentence.
    pub(super) fn edit_slate(&mut self, id: Uuid, edit: SlateEdit) {
        if let Some(open) = &self.open {
            if let Some(slate) = open.project.slates.iter().find(|s| s.id == id) {
                if slate.would_invert(&edit) {
                    let e = SlateError::OutBeforeIn {
                        in_seconds: slate.in_seconds,
                    };
                    return self.emit(Event::Error(UserError::Slate(e.to_string())));
                }
            }
        }
        self.edit_slates(|project| project.edit_slate(id, edit));
    }

    pub(super) fn delete_slate(&mut self, id: Uuid) {
        self.edit_slates(|project| project.delete_slate(id));
    }

    /// Applies `edit` to the slate list as one undo step, unless it changed
    /// nothing — which is how a command naming a slate that is gone costs
    /// neither a save nor a step.
    fn edit_slates(&mut self, edit: impl FnOnce(&mut Project)) {
        let Some(open) = &mut self.open else {
            return;
        };
        let before = open.project.slates.clone();
        edit(&mut open.project);
        let after = open.project.slates.clone();
        if after == before {
            return;
        }
        self.save();
        self.record(UndoAction::EditSlates { before, after });
        self.publish_project();
    }
}
