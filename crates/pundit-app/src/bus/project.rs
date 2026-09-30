//! Project lifecycle (spec D6): read first, then commit folder and project
//! together. macOS set the folder before reading, so after a refused open the
//! next autosave wrote the old project over the file it had just refused.
//!
//! The project folder's own assets live here too: the avatar image, which is
//! copied in, named in `project.json`, and deleted with the project's copy
//! alone (avatar spec A2).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use pundit_core::project::{aspects_match, Project};
use pundit_core::scoreboard::ScoreboardConfig;
use pundit_core::store::{self, StoreError};
use pundit_media::{decode_still, probe, Probe};

use super::{Bus, Event, Open, Snapshot, UserError};

/// Deletes the project's copy of an avatar image. A copy that has already
/// gone is not news: the field is what says there is one, and it is on its
/// way out either way.
fn remove_avatar_file(folder: &Path, name: &str) {
    if let Err(e) = std::fs::remove_file(folder.join(name)) {
        if e.kind() != std::io::ErrorKind::NotFound {
            eprintln!("bus: could not delete the project's {name}: {e}");
        }
    }
}

impl Bus {
    /// Opens `folder`, creating a project if it exists but has no
    /// `project.json`. Any other failure changes nothing — not the file, not
    /// the open project. A folder that doesn't exist is an error: saving
    /// never creates one.
    pub(super) fn open_project(&mut self, folder: PathBuf) {
        let folder = match std::path::absolute(&folder) {
            Ok(folder) => folder,
            Err(e) => return self.emit(Event::Error(UserError::Io(e.to_string()))),
        };
        if !folder.is_dir() {
            return self.emit(Event::Error(UserError::Io(format!(
                "folder not found: {}",
                folder.display()
            ))));
        }
        match store::read(&folder) {
            Ok(project) => self.commit(folder, project),
            Err(StoreError::MissingProjectJson(_)) => {
                let name = folder
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "Untitled".into());
                let mut project = Project::new(name);
                match store::write(&folder, &mut project) {
                    Ok(()) => self.commit(folder, project),
                    Err(e) => self.emit(Event::Error(e.into())),
                }
            }
            Err(e) => self.emit(Event::Error(e.into())),
        }
    }

    /// Reopens the remembered project. Opens an **existing** project only:
    /// creating one would make `store::write` recreate a deleted or unmounted
    /// folder. On any failure the path is forgotten and the UI stays in its
    /// no-project state.
    pub(super) fn restore_last_project(&mut self) {
        let Some(folder) = self.files.last_project() else {
            return;
        };
        match store::read(&folder) {
            Ok(project) => self.commit(folder, project),
            Err(e) => {
                eprintln!("bus: not restoring last project {}: {e}", folder.display());
                self.files.set_last_project(None);
            }
        }
    }

    pub(super) fn rename_project(&mut self, name: String) {
        let name = name.trim();
        let Some(open) = &mut self.open else {
            return;
        };
        if name.is_empty() || name == open.project.name {
            return;
        }
        open.project.name = name.to_owned();
        self.project_changed();
    }

    /// Creates a project in `project_dir` from `videos`, in the order given,
    /// and opens it (spec C2).
    ///
    /// **One command rather than a sequence of the existing four, because the
    /// aspect gate fires between sources.** As `OpenProject` + `AddSource` × n,
    /// a second half whose shape differs is refused *after* the first has been
    /// probed, pushed, saved and published — leaving a named folder holding one
    /// of a game's two halves, and the folder name is the one thing this flow
    /// cannot correct afterwards. So: **nothing touches the disk until every
    /// video has been accepted**, which makes a refused Create leave the coach
    /// in the project he was in, with the sheet still up and his typing in it.
    ///
    /// Three things about it are easy to write down wrongly:
    ///
    /// - **The pre-disk gate is [`aspects_match`], not
    ///   [`Project::check_aspect`].** `check_aspect` gates a candidate against
    ///   a *stored* source and returns `Ok(())` when there is none, so gating
    ///   the pushes below would gate nothing at all on a project that does not
    ///   exist yet, and gating them *as well* would run one rule twice over the
    ///   same numbers — the second time after both `create_dir`s, which is too
    ///   late for the refusal this command exists for.
    /// - **An existing folder is refused only for its `project.json`; an empty
    ///   one is adopted.** That is what [`Bus::open_project`] already does, and
    ///   it is what lets the coach point this flow at the empty folder he made
    ///   by hand. Keyed on `create_dir`'s `AlreadyExists` instead, it would
    ///   recreate the very thing the flow exists to fix.
    /// - **`create_dir`, never `create_dir_all`.** At most two directories are
    ///   ever made — the projects folder's leaf and the match folder under it —
    ///   so a typo in the hand-editable path (spec W3) leaves one stray
    ///   directory under a folder that already existed rather than a tree.
    ///
    /// **Nothing is rolled back** if a step after the directories fails (spec
    /// C3) — the canonicalize, a `SourceRef` whose path is not UTF-8, or
    /// `store::write`: the folder is then an empty one, which the next Create
    /// adopts. The two `remove_dir`s
    /// a draft had were *unreliable rather than impossible* — `store::write`
    /// creates `recordings/` first and writes `.project.json.tmp` before
    /// renaming it, so a failure while serializing rolls back cleanly while
    /// `ENOSPC`, `EIO` or a dropped mount leaves the temp file and the folder
    /// removal returns `ENOTEMPTY`. A rollback that covers the cheap failures
    /// and not the expensive ones is worse than none. Creating a project is not
    /// an undo step either, as opening one is not: [`Bus::commit`] clears the
    /// history.
    ///
    /// **Every refusal is a modal, and every refusal of its own is a
    /// [`UserError::Io`] naming the file or path at fault.** The coach picked
    /// several videos in one dialog and is looking at a sheet waiting for an
    /// answer, so "the file has no video stream" has to say *which* file — which
    /// neither `ProbeError` nor [`UserError::AspectMismatch`] can, and whose
    /// "the project's other videos" is not even true here, there being no
    /// project yet. Those two stay what `add_source` and relink raise, where
    /// both are true. The two refusals this borrows are not `Io`:
    /// [`Bus::refuse_if_busy`]'s `CantExport` and `store::write`'s own error.
    /// Both are modal, which is what the reasoning above needs, and the second
    /// is wrapped here so that it names its folder too.
    pub(super) fn new_match(
        &mut self,
        project_dir: PathBuf,
        scoreboard: ScoreboardConfig,
        videos: Vec<PathBuf>,
    ) {
        match self.built_new_match(&project_dir, &scoreboard, &videos) {
            Ok((folder, project)) => self.commit(folder, project),
            Err(e) => self.emit(Event::Error(e)),
        }
    }

    /// [`Bus::new_match`]'s steps 1 to 7, so that every refusal is one `?` and
    /// the commit is the only thing left to do on success. Takes `&self`: it
    /// writes to the disk and nothing to the bus.
    fn built_new_match(
        &self,
        project_dir: &Path,
        scoreboard: &ScoreboardConfig,
        videos: &[PathBuf],
    ) -> Result<(PathBuf, Project), UserError> {
        let io = |e: std::io::Error, path: &Path| UserError::Io(format!("{}: {e}", path.display()));

        // 1. An export or a preview must not have the project swapped
        // underneath it — and the export sheet is meant to be closed while a
        // run continues, so `New match…` is clickable mid-export.
        self.refuse_if_busy()?;

        if !project_dir.is_absolute() {
            return Err(UserError::Io(format!(
                "not a full path: {}",
                project_dir.display()
            )));
        }
        // `file_name` is `None` for `/` and for a path ending in `..`, which
        // is what this rejects. It is **not** the "exactly one
        // `Component::Normal`" check — `a/b` and `a/` both have one, since
        // `Components` discards a trailing separator. That check is
        // `new_match::valid_folder_name`'s, upstream; the weaker one here still
        // cannot make more than two directories, because only `create_dir` is
        // used and only twice.
        if project_dir.file_name().is_none() {
            return Err(UserError::Io(format!(
                "{} doesn't name a folder to create",
                project_dir.display()
            )));
        }
        let Some(projects_dir) = project_dir.parent() else {
            return Err(UserError::Io(format!(
                "{} has no folder to go in",
                project_dir.display()
            )));
        };
        // W3: the projects folder is hand-editable, so it may not exist yet —
        // but its parent must, because only its leaf is ever created.
        if !projects_dir.is_dir() && !projects_dir.parent().is_some_and(Path::is_dir) {
            return Err(UserError::Io(format!(
                "{} can't be created: the folder it would go in doesn't exist",
                projects_dir.display()
            )));
        }
        if videos.is_empty() {
            return Err(UserError::Io("pick the game's video files first".into()));
        }
        // The one place that decides both whether a scoreboard may be stored and
        // what a project carrying it is called, so that writing
        // `scoreboard: Some(config)` onto a fresh project can neither walk
        // around `Bus::set_scoreboard`'s guard nor invent a second spelling of
        // `<Home> v <Away>`. A modal here, where `set_scoreboard` makes it a
        // notice: this one lands behind the sheet's scrim otherwise, and the
        // coach is waiting on an answer.
        let name = super::scoreboard::storable(scoreboard)
            .map_err(|reason| UserError::Io(reason.into()))?;

        // 2. Probe every video, each gated against the first — the same
        // reference `check_aspect` takes, and the same rule.
        let mut probed: Vec<(&Path, Probe)> = Vec::with_capacity(videos.len());
        for path in videos {
            let this =
                probe(path).map_err(|e| UserError::Io(format!("{}: {e}", path.display())))?;
            if let Some(&(first, reference)) = probed.first() {
                if !aspects_match(reference.display_aspect, this.display_aspect) {
                    return Err(UserError::Io(format!(
                        "{} is {:.3}:1 but {} is {:.3}:1 — a match's videos all have to be \
                         the same shape",
                        first.display(),
                        reference.display_aspect,
                        path.display(),
                        this.display_aspect,
                    )));
                }
            }
            probed.push((path.as_path(), this));
        }

        // 3. The projects folder's leaf, and only if it is missing.
        if !projects_dir.is_dir() {
            std::fs::create_dir(projects_dir).map_err(|e| io(e, projects_dir))?;
        }
        // 4. The match folder. An empty one is adopted; only a `project.json`
        // inside it refuses.
        match std::fs::create_dir(project_dir) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                // `create_dir` says `AlreadyExists` for a regular file of the
                // same name too, and the `project.json` probe below would then
                // find nothing and leave `store::write` to fail with a
                // confusing `ENOTDIR`. One `is_dir` on a branch that is
                // reading the filesystem anyway, as `open_project` guards it.
                if !project_dir.is_dir() {
                    return Err(UserError::Io(format!(
                        "{} is a file, not a folder",
                        project_dir.display()
                    )));
                }
                if project_dir.join(store::PROJECT_FILENAME).exists() {
                    return Err(UserError::Io(format!(
                        "{} already holds a project — open that instead",
                        project_dir.display()
                    )));
                }
            }
            Err(e) => return Err(io(e, project_dir)),
        }

        // 5. Canonical on both sides, because the kernel resolves `..`
        // physically and a cloud-sync mount may be reached through a symlink.
        let folder = project_dir.canonicalize().map_err(|e| io(e, project_dir))?;

        // 6. The project, in memory. Every aspect has already been gated.
        let mut project = Project::new(name);
        project.scoreboard = Some(scoreboard.clone());
        for (path, this) in probed {
            project
                .source_videos
                .push(super::sources::source_ref(&folder, path, this)?);
        }

        // 7. `recordings/`, then `project.json` through a temp file and a
        // rename. Named, like every other refusal here: `StoreError` carries the
        // file name it could not write but not the folder, so a read-only match
        // folder reached the coach as a bare "Permission denied".
        store::write(&folder, &mut project)
            .map_err(|e| UserError::Io(format!("{}: {e}", folder.display())))?;
        Ok((folder, project))
    }

    /// Makes `project` in `folder` the open project and resets everything
    /// tied to the previous one.
    fn commit(&mut self, folder: PathBuf, project: Project) {
        // The folder exists now (it was read or just written). Canonical, so
        // relative source paths computed against it resolve the way the
        // kernel resolves `..`.
        let folder = folder.canonicalize().unwrap_or(folder);
        self.files.set_last_project(Some(&folder));

        // Nothing of the previous project survives: not its requests, its
        // skip burst, nor its frame.
        self.unload();
        // Undo is in-memory only, so the trash it held is unreachable now
        // (Phase 3 spec C4). That includes the previous project's: its
        // history is being cleared, so its trash can never be restored.
        self.history.clear();
        if let Some(open) = &self.open {
            super::clips::empty_trash(&open.folder);
        }
        super::clips::empty_trash(&folder);
        self.player.set_volume(project.preferences.scan_volume);
        self.current = 0;
        self.open = Some(Open { folder, project });
        self.refresh_missing();
        if let Some(snapshot) = self.snapshot() {
            self.emit(Event::ProjectOpened(snapshot));
        }
        // The transcription queue doesn't survive an open either, for the
        // same reason the history doesn't: it holds ids of another project's
        // clips, and a job left running would write its words into a project
        // that is no longer open (Phase 10 spec S5). **After** the
        // `ProjectOpened` it belongs to, so the UI hears it as this project's
        // state rather than the last one's.
        self.reset_transcription();
        self.ensure_loaded(0.0);
    }

    /// Makes `path` the project's avatar (spec A2): decode, copy, save.
    ///
    /// Decoding first is the whole validation — pixels out of
    /// [`decode_still`] are proof the export will get pixels too — and it
    /// happens **before** anything is copied, so a refused pick leaves the
    /// project exactly as it was. The copy goes through a temp file and a
    /// rename in the same directory, as `store::write` writes `project.json`:
    /// a copy cut short leaves no avatar.
    pub(super) fn set_avatar(&mut self, path: PathBuf) {
        let Some(open) = &self.open else {
            return eprintln!("bus: SetAvatar with no project open");
        };
        if let Err(e) = decode_still(&path) {
            return self.emit(Event::Error(UserError::Avatar(e)));
        }
        // The stored name keeps the original extension, so the file opens in
        // a file manager as what it is; the decoder sniffs regardless, which
        // is why a file without one needs no guess.
        let name = match path.extension().and_then(|e| e.to_str()) {
            Some(ext) => format!("avatar.{}", ext.to_ascii_lowercase()),
            None => "avatar".to_owned(),
        };
        let folder = open.folder.clone();
        let temp = folder.join(".avatar.tmp");
        if let Err(e) =
            std::fs::copy(&path, &temp).and_then(|_| std::fs::rename(&temp, folder.join(&name)))
        {
            let _ = std::fs::remove_file(&temp);
            return self.emit(Event::Error(UserError::Io(format!(
                "the image couldn't be copied into the project: {e}"
            ))));
        }
        let Some(open) = &mut self.open else { return };
        // Exactly the file the project named, never a `avatar.*` glob over a
        // folder the coach can also put files in.
        if let Some(old) = open.project.avatar.take().filter(|old| *old != name) {
            remove_avatar_file(&folder, &old);
        }
        open.project.avatar = Some(name);
        self.project_changed();
    }

    /// Drops the avatar: the field, then the project's **copy** of the image.
    /// The coach's original is theirs.
    pub(super) fn clear_avatar(&mut self) {
        let Some(open) = &mut self.open else {
            return eprintln!("bus: ClearAvatar with no project open");
        };
        let Some(old) = open.project.avatar.take() else {
            return;
        };
        let folder = open.folder.clone();
        remove_avatar_file(&folder, &old);
        self.project_changed();
    }

    /// Saves the open project after a mutation and publishes the snapshot. A
    /// failed save is reported; the in-memory change stands, and the next
    /// successful save carries it.
    pub(super) fn project_changed(&mut self) {
        self.save();
        self.publish_project();
    }

    /// [`Bus::project_changed`]'s save, for a caller with more to do before
    /// it publishes. Returns whether it succeeded.
    pub(super) fn save(&mut self) -> bool {
        let Some(open) = &mut self.open else {
            return false;
        };
        match store::write(&open.folder, &mut open.project) {
            Ok(()) => true,
            Err(e) => {
                self.emit(Event::Error(e.into()));
                false
            }
        }
    }

    /// Publishes the open project, with the cached missing flags.
    pub(super) fn publish_project(&self) {
        if let Some(snapshot) = self.snapshot() {
            self.emit(Event::ProjectChanged(snapshot));
        }
    }

    fn snapshot(&self) -> Option<Snapshot> {
        let open = self.open.as_ref()?;
        Some(Snapshot {
            project: Arc::new(open.project.clone()),
            folder: open.folder.clone(),
            missing: self.missing.clone(),
        })
    }
}
