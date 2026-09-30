//! What the New match sheet opens with (new match spec I, W): where projects
//! go, the date, the opponent, and the scoreboard a neighbouring project
//! lends.
//!
//! **The rules live here, not in `main.rs`**, which is wiring and has no test
//! module at all — `fit`, `match_panel`, `drawing` and `zoom_input` are this
//! crate's pattern of "a testable rule is a `pub mod` with its tests beside
//! it". **And the environment is passed in**, exactly as `bus::state`'s
//! `films_dir(videos, home)` and `config_dir(xdg, home)` take theirs: a home
//! directory, a last project and a year bound are arguments, so every rule
//! below is a test with no `$HOME`, no `state.json` and no clock in it.
//!
//! **Nothing here calls `probe`** (spec S4). It blocks for up to ten seconds
//! per file and this runs on the UI thread while the sheet opens; the bus
//! probes, once, during Create. The whole cost of opening the sheet is one
//! `read_dir`, at most four `is_dir` calls, one `metadata` and one read of a
//! file a few kilobytes long.

use std::path::{Path, PathBuf};

use pundit_core::metadata::{CalendarDate, APP_NAME};
use pundit_core::naming;
use pundit_core::scoreboard::ScoreboardConfig;
use pundit_core::store;

use crate::match_panel::blank_config;

/// How far up from the match folder [`projects_dir_for`] looks for an
/// app-named directory: four candidates, the match folder itself being the
/// first (spec W1's depth table).
const WALK_CANDIDATES: usize = 4;

/// Where the projects folder was found, and so what the sheet says about it.
///
/// **An enum rather than a bare `PathBuf`**, because the three tiers are not
/// equally sure of themselves and the sheet says so: only the last carries a
/// provenance line (spec S1), and a test names the tier rather than comparing
/// paths it would have had to build the same way the code did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectsDir {
    /// Spec W1: an existing `<candidate>/pundit` at or above the footage. The
    /// decisive answer, and the coach's own pattern.
    Found(PathBuf),
    /// Spec W2: the folder of the project last opened is *in* a projects
    /// folder, so its parent is one. A fact, not an inference — **which is why
    /// it is checked**: `last_project` is a stored path with no existence
    /// guarantee (`RestoreLastProject` says "if it still exists" for the same
    /// reason), and an unchecked one would silently re-create a projects folder
    /// the coach had deleted, with no line saying it had proposed a directory
    /// that is not there. A stale one falls through to [`ProjectsDir::Proposed`],
    /// which carries that line.
    BesideLastProject(PathBuf),
    /// Spec W2's floor: an app-named directory **beside the footage**, which
    /// does not exist yet. The one tier that proposes something the coach has
    /// never seen, and so the only one that carries a line.
    Proposed(PathBuf),
}

impl ProjectsDir {
    pub fn path(&self) -> &Path {
        match self {
            ProjectsDir::Found(p)
            | ProjectsDir::BesideLastProject(p)
            | ProjectsDir::Proposed(p) => p,
        }
    }

    /// The one line under the field, or none where the answer is strong: a
    /// hint on every field is a hint on none (spec I0).
    pub fn provenance(&self) -> Option<&'static str> {
        match self {
            ProjectsDir::Found(_) | ProjectsDir::BesideLastProject(_) => None,
            ProjectsDir::Proposed(_) => {
                Some("No projects folder found nearby — choose where projects go.")
            }
        }
    }
}

/// Where this match's project folder should go (spec W1, W2).
///
/// **Walks up from `match_folder` for an existing `<candidate>/pundit`**, four
/// candidates — the match folder itself, its parent, and two more — stopping at
/// the filesystem root or at `home` **inclusive**, never above it. The first hit
/// wins. That is the coach's own pattern ("a `pundit` folder and then projects
/// inside it") and it finds the right answer at depth 2 in both of their trees,
/// which have different shapes.
///
/// **The name is [`APP_NAME`], never the literal `"pundit"`**: `bus::state`'s
/// `APP_DIR` is the same string but private to that module, and `CLAUDE.md` is
/// explicit that the name is read from the constant rather than spelled again.
///
/// With no such directory above the footage, `last_project`'s **parent** — a
/// directory that demonstrably held a project a moment ago — and with no last
/// project either, `<match folder's parent>/pundit`, which does not exist yet.
/// **Not the bare parent:** that would propose projects as *siblings* of the
/// match folder, the opposite of the pattern this flow exists to keep.
///
/// The filesystem is reached only through `is_dir`.
pub fn projects_dir_for(
    match_folder: &Path,
    home: Option<&Path>,
    last_project: Option<&Path>,
) -> ProjectsDir {
    let mut candidate = Some(match_folder);
    for _ in 0..WALK_CANDIDATES {
        let Some(dir) = candidate else { break };
        let named = dir.join(APP_NAME);
        if named.is_dir() {
            return ProjectsDir::Found(named);
        }
        // Home is a candidate but nothing above it is: a projects folder found
        // in `/home` or `/` belongs to somebody else.
        if home.is_some_and(|home| dir == home) {
            break;
        }
        candidate = dir.parent();
    }
    if let Some(parent) = last_project.and_then(Path::parent).filter(|p| p.is_dir()) {
        return ProjectsDir::BesideLastProject(parent.to_owned());
    }
    ProjectsDir::Proposed(match_folder.parent().unwrap_or(match_folder).join(APP_NAME))
}

/// The scoreboard the newest neighbouring project lends (spec I5).
///
/// One `read_dir` of `projects_dir`, its subfolders **by the date in their
/// names, descending**, and the first that has a scoreboard to lend. A file that
/// is unreadable, legacy or too new is passed over in silence: this is a
/// prefill, not an operation.
///
/// **"The first that has one", not "the first that reads".** Spec I5 says the
/// latter, and it is the weaker reading of its own sentence: a project made by
/// pointing `Open Project…` at a folder carries `scoreboard: None` until
/// `Set up teams…` is used, so the newest thing in the folder is quite often a
/// project with nothing to lend — and stopping there would hand back
/// [`blank_config`] while the match before it had exactly the kit and format
/// this tier exists to pass on. Looking past it is also one pass rather than
/// two, since the config comes out of the same read that accepted the file.
///
/// **The names are not substituted here.** They are the sheet's, typed after
/// this runs, and every reader of the result overwrites both — so doing it here
/// was work that could only ever be undone.
///
/// **Recency is the folder name's date, and nothing is stat-ed at all.**
/// `project.json` is rewritten on every edit, so its mtime means "last opened",
/// not "last match" — a coach who reopens last season's game to re-export one
/// clip would otherwise seed the next match from it. And because every folder
/// this flow creates is named `YYYY-MM-DD-…`, sorting those names descending
/// **is** recency order. A folder whose name carries no date sorts **below**
/// every dated one, by name descending; the cost is one degraded prefill in a
/// folder where no project is dated, which is a prefill either way.
///
/// A missing or unreadable projects folder gives [`blank_config`] rather than
/// an error: [`ProjectsDir::Proposed`] names a directory that does not exist
/// yet, and `blank_config` is what the setup sheet already opens with on a
/// project that has no scoreboard.
///
/// What is inherited is a **whole valid config** — the previous match's kit in
/// each slot and the format the coach plays — so only the two names are
/// replaced and `Set up teams…` corrects the rest.
pub fn seed_scoreboard(projects_dir: &Path, max_year: i32) -> ScoreboardConfig {
    lent_scoreboard(projects_dir, max_year).unwrap_or_else(blank_config)
}

/// A folder beside the one being created, as [`lent_scoreboard`] orders them.
struct Neighbour {
    /// The date in its name as a comparable tuple, because `Option<T>` orders
    /// `None` below every `Some` — which, sorted descending, *is* "dated before
    /// undated".
    date: Option<(i32, u32, u32)>,
    name: String,
    folder: PathBuf,
}

/// The neighbouring project folders, newest first, and the first scoreboard one
/// of them has to lend.
fn lent_scoreboard(projects_dir: &Path, max_year: i32) -> Option<ScoreboardConfig> {
    let mut neighbours: Vec<Neighbour> = std::fs::read_dir(projects_dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            Neighbour {
                date: naming::parse_date_in(&name, max_year)
                    .map(|d: CalendarDate| (d.year, d.month, d.day)),
                name,
                folder: e.path(),
            }
        })
        .collect();
    // Dated before undated, then by date and by name, both descending.
    neighbours.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| b.name.cmp(&a.name)));
    neighbours
        .into_iter()
        .find_map(|n| store::read(&n.folder).ok()?.scoreboard)
}

/// What the New match sheet cannot show, and what Create needs: the kit and
/// format a neighbour lent, and the videos in the order they will be stored.
///
/// **The paths are held here rather than as a `[string]` on the window**, since
/// `to_string_lossy` would corrupt one that is not UTF-8 and the sheet needs only
/// the file names to show. One struct because the two are written together, at
/// prefill, and read together, at Create.
#[derive(Debug, Clone, PartialEq)]
pub struct Draft {
    pub scoreboard: ScoreboardConfig,
    pub videos: Vec<PathBuf>,
}

/// A draft with nothing in it yet. Hand-written, because `ScoreboardConfig` has
/// no `Default` on purpose — [`blank_config`] is the one this app means by an
/// empty scoreboard, and it is what the sheet opens with when no neighbour has
/// one to lend.
impl Default for Draft {
    fn default() -> Self {
        Draft {
            scoreboard: blank_config(),
            videos: Vec::new(),
        }
    }
}

/// Everything the New match sheet opens with, derived from the videos the coach
/// just picked.
#[derive(Debug, Clone, PartialEq)]
pub struct Prefill {
    /// The deepest common ancestor of the chosen files (spec I1) — in both of
    /// the coach's trees, the directory they are all in.
    pub match_folder: PathBuf,
    /// Where projects go, and how sure of it we are.
    pub projects_dir: ProjectsDir,
    /// `YYYY-MM-DD`, or empty where no date was found. There is **no date
    /// field** (spec N1): this goes into the folder name, where its consequence
    /// is visible.
    pub date: String,
    /// The coach's own club goes in `home`, so the guess prefills `away` (spec
    /// S3, the coach's own decision) — and ⇄ is one click either way.
    pub away: String,
    /// The kit and the format a neighbour lent. **Not the names** — those are
    /// the sheet's, typed after this runs, and the bus builds the project's own
    /// name from them.
    pub scoreboard: ScoreboardConfig,
    /// The videos, in the order the command will store them.
    pub videos: Vec<PathBuf>,
}

/// `<date>-<home>-<away>` as one slug (spec N1).
///
/// **The joined string is slugged, not each part separately**, so the `-`
/// collapse absorbs a blank team name or an absent date and the 64-byte cut
/// applies to the whole name.
pub fn folder_name(date: &str, home: &str, away: &str) -> String {
    naming::folder_slug(&format!("{date}-{home}-{away}"))
}

/// Whether a team name is one a project can be stored with: non-blank after
/// trimming, which is `bus::scoreboard::storable`'s test and so **the same call
/// Create makes** (spec S5).
///
/// It is here rather than written into the sheet because Slint has no test path
/// in this crate — and an untrimmed `!= ""` there is exactly the bug this
/// replaced: a field of one space read good, enabled Create, and sent nothing.
pub fn team_named(name: &str) -> bool {
    !name.trim().is_empty()
}

/// What the sheet opens with, for `videos` as the picker handed them over.
///
/// `home` is the coach's own club and is left to them: nothing in a file or a
/// folder name says which team that is. `now` is this year, from which the date
/// bound is `now + 1` — an argument, because core has no clock and neither does
/// this rule's test.
pub fn prefill(
    mut videos: Vec<PathBuf>,
    home: Option<&Path>,
    last_project: Option<&Path>,
    this_year: i32,
) -> Prefill {
    naming::order_videos(&mut videos);
    let max_year = this_year + 1;
    let match_folder = common_ancestor(&videos);
    let projects_dir = projects_dir_for(&match_folder, home, last_project);
    let away = match_folder
        .file_name()
        .and_then(|n| naming::opponent_from(&n.to_string_lossy()))
        .unwrap_or_default();
    let date = match_date(&videos, &match_folder, max_year)
        .map(|d| format!("{:04}-{:02}-{:02}", d.year, d.month, d.day))
        .unwrap_or_default();
    let scoreboard = seed_scoreboard(projects_dir.path(), max_year);
    Prefill {
        match_folder,
        projects_dir,
        date,
        away,
        scoreboard,
        videos,
    }
}

/// The deepest directory every one of `videos` is under (spec I1). With every
/// file in one directory — both of the coach's trees, every time — that is that
/// directory; with files from two, their shared parent, where the opponent
/// inference will usually find nothing, which is the correct outcome: a blank
/// half of a folder name rather than a wrong one.
fn common_ancestor(videos: &[PathBuf]) -> PathBuf {
    let mut dirs = videos.iter().filter_map(|p| p.parent());
    let Some(first) = dirs.next() else {
        return PathBuf::new();
    };
    dirs.fold(first.to_owned(), |common, dir| {
        let take = common
            .components()
            .zip(dir.components())
            .take_while(|(a, b)| a == b)
            .count();
        common.components().take(take).collect()
    })
}

/// The match's date, in spec I2's order: a full date agreed on by every chosen
/// file's name, else one in the match folder's name, else the **first-in-order**
/// file's mtime, else none.
///
/// **A bare year is never a date** and neither is a six-digit run — the two
/// traps in the coach's own trees, a team folder named for a birth year and a
/// match folder starting with an age-group code. That rule lives in
/// [`naming::parse_date_in`]; this is only the order the sources are tried in.
///
/// **The first in `order_videos`' order, not the earliest mtime**, so there is
/// one rule for "the game's date" rather than two: `bus::export`'s tagging
/// already reads `sources.first()`. Mtime is third because these are downloads
/// — in the coach's trees the footage mtime runs one and two days after the date
/// their own folder name gives. Close enough to prefill, not close enough to
/// trust.
fn match_date(videos: &[PathBuf], match_folder: &Path, max_year: i32) -> Option<CalendarDate> {
    let mut in_names = videos.iter().map(|path| {
        path.file_name()
            .and_then(|n| naming::parse_date_in(&n.to_string_lossy(), max_year))
    });
    if let Some(first) = in_names.next().flatten() {
        if in_names.all(|d| d == Some(first)) {
            return Some(first);
        }
    }
    if let Some(date) = match_folder
        .file_name()
        .and_then(|n| naming::parse_date_in(&n.to_string_lossy(), max_year))
    {
        return Some(date);
    }
    mtime_date(videos.first()?)
}

/// `path`'s modification time as a local calendar date. The one thing in this
/// module that reads a file rather than a name, which is why it is here and not
/// in core.
fn mtime_date(path: &Path) -> Option<CalendarDate> {
    let modified = std::fs::metadata(path).ok()?.modified().ok()?;
    let secs = modified
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    let local = gstreamer::glib::DateTime::from_unix_local(i64::try_from(secs).ok()?).ok()?;
    Some(CalendarDate {
        year: local.year(),
        month: u32::try_from(local.month()).ok()?,
        day: u32::try_from(local.day_of_month()).ok()?,
    })
}

/// Whether `name` is a folder name this flow may create: non-blank, and exactly
/// one `Component::Normal` — no `/`, no `..`, not `.` (spec S5).
///
/// **`file_name() == name` is the whole component test**, and it is stricter
/// than the bus's on purpose. The bus checks that the joined path *has* a
/// `file_name`, which `a/b` and `a/` both do; here the name has to be the last
/// component and nothing else, so what the path line under the field shows is
/// what gets created. A field that read good on `a/b` and then created `b` would
/// be "a field can't read good and then fail to save" failing in the other
/// direction.
///
/// **No length check**, though the name this flow *proposes* is cut at
/// [`naming::folder_slug`]'s 64 bytes. That budget is a proposal, not a limit —
/// the filesystem's is 255 — and the bus has no length refusal either, so
/// marking a hand-typed 70-byte name bad would be the same contract failing the
/// other way: a field reading bad on something that would have saved.
pub fn valid_folder_name(name: &str) -> bool {
    let name = name.trim();
    !name.is_empty() && Path::new(name).file_name().is_some_and(|last| last == name)
}

/// Whether `path` is a projects folder this flow may write in: absolute, and
/// either present or with a parent that is (spec W3).
///
/// **The same call Create will make**, for the same reason: only the leaf is
/// ever created, so a typo makes one stray directory rather than a tree.
pub fn valid_projects_dir(path: &Path) -> bool {
    path.is_absolute() && (path.is_dir() || path.parent().is_some_and(Path::is_dir))
}

/// Whether `<projects_dir>/<folder_name>` already holds a project, which the
/// bus refuses (spec C2 step 4).
///
/// **A courtesy, not the guarantee.** It is one `try_exists` of one path, run at
/// prefill and on every edit; the guarantee is the bus's own, which cannot race.
/// An empty folder is **not** taken: the command adopts one.
pub fn target_taken(projects_dir: &Path, folder_name: &str) -> bool {
    projects_dir
        .join(folder_name.trim())
        .join(store::PROJECT_FILENAME)
        .try_exists()
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pundit_core::project::Project;
    use pundit_core::scoreboard::{MatchFormat, TeamConfig};
    use pundit_core::stroke::Rgba;
    use tempfile::TempDir;

    /// This decade, so `max_year` is never the thing a test is measuring.
    const MAX_YEAR: i32 = 2030;

    fn kit(c: u32) -> Rgba {
        Rgba {
            r: f64::from((c >> 16) as u8) / 255.0,
            g: f64::from((c >> 8) as u8) / 255.0,
            b: f64::from(c as u8) / 255.0,
            a: 1.0,
        }
    }

    /// Makes every directory of `paths` under `root`, and returns `root`.
    fn tree(root: &Path, paths: &[&str]) {
        for p in paths {
            std::fs::create_dir_all(root.join(p)).unwrap();
        }
    }

    /// Writes empty files named `names` in `dir`, and returns their paths.
    fn files(dir: &Path, names: &[&str]) -> Vec<PathBuf> {
        std::fs::create_dir_all(dir).unwrap();
        names
            .iter()
            .map(|n| {
                let path = dir.join(n);
                std::fs::write(&path, b"").unwrap();
                path
            })
            .collect()
    }

    /// A project in `<projects>/<folder>` whose home kit is `c`, so a test can
    /// tell which neighbour was read without comparing whole configs.
    fn neighbour(projects: &Path, folder: &str, c: u32) {
        let dir = projects.join(folder);
        std::fs::create_dir_all(&dir).unwrap();
        let mut project = Project::new(folder);
        project.scoreboard = Some(ScoreboardConfig {
            home: TeamConfig::new("Old Home", kit(c), kit(0xffffff)),
            away: TeamConfig::new("Old Away", kit(0x000000), kit(0xffffff)),
            format: MatchFormat::default(),
            auto_back_anchor_p1: false,
        });
        store::write(&dir, &mut project).unwrap();
    }

    // --- The APP_NAME walk (spec W1) -------------------------------------

    /// Tree A's shape: `<club>/<team>/game-videos/<match>`, with the projects
    /// folder two above the footage.
    #[test]
    fn the_walk_finds_an_app_named_directory_at_depth_two_in_tree_a() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        tree(root, &["club/team/game-videos/match", "club/team/pundit"]);

        let found = projects_dir_for(&root.join("club/team/game-videos/match"), None, None);
        assert_eq!(found, ProjectsDir::Found(root.join("club/team/pundit")));
        assert_eq!(found.provenance(), None);
    }

    /// Tree B's shape: `<club>/<season>/<match>`, with the projects folder two
    /// above the footage again — a different tree, the same depth.
    #[test]
    fn the_walk_finds_an_app_named_directory_at_depth_two_in_tree_b() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        tree(root, &["club/season/match", "club/pundit"]);

        let found = projects_dir_for(&root.join("club/season/match"), None, None);
        assert_eq!(found, ProjectsDir::Found(root.join("club/pundit")));
    }

    /// The match folder itself is the first candidate, not the second.
    #[test]
    fn the_walk_starts_at_the_match_folder() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        tree(root, &["match/pundit"]);

        assert_eq!(
            projects_dir_for(&root.join("match"), None, None),
            ProjectsDir::Found(root.join("match/pundit"))
        );
    }

    /// Four candidates and no more: a `pundit` five levels up is not this
    /// coach's projects folder.
    #[test]
    fn the_walk_stops_after_four_candidates() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        tree(root, &["a/b/c/d/match", "a/pundit"]);
        let footage = root.join("a/b/c/d/match");

        // `a` is the fifth candidate (match, d, c, b, a), so it is not reached.
        assert!(matches!(
            projects_dir_for(&footage, None, None),
            ProjectsDir::Proposed(_)
        ));
        // One level shallower and the same directory is found.
        assert_eq!(
            projects_dir_for(&root.join("a/b/c/match"), None, None),
            ProjectsDir::Found(root.join("a/pundit"))
        );
    }

    /// Home is a candidate; nothing above it is. A `pundit` in `/home` or `/`
    /// belongs to somebody else.
    #[test]
    fn the_walk_stops_at_the_home_directory() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        tree(root, &["home/games/match", "home/pundit", "pundit"]);
        let home = root.join("home");
        let footage = root.join("home/games/match");

        // `<home>/pundit` is found, two candidates up.
        assert_eq!(
            projects_dir_for(&footage, Some(&home), None),
            ProjectsDir::Found(home.join("pundit"))
        );
        // With home's own one gone, the walk stops there rather than taking
        // `<root>/pundit` one level above it.
        std::fs::remove_dir(home.join("pundit")).unwrap();
        assert!(matches!(
            projects_dir_for(&footage, Some(&home), None),
            ProjectsDir::Proposed(_)
        ));
        // Without the bound, the same walk reaches `<root>/pundit`.
        assert_eq!(
            projects_dir_for(&footage, None, None),
            ProjectsDir::Found(root.join("pundit"))
        );
    }

    /// Spec W2: no app-named directory above the footage, so the folder the
    /// last project was in — a directory that demonstrably held one.
    #[test]
    fn with_nothing_above_the_footage_the_last_projects_parent_is_used() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        tree(
            root,
            &["games/match", "elsewhere/pundit/2026-09-21-athletic"],
        );

        let found = projects_dir_for(
            &root.join("games/match"),
            None,
            Some(&root.join("elsewhere/pundit/2026-09-21-athletic")),
        );
        assert_eq!(
            found,
            ProjectsDir::BesideLastProject(root.join("elsewhere/pundit"))
        );
        assert_eq!(found.provenance(), None);
    }

    /// A `last_project` whose folder has gone falls through to `Proposed`, which
    /// carries the line. Unchecked it silently re-created a projects folder the
    /// coach had deleted, and said nothing — and if the grandparent had gone too
    /// the field went red with no line to read, because this tier has none.
    #[test]
    fn a_stale_last_project_falls_through_rather_than_being_taken_as_fact() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        tree(root, &["games/match", "elsewhere/pundit"]);
        let last = root.join("elsewhere/pundit/2026-09-21-athletic");
        std::fs::create_dir_all(&last).unwrap();
        let footage = root.join("games/match");

        // While it is there, it is the answer and needs no explaining.
        assert_eq!(
            projects_dir_for(&footage, None, Some(&last)),
            ProjectsDir::BesideLastProject(root.join("elsewhere/pundit"))
        );

        // Gone, and the whole projects folder with it.
        std::fs::remove_dir_all(root.join("elsewhere")).unwrap();
        let found = projects_dir_for(&footage, None, Some(&last));
        assert_eq!(found, ProjectsDir::Proposed(root.join("games/pundit")));
        assert!(found.provenance().is_some());
    }

    /// Spec W2's floor: an app-named directory **beside** the footage, never
    /// the bare parent — which would propose projects as siblings of the match
    /// folder, the opposite of the pattern this flow keeps. It is the one tier
    /// that carries a line, because it names a directory the coach has never
    /// seen.
    #[test]
    fn with_no_last_project_an_app_named_directory_beside_the_footage_is_proposed() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        tree(root, &["games/match"]);

        let found = projects_dir_for(&root.join("games/match"), None, None);
        assert_eq!(found, ProjectsDir::Proposed(root.join("games/pundit")));
        assert_ne!(found.path(), root.join("games"));
        assert!(found.provenance().is_some());
    }

    // --- The neighbour seeding (spec I5) ---------------------------------

    #[test]
    fn the_newest_neighbour_by_folder_date_lends_its_config() {
        let tmp = TempDir::new().unwrap();
        let projects = tmp.path();
        neighbour(projects, "2026-08-02-rovers", 0x111111);
        neighbour(projects, "2026-09-21-athletic", 0x222222);
        neighbour(projects, "2026-07-14-town", 0x333333);

        let seeded = seed_scoreboard(projects, MAX_YEAR);
        assert_eq!(seeded.home.primary_color, kit(0x222222));
        // The names come with it and are the sheet's to overwrite.
        assert_eq!(seeded.home.name, "Old Home");
    }

    /// The decisive half of "recency is the folder name's date": `project.json`
    /// is rewritten on every edit, so a season-old project reopened to
    /// re-export one clip has the newest mtime in the folder and must not be
    /// the one that seeds the next match.
    #[test]
    fn a_newer_mtime_on_an_older_folder_date_is_not_chosen() {
        let tmp = TempDir::new().unwrap();
        let projects = tmp.path();
        neighbour(projects, "2026-09-21-athletic", 0x222222);
        // Written second, so its mtime is the newer one.
        neighbour(projects, "2025-10-05-rovers", 0x444444);

        let seeded = seed_scoreboard(projects, MAX_YEAR);
        assert_eq!(seeded.home.primary_color, kit(0x222222));
    }

    /// A folder with no parseable date sorts below every dated one, and the
    /// undated ones among themselves by name descending. Still no `stat`.
    #[test]
    fn an_undated_folder_sorts_below_every_dated_one() {
        let tmp = TempDir::new().unwrap();
        let projects = tmp.path();
        neighbour(projects, "friendly", 0x555555);
        neighbour(projects, "2024-04-04-rovers", 0x666666);

        assert_eq!(
            seed_scoreboard(projects, MAX_YEAR).home.primary_color,
            kit(0x666666)
        );

        // With no dated neighbour at all, the undated ones go by name
        // descending: `friendly` before `athletic`.
        let only = TempDir::new().unwrap();
        neighbour(only.path(), "athletic", 0x777777);
        neighbour(only.path(), "friendly", 0x555555);
        assert_eq!(
            seed_scoreboard(only.path(), MAX_YEAR).home.primary_color,
            kit(0x555555)
        );
    }

    /// **A newest neighbour that reads but has nothing to lend is looked past.**
    /// A project made by pointing `Open Project…` at a folder carries
    /// `scoreboard: None` until `Set up teams…` is used, and it would be the
    /// newest thing in the folder — so stopping at "the first that reads" would
    /// hand back the blank config while the match before it had exactly the kit
    /// and format this tier exists to pass on.
    #[test]
    fn a_neighbour_with_no_scoreboard_is_looked_past() {
        let tmp = TempDir::new().unwrap();
        let projects = tmp.path();
        neighbour(projects, "2026-09-21-athletic", 0x222222);
        // Newer, readable, and nothing to lend.
        let bare = projects.join("2026-09-28-friendly");
        std::fs::create_dir_all(&bare).unwrap();
        store::write(&bare, &mut Project::new("Friendly")).unwrap();

        let seeded = seed_scoreboard(projects, MAX_YEAR);
        assert_eq!(seeded.home.primary_color, kit(0x222222));
        assert_ne!(seeded.home.primary_color, blank_config().home.primary_color);
    }

    /// Unreadable, legacy and too-new files are passed over in silence: this is
    /// a prefill, not an operation.
    #[test]
    fn an_unreadable_or_too_new_neighbour_is_skipped() {
        let tmp = TempDir::new().unwrap();
        let projects = tmp.path();
        for (folder, text) in [
            ("2026-09-28-corrupt", "{ this is not json"),
            (
                "2026-09-27-too-new",
                r#"{"formatVersion": 99, "name": "Next"}"#,
            ),
            (
                "2026-09-26-legacy",
                r#"{"formatVersion": 6, "name": "Old"}"#,
            ),
        ] {
            let dir = projects.join(folder);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join(store::PROJECT_FILENAME), text).unwrap();
        }
        neighbour(projects, "2026-09-21-athletic", 0x222222);

        let seeded = seed_scoreboard(projects, MAX_YEAR);
        assert_eq!(seeded.home.primary_color, kit(0x222222));
    }

    /// No neighbour at all, and a projects folder that does not exist yet —
    /// which is exactly what [`ProjectsDir::Proposed`] names — give the config
    /// the setup sheet already opens with, not an error.
    #[test]
    fn no_readable_neighbour_gives_the_blank_config() {
        let tmp = TempDir::new().unwrap();
        for projects in [tmp.path().to_owned(), tmp.path().join("not-created-yet")] {
            let seeded = seed_scoreboard(&projects, MAX_YEAR);
            assert_eq!(seeded.home.primary_color, blank_config().home.primary_color);
            assert_eq!(seeded.format, blank_config().format);
        }
    }

    // --- The prefill (specs I1, I2, I4, N1) ------------------------------

    #[test]
    fn the_match_folder_is_the_deepest_common_ancestor() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let one = files(&root.join("games/match"), &["a.mp4", "b.mp4"]);
        assert_eq!(
            prefill(one, None, None, 2026).match_folder,
            root.join("games/match")
        );

        // Two directories: their shared parent, where the opponent inference
        // will usually find nothing — a blank half of a folder name rather
        // than a wrong one.
        let mut split = files(&root.join("games/cam1"), &["a.mp4"]);
        split.extend(files(&root.join("games/cam2"), &["b.mp4"]));
        assert_eq!(
            prefill(split, None, None, 2026).match_folder,
            root.join("games")
        );
    }

    /// Spec I2 tier 1: a full date every chosen file's name agrees on.
    #[test]
    fn a_date_every_file_name_agrees_on_is_the_match_date() {
        let tmp = TempDir::new().unwrap();
        let videos = files(
            &tmp.path().join("2026-07-04-town"),
            &["20260921-P1.mp4", "20260921-P2.mp4"],
        );
        let got = prefill(videos, None, None, 2026);
        assert_eq!(got.date, "2026-09-21");
        // The folder's own, different date is the second tier and is not used.
        assert_eq!(got.away, "Town");
    }

    /// Files that disagree fall through to the folder's name (tier 2).
    #[test]
    fn file_names_that_disagree_fall_through_to_the_folder() {
        let tmp = TempDir::new().unwrap();
        let videos = files(
            &tmp.path().join("2026-07-04-town"),
            &["20260921-P1.mp4", "20260922-P2.mp4"],
        );
        assert_eq!(prefill(videos, None, None, 2026).date, "2026-07-04");
    }

    /// Tier 3: with no date in any name, the **first-in-order** file's mtime,
    /// which is today for a file a test just wrote. One rule for "the game's
    /// date", not two.
    #[test]
    fn with_no_date_in_any_name_the_first_files_mtime_is_used() {
        gstreamer::init().unwrap();
        let tmp = TempDir::new().unwrap();
        let videos = files(&tmp.path().join("athletic"), &["half one.mp4"]);
        let date = prefill(videos, None, None, 2400).date;
        assert_eq!(date.len(), 10, "{date:?}");
        assert!(date.starts_with("20"), "{date:?}");
    }

    /// Spec I4: the opponent comes from the match folder's name, in the **away**
    /// slot — the coach's own decision, and ⇄ is one click either way.
    #[test]
    fn the_guessed_opponent_prefills_the_away_slot() {
        let tmp = TempDir::new().unwrap();
        let videos = files(&tmp.path().join("170918-city_reserves"), &["a.mp4"]);
        let got = prefill(videos, None, None, 2026);
        assert_eq!(got.away, "City Reserves");
    }

    /// A folder of digits alone yields nothing rather than a wrong guess.
    #[test]
    fn a_folder_of_digits_alone_guesses_no_opponent() {
        let tmp = TempDir::new().unwrap();
        let videos = files(&tmp.path().join("20260917"), &["a.mp4"]);
        let got = prefill(videos, None, None, 2026);
        assert_eq!(got.away, "");
        assert_eq!(got.date, "2026-09-17");
    }

    /// The videos come back in `order_videos`' order — the copy-suffix rule
    /// that stops a game's halves being reversed.
    #[test]
    fn the_videos_come_back_in_order() {
        let tmp = TempDir::new().unwrap();
        let videos = files(
            &tmp.path().join("2026-09-21-athletic"),
            &["Hudson (1).mp4", "Hudson.mp4"],
        );
        let got = prefill(videos, None, None, 2026);
        let names: Vec<&str> = got
            .videos
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(names, ["Hudson.mp4", "Hudson (1).mp4"]);
    }

    // --- The folder name and the validators (specs N1, S5) ---------------

    /// The **joined** string is slugged, so the `-` collapse absorbs a blank
    /// team name or an absent date and the 64-byte cut applies to the whole.
    #[test]
    fn the_folder_name_slugs_the_joined_string() {
        assert_eq!(
            folder_name("2026-09-21", "City FC", "Rovers United"),
            "2026-09-21-city-fc-rovers-united"
        );
        assert_eq!(
            folder_name("2026-09-21", "", "Athletic"),
            "2026-09-21-athletic"
        );
        assert_eq!(folder_name("", "City", "Athletic"), "city-athletic");
        assert_eq!(folder_name("", "", ""), "");
        // The cut applies to the whole name, and never leaves a trailing `-`.
        let long = folder_name("2026-09-21", &"a".repeat(40), &"b".repeat(40));
        assert!(long.len() <= 64, "{long:?} is {} bytes", long.len());
        assert!(!long.ends_with('-'), "{long:?}");
    }

    #[test]
    fn a_folder_name_is_one_normal_component_and_non_blank() {
        for good in ["athletic", "2026-09-21-city-v-rovers", "a b", "..c"] {
            assert!(valid_folder_name(good), "{good:?}");
        }
        // Longer than the name this flow *proposes* — `folder_slug` cuts at 64 —
        // but the filesystem's limit is 255 and the bus has no length refusal, so
        // a hand-typed long name must not be marked bad.
        assert!(valid_folder_name(&"x".repeat(65)));
        for bad in ["", "   ", ".", "..", "/", "a/b", "../a", "a/", "./a"] {
            assert!(!valid_folder_name(bad), "{bad:?}");
        }
    }

    #[test]
    fn a_projects_folder_is_absolute_and_present_or_one_leaf_from_it() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        assert!(valid_projects_dir(root));
        assert!(valid_projects_dir(&root.join("not-created-yet")));
        // Two levels from anything that exists: `create_dir` would fail, so
        // the field says so rather than letting Create fail.
        assert!(!valid_projects_dir(&root.join("nope/deeper")));
        assert!(!valid_projects_dir(Path::new("relative/pundit")));
    }

    /// A `project.json` is what makes a target taken. An **empty** folder is
    /// not: the command adopts one.
    #[test]
    fn only_a_project_json_makes_the_target_taken() {
        let tmp = TempDir::new().unwrap();
        let projects = tmp.path();
        std::fs::create_dir(projects.join("empty")).unwrap();
        neighbour(projects, "taken", 0x222222);

        assert!(target_taken(projects, "taken"));
        assert!(target_taken(projects, " taken "));
        assert!(!target_taken(projects, "empty"));
        assert!(!target_taken(projects, "never-made"));
    }

    #[test]
    fn a_team_is_named_only_when_it_is_more_than_whitespace() {
        for named in ["City", " City ", "1"] {
            assert!(team_named(named), "{named:?}");
        }
        // The case that was shipped as `!= ""` in Slint, marked good, enabled
        // Create and sent nothing.
        for blank in ["", " ", "\t", "   "] {
            assert!(!team_named(blank), "{blank:?}");
        }
    }
}
