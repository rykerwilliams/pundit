//! What a finished export run says, and where it says it (BACKLOG #77 task 3).
//!
//! **A lib module because `main.rs` is not testable.** The harness has no
//! window and a UI test links the library, so nothing in `main.rs`'s event arms
//! can be reached by either — which is the limit `CLAUDE.md` already records
//! for `open_export_sheet`'s seeding. The *decision* a finished run forces is
//! six cases over three kinds of run, so it lives here with its own tests and
//! `show_export` is left doing nothing but setting what this answers.
//!
//! **The three kinds are not interchangeable**, and that is the whole reason
//! this exists:
//!
//! - an **export** run is one the coach started and is standing in front of, so
//!   a failure is a modal — *"a run of one target is still the common case"*;
//! - a **basket** run writes one film under a name that may have been suffixed,
//!   so its line names the **file that was written** rather than the name asked
//!   for (basket spec O1), and it needs a line on the **sheet** as well because
//!   the status bar renders behind the scrim;
//! - a **queue** run is one the coach walked away from, across several
//!   projects. A modal landing over whatever he is doing now is exactly what
//!   `UserError::Scoreboard` and `UserError::Slate` are notices for, and the
//!   Queue sheet's own rows carry which job failed and why. So its failures go
//!   to the notice line and name a count rather than one job (#77 spec §Q7).

use std::path::{Path, PathBuf};

use crate::bus::{ExportRun, TargetState};
use crate::format::sentence;

/// Which kind of run is going, and so what its end means.
///
/// **The authoritative value, and it is Rust's rather than a window
/// property.** `main.rs` derives the sheet's own flag from it, so the two
/// cannot disagree: there is one writer and the flag is a reading of this. A
/// second independent bool is what #77's plan review rejected, and a Slint enum
/// would put the decision somewhere no test can reach it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RunKind {
    /// The export sheet's own Export button: the ticked targets, now.
    #[default]
    Export,
    /// The basket's Start: one film from several matches.
    Basket,
    /// `Start queue`: everything that was enqueued, across projects.
    Queue,
}

/// What to show when a run ends. Every field is independent, and an empty one
/// means "say nothing there" rather than "say nothing".
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Report {
    /// The status bar's line.
    pub notice: Option<String>,
    /// The modal dialog, for a run the coach is standing in front of.
    pub modal: Option<String>,
    /// The sheet's own line — the basket's, which is the only place either its
    /// failure or its file is readable while the scrim is up (basket spec C6).
    pub sheet: Option<String>,
}

/// What `run` says now that it has ended.
///
/// **Called only for a finished run**; `show_export` returns early while one is
/// still going, because every field here is about an outcome.
pub fn report(kind: RunKind, run: &ExportRun) -> Report {
    let failures: Vec<&String> = run
        .targets
        .iter()
        .filter_map(|t| match &t.state {
            TargetState::Failed(why) => Some(why),
            _ => None,
        })
        .collect();

    if let Some(first) = failures.first() {
        // **A run stops at nothing, so the first failure is what to say** —
        // except for a queue, where the rows are the report and a count is the
        // honest summary of several.
        let text = format!("export failed: {first}");
        return match kind {
            RunKind::Export => Report {
                modal: Some(text),
                ..Report::default()
            },
            RunKind::Basket => Report {
                sheet: Some(sentence(&text)),
                modal: Some(text),
                ..Report::default()
            },
            RunKind::Queue => Report {
                notice: Some(failed_count(failures.len(), run.targets.len())),
                ..Report::default()
            },
        };
    }

    let written: Vec<&PathBuf> = run
        .targets
        .iter()
        .filter_map(|t| match &t.state {
            TargetState::Done(path) => Some(path),
            _ => None,
        })
        .collect();
    if written.is_empty() {
        // A cancel, which names nothing: the rows carry it.
        return Report::default();
    }

    match kind {
        // **No folder clause** (#77 task 1): once a project can be opened
        // mid-run it would name the wrong project's folder, and for a queue
        // several folders at once.
        RunKind::Export | RunKind::Queue => Report {
            notice: Some(format!(
                "Exported {} video{}",
                written.len(),
                plural(written.len())
            )),
            ..Report::default()
        },
        RunKind::Basket => {
            let path = written[0];
            let file = name_of(path);
            let folder = path.parent().map(name_of).unwrap_or_default();
            Report {
                notice: Some(format!(
                    "Wrote {file} to the {folder} folder in your videos folder"
                )),
                sheet: Some(format!("Wrote {}", path.display())),
                ..Report::default()
            }
        }
    }
}

/// `"2 of 5 exports failed — the queue's list says which"`.
fn failed_count(failed: usize, total: usize) -> String {
    format!(
        "{failed} of {total} export{} failed — the queue's list says which",
        plural(total)
    )
}

fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}

fn name_of(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::ExportTargetRun;

    fn run(states: &[TargetState]) -> ExportRun {
        ExportRun {
            targets: states
                .iter()
                .enumerate()
                .map(|(i, state)| ExportTargetRun {
                    label: format!("t{i}"),
                    frames: 30,
                    state: state.clone(),
                })
                .collect(),
            rate: None,
        }
    }

    fn done(name: &str) -> TargetState {
        TargetState::Done(PathBuf::from(format!("/films/basket/{name}")))
    }

    /// **A failure the coach is standing in front of is a modal; one he walked
    /// away from is not.** The split #77 §Q7 exists for, and the six cases of
    /// it are the reason this is a function rather than three `if`s in
    /// `main.rs`.
    #[test]
    fn a_failure_is_modal_for_a_run_the_coach_started_and_a_notice_for_a_queue() {
        let failed = run(&[TargetState::Failed("no video".into()), done("a.mp4")]);

        let export = report(RunKind::Export, &failed);
        assert_eq!(export.modal.as_deref(), Some("export failed: no video"));
        assert_eq!(export.notice, None, "it must not say it twice");

        // The basket's line as well, because the status bar renders behind the
        // scrim and that is the only place its failure is readable.
        let basket = report(RunKind::Basket, &failed);
        assert_eq!(basket.modal.as_deref(), Some("export failed: no video"));
        assert_eq!(basket.sheet.as_deref(), Some("Export failed: no video"));

        // And a queue says how many, on the notice line, with no modal at all:
        // its rows are the report.
        let queue = report(RunKind::Queue, &failed);
        assert_eq!(queue.modal, None, "a modal over the project he moved on to");
        assert_eq!(
            queue.notice.as_deref(),
            Some("1 of 2 exports failed — the queue's list says which")
        );
    }

    /// **The success notice names no folder**, for any kind (#77 task 1): once
    /// a project can be opened mid-run, "the project's exports folder" is the
    /// wrong project's — and for a queue it is several at once.
    #[test]
    fn a_finished_run_says_how_many_and_never_which_folder() {
        let two = run(&[done("a.mp4"), done("b.mp4")]);
        for kind in [RunKind::Export, RunKind::Queue] {
            let r = report(kind, &two);
            assert_eq!(r.notice.as_deref(), Some("Exported 2 videos"), "{kind:?}");
            assert_eq!(r.modal, None, "{kind:?}");
        }
        assert_eq!(
            report(RunKind::Export, &run(&[done("a.mp4")]))
                .notice
                .as_deref(),
            Some("Exported 1 video"),
            "one is singular"
        );
    }

    /// **A basket names the file it actually wrote**, not the name asked for:
    /// Start suffixes rather than overwriting (basket spec O1), so the two can
    /// differ and only one of them is true.
    #[test]
    fn a_basket_names_the_file_and_the_folder_it_landed_in() {
        let r = report(RunKind::Basket, &run(&[done("Corners (2).mp4")]));
        assert_eq!(
            r.notice.as_deref(),
            Some("Wrote Corners (2).mp4 to the basket folder in your videos folder")
        );
        assert_eq!(
            r.sheet.as_deref(),
            Some("Wrote /films/basket/Corners (2).mp4")
        );
    }

    /// **A cancel says nothing at all**, for every kind: the rows carry it, and
    /// a notice would be a second account of something already on screen.
    #[test]
    fn a_cancelled_run_says_nothing() {
        let cancelled = run(&[TargetState::Cancelled, TargetState::Cancelled]);
        for kind in [RunKind::Export, RunKind::Basket, RunKind::Queue] {
            assert_eq!(report(kind, &cancelled), Report::default(), "{kind:?}");
        }
        // And a cancel that caught one target already written is still a
        // success for that one — `cancel_leaves_the_targets_already_written_alone`
        // is the behaviour this mirrors.
        let partly = run(&[done("a.mp4"), TargetState::Cancelled]);
        assert_eq!(
            report(RunKind::Export, &partly).notice.as_deref(),
            Some("Exported 1 video")
        );
    }
}
