//! The player during a whiteboard take (BACKLOG #138, plan task 4) against the
//! **real** `AppWindow`, on Slint's headless backend.
//!
//! The claim is that the player takes the diagram's **shape**, not just its
//! picture: strokes are normalised against the content rect, and the export
//! replays them in the diagram's, so a diagram drawn in the footage's 16:9
//! rect would put every stroke somewhere else — while looking nearly right.
//! The inset is placed over that same rect, so it follows for free.
//!
//! `main.rs`'s `Starting { whiteboard }` arm, which sets the flag, is a
//! `[[bin]]` no test reaches (`fit_window.rs` records the limit).

use pundit_app::zoom_input::Viewport;
use pundit_core::zoom::Zoom;
use slint::{ComponentHandle, ModelRc, PhysicalSize, SharedPixelBuffer, VecModel};

use crate::{AppWindow, LiveHighlight, PictureRect};

/// The window with the production letterbox (`main.rs`'s `place-picture`
/// verbatim — an unwired one answers 0×0 and every assertion below would hold
/// on nothing) and 16:9 footage on screen.
fn window() -> AppWindow {
    i_slint_backend_testing::init_no_event_loop();
    let w = AppWindow::new().unwrap();
    w.on_place_picture(|zoom, frame_w, frame_h, area_w, area_h| {
        let zoom = Zoom::new(zoom.scale.into(), zoom.pan_x.into(), zoom.pan_y.into());
        let Some(vp) = Viewport::new(frame_w.into(), frame_h.into(), area_w.into(), area_h.into())
        else {
            return PictureRect::default();
        };
        let r = vp.picture(zoom);
        PictureRect {
            x: r.x as f32,
            y: r.y as f32,
            width: r.width as f32,
            height: r.height as f32,
        }
    });
    w.set_has_project(true);
    w.set_can_play(true);
    w.set_frame_width(1920.0);
    w.set_frame_height(1080.0);
    w.show().unwrap();
    w.window().set_size(PhysicalSize::new(1600, 960));
    w
}

/// A 4:3 diagram, which is the shape that makes a wrong rect visible: against
/// 16:9 footage it pillarboxes where the footage letterboxes.
fn diagram() -> slint::Image {
    slint::Image::from_rgba8(SharedPixelBuffer::new(400, 300))
}

fn aspect(w: &AppWindow) -> f32 {
    let (width, height) = (w.get_content_width(), w.get_content_height());
    assert!(
        width > 0.0 && height > 0.0,
        "no content rect: {width}x{height}"
    );
    width / height
}

/// The diagram's shape while it is shown, the footage's when it is not — and
/// **the diagram being armed is not enough**: it is only the take's. A take
/// with nothing decoded to show keeps the footage's shape rather than
/// letterboxing a 0×0 picture.
#[test]
fn the_content_rect_is_the_diagrams_for_the_take_alone() {
    let w = window();
    let footage = aspect(&w);
    assert!((footage - 16.0 / 9.0).abs() < 0.01, "{footage}");

    w.set_whiteboard_take(true);
    assert!((aspect(&w) - footage).abs() < 0.01, "nothing to show");
    w.set_whiteboard_take(false);

    w.set_backdrop(diagram());
    assert!((aspect(&w) - footage).abs() < 0.01, "armed is not shown");

    w.set_whiteboard_take(true);
    assert_eq!(
        (w.get_picture_width(), w.get_picture_height()),
        (400.0, 300.0)
    );
    let board = aspect(&w);
    assert!((board - 4.0 / 3.0).abs() < 0.01, "{board}");

    w.set_whiteboard_take(false);
    assert!(
        (aspect(&w) - footage).abs() < 0.01,
        "the footage's shape is back"
    );
}

/// A ring belongs to the footage, which is behind the diagram — and the
/// export draws none on a whiteboard entry — so the live layer hides them,
/// and the H tool is off, since a drag on a diagram is the pen's. Found by its
/// label, which a `HighlightLabel` exposes as a `Text`.
#[test]
fn the_rings_and_the_highlight_tool_are_off_over_the_diagram() {
    let w = window();
    w.set_highlight_tool(true);
    assert!(w.get_in_highlight_tool());
    w.set_live_highlights(ModelRc::new(VecModel::from(vec![LiveHighlight {
        label: "17".into(),
        ..LiveHighlight::default()
    }])));
    let ring = |w: &AppWindow| {
        i_slint_backend_testing::ElementHandle::find_by_accessible_label(w, "17").count()
    };
    assert_eq!(ring(&w), 1, "the ring is there over the footage");
    w.set_backdrop(diagram());
    w.set_whiteboard_take(true);
    assert_eq!(ring(&w), 0, "and gone over the diagram");
    assert!(!w.get_in_highlight_tool(), "the drag is the pen's");
}

/// **The popover's Whiteboard row is wired** (plan task 5): it names what is
/// armed beside its thumbnail, and its Remove reaches `disarm-backdrop`. Found
/// by the words on it and pressed with the accessible action, as
/// `export_sheet.rs` does — the popover opens from the real Devices button.
///
/// The avatar's row is the same component, so with no avatar picked its own
/// Remove is absent and the one on screen can only be the diagram's. Its
/// Choose… is not absent, so both are pressed and each callback must fire
/// **once** — which catches one left unwired and the two crossed alike.
#[test]
fn the_popovers_whiteboard_row_shows_the_arm_and_disarms() {
    use i_slint_backend_testing::ElementHandle;
    use std::cell::Cell;
    use std::rc::Rc;

    let w = window();
    let disarmed = Rc::new(Cell::new(false));
    w.on_disarm_backdrop({
        let disarmed = Rc::clone(&disarmed);
        move || disarmed.set(true)
    });
    let chosen = Rc::new(Cell::new((0, 0)));
    w.on_choose_avatar({
        let chosen = Rc::clone(&chosen);
        move || chosen.set((chosen.get().0 + 1, chosen.get().1))
    });
    w.on_choose_backdrop({
        let chosen = Rc::clone(&chosen);
        move || chosen.set((chosen.get().0, chosen.get().1 + 1))
    });
    w.set_backdrop(diagram());
    w.set_backdrop_name("pitch.png".into());
    let press = |label: &str| {
        let found: Vec<_> = ElementHandle::find_by_accessible_label(&w, label).collect();
        assert_eq!(found.len(), 1, "one {label:?} on screen");
        found[0].invoke_accessible_default_action();
    };
    press("Devices…");
    assert_eq!(
        ElementHandle::find_by_accessible_label(&w, "pitch.png").count(),
        1,
        "the armed file is named"
    );
    for choose in ElementHandle::find_by_accessible_label(&w, "Choose…") {
        choose.invoke_accessible_default_action();
    }
    assert_eq!(chosen.get(), (1, 1), "(avatar, diagram) choices");
    press("Remove");
    assert!(disarmed.get(), "Remove reached disarm-backdrop");
}

/// **A whiteboard clip previews with its game video missing** (task 3's
/// review, closed here): the bus allows it, and the window used to grey
/// Preview whenever any half was missing, so the coach could never ask. The
/// inspector's button follows the selected clip; a footage clip in the same
/// project stays greyed. **Not reached:** the clip-list menu's own use of
/// `can-preview(clip.whiteboard)`, and `main.rs` seeding `ClipRow::whiteboard`
/// and `clip-whiteboard` — a `[[bin]]` no test reaches.
#[test]
fn preview_is_offered_for_a_whiteboard_clip_with_the_footage_missing() {
    use i_slint_backend_testing::ElementHandle;

    let w = window();
    // What `show_project` sets with a half missing.
    w.set_can_play(false);
    w.set_selected_clip("c1".into());
    let preview = |w: &AppWindow| {
        let found: Vec<_> = ElementHandle::find_by_accessible_label(w, "Preview").collect();
        assert_eq!(found.len(), 1, "one Preview button");
        found[0].accessible_enabled()
    };
    assert_eq!(preview(&w), Some(false), "a footage clip needs its footage");
    w.set_clip_whiteboard(true);
    assert_eq!(
        preview(&w),
        Some(true),
        "a whiteboard clip needs its diagram"
    );
}
