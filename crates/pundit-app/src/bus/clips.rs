//! Clip management (Phase 3 spec C1–C5): field edits, delete to the trash,
//! reorder and sort, jump, and the undo history they share.
//!
//! Every mutation is diffed first (unchanged ⇒ no save, no undo step), then
//! applied, saved, recorded and published: recorded before it's published,
//! so an event always follows any shred the push caused. Every push goes
//! through [`Bus::record`], so a delete the history drops always has its
//! trashed file shredded.
//!
//! **The trash.** A deleted clip's recording moves to
//! `recordings/.trash/<file>` only once the project without it has saved, and
//! comes back before the project with it is saved. So `project.json` never
//! lists a clip whose file is in `.trash`, which every open empties; a crash
//! leaves at worst an unreferenced recording in `recordings/` (BACKLOG #38).
//! Shredding only ever touches `.trash`.

use std::io;
use std::path::{Path, PathBuf};

use pundit_core::project::Clip;
use pundit_core::store::RECORDINGS_DIRNAME;
use pundit_core::undo::{ClipEdit, UndoAction};
use uuid::Uuid;

use super::{Bus, Event, UserError};

/// Inside `recordings/`, so moves in and out are same-filesystem renames.
const TRASH_DIRNAME: &str = ".trash";

impl Bus {
    pub(super) fn edit_clip(&mut self, id: Uuid, edit: ClipEdit) {
        let Some(open) = &mut self.open else {
            return;
        };
        let Some(before) = open.project.apply_edit(id, edit.clone()) else {
            return eprintln!("bus: EditClip on a clip that isn't there: {id}");
        };
        // The inset's size and corner are **sticky**: the value the coach last
        // set on a clip is what the next recording is given (#88 spec I6), as
        // the export sheet's three pickers are written back in `export.rs`.
        // Here, before the save that carries it — after it, the preference
        // would be lost until some unrelated edit wrote the project again.
        //
        // **Before the no-op guard below, and reason enough to save on its
        // own.** Nothing to save about the *document* is not the same as nothing
        // the coach asked for: `ComboBoxBase::select` calls `selected` whatever
        // was showing, so picking the value a clip already has is a real
        // gesture — and it is the only way to say "put my takes back in the
        // usual corner" about a clip that is already there, or to undo a sticky
        // choice after a Ctrl+Z. Behind the guard it was swallowed, leaving the
        // next recording in the corner the coach had just moved away from and
        // the picker showing exactly what they chose.
        //
        // **Undo does not put it back**: a last-used value is not part of the
        // document's meaning, which is why the `last_export_*` preferences sit
        // outside the undo history too — and so a preference-only change files
        // no undo step.
        let sticky = match &edit {
            ClipEdit::InsetSize(v) => {
                std::mem::replace(&mut open.project.preferences.last_inset_size, *v) != *v
            }
            ClipEdit::InsetCorner(v) => {
                std::mem::replace(&mut open.project.preferences.last_inset_corner, *v) != *v
            }
            _ => false,
        };
        if before == edit {
            if sticky {
                self.save();
                self.publish_project();
            }
            return;
        }
        self.save();
        self.record(UndoAction::EditClip {
            id,
            before,
            after: edit,
        });
        self.publish_project();
    }

    pub(super) fn delete_clip(&mut self, id: Uuid) {
        match self.trash_clip(id) {
            Some(clip) => {
                self.record(UndoAction::DeleteClip(clip));
                self.publish_project();
            }
            None => eprintln!("bus: DeleteClip on a clip that isn't there: {id}"),
        }
    }

    pub(super) fn move_clip(&mut self, from: usize, to: usize) {
        let Some(open) = &self.open else {
            return;
        };
        let len = open.project.clips.len();
        if from >= len || to >= len {
            return eprintln!("bus: MoveClip {{ {from} -> {to} }} out of range");
        }
        let order = open.project.moved_order(from, to);
        self.reorder_clips(order);
    }

    pub(super) fn sort_clips_by_source(&mut self) {
        if let Some(open) = &self.open {
            let order = open.project.source_sorted_order();
            self.reorder_clips(order);
        }
    }

    /// Applies `after` as the clip order, unless it already is.
    fn reorder_clips(&mut self, after: Vec<Uuid>) {
        let Some(open) = &mut self.open else {
            return;
        };
        let before = open.project.clip_order();
        if after == before {
            return;
        }
        open.project.apply_clip_order(&after);
        self.save();
        self.record(UndoAction::ReorderClips { before, after });
        self.publish_project();
    }

    /// Pauses the game video at the clip's start, through the `park_at` a
    /// slate row's jump shares (BACKLOG #104): the same row in the same panel
    /// lands the same way.
    pub(super) fn jump_to_clip(&mut self, id: Uuid) {
        let Some(clip) = self
            .open
            .as_ref()
            .and_then(|open| open.project.clips.iter().find(|c| c.id == id))
        else {
            return eprintln!("bus: JumpToClip on a clip that isn't there: {id}");
        };
        let (index, secs) = (clip.source_index, clip.start_source_seconds);
        self.park_at(index, secs);
    }

    pub(super) fn undo(&mut self) {
        let Some(action) = self.history.take_undo() else {
            return;
        };
        match action {
            UndoAction::DeleteClip(clip) => {
                let id = clip.id;
                if self.restore_clip(&clip) {
                    self.history.file_redo(UndoAction::DeleteClip(clip));
                    self.emit(Event::Select(id));
                } else {
                    // Still trashed: it stays undoable.
                    self.history.file_undo(UndoAction::DeleteClip(clip));
                }
            }
            action => {
                if let Some(action) = self.replay(action, false) {
                    self.history.file_redo(action);
                }
            }
        }
    }

    pub(super) fn redo(&mut self) {
        let Some(action) = self.history.take_redo() else {
            return;
        };
        match action {
            // Files the clip as it is now, not the old snapshot: its source
            // index may have been remapped since it was restored.
            UndoAction::DeleteClip(clip) => match self.trash_clip(clip.id) {
                Some(clip) => {
                    self.history.file_undo(UndoAction::DeleteClip(clip));
                    self.publish_project();
                }
                None => eprintln!(
                    "bus: redo: dropped the delete of a missing clip {}",
                    clip.id
                ),
            },
            action => {
                if let Some(action) = self.replay(action, true) {
                    self.history.file_undo(action);
                }
            }
        }
    }

    /// Applies an edit, a reorder or a match-event or highlight snapshot for a redo
    /// (`forward`) or an undo, saves it, and returns it to be filed. `None`
    /// (and nothing saved) for an edit of a clip that's gone, which eviction's
    /// purge makes unreachable. A delete moves a file, so `undo` and `redo`
    /// handle it themselves.
    fn replay(&mut self, action: UndoAction, forward: bool) -> Option<UndoAction> {
        let open = self.open.as_mut()?;
        match &action {
            UndoAction::EditClip { id, before, after } => {
                let value = if forward { after } else { before };
                if open.project.apply_edit(*id, value.clone()).is_none() {
                    eprintln!("bus: dropped an edit of a missing clip {id}");
                    return None;
                }
                let id = *id;
                self.project_changed();
                self.emit(Event::Select(id));
            }
            UndoAction::ReorderClips { before, after } => {
                open.project
                    .apply_clip_order(if forward { after } else { before });
                self.project_changed();
            }
            // The whole list either way: a source change purges these, so
            // neither side can hold an index the project has moved on from.
            UndoAction::EditMatchEvents { before, after } => {
                open.project.match_events = if forward { after } else { before }.clone();
                self.project_changed();
            }
            UndoAction::EditHighlights { before, after } => {
                open.project.player_highlights = if forward { after } else { before }.clone();
                self.project_changed();
            }
            UndoAction::EditSlates { before, after } => {
                open.project.slates = if forward { after } else { before }.clone();
                self.project_changed();
            }
            UndoAction::DeleteClip(_) => unreachable!("deletes aren't replayed"),
        }
        Some(action)
    }

    /// Drops the history entries a source move or removal invalidated — the
    /// deletes on the undo stack (spec C4) and the match-event and highlight
    /// snapshots on both (Phase 9 spec S5, match vision spec H3) — and shreds
    /// the trashed recordings that leaves unreachable.
    pub(super) fn purge_history_for_source_change(&mut self) {
        let evicted = self.history.purge_for_source_change();
        self.shred(evicted);
    }

    /// Pushes `action` onto the history and shreds any delete the cap drops.
    /// The one way onto the history.
    pub(super) fn record(&mut self, action: UndoAction) {
        let dropped = self.history.push(action);
        self.shred(dropped);
    }

    /// Removes clip `id` and saves; only if the save succeeded, moves its
    /// recording into `.trash`. Returns the clip as removed, or `None` if it
    /// isn't there. The caller files it, then publishes, so the snapshot
    /// follows the file.
    fn trash_clip(&mut self, id: Uuid) -> Option<Clip> {
        // Its recording is about to move into `.trash`, and a preview holds
        // that file open (spec P5) -- as does a transcription of it, which
        // would otherwise fail against a file that has moved and leave a
        // message naming a clip that is gone (Phase 10 spec S5).
        // The transcription first: closing a preview looks for a queued job
        // to start, and this clip's is the one it would find.
        self.cancel_transcription_of(id);
        self.close_preview_of(id);
        let open = self.open.as_mut()?;
        let clip = open.project.remove_clip(id)?;
        // **The third holder of this recording, and dropped only once the
        // delete has actually happened** (BACKLOG #77 spec §Q6). A queued
        // export job's `ClipMedia::recording` is `recordings/<file>`, so from
        // the rename below `Pip::open` cannot read it and that entry falls back
        // to the 1x1 GL filler — the job would still write a film, with the
        // coach's commentary silently gone and its text bar and chapter still
        // in place. A silent quality loss is worse than a failure.
        //
        // **Below `remove_clip`, unlike the two calls above it**, and that is
        // deliberate rather than tidy: both `?`s above can make this function a
        // no-op, and a cancelled transcription and a closed preview are both
        // recoverable where **the queue has no undo**. A delete that does not
        // happen must not destroy queue entries.
        //
        // Not a refusal: `source_is_referenced` is the precedent for one, but
        // that exists because a removed source leaves stored indices pointing
        // at the wrong file. Deleting a clip corrupts nothing and is undoable,
        // and the coach's work on his project must not be held up by a queue he
        // may have forgotten. Undo restores the clip, not the job.
        let dropped = self.drop_queued_for_clip(id);
        if dropped > 0 {
            let plural = if dropped == 1 { "" } else { "s" };
            self.emit(Event::Error(UserError::Queue(format!(
                "dropped {dropped} queued export{plural} that needed this clip"
            ))));
        }
        if self.save() {
            let recordings = self.open.as_ref()?.folder.join(RECORDINGS_DIRNAME);
            let trash = recordings.join(TRASH_DIRNAME);
            let moved = std::fs::create_dir_all(&trash).and_then(|()| {
                rename_if_present(
                    &recordings.join(&clip.recording_filename),
                    &trash.join(&clip.recording_filename),
                )
            });
            // Left in `recordings/`, it is only an orphan: undo still works.
            if let Err(e) = moved {
                eprintln!("bus: couldn't trash {}: {e}", clip.recording_filename);
            }
        }
        Some(clip)
    }

    /// Moves the clip's recording back from `.trash`, then reinserts the clip
    /// at its position and saves. False if the file couldn't be moved back:
    /// the clip stays out, since the next open would shred a restored clip's
    /// file.
    fn restore_clip(&mut self, clip: &Clip) -> bool {
        let Some(open) = &mut self.open else {
            return false;
        };
        let recordings = open.folder.join(RECORDINGS_DIRNAME);
        let restored = rename_if_present(
            &trash_path(&open.folder, clip),
            &recordings.join(&clip.recording_filename),
        );
        if let Err(e) = restored {
            self.emit(Event::Error(UserError::Io(format!(
                "couldn't restore the clip's recording: {e}"
            ))));
            return false;
        }
        open.project.insert_clip(clip.clone());
        self.project_changed();
        true
    }

    /// Deletes the trashed files of clips the history has dropped.
    fn shred(&self, clips: Vec<Clip>) {
        let Some(open) = &self.open else {
            return;
        };
        for clip in clips {
            let path = trash_path(&open.folder, &clip);
            if let Err(e) = ignore_not_found(std::fs::remove_file(&path)) {
                eprintln!("bus: couldn't shred {}: {e}", path.display());
            }
        }
    }
}

/// Empties `recordings/.trash` in `folder`, as every open does: undo is
/// in-memory only, so nothing can restore what's there.
pub(super) fn empty_trash(folder: &Path) {
    let trash = folder.join(RECORDINGS_DIRNAME).join(TRASH_DIRNAME);
    if let Err(e) = ignore_not_found(std::fs::remove_dir_all(&trash)) {
        eprintln!("bus: couldn't empty {}: {e}", trash.display());
    }
}

/// Where a deleted clip's recording waits: `recordings/.trash/<file>`.
fn trash_path(folder: &Path, clip: &Clip) -> PathBuf {
    folder
        .join(RECORDINGS_DIRNAME)
        .join(TRASH_DIRNAME)
        .join(&clip.recording_filename)
}

/// Renames `from` to `to`, replacing it; a missing `from` is not an error.
fn rename_if_present(from: &Path, to: &Path) -> io::Result<()> {
    ignore_not_found(std::fs::rename(from, to))
}

fn ignore_not_found(result: io::Result<()>) -> io::Result<()> {
    match result {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        result => result,
    }
}
