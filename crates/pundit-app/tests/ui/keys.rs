//! **Every key the app binds, pressed on the real `AppWindow`** (BACKLOG #96,
//! plan task 3) — the first automated cover the shortcuts have ever had.
//!
//! The spec said this could not be tested and listed all of it as manual. It
//! can: `handle-key` runs in the window's capture phase, so a dispatched
//! `WindowEvent::KeyPressed` reaches exactly the code a coach's keyboard
//! reaches, and the window's callbacks are where the effect is observable.
//! What stays manual is the half a callback cannot show — that the picture,
//! the readout and the window actually move.
//!
//! **The lookup under test is the production one.** `key_action::wire_keys` is
//! a module of both this binary and the app's, so the 29-arm
//! `Action` -> `KeyAction` map these tests press against is the shipped one; a
//! handler written again here would pin the copy and let a crossed arm ship
//! (`fit_window.rs`'s header is the record of that trap).
//!
//! **Three things are pinned here that nothing else can pin.**
//!
//! - **Every default binding still reaches the callback it used to**, which is
//!   what makes a 61-comparison chain safe to replace with one lookup.
//! - **The far skip is its own action** (spec G2) — `-10` against `-3` is the
//!   one behaviour change in this task, and an assertion rather than a
//!   sentence. With it, a shifted *letter* now fires nothing, which is its
//!   named cost and its own test.
//! - **An arrow release is swallowed too.** `handle-key` is called from
//!   `capture-key-released` as well, and a slider fires `released` on an arrow
//!   key's release — so the lookup runs before any `pressed` test and every
//!   `accept` is outside its guard. Put the lookup behind `if (pressed)` and
//!   only the release leaks, to the one place it does damage.

use i_slint_backend_testing::ElementQuery;
use pundit_app::keymap::{Action, Keymap};
use slint::platform::{Key, PointerEventButton, WindowEvent};
use slint::{ComponentHandle, PhysicalSize, SharedString};
use std::cell::RefCell;
use std::rc::Rc;

use crate::key_action;
use crate::{AppWindow, MatchTag, RecordingPhase, ScanStep};

/// What a press must produce.
enum Fires {
    /// The line the recorded callback pushes. Every branch but one ends in a
    /// callback, so this is the usual case.
    Callback(&'static str),
    /// `h` has no callback: it flips a window property, because the tool is
    /// the window's own state. Checked by reading it back.
    HighlightTool,
    /// `F1` has none either: it opens the Keys sheet, which is a window
    /// property too (#96 task 4). What the sheet then *shows* is
    /// `keys_sheet.rs`'s; this row's job is the census — that the key reaches
    /// the action at all.
    KeysSheet,
}

/// One default binding, and what pressing it must do.
///
/// **The key and the action are both here and checked against each other**
/// (`binds_what_it_says`), so a default moved in `keymap.rs` fails this file by
/// name rather than by a silent miss, and a row cannot drift into testing a key
/// the table no longer holds.
struct Case {
    action: Action,
    /// The event text, as the backend delivers it: a letter is the *shifted*
    /// reading, so Shift+A arrives as `"A"`, and a named key is its character
    /// from **Slint's own enum**, never a code point written out here.
    text: SharedString,
    ctrl: bool,
    shift: bool,
    /// The window state this branch's gate needs beyond [`base`]'s.
    prepare: fn(&AppWindow),
    fires: Fires,
}

fn case(action: Action, text: impl Into<SharedString>, fires: Fires) -> Case {
    Case {
        action,
        text: text.into(),
        ctrl: false,
        shift: false,
        prepare: |_| {},
        fires,
    }
}

fn fires(action: Action, text: impl Into<SharedString>, line: &'static str) -> Case {
    case(action, text, Fires::Callback(line))
}

fn ctrl(mut c: Case) -> Case {
    c.ctrl = true;
    c
}

fn shift(mut c: Case) -> Case {
    c.shift = true;
    c
}

/// **Every default binding in the table, one row each** — all 34 of them, and
/// the one action it leaves out is named rather than forgotten: `showRecents`
/// ships with no default binding at all, so there is no key to press.
///
/// **A new action belongs here.** Slint's `if` chain over `KeyAction` cannot be
/// made exhaustive and nothing in either language will say a mapped action has
/// no branch (plan Risk 1). This table is the check, and `covers_every_action`
/// is what makes leaving a row out a failure instead of a silence.
fn cases() -> Vec<Case> {
    vec![
        ctrl(fires(Action::OpenProject, "o", "open-project")),
        ctrl(fires(Action::Undo, "z", "undo")),
        ctrl(shift(fires(Action::Redo, "Z", "redo"))),
        ctrl(fires(Action::Redo, "y", "redo")),
        Case {
            // The one branch gated on a *recording*: `can-draw` is the
            // recording phase, so this row is the only one not in `base`'s
            // state.
            prepare: |w| w.set_recording_phase(RecordingPhase::Recording),
            ..fires(Action::ClearDrawings, "c", "clear-drawings")
        },
        fires(Action::MarkIn, "i", "mark-in"),
        fires(Action::MarkOut, "o", "mark-out"),
        fires(Action::TagHomeGoal, "z", "tag home-goal"),
        fires(Action::TagAwayGoal, "x", "tag away-goal"),
        fires(Action::TagPeriod, "v", "tag start-stop"),
        case(Action::HighlightTool, "h", Fires::HighlightTool),
        fires(Action::ToggleRecording, "r", "record \"\""),
        fires(Action::DeleteClip, Key::Delete, "delete \"c1\""),
        fires(Action::TogglePlay, " ", "toggle-play"),
        fires(Action::SkipBack, Key::LeftArrow, "skip -3"),
        fires(Action::SkipBack, "a", "skip -3"),
        fires(Action::SkipForward, Key::RightArrow, "skip 3"),
        fires(Action::SkipForward, "d", "skip 3"),
        shift(fires(Action::SkipBackFar, Key::LeftArrow, "skip -10")),
        shift(fires(Action::SkipBackFar, "A", "skip -10")),
        shift(fires(Action::SkipForwardFar, Key::RightArrow, "skip 10")),
        shift(fires(Action::SkipForwardFar, "D", "skip 10")),
        fires(Action::StepBack, ",", "step back"),
        fires(Action::StepForward, ".", "step forward"),
        fires(Action::PrevEvent, "[", "jump back"),
        fires(Action::NextEvent, "]", "jump forward"),
        fires(Action::ScanSlower, "j", "scan slower"),
        fires(Action::ScanFaster, "l", "scan faster"),
        fires(Action::FitWindow, "f", "fit-window"),
        fires(Action::ZoomReset, "1", "zoom-reset"),
        ctrl(fires(Action::ZoomReset, "0", "zoom-reset")),
        fires(Action::ZoomOut, "2", "zoom -0.25"),
        fires(Action::ZoomIn, "3", "zoom 0.25"),
        case(Action::ShowKeys, Key::F1, Fires::KeysSheet),
    ]
}

/// The one action with no branch in `handle-key`, so `covers_every_action`
/// names it rather than counting to 28. It has no default binding either, and
/// the branch belongs with the task that can first bind it (#96 task 5).
const UNWIRED: [Action; 1] = [Action::ShowRecents];

/// Every window callback a key can reach, pushed onto one list in the order
/// they fire. One list rather than a flag each: a key that fires the *wrong*
/// action is the failure a per-callback counter hides.
fn window() -> (AppWindow, Rc<RefCell<Vec<String>>>) {
    i_slint_backend_testing::init_no_event_loop();
    let w = AppWindow::new().unwrap();
    key_action::wire_keys(&w, Keymap::defaults());

    let fired: Rc<RefCell<Vec<String>>> = Rc::default();
    let say = |what: &'static str| {
        let fired = Rc::clone(&fired);
        move || fired.borrow_mut().push(what.to_owned())
    };
    w.on_open_project(say("open-project"));
    w.on_undo(say("undo"));
    w.on_redo(say("redo"));
    w.on_clear_drawings(say("clear-drawings"));
    w.on_mark_in(say("mark-in"));
    w.on_mark_out(say("mark-out"));
    w.on_toggle_play(say("toggle-play"));
    w.on_fit_window(say("fit-window"));
    w.on_zoom_reset(say("zoom-reset"));
    w.on_tag_match_event({
        let fired = Rc::clone(&fired);
        move |kind| {
            let which = match kind {
                MatchTag::HomeGoal => "home-goal",
                MatchTag::AwayGoal => "away-goal",
                MatchTag::StartStop => "start-stop",
            };
            fired.borrow_mut().push(format!("tag {which}"));
        }
    });
    w.on_toggle_recording({
        let fired = Rc::clone(&fired);
        move |slate| fired.borrow_mut().push(format!("record {slate:?}"))
    });
    w.on_delete_clip({
        let fired = Rc::clone(&fired);
        move |clip| fired.borrow_mut().push(format!("delete {clip:?}"))
    });
    w.on_skip({
        let fired = Rc::clone(&fired);
        move |by| fired.borrow_mut().push(format!("skip {by}"))
    });
    w.on_step_frame({
        let fired = Rc::clone(&fired);
        move |forward| fired.borrow_mut().push(format!("step {}", way(forward)))
    });
    w.on_jump_chapter({
        let fired = Rc::clone(&fired);
        move |forward| fired.borrow_mut().push(format!("jump {}", way(forward)))
    });
    w.on_step_scan_speed({
        let fired = Rc::clone(&fired);
        move |step| {
            let which = match step {
                ScanStep::Faster => "faster",
                ScanStep::Slower => "slower",
                ScanStep::Cycle => "cycle",
            };
            fired.borrow_mut().push(format!("scan {which}"));
        }
    });
    w.on_zoom_step({
        let fired = Rc::clone(&fired);
        move |by, _hover, _x, _y| fired.borrow_mut().push(format!("zoom {by}"))
    });
    w.show().unwrap();
    (w, fired)
}

fn way(forward: bool) -> &'static str {
    if forward {
        "forward"
    } else {
        "back"
    }
}

/// The state every gate but `can-draw`'s is satisfied by: footage on screen and
/// playing, a clip selected, nothing recording and nothing previewing.
///
/// Re-applied before each row, so one row's `prepare` cannot leak into the next
/// — the clear-drawings row leaves a recording running, which would refuse
/// undo, redo, delete and open, and the `showKeys` row leaves a **modal sheet**
/// up, which would swallow every key after it.
fn base(w: &AppWindow) {
    w.set_keys_sheet_open(false);
    w.set_can_play(true);
    w.set_playing(true);
    w.set_can_fit(true);
    w.set_previewing_clip(SharedString::new());
    w.set_recording_phase(RecordingPhase::Idle);
    w.set_selected_clip("c1".into());
    w.set_selected_slate(SharedString::new());
    w.set_highlight_tool(false);
}

/// A press and its release, with the modifiers held across both as a keyboard
/// holds them: Slint reads a modifier off the modifier key's own event, so
/// `Ctrl+O` is four events and not one carrying a flag.
fn press(w: &AppWindow, text: &SharedString, ctrl: bool, shift: bool) {
    let mods: Vec<SharedString> = [(ctrl, Key::Control), (shift, Key::Shift)]
        .into_iter()
        .filter(|(on, _)| *on)
        .map(|(_, key)| SharedString::from(key))
        .collect();
    for m in &mods {
        w.window()
            .dispatch_event(WindowEvent::KeyPressed { text: m.clone() });
    }
    w.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    w.window()
        .dispatch_event(WindowEvent::KeyReleased { text: text.clone() });
    for m in mods.iter().rev() {
        w.window()
            .dispatch_event(WindowEvent::KeyReleased { text: m.clone() });
    }
}

/// **The table and the test agree about which key is which action.** Cheap, and
/// it is what stops a row drifting into pressing a key the defaults no longer
/// hold — which would leave the row green and the action untested.
#[test]
fn binds_what_it_says() {
    let keymap = Keymap::defaults();
    for c in cases() {
        assert_eq!(
            keymap.action_for(&c.text, c.ctrl, c.shift, false),
            Some(c.action),
            "{:?}'s row presses {:?}, which the defaults give to something else",
            c.action,
            c.text
        );
    }
}

/// **Every action with a branch has a row.** The pairing Slint cannot check: an
/// action mapped in `key_action.rs` with no branch in `handle-key` is a listed
/// key that silently does nothing, and this is the one place that is caught.
#[test]
fn covers_every_action() {
    let cases = cases();
    for action in Action::ALL {
        let tested = cases.iter().any(|c| c.action == action);
        if UNWIRED.contains(&action) {
            assert!(
                !tested,
                "{action:?} has a row, so it is wired now and does not belong in UNWIRED"
            );
            continue;
        }
        assert!(
            tested,
            "{action:?} has no row here, so nothing checks that its key does anything"
        );
    }
}

/// **Every default binding still does what it did**, pressed on the real
/// window: 33 of the table's 34, one row each.
///
/// The failure this exists for is a crossed arm in `key_action.rs`'s map or a
/// missing branch in `handle-key` — both of which leave the app building, the
/// key listed and nothing happening.
#[test]
fn every_default_binding_still_does_what_it_did() {
    let (w, fired) = window();
    for c in cases() {
        base(&w);
        (c.prepare)(&w);
        fired.borrow_mut().clear();
        let before = w.get_highlight_tool();

        press(&w, &c.text, c.ctrl, c.shift);

        match c.fires {
            Fires::Callback(line) => assert_eq!(
                *fired.borrow(),
                [line.to_owned()],
                "{:?} on {:?}",
                c.action,
                c.text
            ),
            Fires::HighlightTool => {
                assert!(fired.borrow().is_empty(), "{:?} fired a callback", c.action);
                assert_ne!(
                    w.get_highlight_tool(),
                    before,
                    "{:?} on {:?} did not toggle the tool",
                    c.action,
                    c.text
                );
            }
            Fires::KeysSheet => {
                assert!(fired.borrow().is_empty(), "{:?} fired a callback", c.action);
                assert!(
                    w.get_keys_sheet_open(),
                    "{:?} on {:?} did not open the Keys sheet",
                    c.action,
                    c.text
                );
            }
        }
    }
}

/// **A shifted letter fires nothing** — spec G2's named cost, pinned at the
/// window rather than only in `keymap.rs`'s matcher.
///
/// The three modifiers are compared exactly, so `Shift+R` is not `r`: a coach
/// who wants the far skip on a letter gets `Shift+A`, and nothing else gains a
/// shifted twin. **Caps Lock is a different matter and stays free** — the
/// backend reports `"R"` with `shift: false` under it, so the matcher's
/// lowercasing is what keeps `r` recording, and `keymap.rs` is where that is
/// pinned.
#[test]
fn a_shifted_letter_fires_nothing() {
    let (w, fired) = window();
    base(&w);
    for text in ["R", "Z", "F", "C", "V", "I", "O"] {
        fired.borrow_mut().clear();
        press(&w, &text.into(), false, true);
        assert!(
            fired.borrow().is_empty(),
            "Shift+{text} fired {:?}",
            fired.borrow()
        );
    }
}

/// **An arrow never reaches a touched slider, on the way up or the way down**
/// (spec D10). The reason every shortcut is handled in the *capture* phase, and
/// the reason the lookup runs before any `pressed` test.
///
/// The volume slider is the one control in this window that answers an arrow,
/// and it answers on both halves: `SliderBase`'s `key-pressed` decrements and
/// its `key-released` fires `released(value)`. So a coach who nudged the volume
/// with the mouse and then pressed `←` to go back three seconds would hear the
/// volume drop instead — which is what `handle-key` running first prevents.
///
/// **The click is also what makes this test non-vacuous.** `SliderBase`'s
/// `TouchArea` calls `focus-scope.focus()` from its own pointer handler, so one
/// left click both moves the value and takes the keyboard focus. The two
/// assertions on that click are therefore the proof the slider is focused: with
/// no focus, nothing below could fire whatever `handle-key` did.
#[test]
fn an_arrow_never_reaches_a_touched_slider() {
    let (w, fired) = window();
    base(&w);
    // `ElementHandle` skips a subtree its geometry puts outside the enclosing
    // clip, so the transport row has to be on screen to be clickable at all.
    w.window().set_size(PhysicalSize::new(1920, 2400));
    let volume: Rc<RefCell<Vec<String>>> = Rc::default();
    w.on_volume_changed({
        let volume = Rc::clone(&volume);
        move |v| volume.borrow_mut().push(format!("changed {v}"))
    });
    w.on_volume_released({
        let volume = Rc::clone(&volume);
        move |v| volume.borrow_mut().push(format!("released {v}"))
    });

    // **By id, not by type.** `match_inherits("Slider")` answers *two*
    // elements for the one `Slider` in `app.slint` -- the widget and the
    // element inside it that the style builds it from -- and which of the two
    // a click lands on is not a thing to leave to `find_first`.
    let sliders = ElementQuery::from_root(&w)
        .match_id("AppWindow::volume-slider")
        .find_all();
    let [slider] = sliders.as_slice() else {
        panic!(
            "expected the transport's one volume slider, found {}",
            sliders.len()
        );
    };
    slider.mock_single_click(PointerEventButton::Left);
    assert!(
        volume.borrow().iter().any(|l| l.starts_with("changed")),
        "the click never reached the slider, so nothing below is tested: {:?}",
        volume.borrow()
    );
    assert!(
        volume.borrow().iter().any(|l| l.starts_with("released")),
        "the click never released on the slider: {:?}",
        volume.borrow()
    );

    volume.borrow_mut().clear();
    fired.borrow_mut().clear();
    let left = SharedString::from(Key::LeftArrow);
    w.window()
        .dispatch_event(WindowEvent::KeyPressed { text: left.clone() });
    w.window()
        .dispatch_event(WindowEvent::KeyReleased { text: left });

    assert_eq!(
        *fired.borrow(),
        ["skip -3".to_owned()],
        "the left arrow should skip, once, on the press alone"
    );
    assert!(
        volume.borrow().is_empty(),
        "the arrow reached the slider: {:?}. The press is swallowed by the \
         branch; the **release** is swallowed only because the lookup runs \
         before any `pressed` test",
        volume.borrow()
    );
}

/// **A gate refuses the action and the key is swallowed anyway.** `can-fit`
/// false greys the button and silences the key, and the `accept` stays outside
/// the guard so the key does not fall through to whatever has focus.
///
/// Only the first half is observable for `f`: nothing in this window answers
/// it, so there is no child to watch it land on. The fall-through half is
/// tested where it can be — `an_arrow_never_reaches_a_touched_slider`, where
/// the release passes no guard at all and is swallowed regardless.
#[test]
fn a_refused_gate_still_swallows_the_key() {
    let (w, fired) = window();
    base(&w);
    w.set_can_fit(false);
    press(&w, &"f".into(), false, false);
    assert!(
        fired.borrow().is_empty(),
        "f fitted a window with nothing to take off: {:?}",
        fired.borrow()
    );
}
