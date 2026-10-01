//! What a row of the recent-projects popover shows (spec D1, D2, D3).
//!
//! The rules live here rather than in `main.rs` because `main.rs` is wiring and
//! has no `#[cfg(test)]` module at all — the same reason [`crate::new_match`]
//! and [`crate::fit`] are their own modules.
//!
//! **Nothing here reaches the bus.** A row is a label and two flags; the click
//! that acts on one is a `Command::OpenProject` like any other, with the same
//! refusals.

use std::path::{Path, PathBuf};

use pundit_core::{metadata, store};

/// How many projects the popover remembers (spec S3).
///
/// Eight, which is an estimate rather than a measurement: a row is two lines,
/// so eight stand about 320px over the player. The coach has three projects and
/// a season has twenty-odd, and a list you scroll is a picker — which they
/// already have.
pub const RECENT_PROJECTS: usize = 8;

/// One row of the popover.
///
/// The row's **path does not reach Slint** — the callback carries the index, and
/// `main.rs` re-reads the list to resolve it. So this type is the whole of what
/// a row knows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub path: PathBuf,
    /// Line one, **always populated**: the match's name where the project
    /// resolved, the folder's own file name where it did not.
    ///
    /// Never empty-as-a-signal. An earlier draft left it empty for an
    /// unresolved row, which draws a blank first line with the folder name
    /// underneath — the opposite of spec D2's *"shows the folder name alone,
    /// dimmed"* — and made one string carry two facts, so `main.rs` would have
    /// re-derived `dimmed` as `label.is_empty()`. [`Row::resolved`] is that
    /// fact, once.
    pub label: String,
    /// Line two, empty where it would only repeat [`Row::label`] (spec D1).
    ///
    /// The collision it resolves is **two matches against the same opponent**:
    /// a New match project is named `<Home> v <Away>`, so `match_label` is
    /// identical for every match against that opponent — the normal case across
    /// a season — and the folder name is what tells them apart. For an
    /// unresolved row the two strings *are* the same, so D1's rule covers it
    /// with no special case.
    pub second_line: String,
    /// Whether the project read. `false` **dims the row and nothing more**: a
    /// dimmed row is still clickable, because the project on the drive the coach
    /// is about to plug in must not be the one row that refuses. The click's
    /// refusal already names the folder and is already modal.
    pub resolved: bool,
    /// The open project: ticked, and **the one row that is not clickable**.
    ///
    /// Not cosmetic. `OpenProject` on the already-open folder reaches `commit`,
    /// which clears the undo history and runs `clips::empty_trash` on both the
    /// outgoing and the incoming folder — so a re-open of the open project
    /// would silently and permanently destroy the coach's undo stack and the
    /// clip recordings in it.
    pub open: bool,
}

/// A row per path, in list order.
///
/// `open` is the folder of the project that is open and its label, if one is —
/// an `Option`, because a first launch, a refused `pundit <folder>` run and a
/// launch whose restore failed each have a list and no open project.
///
/// **The open project's row is labelled from `open`, not from disk**, which is
/// the basket's own rule (`bus/basket.rs`'s `project_for`: the open project from
/// memory, every other from [`store::read`]). It is not only cheaper but more
/// correct — a rename stands in memory while a failed save leaves the old name
/// on disk, so a re-read would show the popover disagreeing with the window
/// title. Its path is therefore **not** read, which is also what keeps the cost
/// at one read per *other* row.
///
/// **The tick is a path match, never a position** (spec D3). The head is not the
/// open project after a launch whose restore failed, on a first launch, or after
/// a refused `pundit <folder>` run; ticking the head would put a tick and a
/// disabled state on a greyed, unreachable row — and the one row the coach most
/// needs, to retry once the drive is mounted, is the one that would refuse.
///
/// **No injected reader.** The one valuable test here needs four real
/// [`store::StoreError`]s — absent, malformed, legacy, too new — which a stub
/// cannot produce honestly. `new_match::lent_scoreboard` is the comparable
/// function and takes a `&Path` for the same reason.
pub fn rows(paths: &[PathBuf], open: Option<(&Path, &str)>) -> Vec<Row> {
    paths
        .iter()
        .map(|path| {
            let folder_name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();

            // The open project's label comes from the caller; every other
            // row's from its own `project.json`.
            let (label, resolved) = match open {
                Some((folder, label)) if folder == path.as_path() => (label.to_owned(), true),
                _ => match store::read(path) {
                    Ok(project) => (metadata::match_label(&project), true),
                    // Gone, unreadable, legacy or too new: the folder name
                    // alone, dimmed. Not an error — the popover is a list, not
                    // an operation, so the refusal's kind does not matter here.
                    Err(_) => (folder_name.clone(), false),
                },
            };

            Row {
                open: open.is_some_and(|(folder, _)| folder == path.as_path()),
                second_line: if folder_name == label {
                    String::new()
                } else {
                    folder_name
                },
                path: path.clone(),
                label,
                resolved,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::match_panel::blank_config;
    use pundit_core::project::Project;

    /// A project folder holding a `project.json`.
    ///
    /// With `teams`, it carries a scoreboard naming them, which is what a New
    /// match project has and what `match_label` reads first. Without, it is
    /// named after its own folder — which is what `open_project` builds for a
    /// folder holding no `project.json` (`bus/project.rs`, pinned by
    /// `project_and_sources.rs`), so `match_label` falls through to the name.
    fn project_at(dir: &Path, folder: &str, teams: Option<(&str, &str)>) -> PathBuf {
        let path = dir.join(folder);
        std::fs::create_dir(&path).unwrap();
        let mut project = match teams {
            Some((home, away)) => {
                let mut project = Project::new(format!("{home} v {away}"));
                let mut config = blank_config();
                config.home.name = home.to_owned();
                config.away.name = away.to_owned();
                project.scoreboard = Some(config);
                project
            }
            None => Project::new(folder.to_owned()),
        };
        store::write(&path, &mut project).unwrap();
        path
    }

    /// **The base case, and D1's reason in one fixture: two matches against the
    /// same opponent.** `match_label` is identical for both — which is the
    /// normal case across a season — so the folder name is what tells them
    /// apart, and it is shown on both rows. Order is the list's own.
    #[test]
    fn two_matches_against_one_opponent_are_told_apart_by_their_folders() {
        let dir = tempfile::tempdir().unwrap();
        let september = project_at(
            dir.path(),
            "20260917-rovers-athletic",
            Some(("Rovers", "Athletic")),
        );
        let october = project_at(
            dir.path(),
            "20261008-rovers-athletic",
            Some(("Rovers", "Athletic")),
        );

        let rows = rows(&[october.clone(), september.clone()], None);

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].path, october, "the list's order is kept");
        assert_eq!(rows[0].label, "Rovers v Athletic");
        assert_eq!(rows[0].second_line, "20261008-rovers-athletic");
        assert_eq!(rows[1].path, september);
        assert_eq!(rows[1].label, "Rovers v Athletic");
        assert_eq!(
            rows[1].second_line, "20260917-rovers-athletic",
            "the same label on both rows is exactly why the folder is shown"
        );
        assert!(rows.iter().all(|r| r.resolved && !r.open));
    }

    /// A project whose name **is** its folder name — which is what
    /// `open_project` builds for a folder with no `project.json`
    /// (`bus/project.rs`, pinned by `project_and_sources.rs`) — draws its name
    /// once, not twice (spec D1).
    #[test]
    fn a_project_named_after_its_folder_has_no_second_line() {
        let dir = tempfile::tempdir().unwrap();
        let path = project_at(dir.path(), "Saturday Game", None);

        let rows = rows(&[path], None);

        assert_eq!(rows[0].label, "Saturday Game");
        assert_eq!(rows[0].second_line, "");
        assert!(rows[0].resolved);
    }

    /// **Four kinds of refusal, four rows — not four dropped rows.** Gone,
    /// malformed, legacy and too new: each shows its folder name alone, dimmed,
    /// and stays in the list. The refusal's *kind* deliberately does not
    /// matter, which is why all four are asserted the same way.
    #[test]
    fn a_project_that_does_not_read_is_a_dimmed_row_with_its_folder_name() {
        let dir = tempfile::tempdir().unwrap();
        let mut paths = Vec::new();
        for (folder, text) in [
            ("absent", None),
            ("corrupt", Some("{ this is not json")),
            ("legacy", Some(r#"{"formatVersion": 6, "name": "Old"}"#)),
            ("too-new", Some(r#"{"formatVersion": 99, "name": "New"}"#)),
        ] {
            let path = dir.path().join(folder);
            std::fs::create_dir(&path).unwrap();
            if let Some(text) = text {
                std::fs::write(path.join("project.json"), text).unwrap();
            }
            paths.push(path);
        }
        // And one that was never created at all.
        paths.push(dir.path().join("never-existed"));

        let rows = rows(&paths, None);

        assert_eq!(rows.len(), 5, "no row is dropped");
        for (row, folder) in
            rows.iter()
                .zip(["absent", "corrupt", "legacy", "too-new", "never-existed"])
        {
            assert!(!row.resolved, "{folder}");
            assert_eq!(row.label, folder, "the folder name is line one");
            assert_eq!(row.second_line, "", "and it is not repeated");
            assert!(!row.open, "{folder}");
        }
    }

    /// The open project's row is the caller's, **never a re-read**: a rename
    /// stands in memory while a failed save leaves the old name on disk, and the
    /// popover must agree with the window title.
    #[test]
    fn the_open_projects_row_is_labelled_from_memory_not_from_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = project_at(
            dir.path(),
            "20260917-rovers-athletic",
            Some(("Rovers", "Athletic")),
        );

        let rows = rows(std::slice::from_ref(&path), Some((&path, "Rovers v Town")));

        assert_eq!(
            rows[0].label, "Rovers v Town",
            "the argument wins over the name on disk"
        );
        assert_eq!(rows[0].second_line, "20260917-rovers-athletic");
        assert!(rows[0].resolved);
        assert!(rows[0].open);
    }

    /// No open project at all — a first launch, a refused `pundit <folder>` run,
    /// or a launch whose restore failed.
    #[test]
    fn with_no_open_project_no_row_is_ticked() {
        let dir = tempfile::tempdir().unwrap();
        let path = project_at(
            dir.path(),
            "20260917-rovers-athletic",
            Some(("Rovers", "Athletic")),
        );

        let rows = rows(&[path], None);

        assert!(!rows[0].open);
        assert!(rows[0].resolved);
    }

    /// **Spec D3, and the design bug the spec's review found.** After a launch
    /// whose restore failed, the head is a project that is **not** open — so the
    /// tick follows the path, and the head is dimmed, unticked and still
    /// clickable. "Tick the head" would put the tick and the one disabled row on
    /// the greyed entry the coach most needs to click.
    #[test]
    fn an_unresolved_head_is_not_ticked_when_another_project_is_open() {
        let dir = tempfile::tempdir().unwrap();
        let gone = dir.path().join("20261008-rovers-athletic");
        let open = project_at(dir.path(), "20260917-rovers-town", Some(("Rovers", "Town")));

        let rows = rows(
            &[gone.clone(), open.clone()],
            Some((&open, "Rovers v Town")),
        );

        assert_eq!(rows[0].path, gone, "the head is still the head");
        assert!(!rows[0].resolved, "and it is dimmed");
        assert!(!rows[0].open, "but it is NOT the ticked row");
        assert_eq!(rows[0].label, "20261008-rovers-athletic");

        assert!(rows[1].open, "the ticked row is the one whose path matches");
        assert!(rows[1].resolved);
    }

    /// An empty list is an empty list, not a row.
    #[test]
    fn no_paths_is_no_rows() {
        assert!(rows(&[], None).is_empty());
    }
}
