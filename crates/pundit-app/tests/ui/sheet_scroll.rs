//! **No sheet is taller than the window, and its buttons can be reached**
//! (BACKLOG #134).
//!
//! All six modals are one `Sheet`, and until this change that component was
//! `height: body.preferred-height` with no scroll: a card that outgrew the
//! window had its bottom — the action buttons — simply off the screen, with no
//! way to reach them. Measured at the declared 1100x700 minimum, two of the six
//! were over it: the export sheet reopened over a run (**823px**) and New match
//! with twenty videos (**704px**).
//!
//! **What this file pins is the rule, not those two cases.** `Scrim`'s column
//! caps every sheet at the window and the body scrolls the rest, so the
//! assertion below is over all six sheets at once, in the worst state each is
//! configured in — which is what makes it a tripwire for the seventh (#96's
//! keys sheet is 29 rows) rather than a measurement of today's tallest.
//!
//! **The buttons scroll with the body**, because each sheet puts its own button
//! row in `@children`. That is the accepted trade, so the reachability test
//! scrolls before it presses Export; it does *not* assert the button is out of
//! view first, which would stand in the way of ever pinning the row.

use crate::key_action;
use crate::{AppWindow, BasketPieceRow, MatchEditorRow, RunRow, TargetRow};
use i_slint_backend_testing::{AccessibleRole, ElementHandle};
use pundit_app::keymap::Keymap;
use slint::platform::{Key, WindowEvent};
use slint::{ComponentHandle, LogicalPosition, LogicalSize, ModelRc, SharedString, VecModel};
use std::cell::Cell;
use std::ops::ControlFlow;
use std::rc::Rc;
use std::time::Duration;

/// One of the six modals: how a test opens it, the element type name its card
/// is found by, and the worst state it is configured in. Paired here so a sheet
/// that scrolls and a sheet that overflows are two different failures, named.
struct Sheet {
    /// The card's element type name, as `app.slint` declares the component.
    card: &'static str,
    /// Opens it, loaded with the worst case measured for it.
    open: fn(&AppWindow),
    /// Whether it is shut — what Esc has to achieve.
    shut: fn(&AppWindow) -> bool,
}

/// All six, in the order `app.slint` declares them. The error dialog is the one
/// with no component of its own, so its card is the bare `Sheet`.
fn sheets() -> Vec<Sheet> {
    vec![
        Sheet {
            card: "ExportSheet",
            open: |w| {
                w.set_export_targets(targets(12));
                w.set_export_run(run(8));
                w.set_export_finish("Finishes at 3:42 PM".into());
                w.set_export_whole_match_ticked(true);
                w.set_export_scoreboard(2);
                w.set_export_cues(false);
                w.set_export_sheet_open(true);
            },
            shut: |w| !w.get_export_sheet_open(),
        },
        Sheet {
            card: "BasketSheet",
            open: |w| {
                w.set_basket_pieces(pieces(30));
                w.set_basket_run(true);
                w.set_export_run(run(8));
                w.set_basket_message("A piece of this film can't be read.".into());
                w.set_basket_sheet_open(true);
            },
            shut: |w| !w.get_basket_sheet_open(),
        },
        Sheet {
            card: "MatchSetupSheet",
            open: |w| w.set_match_sheet_open(true),
            shut: |w| !w.get_match_sheet_open(),
        },
        Sheet {
            card: "MatchEditorSheet",
            open: |w| {
                w.set_match_editor_rows(editor_rows(40));
                w.set_match_editor_message("There is no video 900.".into());
                w.set_match_editor_open(true);
            },
            shut: |w| !w.get_match_editor_open(),
        },
        Sheet {
            card: "NewMatchSheet",
            open: |w| {
                w.set_new_match_videos(videos(20));
                w.set_new_match_provenance("Beside the footage.".into());
                w.set_new_match_sheet_open(true);
            },
            shut: |w| !w.get_new_match_sheet_open(),
        },
        Sheet {
            card: "Sheet",
            open: |w| {
                w.set_error_message(
                    "Something went wrong, and here is a long account of it. "
                        .repeat(30)
                        .into(),
                )
            },
            shut: |w| w.get_error_message().is_empty(),
        },
    ]
}

/// A window at the floor it declares — the size the card has to fit, and the
/// size the testing backend opens at anyway, stated rather than assumed.
fn window() -> AppWindow {
    i_slint_backend_testing::init_no_event_loop();
    let w = AppWindow::new().unwrap();
    // **The keys come from the table now** (BACKLOG #96): with `action-for`
    // unwired every shortcut reads as `KeyAction.none`, so the arrow-key test
    // below would assert "no skip reached the window" against a window that
    // could not have skipped either way — passing on nothing. This file was
    // written on a branch that predated the table, and the merge is what
    // caught it.
    key_action::wire_keys(&w, &crate::scratch_state(), Keymap::defaults());
    w.show().unwrap();
    let (min_w, min_h) = (w.get_min_window_width(), w.get_min_window_height());
    w.window().set_size(LogicalSize::new(min_w, min_h));
    w
}

/// The one card of this type on screen.
fn card(w: &AppWindow, type_name: &str) -> ElementHandle {
    ElementHandle::find_by_element_type_name(w, type_name)
        .next()
        .unwrap_or_else(|| panic!("no {type_name} on screen with the sheet open"))
}

/// Flicks the sheet's body to its end. The wheel lands in the body's own top
/// padding — a point inside the card that no child of it covers, so neither the
/// target list nor the run list (both `ListView`s, and so `Flickable`s of their
/// own) takes the event instead. Deliberately far more scroll than any sheet
/// has: the `Flickable` clamps, and the mock time is what lets its physics
/// animation arrive.
fn flick_to_the_end(w: &AppWindow, card: &ElementHandle) {
    let at = card.absolute_position();
    let position = LogicalPosition::new(at.x + 10.0, at.y + 10.0);
    for _ in 0..20 {
        w.window()
            .dispatch_event(WindowEvent::PointerMoved { position });
        w.window().dispatch_event(WindowEvent::PointerScrolled {
            position,
            delta_x: 0.0,
            delta_y: -240.0,
        });
        i_slint_backend_testing::mock_elapsed_time(Duration::from_millis(100));
    }
}

/// The button reading `label` inside this card. **Found by role as well as by
/// words**: the export sheet's heading is the word "Export" too, and a test
/// that pressed the title would pass without a button at all.
fn button(card: &ElementHandle, label: &str) -> Option<ElementHandle> {
    card.visit_descendants(|e| {
        if e.accessible_role() == Some(AccessibleRole::Button)
            && e.accessible_label().as_deref() == Some(label)
        {
            return ControlFlow::Break(e);
        }
        ControlFlow::Continue(())
    })
}

/// **Every sheet fits the window it is shown in**, in the worst state each is
/// configured in. The one assertion #134 was about: a card taller than the
/// window is a card with its buttons off the bottom of it.
///
/// Measured before this change and after, at 1100x700 (so the export sheet's
/// target list is at its own 210px cap and the run list at its 140px one):
///
/// | sheet | before | after |
/// |---|---|---|
/// | export, a run in progress | 823px | 660px |
/// | export, as an export is set up | 623px | 623px |
/// | basket, 30 pieces and a run | 657px | 657px |
/// | match setup | 523px | 523px |
/// | match editor, 40 events | 561px | 561px |
/// | New match, 20 videos | 704px | 660px |
/// | error, a long message | 579px | 579px |
///
/// 660px is 700 less the scrim's 20px either side; every sheet that fits is
/// unchanged to the pixel, which is the other half of what this pins.
#[test]
fn no_sheet_outgrows_the_window() {
    let w = window();

    for sheet in sheets() {
        (sheet.open)(&w);
        let it = card(&w, sheet.card);
        let height = it.size().height;
        let window = w.window().size();
        let window_height = window.height as f32;
        assert!(
            height <= window_height,
            "the {} is {height}px tall in a {window_height}px window: its \
             buttons are off the bottom of it",
            sheet.card
        );
        // **Still centred**, which is the other half of what `Scrim`'s column
        // does: a box layout left-aligns a fixed-width cell unless it is told
        // `cross-axis-alignment: center`, and a card drawn flush against the
        // left edge of the window is not a modal anyone would ship.
        let slack = window.width as f32 - it.size().width;
        assert!(
            (it.absolute_position().x - slack / 2.0).abs() < 1.0,
            "the {} sits at x={} in a {}px window, not centred",
            sheet.card,
            it.absolute_position().x,
            window.width
        );
        press(&w, Key::Escape);
    }
}

/// **A sheet taller than the window can still be exported from.** The test
/// #134 never had: the export sheet loaded past the window's height, flicked
/// to its end, and its Export button pressed — the one button a coach cannot
/// do without, and the one that was off the bottom of the window.
///
/// **`invoke_accessible_default_action`, not a click**, as the rest of this
/// sheet's tests: it is the `Button`'s own default action, and what the test
/// needs to prove is that the button is *found* — `ElementHandle` skips
/// anything clipped away, so a button scrolled out of the card is a button that
/// does not exist as far as this search is concerned.
///
/// **Being found is not enough, so the button's own rectangle is checked
/// against the window too.** Clipping is what the search tests, and a card
/// taller than the window is not clipped by it — remove the cap and the Export
/// button is at y≈790 of a 700px window, in the tree and on nobody's screen.
/// `no_sheet_outgrows_the_window` is the assertion that catches that; this one
/// catches it as well, which is the point of measuring the button and not only
/// pressing it.
#[test]
fn a_sheet_taller_than_the_window_can_still_be_exported_from() {
    let w = window();
    let exported = Rc::new(Cell::new(false));
    let fired = exported.clone();
    w.on_start_export(move || fired.set(true));

    w.set_export_targets(targets(12));
    w.set_export_any_ticked(true);
    w.set_export_run(run(8));
    w.set_export_finish("Finishes at 3:42 PM".into());
    w.set_export_whole_match_ticked(true);
    w.set_export_scoreboard(2);
    w.set_export_cues(false);
    w.set_export_sheet_open(true);

    let sheet = card(&w, "ExportSheet");
    flick_to_the_end(&w, &sheet);
    let export =
        button(&sheet, "Export").expect("the Export button, after flicking the sheet to its end");

    let bottom = export.absolute_position().y + export.size().height;
    let window_height = w.window().size().height as f32;
    assert!(
        bottom <= window_height,
        "the Export button's bottom edge is at {bottom}px of a {window_height}px \
         window: it is in the tree and off the screen"
    );

    export.invoke_accessible_default_action();
    assert!(
        exported.get(),
        "Export was pressed and the window's `start-export` never ran"
    );
}

/// **Esc still closes every sheet.** The cascade in `handle-key` runs in the
/// capture phase, ahead of whatever in the sheet has focus, and a `ScrollView`
/// in the card is exactly such a child — so this is the regression the scroll
/// could plausibly have caused and did not.
#[test]
fn esc_still_closes_every_sheet() {
    let w = window();

    for sheet in sheets() {
        (sheet.open)(&w);
        assert!(
            !(sheet.shut)(&w),
            "the {} did not open, so Esc proves nothing",
            sheet.card
        );
        press(&w, Key::Escape);
        assert!((sheet.shut)(&w), "Esc did not close the {}", sheet.card);
    }
}

/// **The arrow keys are the window's, and a scrolling sheet does not take
/// them.** `handle-key` is `capture-key-pressed`, so every shortcut runs ahead
/// of whichever child has focus (spec D10) — the rule that stops a touched
/// slider turning the arrows into volume, and the one a `Flickable` inside the
/// card would have broken if Slint's scrolling were keyboard-driven. It is not:
/// a `Flickable` reads the wheel and a drag and no keys at all.
///
/// So both halves are asserted: a sheet open swallows the arrows (modal, and
/// the skip never reaches the bus), and the sheet closed hands them straight
/// back to the shortcut.
#[test]
fn a_scrolling_sheet_does_not_take_the_arrow_keys() {
    let w = window();
    let skipped = Rc::new(Cell::new(0));
    let count = skipped.clone();
    w.on_skip(move |_| count.set(count.get() + 1));

    for sheet in sheets() {
        (sheet.open)(&w);
        press(&w, Key::RightArrow);
        press(&w, Key::LeftArrow);
        assert_eq!(
            skipped.get(),
            0,
            "an arrow key behind the {} reached the window's skip",
            sheet.card
        );
        press(&w, Key::Escape);
    }

    press(&w, Key::RightArrow);
    assert_eq!(
        skipped.get(),
        1,
        "with every sheet shut the arrow keys are the window's again"
    );
}

/// A key, down and up: `handle-key` is called for releases too.
fn press(w: &AppWindow, key: impl Into<SharedString> + Clone) {
    w.window().dispatch_event(WindowEvent::KeyPressed {
        text: key.clone().into(),
    });
    w.window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}

// --- The models each sheet is loaded with --------------------------------

fn targets(n: usize) -> ModelRc<TargetRow> {
    ModelRc::new(VecModel::from(
        (0..n)
            .map(|i| TargetRow {
                label: format!("Tag {i}").into(),
                detail: "4 clips · 3:12".into(),
                ticked: true,
            })
            .collect::<Vec<_>>(),
    ))
}

fn run(n: usize) -> ModelRc<RunRow> {
    ModelRc::new(VecModel::from(
        (0..n)
            .map(|i| RunRow {
                label: format!("Tag {i}").into(),
                rendering: i == 0,
                progress: 0.5,
                status: "45%".into(),
            })
            .collect::<Vec<_>>(),
    ))
}

fn pieces(n: usize) -> ModelRc<BasketPieceRow> {
    ModelRc::new(VecModel::from(
        (0..n)
            .map(|i| BasketPieceRow {
                match_label: format!("Rovers v Athletic {i}").into(),
                clip_label: "Corner, second half".into(),
                length: "0:14".into(),
                problem: "".into(),
            })
            .collect::<Vec<_>>(),
    ))
}

fn editor_rows(n: usize) -> ModelRc<MatchEditorRow> {
    ModelRc::new(VecModel::from(
        (0..n)
            .map(|i| MatchEditorRow {
                id: format!("id-{i}").into(),
                where_text: "2 · 14:05.0".into(),
                label: "Home goal".into(),
                role_less: false,
            })
            .collect::<Vec<_>>(),
    ))
}

fn videos(n: usize) -> ModelRc<SharedString> {
    ModelRc::new(VecModel::from(
        (0..n)
            .map(|i| SharedString::from(format!("{}. half-{i}.mp4", i + 1)))
            .collect::<Vec<_>>(),
    ))
}
