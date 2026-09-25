//! Slates: a range marked while watching, waiting for its commentary.
//!
//! **The times are the caller's**, like every logged position: the displayed
//! frame's source time, captured on the UI thread at the key press. The bus
//! asking the player where it is would mark whatever frame the queue delay had
//! reached.
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

use pundit_core::project::{Project, SlateEdit};
use pundit_core::undo::UndoAction;
use pundit_core::zoom::Zoom;
use uuid::Uuid;

use super::{Bus, Event, UserError};

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
        let from = (id, slate.source_index, slate.in_seconds);
        self.start_recording_from_slate(zoom, from);
    }

    pub(super) fn edit_slate(&mut self, id: Uuid, edit: SlateEdit) {
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
