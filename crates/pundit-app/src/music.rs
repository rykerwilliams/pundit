//! A goals reel's music folder (BACKLOG #84, spec §M1): which genres it holds
//! and which tracks each genre has.
//!
//! **One rule, read by two callers**: the window lists genres to fill the
//! export sheet's Music picker, and the bus lists a genre's tracks to choose
//! one. Both read the disk each time — a `read_dir`, cheap on either thread —
//! so a folder added while the app runs appears on the next look.

use std::path::{Path, PathBuf};

/// The extensions a track may have, lower-cased. They filter the listing, so a
/// `cover.jpg` is never chosen; they are not the decoder's gate, which is
/// `decodebin3`'s.
pub(crate) const EXTENSIONS: [&str; 7] = ["mp3", "ogg", "oga", "opus", "flac", "wav", "m4a"];

/// The genres in `folder`: its immediate, non-hidden subfolders that hold at
/// least one track, sorted. Files at the folder's top level are not a genre —
/// a genre the coach did not name would be a guess. An unreadable folder has
/// none. **Symlinks are followed**, for a folder and a song alike: the folder is
/// the coach's own, and one built from links into `~/Music` must list.
pub fn genres(folder: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return Vec::new();
    };
    let mut genres: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|name| !name.starts_with('.'))
        .filter(|name| track_files(&folder.join(name)).next().is_some())
        .collect();
    genres.sort();
    genres
}

/// The tracks of `genre` in `folder`, sorted by file name — **only if
/// [`genres`] would list it**. The genre comes from `state.json`, which a hand
/// edit can make anything; being listed is the whole check, so `../x`, an
/// absolute path, a hidden or an empty folder all read nothing.
pub fn tracks(folder: &Path, genre: &str) -> Vec<PathBuf> {
    if !genres(folder).iter().any(|g| g == genre) {
        return Vec::new();
    }
    let mut tracks: Vec<PathBuf> = track_files(&folder.join(genre)).collect();
    tracks.sort();
    tracks
}

/// The Music picker's first row, which is no music at all.
pub const NONE: &str = "None";

/// What the export sheet's Music picker shows (spec §M2): its rows — [`NONE`]
/// then the folder's `genres` — and the row selected.
#[derive(Debug, PartialEq, Eq)]
pub struct Picker {
    pub rows: Vec<String>,
    pub index: usize,
    /// A genre was remembered that `genres` no longer lists. The caller clears
    /// it, so the None on screen is the None the bus reads: the bus refuses a
    /// genre it cannot honour, and a refusal over music the sheet does not show
    /// would be one the coach cannot act on.
    pub stale: bool,
}

/// The picker for `genres` with `remembered` selected.
pub fn picker(genres: Vec<String>, remembered: Option<&str>) -> Picker {
    let found = remembered.and_then(|g| genres.iter().position(|x| x == g));
    let mut rows = vec![NONE.to_owned()];
    rows.extend(genres);
    Picker {
        rows,
        index: found.map_or(0, |i| i + 1),
        stale: remembered.is_some() && found.is_none(),
    }
}

/// The genre at picker row `index` of `rows`: none for [`NONE`]'s row, and none
/// for an index off the list (a `ComboBox` reports -1 for no selection).
pub fn genre_at(rows: &[String], index: i32) -> Option<String> {
    let i = usize::try_from(index).ok().filter(|&i| i > 0)?;
    rows.get(i).cloned()
}

/// The files in `dir` with a listed extension, in directory order.
fn track_files(dir: &Path) -> impl Iterator<Item = PathBuf> {
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .filter(|p| {
            p.extension()
                .and_then(|x| x.to_str())
                .is_some_and(|x| EXTENSIONS.contains(&x.to_ascii_lowercase().as_str()))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(path: &Path) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, b"").unwrap();
    }

    #[test]
    fn a_genre_is_a_visible_subfolder_with_a_track() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        touch(&root.join("rock/b.mp3"));
        touch(&root.join("edm/a.OGG"));
        touch(&root.join(".hidden/x.mp3"));
        touch(&root.join("art/cover.jpg"));
        std::fs::create_dir(root.join("empty")).unwrap();
        touch(&root.join("loose.mp3"));
        assert_eq!(genres(root), ["edm", "rock"]);
    }

    #[test]
    fn a_genres_tracks_are_its_listed_files_sorted() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        for name in ["b.flac", "a.mp3", "c.M4A", "notes.txt", "cover.jpg"] {
            touch(&root.join("rock").join(name));
        }
        let names: Vec<String> = tracks(root, "rock")
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["a.mp3", "b.flac", "c.M4A"]);
    }

    /// The genre comes from `state.json`, which a hand edit can make anything:
    /// a path out of the folder reads nothing.
    #[test]
    fn a_genre_that_is_a_path_reads_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("music");
        touch(&dir.path().join("outside/x.mp3"));
        touch(&root.join("rock/a.mp3"));
        assert!(tracks(&root, "../outside").is_empty());
        assert!(tracks(&root, &dir.path().join("outside").to_string_lossy()).is_empty());
        assert!(tracks(&root, "").is_empty());
        assert_eq!(tracks(&root, "rock").len(), 1);
    }

    #[test]
    fn the_picker_puts_none_first_and_selects_the_remembered_genre() {
        let genres = || vec!["edm".to_owned(), "rock".to_owned()];
        let p = picker(genres(), Some("rock"));
        assert_eq!(p.rows, ["None", "edm", "rock"]);
        assert_eq!((p.index, p.stale), (2, false));
        assert_eq!(genre_at(&p.rows, p.index as i32).as_deref(), Some("rock"));
        assert_eq!(picker(genres(), None).index, 0);
    }

    #[test]
    fn a_remembered_genre_no_longer_listed_is_stale_and_shows_none() {
        let p = picker(vec!["edm".to_owned()], Some("rock"));
        assert_eq!((p.index, p.stale), (0, true));
        assert!(picker(Vec::new(), Some("rock")).stale);
    }

    #[test]
    fn none_and_an_index_off_the_list_are_no_genre() {
        let rows = picker(vec!["edm".to_owned()], None).rows;
        assert_eq!(genre_at(&rows, 0), None);
        assert_eq!(genre_at(&rows, -1), None);
        assert_eq!(genre_at(&rows, 2), None);
        assert_eq!(genre_at(&rows, 1).as_deref(), Some("edm"));
    }

    /// A music folder built from links lists like a real one.
    #[test]
    fn symlinked_genres_and_songs_are_followed() {
        let dir = tempfile::tempdir().unwrap();
        let elsewhere = dir.path().join("elsewhere");
        touch(&elsewhere.join("real/a.mp3"));
        let root = dir.path().join("music");
        std::fs::create_dir(&root).unwrap();
        std::os::unix::fs::symlink(elsewhere.join("real"), root.join("rock")).unwrap();
        std::fs::create_dir(root.join("edm")).unwrap();
        std::os::unix::fs::symlink(elsewhere.join("real/a.mp3"), root.join("edm/b.mp3")).unwrap();
        assert_eq!(genres(&root), ["edm", "rock"]);
        assert_eq!(tracks(&root, "edm").len(), 1);
    }
}
