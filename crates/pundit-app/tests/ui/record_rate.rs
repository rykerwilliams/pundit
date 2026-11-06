//! The recorded playback-speed control (#140 task 7, spec §R): a small cycle
//! button in the recording transport that slows the game footage so the coach
//! talks over slow motion. The commentary stays real time; the slow rate is
//! baked into the clip's edit by the bus and walked into export/preview.
//!
//! **What `tests/ui` can reach is the window's own presentation**, not the
//! bus: the control is shown only while recording, and cycling it steps the
//! window's `record-rate` through the slow-motion ladder (1 → 0.75 → 0.5 →
//! 0.25 → 1). The bus honouring that rate — the footage actually playing slow,
//! the `Play` event carrying it — is the harness's `recording.rs`. Each
//! `#[test]` shares the process and inits the backend for itself
//! (`tests/ui/main.rs`'s note).

use crate::{one_source, AppWindow, RecordingPhase};
use i_slint_backend_testing::ElementHandle;
use slint::ComponentHandle;

/// A window with a project and a source, so the transport row is live.
fn transport() -> AppWindow {
    let w = AppWindow::new().unwrap();
    w.set_has_project(true);
    w.set_can_play(true);
    one_source(&w);
    w.show().unwrap();
    w
}

/// The recording speed button, by its distinct accessible label (the scan
/// button beside Play can read the same "1×").
fn speed_button(w: &AppWindow) -> Option<ElementHandle> {
    ElementHandle::find_by_accessible_label(w, &format!("Recording speed {}×", w.get_record_rate()))
        .next()
}

/// The control is in the recording transport, and only while recording: scan
/// speed owns the viewing rate the rest of the time (§R4), so the two never
/// share the row.
#[test]
fn the_speed_control_shows_only_while_recording() {
    i_slint_backend_testing::init_no_event_loop();
    let w = transport();

    w.set_recording_phase(RecordingPhase::Idle);
    assert!(
        speed_button(&w).is_none(),
        "the recording speed control is shown outside a take"
    );

    w.set_recording_phase(RecordingPhase::Recording);
    assert!(
        speed_button(&w).is_some(),
        "the recording speed control is missing during a take"
    );
}

/// Cycling the control steps `record-rate` through the slow-motion ladder and
/// wraps back to 1×. That property is what `on_toggle_play` reads and sends to
/// the bus (caller-captured per the bus contract), so proving it flows here is
/// proving the value the command carries.
#[test]
fn cycling_the_control_steps_the_recorded_rate() {
    i_slint_backend_testing::init_no_event_loop();
    let w = transport();
    w.set_recording_phase(RecordingPhase::Recording);

    assert_eq!(w.get_record_rate(), 1.0);
    for expected in [0.75_f32, 0.5, 0.25, 1.0] {
        speed_button(&w)
            .expect("the recording speed control")
            .invoke_accessible_default_action();
        assert_eq!(
            w.get_record_rate(),
            expected,
            "the ladder did not reach {expected}×"
        );
    }
}
