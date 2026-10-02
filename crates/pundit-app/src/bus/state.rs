//! Where the app's own files live ([`AppFiles`]), and the one it reads and
//! writes itself: `$XDG_CONFIG_HOME/pundit/state.json`, holding the last
//! successfully opened project folder (spec D6), which speech model
//! transcription runs (Phase 10 S3), which pen the coach draws with, how big
//! the window was and how wide its two side columns are. **None is a
//! project's.** The model describes how fast this machine is, not the match,
//! the pen, the window and its columns are the coach's habit, and
//! `Preferences` lives
//! in `project.json`, where a new field is a format change that
//! [`store::read`](pundit_core::store::read)'s exact-version guard would
//! make every existing project unreadable for.
//!
//! Losing this file only costs the user a re-open and a re-pick, so every
//! failure here is logged and otherwise ignored.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use gstreamer::glib;
use pundit_media::WhisperModel;
use serde::{Deserialize, Serialize};

use crate::drawing::{Pen, PenWidth};

/// The app's own directory under whichever XDG base directory is in play.
pub(super) const APP_DIR: &str = "pundit";

/// How many projects are remembered (recents spec S3).
///
/// Eight, which is an estimate rather than a measurement: a popover row is two
/// lines, so eight stand about 320px over the player. The coach has three
/// projects and a season has twenty-odd, and a list you scroll is a picker —
/// which they already have.
///
/// It lives here, with [`AppFiles::push_recent_project`] and [`AppFiles::read`],
/// because those two are what enforce it. The popover only displays what it is
/// handed.
pub const RECENT_PROJECTS: usize = 8;
const FILE: &str = "state.json";

/// **Every field defaults**, and a file written by a later version keeps the
/// fields this one doesn't know only insofar as it rewrites the whole
/// document — it doesn't. A lost field costs a re-open or a re-pick.
///
/// **Two attributes carry the whole of BACKLOG #100, and a new field needs
/// neither.** `default` is on the **container**, so an absent key reads as
/// [`State::default()`]'s value for it — `deserialize_with` is not called for a
/// key that isn't there, and a field-level `default` beside it would be the
/// same rule written twice. [`lenient`] is on each field, so a value this build
/// can't read costs that field and **not the document**: before it, one bad
/// `panels` took the last project, the pen and the speech model with it.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct State {
    /// The projects the coach has had open, newest first (spec S1).
    ///
    /// **One field, not two.** [`AppFiles::last_project`] is this list's head,
    /// derived — never a second copy of it, which would be free to disagree.
    #[serde(deserialize_with = "lenient")]
    recent_projects: Vec<PathBuf>,
    /// **A read-only seed, and dated.** It is what a `state.json` written
    /// before `recentProjects` existed carries, read once in [`AppFiles::read`]
    /// to fill the list.
    ///
    /// `#[serde(skip_serializing)]` is what makes "read-only" true rather than
    /// merely intended: [`AppFiles::save`] re-serialises the whole `State`, so
    /// without it every setter would rewrite `lastProject` at its pre-upgrade
    /// value forever. With it the key self-cleans out of the file on the first
    /// save. **The named cost, traced rather than guessed:** a build predating
    /// this one has no `recent_projects` field, so its first setter rewrites the
    /// document and drops the **whole list**, not just the head — and since
    /// `lastProject` is already gone by then, a downgrade that only changes a
    /// pen leaves neither key. So the cost of a downgrade after any save is the
    /// list, permanently, and a re-pick of the project to restore. Nothing else
    /// is unrecoverable, and this file's header already prices a lost field at a
    /// re-open and a re-pick.
    ///
    /// Dated on `adopt_old_name`'s pattern: BACKLOG #109 deletes it once no
    /// installation predating this version is left to upgrade.
    #[serde(skip_serializing, deserialize_with = "lenient")]
    last_project: Option<PathBuf>,
    /// [`WhisperModel::label`], not the enum: the file is hand-readable, and
    /// a label this version doesn't know reads as the default rather than
    /// throwing the whole document away.
    #[serde(deserialize_with = "lenient")]
    whisper_model: Option<String>,
    /// [`Pen::label`], for the same reasons.
    #[serde(deserialize_with = "lenient")]
    pen: Option<String>,
    /// [`PenWidth::label`] (BACKLOG #116), for the same reasons again. One
    /// attribute rather than two, which is what moving `default` to the
    /// container bought.
    #[serde(deserialize_with = "lenient")]
    pen_width: Option<String>,
    #[serde(deserialize_with = "lenient")]
    window: Option<WindowSize>,
    /// No `Option`: [`PanelWidths`]'s own container default already fills a
    /// file that doesn't mention the panels, so a second "absent" state would
    /// mean the same thing twice.
    #[serde(deserialize_with = "lenient")]
    panels: PanelWidths,
    /// No `Option`, for `panels`' reason: [`Folds`]' own container default
    /// already fills a file that doesn't mention them.
    #[serde(deserialize_with = "lenient")]
    folds: Folds,
}

/// Any value this build can't read falls back to the field's default, so one
/// bad value costs that field and not the whole document (BACKLOG #100).
///
/// **`DeserializeOwned`, not `Deserialize<'de>`.** `&Value` is a deserializer
/// for a borrow of the local `value`, which is strictly shorter than the outer
/// `'de` — `Deserialize<'de>` does not supply it and the function does not
/// compile (E0597, *"argument requires that `value` is borrowed for `'de`"*).
///
/// **It moves the loss from the document to the field, and no further.** One
/// malformed *element* would still cost a whole list, because a `Vec` fails
/// whole — the bargain `bus/basket.rs` already strikes for its `pieces`, and
/// struck for the reason this file's own header gives: what is lost here is a
/// re-pick, never a format change. And it does nothing about a lost *update*:
/// every setter is `read` then [`AppFiles::save`] over the whole document, and
/// there are two [`AppFiles`] handles, so a bus-side write interleaving with a
/// UI-side one still loses a field. The write is a temp file and a rename, so
/// nothing tears.
fn lenient<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::de::DeserializeOwned + Default,
{
    let value = serde_json::Value::deserialize(d)?;
    if let Ok(parsed) = T::deserialize(&value) {
        return Ok(parsed);
    }
    eprintln!("bus: ignoring an unreadable state.json value: {value}");
    Ok(T::default())
}

/// The main window's size, in logical pixels.
///
/// `default` is on the **container**, so a **partial object** —
/// `{"window": {"width": 1600}}` — reads as 1600x960, keeping the width the
/// coach actually stored. **That is a finer grain than [`lenient`] reaches and
/// is why this attribute stays**: `lenient` rescues the rest of the *document*
/// from a bad `window`, but it hands back `None` and loses the 1600 with it.
/// Field-level `#[serde(default)]` would be the hazard `project.rs` names: it
/// resolves to `Default::default()`, i.e. a height of `0`, which `main.rs`
/// hands straight to `set_size` — so the hand-written [`Default`] impl below is
/// load-bearing, not decoration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

impl Default for WindowSize {
    /// The first launch's: roomy on a 1920x1080 laptop screen without
    /// filling it. The window's minimum is well under it, for a smaller
    /// screen.
    fn default() -> Self {
        WindowSize {
            width: 1600,
            height: 960,
        }
    }
}

/// How wide the coach dragged the two side columns, in logical pixels (spec W7).
///
/// One field holding both, because they are read and written together and a
/// file with one but not the other is a state nobody wants to reason about.
/// `default` is on the container for [`WindowSize`]'s reason and reaches exactly
/// as far: a partial `{"panels": {"sidebar": 400}}` keeps the 400, which
/// [`lenient`] would not. `"wide"`, `{"sidebar": -5}`, `{"sidebar": 1.5}` and
/// `null` each cost **this field alone** — they used to cost the document
/// (BACKLOG #100), and `lenient` is what changed that.
///
/// **Unlike [`WindowSize`], a field-level one here would be merely untidy**, and
/// the asymmetry is worth knowing. The layout bounds a stored `0` back up to
/// `sidebar-min` before anything is drawn — measured, stored `(0, 0)` at
/// 1600x960 lays out 240 / 280 / player 1068, the defaults to the pixel — so no
/// width a `Default::default()` could invent ever reaches a resize, where
/// `WindowSize`'s zero height goes straight to `set_size`. The container default
/// is here for the **document**, not for the column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PanelWidths {
    pub sidebar: u32,
    pub inspector: u32,
}

impl Default for PanelWidths {
    /// Today's fixed widths, which are also the minima: the panels grow, they
    /// do not shrink (spec W4 — 280 is the width the inspector's transcript row
    /// was fitted to).
    fn default() -> Self {
        PanelWidths {
            sidebar: 240,
            inspector: 280,
        }
    }
}

/// Which panel sections the coach has folded away (BACKLOG #113).
///
/// `default` is on the **container** for [`PanelWidths`]' reason, and the
/// derived `Default` is right here rather than hand-written: **every section
/// starts open**, which is what the app has always looked like and what a file
/// written before this feature means.
///
/// **Bools, which the format rules forbid in `project.json` and allow here.**
/// The hazard there is a field-level `#[serde(default)]` resolving to `false`
/// where `false` is not what an older file means; here the container fills from
/// `Default` and `false` — open — is exactly what it means.
///
/// One struct rather than three keys, so a new foldable section is one field and
/// the whole set reads and writes together.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Folds {
    pub match_panel: bool,
    pub project: bool,
    pub sources: bool,
}

/// Where the app's own files live — three things, which is why this is not
/// named after any one of them:
///
/// - `state.json`, which this type reads and writes through its own accessors;
///   `None` when there is no config directory at all (no `$XDG_CONFIG_HOME` and
///   no `$HOME`), in which case nothing is remembered.
/// - the app's other files beside it, found through [`AppFiles::sibling`] —
///   today just the basket's.
/// - the folder a basket's film is written into
///   ([`AppFiles::basket_dir`]), which is under the **user's videos**
///   directory: neither state nor configuration, and nobody's project.
#[derive(Debug, Clone)]
pub struct AppFiles {
    path: Option<PathBuf>,
    films: PathBuf,
}

impl AppFiles {
    /// `$XDG_CONFIG_HOME/pundit/state.json`, falling back to
    /// `~/.config/pundit/state.json`; films in `<XDG Videos>/pundit`.
    pub fn default_location() -> Self {
        AppFiles {
            path: config_dir(
                std::env::var_os("XDG_CONFIG_HOME"),
                std::env::var_os("HOME"),
            )
            .map(|dir| dir.join(APP_DIR).join(FILE)),
            films: films_dir(
                glib::user_special_dir(glib::UserDirectory::Videos),
                std::env::var_os("HOME"),
            ),
        }
    }

    /// The app's files under `config_dir` instead of the user's, for tests —
    /// **films included**, so no test writes a film into the coach's own
    /// videos folder.
    pub fn in_config_dir(config_dir: &Path) -> Self {
        AppFiles {
            path: Some(config_dir.join(APP_DIR).join(FILE)),
            films: config_dir.join("videos"),
        }
    }

    /// Another of the app's own files, beside `state.json`: the basket's
    /// (basket spec H1), which is deliberately **not** a key in this one.
    /// `None` when there is no config directory, where nothing is remembered.
    pub(super) fn sibling(&self, file: &str) -> Option<PathBuf> {
        Some(self.path.as_ref()?.with_file_name(file))
    }

    /// The folder a basket's film is written into (basket spec O1), created on
    /// demand by the run that writes one.
    pub fn basket_dir(&self) -> PathBuf {
        self.films.clone()
    }

    /// The projects the coach has had open, newest first (spec S1). An
    /// unreadable file reads as none of them.
    pub fn recent_projects(&self) -> Vec<PathBuf> {
        self.read().recent_projects
    }

    /// The remembered project folder, if any — **the head of
    /// [`AppFiles::recent_projects`]**, derived rather than stored.
    ///
    /// It survives as an accessor because two callers want exactly the head and
    /// nothing else: `Bus::restore_last_project`, and the New match flow's
    /// projects-folder tier in `main.rs`.
    pub fn last_project(&self) -> Option<PathBuf> {
        self.read().recent_projects.into_iter().next()
    }

    /// Puts `folder` at the head of the list, removing it from wherever else it
    /// was and dropping the oldest beyond [`RECENT_PROJECTS`].
    ///
    /// **This is the only writer**, which is why `set_last_project` was deleted
    /// rather than left with no callers: a second way to write one field, and
    /// nobody could say what it meant — write a one-element list, replace the
    /// head, or truncate the rest?
    ///
    /// **`==` is the whole de-duplication** (spec S4). `Bus::commit` stores
    /// `folder.canonicalize().unwrap_or(folder)` with `open_project` having
    /// made it absolute first, so equal paths are the same project and
    /// re-opening one moves it to the head. Where `canonicalize` failed — an
    /// EACCES on a component, a mount dropped between the read and the commit —
    /// a non-canonical path is stored and the list can hold two entries for one
    /// project. That costs one duplicate row and is not worth normalizing for.
    pub fn push_recent_project(&self, folder: &Path) {
        let mut state = self.read();
        state.recent_projects.retain(|p| p != folder);
        state.recent_projects.insert(0, folder.to_path_buf());
        state.recent_projects.truncate(RECENT_PROJECTS);
        self.save(&state);
    }

    /// Which speech model transcription runs (Phase 10 S3). A file that
    /// doesn't say, or says something this version doesn't know, reads as the
    /// default.
    pub fn whisper_model(&self) -> WhisperModel {
        self.read()
            .whisper_model
            .as_deref()
            .and_then(WhisperModel::from_label)
            .unwrap_or_default()
    }

    /// Remembers `model` for every project on this machine.
    pub fn set_whisper_model(&self, model: WhisperModel) {
        let mut state = self.read();
        state.whisper_model = Some(model.label().to_owned());
        self.save(&state);
    }

    /// The pen new strokes are drawn with. A file that doesn't say, or names
    /// a pen this version doesn't have, reads as the default.
    pub fn pen(&self) -> Pen {
        self.read()
            .pen
            .as_deref()
            .and_then(Pen::from_label)
            .unwrap_or_default()
    }

    /// Remembers `pen` for every project on this machine.
    pub fn set_pen(&self, pen: Pen) {
        let mut state = self.read();
        state.pen = Some(pen.label().to_owned());
        self.save(&state);
    }

    /// How thick the pen draws (BACKLOG #116). A file that doesn't say, or
    /// names a width this version doesn't have, reads as the default — which is
    /// the width every drawing before this feature was made at.
    pub fn pen_width(&self) -> PenWidth {
        self.read()
            .pen_width
            .as_deref()
            .and_then(PenWidth::from_label)
            .unwrap_or_default()
    }

    /// Remembers `width` for every project on this machine.
    pub fn set_pen_width(&self, width: PenWidth) {
        let mut state = self.read();
        state.pen_width = Some(width.label().to_owned());
        self.save(&state);
    }

    /// The window's size when it last closed. A file that doesn't say reads
    /// as the default.
    pub fn window_size(&self) -> WindowSize {
        self.read().window.unwrap_or_default()
    }

    /// Remembers `size` for every project on this machine.
    pub fn set_window_size(&self, size: WindowSize) {
        let mut state = self.read();
        state.window = Some(size);
        self.save(&state);
    }

    /// Which sections are folded away (BACKLOG #113). A file that doesn't say
    /// reads as all open, which is what the app looked like before it.
    pub fn folds(&self) -> Folds {
        self.read().folds
    }

    /// Remembers `folds` for every project on this machine — how the coach
    /// likes the window laid out, like the column widths beside it.
    pub fn set_folds(&self, folds: Folds) {
        let mut state = self.read();
        state.folds = folds;
        self.save(&state);
    }

    /// How wide the side columns were. A file that doesn't say reads as
    /// today's widths.
    pub fn panel_widths(&self) -> PanelWidths {
        self.read().panels
    }

    /// Remembers `widths` for every project on this machine. Called on the
    /// release of a drag and nowhere else (spec W5): a drag is 30 events a
    /// second and every setter here rewrites the whole document.
    pub fn set_panel_widths(&self, widths: PanelWidths) {
        let mut state = self.read();
        state.panels = widths;
        self.save(&state);
    }

    /// The file as it stands, defaulted where it is absent or unreadable.
    ///
    /// **Every write reads first**, so a field one setter doesn't know about
    /// survives the other's write: the document is rewritten whole.
    ///
    /// The `eprintln!` below is now reached only by a document [`lenient`]
    /// never sees a field of: `"{not json"`, `"hello"`, `5`, `true`, `null`, a
    /// truncated file and an empty one. **A JSON array is not one of them** —
    /// a derived struct deserialises from a sequence positionally, so `[1,2]`
    /// parses as all-defaults and logs through `lenient` twice instead.
    fn read(&self) -> State {
        let Some(path) = self.path.as_ref() else {
            return State::default();
        };
        let Ok(text) = std::fs::read_to_string(path) else {
            return State::default();
        };
        let mut state = serde_json::from_str::<State>(&text).unwrap_or_else(|e| {
            eprintln!("bus: ignoring unreadable {}: {e}", path.display());
            State::default()
        });
        // **The migration, and it belongs here rather than in the accessor.**
        // Every setter is `read` → mutate → `save` over the raw field. Seeded
        // in `recent_projects()` instead, that field is still empty on a
        // just-upgraded install while `lastProject` is skipped on serialize —
        // so the **first save of any setting**, a pen change included, writes
        // an empty `recentProjects` and no `lastProject`, and the pointer is
        // gone for good. Measured: moving the seeding there fails two tests,
        // one of them on a `set_pen`. Seeding here closes it in one place and
        // saves `push_recent_project` a second read.
        //
        // **The fallback is on the empty value, from any cause** — not on the
        // key being absent. A per-field read cannot tell a missing
        // `recentProjects` from a malformed one, so keying on absence would
        // lose the pointer for a document holding a good `lastProject` beside a
        // bad list: BACKLOG #100's symptom, reintroduced by its own fix.
        if state.recent_projects.is_empty() {
            // `take`, not `clone`: read once is exactly what this field is for.
            state.recent_projects = state.last_project.take().into_iter().collect();
        }
        // **Bounded on the way in as well as on the way out.** The cap used to
        // be `push_recent_project`'s alone, so a hand-edited or
        // downgrade-written file holding two hundred entries would have had all
        // two hundred read — two hundred `store::read`s on the UI thread when
        // the popover opened, against a budget of seven — and lowering
        // `RECENT_PROJECTS` would not have taken effect until the next push.
        // Here it is a property of the value rather than of one writer.
        state.recent_projects.truncate(RECENT_PROJECTS);
        state
    }

    fn save(&self, state: &State) {
        let Some(path) = &self.path else {
            return;
        };
        if let Err(e) = write(path, state) {
            eprintln!("bus: could not write {}: {e}", path.display());
        }
    }
}

fn write(path: &Path, state: &State) -> std::io::Result<()> {
    // Fails only for a non-UTF-8 path, which then simply isn't remembered.
    // Self-limiting: the rename never happens, so the file is left exactly as it
    // was and the next setter reads a clean one.
    let text = serde_json::to_string_pretty(state).map_err(std::io::Error::other)?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, path)
}

/// The XDG base-directory rule: the `$XDG_*_HOME` variable if it is set to an
/// absolute path, else `$HOME/<fallback>`.
fn base_dir(xdg: Option<OsString>, home: Option<OsString>, fallback: &str) -> Option<PathBuf> {
    xdg.map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| {
            home.map(PathBuf::from)
                .filter(|p| p.is_absolute())
                .map(|h| h.join(fallback))
        })
}

/// `$XDG_CONFIG_HOME`, else `~/.config`: where this file lives.
fn config_dir(xdg: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    base_dir(xdg, home, ".config")
}

/// Where a basket's film is written (basket spec O1): `<videos>/`[`APP_DIR`],
/// under the user's **videos** folder, because a film whose pieces come from
/// several matches belongs to no project and so can't go in one's `exports/`.
/// `videos` is that folder as glib reports it — `$HOME/Videos/pundit` when it
/// reports none (a machine whose `user-dirs.dirs` has no entry), and a relative
/// `pundit`, in the working directory, when there is no home either. The rule is
/// worth a test, and the environment is passed in as [`config_dir`] takes it.
fn films_dir(videos: Option<PathBuf>, home: Option<OsString>) -> PathBuf {
    videos
        .or_else(|| {
            home.map(PathBuf::from)
                .filter(|p| p.is_absolute())
                .map(|h| h.join("Videos"))
        })
        .unwrap_or_default()
        .join(APP_DIR)
}

/// `$XDG_CACHE_HOME`, else `~/.cache`: where the whisper models are looked
/// for and downloaded to (Phase 10 spec S3, Phase 11 S3). Hundreds of
/// megabytes of downloaded weights are a cache, not configuration.
pub(super) fn cache_dir(xdg: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    base_dir(xdg, home, ".cache")
}

/// The name the app went by before 0.8.0, and so the name on the directories
/// an existing installation left behind. Delete this and everything that reads
/// it once no 0.7.x installation is left to upgrade (`BACKLOG.md` #93).
const OLD_APP_DIR: &str = "coach-cuts";

/// Take over the directories the old name left behind, before anything reads
/// them. The cache directory holds a speech model that costs hundreds of
/// megabytes to fetch again, and the config directory holds the project the
/// coach last had open, the basket they filled and the pen they draw with — all
/// of it still theirs after a rename.
///
/// A rename, never a copy: the two names sit in the same base directory, so
/// this is one syscall and no chance of half a copy. Every failure is logged and
/// ignored, which costs the coach a re-open and (only if it was the cache) a
/// re-download, rather than a start that fails.
pub fn adopt_old_name() {
    let bases = [
        config_dir(
            std::env::var_os("XDG_CONFIG_HOME"),
            std::env::var_os("HOME"),
        ),
        cache_dir(std::env::var_os("XDG_CACHE_HOME"), std::env::var_os("HOME")),
    ];
    for base in bases.into_iter().flatten() {
        adopt_in(&base);
    }
}

/// [`adopt_old_name`] under one base directory, which is what a test can call.
///
/// **What stops it is a new directory with something in it, not a new directory**
/// — an empty one is adopted into. `mkdir` costs nothing and happens by
/// accident: a build run before this function existed, a write that created the
/// directory and then failed. Guarding on mere existence would forfeit the whole
/// carry-over, permanently and in silence, for an empty directory nobody meant
/// to create.
fn adopt_in(base: &Path) {
    let (old, new) = (base.join(OLD_APP_DIR), base.join(APP_DIR));
    let occupied = std::fs::read_dir(&new).is_ok_and(|mut dir| dir.next().is_some());
    if occupied || !old.is_dir() {
        return;
    }
    match std::fs::rename(&old, &new) {
        Ok(()) => eprintln!("state: adopted {} as {}", old.display(), new.display()),
        Err(err) => eprintln!(
            "state: could not adopt {} as {}: {err}",
            old.display(),
            new.display()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xdg_wins_when_absolute() {
        assert_eq!(
            config_dir(Some("/x/cfg".into()), Some("/home/u".into())),
            Some(PathBuf::from("/x/cfg"))
        );
    }

    #[test]
    fn relative_or_empty_xdg_falls_back_to_home() {
        for xdg in ["", "rel/cfg"] {
            assert_eq!(
                config_dir(Some(xdg.into()), Some("/home/u".into())),
                Some(PathBuf::from("/home/u/.config")),
                "{xdg:?}"
            );
        }
        assert_eq!(
            config_dir(None, Some("/home/u".into())),
            Some(PathBuf::from("/home/u/.config"))
        );
    }

    #[test]
    fn no_config_dir_without_xdg_or_home() {
        assert_eq!(config_dir(None, None), None);
    }

    /// The same rule, a different base directory: the model is a cache.
    #[test]
    fn the_cache_directory_follows_its_own_variable() {
        assert_eq!(
            cache_dir(Some("/x/cache".into()), Some("/home/u".into())),
            Some(PathBuf::from("/x/cache"))
        );
        assert_eq!(
            cache_dir(None, Some("/home/u".into())),
            Some(PathBuf::from("/home/u/.cache"))
        );
        assert_eq!(cache_dir(None, None), None);
    }

    /// Where the films go, and the fallbacks for a machine that doesn't say.
    #[test]
    fn the_films_folder_follows_the_videos_directory() {
        assert_eq!(
            films_dir(Some("/v".into()), Some("/home/u".into())),
            PathBuf::from("/v/pundit")
        );
        assert_eq!(
            films_dir(None, Some("/home/u".into())),
            PathBuf::from("/home/u/Videos/pundit")
        );
        // No home at all: a relative folder in the working directory, which is
        // at least somewhere the run can name.
        assert_eq!(films_dir(None, None), PathBuf::from("pundit"));
    }

    /// A test's films land under its own config directory, never in the
    /// coach's videos folder.
    #[test]
    fn a_test_state_file_keeps_its_films_beside_itself() {
        let dir = Path::new("/x/cfg");
        assert_eq!(
            AppFiles::in_config_dir(dir).basket_dir(),
            PathBuf::from("/x/cfg/videos")
        );
        assert_eq!(
            AppFiles::in_config_dir(dir).sibling("basket.json"),
            Some(PathBuf::from("/x/cfg/pundit/basket.json"))
        );
    }

    /// The list's three rules: a new path goes to the head, one already in it
    /// **moves** rather than duplicating, and the oldest beyond the cap drops.
    ///
    /// This replaces `remembers_and_forgets`, whose name promised the forgetting
    /// that spec E2 deliberately deleted; its first two assertions are the first
    /// two here.
    #[test]
    fn push_recent_project_heads_the_list_moves_a_repeat_and_caps_it() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppFiles::in_config_dir(dir.path());
        assert_eq!(state.last_project(), None);
        assert!(state.recent_projects().is_empty());

        state.push_recent_project(Path::new("/p/game"));
        assert_eq!(state.last_project(), Some(PathBuf::from("/p/game")));
        assert!(dir.path().join("pundit/state.json").is_file());

        state.push_recent_project(Path::new("/p/other"));
        assert_eq!(
            state.recent_projects(),
            [PathBuf::from("/p/other"), PathBuf::from("/p/game")],
            "newest first"
        );

        state.push_recent_project(Path::new("/p/game"));
        assert_eq!(
            state.recent_projects(),
            [PathBuf::from("/p/game"), PathBuf::from("/p/other")],
            "a repeat moves to the head, it does not duplicate"
        );

        // One past the cap: the oldest goes and the length holds. Derived from
        // the constant, so changing the cap cannot fail this test for a reason
        // that has nothing to do with the rule.
        let last = RECENT_PROJECTS - 1;
        for i in 0..RECENT_PROJECTS {
            state.push_recent_project(&PathBuf::from(format!("/p/{i}")));
        }
        let list = state.recent_projects();
        assert_eq!(list.len(), RECENT_PROJECTS);
        assert_eq!(
            list[0],
            PathBuf::from(format!("/p/{last}")),
            "the newest is the head"
        );
        assert!(
            !list.contains(&PathBuf::from("/p/other")),
            "the oldest dropped off the end"
        );
    }

    /// **The migration's three shapes** (spec S1). The fallback is on the
    /// **value**, so a document carrying only `lastProject` reads as a
    /// one-entry list.
    #[test]
    fn a_file_from_before_the_list_reads_its_last_project_as_the_list() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppFiles::in_config_dir(dir.path());
        std::fs::create_dir_all(dir.path().join(APP_DIR)).unwrap();
        let file = dir.path().join(APP_DIR).join(FILE);

        for (text, expected) in [
            // Only the old key: it becomes the list.
            (r#"{"lastProject":"/p/game"}"#, vec!["/p/game"]),
            // Both: the list wins, and the old key is ignored entirely.
            (
                r#"{"lastProject":"/p/game","recentProjects":["/p/a","/p/b"]}"#,
                vec!["/p/a", "/p/b"],
            ),
            // Neither: empty, and no phantom entry.
            (r#"{"pen":"blue"}"#, vec![]),
        ] {
            std::fs::write(&file, text).unwrap();
            let expected: Vec<PathBuf> = expected.iter().map(PathBuf::from).collect();
            assert_eq!(state.recent_projects(), expected, "{text}");
            assert_eq!(state.last_project(), expected.first().cloned(), "{text}");
        }
    }

    /// **The cross-case, and what sabotage proof 1 targets.** A malformed
    /// `recentProjects` beside a good `lastProject`: `lenient` defaults the list
    /// to empty, so the value-based fallback fires and the pointer survives.
    ///
    /// Keying the migration on the **key being absent** instead would lose it
    /// here — BACKLOG #100's own symptom, reintroduced by its fix.
    #[test]
    fn a_malformed_list_still_reads_the_last_project() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppFiles::in_config_dir(dir.path());
        std::fs::create_dir_all(dir.path().join(APP_DIR)).unwrap();
        let file = dir.path().join(APP_DIR).join(FILE);

        for bad in [r#""nonsense""#, "5", "null", r#"["/p/a",5]"#, "{}"] {
            let text = format!(r#"{{"lastProject":"/p/game","recentProjects":{bad}}}"#);
            std::fs::write(&file, &text).unwrap();
            assert_eq!(
                state.recent_projects(),
                [PathBuf::from("/p/game")],
                "{text}"
            );
            assert_eq!(
                state.last_project(),
                Some(PathBuf::from("/p/game")),
                "{text}"
            );
        }
    }

    /// **The seeded push, which is what the plan's trap 2 pins.** A
    /// just-upgraded document holds only `lastProject`; one switch must leave
    /// **both** projects in the list, newest first.
    ///
    /// **Measured by moving the seeding into the accessor: two tests fail, and
    /// the damage is worse than "the first switch".** Every setter is `read` →
    /// mutate → `save` over the raw field, which is empty on a just-upgraded
    /// install, while `lastProject` is skipped on serialize — so the *first save
    /// of any setting at all* writes an empty `recentProjects` and no
    /// `lastProject`, and the pointer is gone. Changing the pen would lose the
    /// coach's project list. This test fails on the switch;
    /// `the_old_key_self_cleans_out_of_the_file_on_the_first_save` fails on the
    /// pen, which is the one that shows how wide it is.
    #[test]
    fn the_first_switch_after_an_upgrade_keeps_the_project_before_it() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppFiles::in_config_dir(dir.path());
        std::fs::create_dir_all(dir.path().join(APP_DIR)).unwrap();
        std::fs::write(
            dir.path().join(APP_DIR).join(FILE),
            r#"{"lastProject":"/p/yesterday"}"#,
        )
        .unwrap();

        state.push_recent_project(Path::new("/p/today"));

        assert_eq!(
            state.recent_projects(),
            [PathBuf::from("/p/today"), PathBuf::from("/p/yesterday")],
            "the project before the upgrade is still in the list"
        );
    }

    /// `lastProject` is **read-only**: one save and the key is gone from the
    /// file, with the list carrying what it held. Without
    /// `#[serde(skip_serializing)]` every setter would rewrite it at its
    /// pre-upgrade value forever, and the doc comment calling it read-only
    /// would be a lie the next reader inherits.
    #[test]
    fn the_old_key_self_cleans_out_of_the_file_on_the_first_save() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppFiles::in_config_dir(dir.path());
        std::fs::create_dir_all(dir.path().join(APP_DIR)).unwrap();
        let file = dir.path().join(APP_DIR).join(FILE);
        std::fs::write(&file, r#"{"lastProject":"/p/game"}"#).unwrap();

        state.set_pen(Pen::Blue);

        let text = std::fs::read_to_string(&file).unwrap();
        assert!(!text.contains("lastProject"), "{text}");
        assert!(text.contains("recentProjects"), "{text}");
        assert_eq!(state.last_project(), Some(PathBuf::from("/p/game")));
    }

    /// The model is machine-wide and survives a restart, which is the whole
    /// point of it being here rather than in `project.json`.
    #[test]
    fn remembers_the_speech_model() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppFiles::in_config_dir(dir.path());
        assert_eq!(state.whisper_model(), WhisperModel::default());
        state.set_whisper_model(WhisperModel::Base);
        // A second handle on the same file: what a relaunch sees.
        assert_eq!(
            AppFiles::in_config_dir(dir.path()).whisper_model(),
            WhisperModel::Base
        );
    }

    /// The pen likewise: machine-wide, surviving a restart, red until picked.
    #[test]
    fn remembers_the_pen() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppFiles::in_config_dir(dir.path());
        assert_eq!(state.pen(), Pen::Red);
        state.set_pen(Pen::Yellow);
        assert_eq!(AppFiles::in_config_dir(dir.path()).pen(), Pen::Yellow);
    }

    /// The window likewise: machine-wide, surviving a restart, and the
    /// first launch's size until it has closed once.
    #[test]
    fn remembers_the_window_size() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppFiles::in_config_dir(dir.path());
        assert_eq!(state.window_size(), WindowSize::default());
        let size = WindowSize {
            width: 1400,
            height: 900,
        };
        state.set_window_size(size);
        assert_eq!(AppFiles::in_config_dir(dir.path()).window_size(), size);
    }

    /// **No setter may clobber another's field.** Each write rewrites the
    /// whole document, so one that didn't read first would forget the project
    /// every time the model, the pen or the window changed, and so on round.
    #[test]
    fn the_settings_are_independent() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppFiles::in_config_dir(dir.path());
        state.push_recent_project(Path::new("/p/game"));
        state.set_whisper_model(WhisperModel::Base);
        state.set_pen(Pen::Pink);
        let size = WindowSize {
            width: 1400,
            height: 900,
        };
        state.set_window_size(size);
        let widths = PanelWidths {
            sidebar: 400,
            inspector: 320,
        };
        state.set_panel_widths(widths);
        assert_eq!(state.last_project(), Some(PathBuf::from("/p/game")));
        assert_eq!(state.whisper_model(), WhisperModel::Base);
        // (Base, not the default Small: a clobbered model must be visible.)
        state.push_recent_project(Path::new("/p/other"));
        assert_eq!(state.pen(), Pen::Pink);
        state.set_pen(Pen::Blue);
        assert_eq!(state.last_project(), Some(PathBuf::from("/p/other")));
        assert_eq!(state.whisper_model(), WhisperModel::Base);
        assert_eq!(state.window_size(), size);
        assert_eq!(state.panel_widths(), widths);
        state.set_window_size(WindowSize {
            width: 1920,
            height: 1012,
        });
        assert_eq!(state.pen(), Pen::Blue);
        assert_eq!(state.last_project(), Some(PathBuf::from("/p/other")));
        assert_eq!(state.panel_widths(), widths);
    }

    /// A state file from before the picker, and one from a version that knows
    /// a model this one doesn't: both read as the default rather than as a
    /// failure.
    #[test]
    fn an_unknown_or_absent_model_reads_as_the_default() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppFiles::in_config_dir(dir.path());
        std::fs::create_dir_all(dir.path().join(APP_DIR)).unwrap();
        let file = dir.path().join(APP_DIR).join(FILE);
        for text in [
            r#"{"lastProject":"/p/game"}"#,
            r#"{"lastProject":"/p/game","whisperModel":"medium.en"}"#,
        ] {
            std::fs::write(&file, text).unwrap();
            assert_eq!(state.whisper_model(), WhisperModel::default(), "{text}");
            assert_eq!(
                state.last_project(),
                Some(PathBuf::from("/p/game")),
                "{text}"
            );
        }
    }

    /// The width is machine-wide like the pen, and an unknown one reads as the
    /// default — which is the width every drawing before #116 was made at, so a
    /// file this build can't read never thickens anybody's pen.
    #[test]
    fn remembers_the_pen_width_and_defaults_an_unknown_one() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppFiles::in_config_dir(dir.path());
        assert_eq!(state.pen_width(), PenWidth::Normal);

        state.set_pen_width(PenWidth::Thick);
        assert_eq!(
            AppFiles::in_config_dir(dir.path()).pen_width(),
            PenWidth::Thick,
            "what a relaunch sees"
        );

        std::fs::write(
            dir.path().join(APP_DIR).join(FILE),
            r#"{"pen":"blue","penWidth":"enormous"}"#,
        )
        .unwrap();
        assert_eq!(state.pen_width(), PenWidth::Normal);
        assert_eq!(state.pen(), Pen::Blue, "and it costs the pen nothing");
    }

    /// Folds are machine-wide like the widths, and a file from before them
    /// reads as every section open — which is what the app looked like.
    #[test]
    fn remembers_the_folds_and_defaults_to_every_section_open() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppFiles::in_config_dir(dir.path());
        assert_eq!(state.folds(), Folds::default());
        assert!(!Folds::default().match_panel, "open is the default");

        let folded = Folds {
            match_panel: true,
            project: true,
            sources: false,
        };
        state.set_folds(folded);
        assert_eq!(AppFiles::in_config_dir(dir.path()).folds(), folded);

        // And a bad value costs the folds alone (BACKLOG #100).
        std::fs::write(
            dir.path().join(APP_DIR).join(FILE),
            r#"{"pen":"blue","folds":"all"}"#,
        )
        .unwrap();
        assert_eq!(state.folds(), Folds::default());
        assert_eq!(state.pen(), Pen::Blue);
    }

    /// The panels likewise, and today's widths until one is dragged.
    #[test]
    fn remembers_the_panel_widths() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppFiles::in_config_dir(dir.path());
        assert_eq!(state.panel_widths(), PanelWidths::default());
        let widths = PanelWidths {
            sidebar: 400,
            inspector: 320,
        };
        state.set_panel_widths(widths);
        assert_eq!(AppFiles::in_config_dir(dir.path()).panel_widths(), widths);
    }

    /// Every state file written before this pass: no `panels` key at all, which
    /// reads as today's widths rather than as a pair of zero-width columns.
    #[test]
    fn a_file_from_before_the_panels_reads_the_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppFiles::in_config_dir(dir.path());
        std::fs::create_dir_all(dir.path().join(APP_DIR)).unwrap();
        std::fs::write(
            dir.path().join(APP_DIR).join(FILE),
            r#"{"lastProject":"/p/game","window":{"width":1400,"height":900}}"#,
        )
        .unwrap();
        assert_eq!(state.panel_widths(), PanelWidths::default());
        assert_eq!(state.last_project(), Some(PathBuf::from("/p/game")));
    }

    /// **`WindowSize`'s and `PanelWidths`' own container `#[serde(default)]` is
    /// what this pins, and it is finer-grained than `lenient`.** A half-written
    /// object — a hand edit, a build that wrote one field of two — keeps the
    /// field it *does* carry: `{"width": 1600}` reads as 1600x960, not as the
    /// default pair. `lenient` alone would give back `None` here and lose the
    /// 1600, so the two attributes are not redundant and neither replaces the
    /// other.
    ///
    /// The missing field reads as its **real** default — 960, not the `0` a
    /// field-level attribute would resolve to and `main.rs` would hand to
    /// `set_size`.
    #[test]
    fn a_partial_window_or_panels_object_costs_nothing_else() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppFiles::in_config_dir(dir.path());
        std::fs::create_dir_all(dir.path().join(APP_DIR)).unwrap();
        let file = dir.path().join(APP_DIR).join(FILE);
        let rest = r#""lastProject":"/p/game","pen":"blue","whisperModel":"base.en""#;
        for (text, window, panels) in [
            (
                format!(r#"{{{rest},"window":{{"width":1600}}}}"#),
                WindowSize {
                    width: 1600,
                    height: 960,
                },
                PanelWidths::default(),
            ),
            (
                format!(r#"{{{rest},"panels":{{"sidebar":400}}}}"#),
                WindowSize::default(),
                PanelWidths {
                    sidebar: 400,
                    inspector: 280,
                },
            ),
        ] {
            std::fs::write(&file, &text).unwrap();
            assert_eq!(
                state.last_project(),
                Some(PathBuf::from("/p/game")),
                "{text}"
            );
            assert_eq!(state.pen(), Pen::Blue, "{text}");
            assert_eq!(state.whisper_model(), WhisperModel::Base, "{text}");
            assert_eq!(state.window_size(), window, "{text}");
            assert_eq!(state.panel_widths(), panels, "{text}");
        }
    }

    /// **BACKLOG #100: one bad value costs that value and nothing else.**
    /// These are the five shapes #100 measured as still costing the whole
    /// document after the container defaults of #87 — so each one here also
    /// carries a good `lastProject`, `pen` and `whisperModel`, and the point of
    /// the test is that those three survive.
    ///
    /// `lenient` is what makes them survive. Without it on the named field the
    /// cross-case assertions below fail with `last_project` coming back `None`.
    #[test]
    fn one_unreadable_value_costs_that_field_alone() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppFiles::in_config_dir(dir.path());
        std::fs::create_dir_all(dir.path().join(APP_DIR)).unwrap();
        let file = dir.path().join(APP_DIR).join(FILE);
        let rest = r#""lastProject":"/p/game","pen":"blue","whisperModel":"base.en""#;
        // The good values are stated once; each row names **which** field it
        // breaks, so the "and the bad field defaulted" assertion is about the
        // field that document actually mentions. Asserting both every time left
        // one of the two vacuous on every row, which would not have noticed a
        // change that broke the other.
        let panels = |s: &AppFiles| s.panel_widths() != PanelWidths::default();
        let window = |s: &AppFiles| s.window_size() != WindowSize::default();
        for (bad, still_set) in [
            (r#""panels":"wide""#, &panels as &dyn Fn(&AppFiles) -> bool),
            (r#""panels":{"sidebar":-5}"#, &panels),
            (r#""panels":{"sidebar":1.5}"#, &panels),
            (r#""panels":null"#, &panels),
            (r#""window":{"height":-1}"#, &window),
        ] {
            let text = format!("{{{rest},{bad}}}");
            std::fs::write(&file, &text).unwrap();
            assert_eq!(
                state.last_project(),
                Some(PathBuf::from("/p/game")),
                "{text}"
            );
            assert_eq!(state.pen(), Pen::Blue, "{text}");
            assert_eq!(state.whisper_model(), WhisperModel::Base, "{text}");
            assert!(
                !still_set(&state),
                "the broken field should have defaulted: {text}"
            );
        }
    }

    /// A document `lenient` never sees a field of — not JSON at all — which is
    /// the class `read`'s own log still covers.
    #[test]
    fn corrupt_file_reads_as_none() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppFiles::in_config_dir(dir.path());
        std::fs::create_dir_all(dir.path().join(APP_DIR)).unwrap();
        std::fs::write(dir.path().join(APP_DIR).join(FILE), "{not json").unwrap();
        assert_eq!(state.last_project(), None);
    }

    /// The 0.8.0 rename: what the old name left behind is carried over, once.
    #[test]
    fn the_old_names_directory_is_adopted_with_what_is_in_it() {
        let base = tempfile::tempdir().unwrap();
        let old = base.path().join(OLD_APP_DIR);
        std::fs::create_dir_all(&old).unwrap();
        std::fs::write(old.join(FILE), r#"{"lastProject":"/p/game"}"#).unwrap();

        adopt_in(base.path());

        assert!(!old.exists(), "the old directory is gone, not copied");
        assert_eq!(
            AppFiles::in_config_dir(base.path()).last_project(),
            Some(PathBuf::from("/p/game"))
        );
    }

    /// An **empty** directory under the new name does not forfeit the
    /// carry-over: `mkdir` happens by accident, and the coach's speech model
    /// and last project must not depend on one. Without the emptiness test
    /// this fails — `rename(2)` itself would happily do it, so the guard is
    /// what this pins.
    #[test]
    fn an_empty_new_directory_is_adopted_into() {
        let base = tempfile::tempdir().unwrap();
        let old = base.path().join(OLD_APP_DIR);
        std::fs::create_dir_all(&old).unwrap();
        std::fs::write(old.join(FILE), r#"{"lastProject":"/p/game"}"#).unwrap();
        std::fs::create_dir_all(base.path().join(APP_DIR)).unwrap();

        adopt_in(base.path());

        assert_eq!(
            AppFiles::in_config_dir(base.path()).last_project(),
            Some(PathBuf::from("/p/game"))
        );
    }

    /// The other side of it: a new directory that **holds** something is this
    /// version's own, so it is left exactly as it is — which is what makes
    /// every run after the first a no-op — and nothing is invented where the
    /// old name never was.
    #[test]
    fn adopting_leaves_an_occupied_new_name_alone_and_invents_nothing() {
        let base = tempfile::tempdir().unwrap();
        adopt_in(base.path());
        assert!(!base.path().join(APP_DIR).exists());

        let (old, new) = (base.path().join(OLD_APP_DIR), base.path().join(APP_DIR));
        std::fs::create_dir_all(&old).unwrap();
        std::fs::create_dir_all(&new).unwrap();
        std::fs::write(old.join(FILE), "old").unwrap();
        std::fs::write(new.join(FILE), "new").unwrap();

        adopt_in(base.path());

        assert_eq!(std::fs::read_to_string(new.join(FILE)).unwrap(), "new");
        assert!(old.is_dir(), "the old one is left for the coach to delete");
    }
}
