//! File pickers: the desktop's own dialogs through `rfd`'s XDG portal backend,
//! awaited on the UI thread's event loop (`slint::spawn_local`), parented to
//! the window. Never the blocking dialog: it would stall the event loop and
//! with it the video.

use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;

use pundit_core::naming::order_videos;
use rfd::AsyncFileDialog;
use slint::ComponentHandle;

use crate::AppWindow;

/// What a picker looks for.
pub enum Pick {
    /// A project folder; `title` names what it's for, because this flow opens
    /// it twice for two different reasons — the project to open, and where
    /// projects go (new match spec W3).
    ProjectFolder { title: &'static str },
    /// A video file; `title` names what it's for.
    Video { title: &'static str },
    /// One or more video files at once: a game is often several camera files.
    Videos { title: &'static str },
    /// A still image for the avatar (avatar spec A1). The filter offers PNG
    /// and JPEG; what is *accepted* is what `media::decode_still` decodes, so
    /// a `.png` that is really something else is refused by the bus, not here.
    Image { title: &'static str },
}

/// Video extensions offered by default. Both cases: a portal's glob match
/// may be case-sensitive, and cameras write `.MP4`.
const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "MP4", "mov", "MOV", "m4v", "M4V", "mkv", "MKV", "webm", "WEBM", "avi", "AVI", "mts",
    "MTS",
];

/// Avatar image extensions, both cases for the same reason.
const IMAGE_EXTENSIONS: &[&str] = &["png", "PNG", "jpg", "JPG", "jpeg", "JPEG"];

/// Opens one picker at a time, answering with each chosen path.
#[derive(Clone, Default)]
pub struct Pickers {
    /// A picker is up: further requests are ignored rather than stacked.
    busy: Rc<Cell<bool>>,
}

impl Pickers {
    /// Shows the picker and calls `then` on the UI thread with each path the
    /// user chose — at most once for a single pick, once per file for
    /// [`Pick::Videos`], in [`order_videos`]'s order. Returns at once, and does
    /// not call `then` at all when the dialog is cancelled.
    ///
    /// **Name order, but not a byte-wise one** (`core::naming::order_videos`): a
    /// portal returns a multiple selection in no promised order, and camera
    /// files are named by when they were shot, so ordering by name is what puts
    /// a game's halves in sequence — except that a ` (n)` copy suffix sorts
    /// *before* no suffix, which silently reversed them. That rule is in core so
    /// it is tested without a picker.
    pub fn open(&self, window: &AppWindow, pick: Pick, mut then: impl FnMut(PathBuf) + 'static) {
        self.open_many(window, pick, move |paths| {
            for path in paths {
                then(path);
            }
        });
    }

    /// Shows the picker and calls `then` **once**, on the UI thread, with the
    /// whole chosen set in [`order_videos`]'s order — and **not at all** when
    /// the dialog is cancelled, exactly as [`Pickers::open`] calls back not at
    /// all.
    ///
    /// **A second method rather than having this flow's caller collect the
    /// paths**: [`Pickers::open`] calls `then` once per path and nothing fires
    /// afterwards, so there is no defined moment at which the set is complete,
    /// and relying on the callbacks draining inside one `spawn_local` task
    /// would be accidental correctness.
    ///
    /// **And not by changing [`Pickers::open`]'s signature to a `Vec`**: a
    /// cancelled dialog would then call back with an empty one, which the
    /// project-folder, single-video, relink and avatar pickers would all have
    /// to learn to ignore — and three of those raise an error on it.
    ///
    /// This is both shapes' body: one picker at a time, awaited on the event
    /// loop, the paths ordered, and `then` skipped entirely on an empty
    /// selection — which is what a cancel is.
    pub fn open_many(
        &self,
        window: &AppWindow,
        pick: Pick,
        then: impl FnOnce(Vec<PathBuf>) + 'static,
    ) {
        if self.busy.replace(true) {
            return;
        }
        let dialog = AsyncFileDialog::new().set_parent(&window.window().window_handle());
        let busy = self.busy.clone();
        let videos = |dialog: AsyncFileDialog, title| {
            dialog
                .set_title(title)
                .add_filter("Video", VIDEO_EXTENSIONS)
                .add_filter("All files", &["*"])
        };
        let spawned = slint::spawn_local(async move {
            let chosen: Vec<_> = match pick {
                Pick::ProjectFolder { title } => dialog
                    .set_title(title)
                    .pick_folder()
                    .await
                    .into_iter()
                    .collect(),
                Pick::Video { title } => videos(dialog, title)
                    .pick_file()
                    .await
                    .into_iter()
                    .collect(),
                Pick::Videos { title } => {
                    videos(dialog, title).pick_files().await.unwrap_or_default()
                }
                Pick::Image { title } => dialog
                    .set_title(title)
                    .add_filter("Image", IMAGE_EXTENSIONS)
                    .add_filter("All files", &["*"])
                    .pick_file()
                    .await
                    .into_iter()
                    .collect(),
            };
            busy.set(false);
            let mut paths: Vec<PathBuf> = chosen.iter().map(|c| c.path().to_path_buf()).collect();
            if paths.is_empty() {
                return;
            }
            order_videos(&mut paths);
            then(paths);
        });
        if let Err(e) = spawned {
            eprintln!("ui: could not show the file picker: {e}");
            self.busy.set(false);
        }
    }
}
