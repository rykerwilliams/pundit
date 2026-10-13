//! What `pundit-core`'s test files share.

use pundit_core::project::{Clip, Inset, InsetCorner, InsetSize};
use uuid::Uuid;

/// A `Clip` for a test helper to spread: `Clip { <its own fields>, ..clip_stub() }`
/// (BACKLOG #106).
///
/// It fixes what every helper in these tests agreed on — a camera inset shown
/// at the default size and corner, no notes, no transcript, no slate, no
/// backdrop, first in the order, on the first video — and leaves the rest as
/// placeholders (a nil id, empty strings and lists, zero times) for a helper to
/// override with its own. **A new `Clip` field goes here and in
/// `project_format.rs`'s `sample_clip`, and nowhere else in these tests**:
/// before this existed every format field cost a hand edit in every test file
/// that built a `Clip`, setting it to the value an older file means.
///
/// **This is not a `Default for Clip` in production code**, which is the
/// tempting shortcut: a document struct that can be built with a nil `Uuid` and
/// an empty recording file name is a worse hazard than the toll this removes,
/// and `Project::add_recorded_clip` is deliberately the one constructor.
pub fn clip_stub() -> Clip {
    Clip {
        id: Uuid::nil(),
        name: String::new(),
        notes: String::new(),
        tags: Vec::new(),
        source_index: 0,
        start_source_seconds: 0.0,
        recording_duration: 0.0,
        recording_filename: String::new(),
        events: Vec::new(),
        show_pip: true,
        inset: Inset::Camera,
        inset_size: InsetSize::Medium,
        inset_corner: InsetCorner::BottomRight,
        sort_index: 0,
        created_at: String::new(),
        transcript: String::new(),
        slate_id: None,
        backdrop: None,
    }
}
