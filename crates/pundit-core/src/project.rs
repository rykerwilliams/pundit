//! The project document.
//!
//! A project is a folder: `project.json` plus a `recordings/` subdirectory of
//! commentary `.mkv` files. Sources are referenced, never copied.
//!
//! **Field-level `#[serde(default)]` is a hazard on an `f64` or a `bool`**: it
//! resolves to `Default::default()`, which is `0.0` and `false`, so applying it
//! per field would silently mute every volume and turn PiP off.
//! `Preferences` puts `default` on the container instead, which fills from its
//! own `Default` impl. On an `Option` or a `Vec` a field-level default is
//! exactly right: `None` and empty are what an older file, written before the
//! field existed, means (spec F2). So a field added to an existing struct is an
//! `Option` or a `Vec` with a field-level default, and a new struct's fields
//! get none — or, since the reason is the rule, any type whose `Default` is
//! what an older file means, which is why `Inset` defaults to `Camera`. And only
//! genuinely optional keys get a default at all: defaulting `clips` would let a
//! truncated `project.json` load as an empty project, after which the next save
//! destroys the user's work.
//!
//! **`Option` is serde's own exception to "a new struct's fields get none"**: a
//! missing field of type `Option` reads as `None` rather than failing, whatever
//! the struct says. `Slate::out_seconds` leans on that and is harmless, because
//! an open range is a legal state — but a new struct whose `Option` means
//! something load-bearing will not get the malformed-file error this header
//! otherwise promises, so say what a missing one means.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::event::CommentaryEvent;
use crate::highlight::PlayerHighlight;
use crate::plan::ScoreboardMode;
use crate::recording::PendingClip;
use crate::scoreboard::{MatchEventRecord, ScoreboardConfig};
use crate::undo::ClipEdit;

/// Export frame size. `source` is deliberately absent — it was ill-defined
/// (undefined for a compilation mixing sources of different dimensions) and
/// partly broken in the macOS original.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Resolution {
    R720,
    #[default]
    R1080,
    R2160,
}

/// What a clip's inset holds: the webcam it was recorded with, or the
/// project's avatar image.
///
/// It is a clip's own fact, recorded at capture, so a project can hold both
/// kinds and each entry of one export renders what it was made with. Not
/// derived from the recording's lack of a video track: a webcam take whose
/// camera died has none either, and deriving would draw the coach's face over
/// a take they recorded on camera — a *wrong* picture, not a missing one
/// (spec B2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Inset {
    #[default]
    Camera,
    Avatar,
}

/// How wide a clip's inset is drawn, as one of three named steps. v13.
///
/// Three named steps rather than a free ratio because this app has no live
/// preview of the composite: a coach dragging a slider is guessing until they
/// export, where each of three steps can be measured against the longest
/// caption the app produces (spec I1). The widths themselves are
/// [`crate::layout::inset_ratio`]'s — a stored enum is a name for a step, not a
/// number, as [`Resolution`] is.
///
/// `Hash`, alone among the three types here: the export keys its avatar
/// textures on the image's path **and** the size, because a pixmap is square
/// and sized by the size alone (`composite::export`). The corner is not in that
/// key, and so needs none.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InsetSize {
    Small,
    #[default]
    Medium,
    Large,
}

/// Which corner a clip's inset is flush into. v13.
///
/// **Top-left is deliberately absent.** The scoreboard is locked into it (the
/// coach, 2026-09-25) and the composite mixes the board *over* the inset, so a
/// top-left inset would corrupt nothing and simply be drawn under the board —
/// a half-hidden face, and a coach left to work out why (spec I2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InsetCorner {
    #[default]
    BottomRight,
    BottomLeft,
    TopRight,
}

/// Where a clip's inset goes and how big: [`Clip::inset_size`] and
/// [`Clip::inset_corner`] as one value.
///
/// **Not serialized** — it is read off the clip, through
/// [`Clip::inset_placement`]. One type threaded through the geometry is less
/// code than two arguments that can be passed in the wrong order, and it gives
/// "Medium in the bottom-right corner", which is every inset the app has ever
/// drawn, a single home in its derived `Default`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InsetPlacement {
    pub size: InsetSize,
    pub corner: InsetCorner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Quality {
    Low,
    #[default]
    Medium,
    High,
}

/// User preferences, persisted with the project.
///
/// `default` is on the **container**, so a missing key is filled from the
/// hand-written `Default` impl below — one copy of the defaults, not two.
/// (Field-level `#[serde(default)]` is the hazard: it resolves to
/// `Default::default()`, i.e. `0.0` and `false`.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Preferences {
    pub scan_volume: f64,
    /// How loud the game video is in an **export**, which the export sheet's
    /// "Mute source audio" writes as `0.0` or `1.0`. v15.
    ///
    /// **Renamed from `previewSourceVolume`**, which the alias still reads: the
    /// old name was always wrong — a preview carries no source audio at all
    /// (BACKLOG #125), so this has only ever reached the file. The alias is
    /// matched verbatim against the camelCase key the container's
    /// `rename_all` produces, and is deserialize-only, so a save writes one
    /// key and the old one self-cleans out of the document.
    ///
    /// **Zero is "no game region", not a gain of zero** — `crate::audio` makes
    /// that call, from the value it is handed rather than from here (spec M2),
    /// so a basket can mute without faking a non-default `Preferences`. The
    /// `f64` is kept over a `bool` because the coach may want a level
    /// (BACKLOG #126), which is then UI work and no format change.
    #[serde(alias = "previewSourceVolume")]
    pub export_source_volume: f64,
    pub preview_commentary_volume: f64,
    pub last_export_resolution: Resolution,
    pub last_export_quality: Quality,
    /// Which way the export sheet last carried the scoreboard, or `None` for
    /// its per-target default ([`crate::plan::default_scoreboard_mode`]). v11.
    pub last_export_scoreboard: Option<ScoreboardMode>,
    /// Whether an export writes its chapters. v16.
    ///
    /// **One switch, both forms**: the `chpl` box inside the file *and* the
    /// `.chapters.txt` beside it. A coach who turns chapters off and still
    /// finds them in the file has been told a half-truth, and the in-file box
    /// is the half he cannot see. "Off" is expressed by clearing
    /// `CompilationPlan::chapters`, which both readers already mean "no
    /// chapters" for — there is no flag in media (spec S1, S2).
    pub last_export_chapters: bool,
    /// Whether an export writes the scoreboard as subtitles. v16.
    ///
    /// Both forms again: the `.srt` beside the file *and* the `tx3g` track
    /// inside a copy, which `composite/copy`'s own header already states as one
    /// decision. "Off" is `ExportJob::cues` as `Some(Vec::new())`, media's own
    /// "no subtitles" — again no flag.
    ///
    /// **Named for `job.cues` and `crate::cues`**, the words the code already
    /// uses for this pair of outputs: `last_export_scoreboard_subtitles` would
    /// read as a qualifier on `last_export_scoreboard` right above it.
    pub last_export_cues: bool,
    /// The inset size and corner the coach last set on a clip, seeding the next
    /// recording ([`Project::add_recorded_clip`]). v13.
    ///
    /// **Sticky last-used, like the three `last_export_*` fields above**, and
    /// that is what gives both readings of #88 out of one mechanism: the clip's
    /// own fields are "fix it afterwards", and this pair — written back
    /// whenever a clip's is changed — is "pick it before", with no field the
    /// coach cannot reach and no UI beyond the clip's two controls (spec I6).
    /// Deliberately **not** modelled on `pip_for_new_recordings` below, which
    /// has no control at all and so is written only by tests.
    pub last_inset_size: InsetSize,
    pub last_inset_corner: InsetCorner,
    /// Stable identifier for the preferred camera: its PipeWire `node.name`.
    /// A hint: if the device is absent at launch the app falls back to the
    /// default **without clearing this**, so the preference is restored if the
    /// device reappears.
    pub preferred_camera_id: Option<String>,
    /// Same semantics as `preferred_camera_id`.
    pub preferred_mic_id: Option<String>,
    pub pip_for_new_recordings: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Preferences {
            scan_volume: 1.0,
            export_source_volume: 1.0,
            preview_commentary_volume: 1.0,
            last_export_resolution: Resolution::R1080,
            last_export_quality: Quality::Medium,
            last_export_scoreboard: None,
            last_export_chapters: true,
            last_export_cues: true,
            last_inset_size: InsetSize::Medium,
            last_inset_corner: InsetCorner::BottomRight,
            preferred_camera_id: None,
            preferred_mic_id: None,
            pip_for_new_recordings: true,
        }
    }
}

/// A referenced source video.
///
/// `duration_seconds` is **the** duration authority. Phase 2's probe writes it
/// back on add and relink; everything else reads it. Two duration sources would
/// let the preview clock and the export clock disagree at EOF for the same
/// clip.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceRef {
    /// Relative to the project folder, POSIX `/` separators. May traverse
    /// `..`. Breaks if the user moves the file; Phase 2 owns relink.
    pub relative_path: String,
    pub display_name: String,
    pub duration_seconds: f64,
    /// Width / height after pixel aspect ratio, stored at probe time.
    ///
    /// Read **only** by the aspect gate ([`Project::check_aspect`]); rendering
    /// uses the live caps. Required rather than defaulted: a `0.0` default
    /// would fail every gate comparison, and no file without it was ever
    /// written outside tests (the field amends the unshipped v7).
    pub display_aspect: f64,
}

/// [`Project::remove_source`] refused because a clip, a match event or a
/// player highlight still points at the source. Silently retargeting them
/// would produce subtly wrong playback, so the user must delete them first.
#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("source {index} is still used by a clip, a match event, a highlight or a slate")]
pub struct SourceReferenced {
    pub index: usize,
}

/// [`Project::check_aspect`] refused: every source in a project shares one
/// display aspect, and `attempted` differs from the project's `existing` one.
#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq)]
#[error("aspect {attempted:.4} does not match the project's {existing:.4}")]
pub struct AspectMismatch {
    pub existing: f64,
    pub attempted: f64,
}

/// The aspect gate's rule for two aspects, with no project in it (macOS
/// `aspectsMatch`): both `> 0.0` and `|a − b| / max(a, b) < 0.005`.
///
/// **A `bool`, not a `Result`:** [`AspectMismatch`] carries both numbers, and a
/// caller of this function is already holding them.
///
/// It is public because the pairwise rule has a caller of its own.
/// [`Project::check_aspect`] gates a candidate against a **stored** source and
/// returns `Ok(())` when there is none — but a set of videos picked for a
/// project that does not exist yet has to be gated against each other, and at
/// that moment there is no `Project` and cannot be: a [`SourceRef`] needs a
/// `relative_path`, which needs a canonical folder nothing has created. A gate
/// written there against `check_aspect` would therefore gate **nothing at
/// all**, and hand-coding the tolerance at the call site would be a second
/// definition of one rule.
pub fn aspects_match(existing: f64, candidate: f64) -> bool {
    existing > 0.0
        && candidate > 0.0
        && (existing - candidate).abs() / existing.max(candidate) < 0.005
}

/// One tagged moment with its commentary recording.
///
/// A clip **is** a recording: `recording_filename` is not optional, so clips
/// only come into existence once capture has produced a file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Clip {
    pub id: Uuid,
    pub name: String,
    pub notes: String,
    pub tags: Vec<String>,

    pub source_index: usize,
    pub start_source_seconds: f64,
    pub recording_duration: f64,

    /// `<uuid>.mkv`, relative to the project's `recordings/` directory.
    pub recording_filename: String,

    pub events: Vec<CommentaryEvent>,
    pub show_pip: bool,
    /// v10. Which inset this clip was recorded with; `show_pip` still decides
    /// whether one is drawn at all. Read through [`Clip::camera_placement`]
    /// and [`Clip::avatar_placement`], never on its own.
    #[serde(default)]
    pub inset: Inset,
    /// v13. How wide this clip's inset is drawn. Read through
    /// [`Clip::inset_placement`], never on its own: `show_pip` still decides
    /// whether an inset is drawn at all.
    #[serde(default)]
    pub inset_size: InsetSize,
    /// v13. Which corner this clip's inset is flush into. Same reading rule as
    /// [`Clip::inset_size`].
    #[serde(default)]
    pub inset_corner: InsetCorner,
    pub sort_index: i64,

    /// RFC3339, opaque. Nothing reads it — ordering is by `sort_index` — so it
    /// is stored as a string rather than justifying a date dependency in a
    /// crate that otherwise needs none.
    pub created_at: String,

    #[serde(default)]
    pub transcript: String,

    /// v12. The [`Slate`] this take was shot from, when it was shot from one:
    /// the clip inherited that slate's name and tags.
    ///
    /// **The link runs this way round so that nothing can dangle.** A slate
    /// asking "have I been shot?" is `clips.iter().any(|c| c.slate_id ==
    /// Some(id))`, which stays right across a delete, an undo and a
    /// re-record — where a pointer stored on the slate would need clearing in
    /// three places and would still outlive a trashed clip.
    #[serde(default)]
    pub slate_id: Option<Uuid>,
}

impl Clip {
    /// Set one field, returning its previous value as the same variant: the
    /// one definition of a field change. Unchanged if the two are equal.
    pub fn set(&mut self, edit: ClipEdit) -> ClipEdit {
        match edit {
            ClipEdit::Name(v) => ClipEdit::Name(std::mem::replace(&mut self.name, v)),
            ClipEdit::Tags(v) => ClipEdit::Tags(std::mem::replace(&mut self.tags, v)),
            ClipEdit::Notes(v) => ClipEdit::Notes(std::mem::replace(&mut self.notes, v)),
            ClipEdit::ShowPip(v) => ClipEdit::ShowPip(std::mem::replace(&mut self.show_pip, v)),
            ClipEdit::InsetSize(v) => {
                ClipEdit::InsetSize(std::mem::replace(&mut self.inset_size, v))
            }
            ClipEdit::InsetCorner(v) => {
                ClipEdit::InsetCorner(std::mem::replace(&mut self.inset_corner, v))
            }
            ClipEdit::Transcript(v) => {
                ClipEdit::Transcript(std::mem::replace(&mut self.transcript, v))
            }
        }
    }

    /// Where this clip's inset goes and how big — or `None` when none is drawn.
    ///
    /// **`None` rather than a placement the caller has to remember to
    /// discard.** `layout::bar_rect` takes exactly this, so a clip with the
    /// inset switched off cuts the caption bar at nothing and keeps the full
    /// width it has always had. An accessor returning a bare
    /// [`InsetPlacement`] would make `clip.map(Clip::inset_placement)` compile
    /// and quietly cut *every* clip's bar at the inset's column, in export and
    /// preview alike.
    ///
    /// **This one is kind-blind, and that is the bar's whole reading of the
    /// inset:** it is `show_pip` alone, because every [`Inset`] is drawn
    /// somewhere, so a clip that shows one shows one whatever it picked. Written
    /// as "camera or avatar" it would read as a claim about the variants that it
    /// cannot make — a third kind nobody drew would still fill this column, and
    /// the or-form would suggest it wouldn't.
    pub fn inset_placement(&self) -> Option<InsetPlacement> {
        self.show_pip.then_some(InsetPlacement {
            size: self.inset_size,
            corner: self.inset_corner,
        })
    }

    // The two halves of one decision, written together so they cannot drift:
    // `show_pip × inset` is interpreted here and nowhere else. Each answers
    // *where* in the same breath as *whether*, because every caller that asks
    // one asks the other — the webcam PiP pad and the avatar's own pad both
    // need a rect, and a bare predicate would send them back for it. They are
    // never both `Some`. `inset_placement` above is the third reading, the
    // kind-blind one, and it belongs to the caption bar alone.
    //
    // **A third `Inset` variant would need a third accessor.** Nothing in this
    // codebase matches `Inset` exhaustively — every read of it is an `==`, the
    // two here and the inspector's one word — so a new variant would simply be
    // `None` from both of these: export falls back to its filler and preview
    // asks for no pad. Safe, and silent.

    /// The webcam PiP pad carries this clip's recording, at this placement.
    pub fn camera_placement(&self) -> Option<InsetPlacement> {
        self.inset_placement()
            .filter(|_| self.inset == Inset::Camera)
    }

    /// The inset pad carries the project's avatar for this clip, at this
    /// placement.
    pub fn avatar_placement(&self) -> Option<InsetPlacement> {
        self.inset_placement()
            .filter(|_| self.inset == Inset::Avatar)
    }
}

/// The project document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub format_version: u32,
    pub name: String,
    pub source_videos: Vec<SourceRef>,
    pub clips: Vec<Clip>,
    #[serde(default)]
    pub preferences: Preferences,
    #[serde(default)]
    pub scoreboard: Option<ScoreboardConfig>,
    #[serde(default)]
    pub match_events: Vec<MatchEventRecord>,
    /// v9. Rings on the footage, not on a clip — see [`crate::highlight`].
    #[serde(default)]
    pub player_highlights: Vec<PlayerHighlight>,
    /// v10. The avatar image's **file name**, relative to the project folder
    /// — never a path. It *is* the mode: `Some` and takes record commentary
    /// only, with that picture for the inset; `None` and they record on
    /// camera, as they always did. There is no second flag to disagree with
    /// it (spec B1).
    #[serde(default)]
    pub avatar: Option<String>,
    /// v12. Ranges marked while watching, waiting for their commentary — see
    /// [`Slate`]. Ordered for reading by [`Project::slates_sorted`]; the stored
    /// order is only the order they were marked in.
    #[serde(default)]
    pub slates: Vec<Slate>,
}

/// A range on the footage, named and tagged, whose commentary has not been
/// recorded yet (spec `2026-09-25-slates-design.md`).
///
/// **A slate is not a clip, and cannot become one by relaxing a rule:** a clip
/// *is* a recording, which is what lets every clip replay, preview and export
/// with no special case. Shooting a slate *produces* a clip, and the link runs
/// from that clip ([`Clip::slate_id`]) so that nothing here can dangle.
///
/// It carries no `created_at` and no `sort_index`. `Clip` has both because
/// clips are hand-ordered and their position is burned into an export's
/// caption; slates are never exported, so the only order that means anything
/// is `(source_index, in_seconds)` — which is also the order a coach marking
/// ranges in one pass produces.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Slate {
    pub id: Uuid,
    pub source_index: usize,
    pub in_seconds: f64,
    /// `None` until the out point is marked. **An open slate is stored**, not
    /// held in the UI: a range half-marked when the app closes is then a
    /// visible row the coach can finish or delete, rather than a press that
    /// vanished. It is also why marking needs no in-progress state to
    /// invalidate when the source list changes.
    pub out_seconds: Option<f64>,
    pub name: String,
    pub tags: Vec<String>,
}

impl Slate {
    /// Whether applying `edit` would leave this slate's out point at or before
    /// its in point (BACKLOG #119) — the question the bus asks before it
    /// mutates, since [`Project::edit_slate`] is infallible.
    ///
    /// **Both directions**, which is the half an earlier draft missed: moving
    /// the *in* point past an existing out is the likelier mistake, and nothing
    /// guarded an in point at all before this. A slate with no out point
    /// constrains nothing, so a half-marked range's in point moves freely.
    ///
    /// The comparison is `<=`, matching [`Project::mark_slate_out`]: an out
    /// point equal to the in point is already refused there, and a zero-length
    /// range is not a range.
    pub fn would_invert(&self, edit: &SlateEdit) -> bool {
        match edit {
            SlateEdit::In(seconds) => self.out_seconds.is_some_and(|out| out <= *seconds),
            SlateEdit::Out(seconds) => *seconds <= self.in_seconds,
            SlateEdit::Name(_) | SlateEdit::Tags(_) => false,
        }
    }
}

/// One field of a [`Slate`], for [`Project::edit_slate`].
///
/// **No `Eq`**, because [`SlateEdit::In`] and [`SlateEdit::Out`] carry an `f64`
/// and `f64` is not `Eq`. Nothing needs it: `edit_slates`' undo step compares
/// whole `Vec<Slate>` by `PartialEq`, and the tests only `assert_eq!`.
#[derive(Debug, Clone, PartialEq)]
pub enum SlateEdit {
    Name(String),
    /// Already normalized by [`crate::tag::normalize_tags`], as a clip's are.
    Tags(Vec<String>),
    /// Move the in point (BACKLOG #119). **The caller has already checked the
    /// ordering** — see [`Project::edit_slate`]'s note on why this setter stays
    /// infallible.
    In(f64),
    /// Move the out point. A slate with no out point gains one, which is how a
    /// half-marked range is finished from the editor rather than from `o`.
    Out(f64),
}

/// [`Project::mark_slate_out`] refused; the project is unchanged.
#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq)]
pub enum SlateError {
    #[error("press i first: there is no range open on this video")]
    NothingOpen,
    #[error("that is before the range started, at {in_seconds:.1}s")]
    OutBeforeIn { in_seconds: f64 },
}

impl Project {
    pub fn new(name: impl Into<String>) -> Self {
        Project {
            format_version: crate::store::CURRENT_FORMAT_VERSION,
            name: name.into(),
            source_videos: Vec::new(),
            clips: Vec::new(),
            slates: Vec::new(),
            preferences: Preferences::default(),
            scoreboard: None,
            match_events: Vec::new(),
            player_highlights: Vec::new(),
            avatar: None,
        }
    }

    /// Sum of every source's duration.
    pub fn total_source_duration(&self) -> f64 {
        self.source_videos.iter().map(|s| s.duration_seconds).sum()
    }

    /// Where source `source_index` starts on the virtual-concat timeline.
    ///
    /// Clamped to the source count, so an index past the end returns the total
    /// duration rather than panicking.
    pub fn cumulative_offset(&self, source_index: usize) -> f64 {
        let end = source_index.min(self.source_videos.len());
        self.source_videos[..end]
            .iter()
            .map(|s| s.duration_seconds)
            .sum()
    }

    /// Absolute time on the virtual-concat timeline.
    ///
    /// The match clock runs on this timeline, which is why it lives with the
    /// format rather than with the player.
    pub fn abs_seconds(&self, source_index: usize, source_seconds: f64) -> f64 {
        self.cumulative_offset(source_index) + source_seconds
    }

    /// Concat time → `(source_index, source_seconds)`. The inverse is
    /// [`Project::abs_seconds`].
    ///
    /// Ported from macOS `Workspace.sourceTime(at:)`:
    ///
    /// - the first source with `abs < cumulative + duration` contains it;
    /// - an instant exactly on a boundary belongs to the **next** source, at 0;
    /// - past the end, it clamps to `(last, last_duration)`;
    /// - with no sources, it is `(0, 0)`.
    ///
    /// Uses the stored durations (the duration authority), so a zero-length
    /// source is never located into.
    pub fn locate(&self, abs_seconds: f64) -> (usize, f64) {
        let Some(last) = self.source_videos.len().checked_sub(1) else {
            return (0, 0.0);
        };
        let mut cumulative = 0.0;
        for (i, src) in self.source_videos.iter().enumerate() {
            let next = cumulative + src.duration_seconds;
            if abs_seconds < next {
                // `f64::max` returns the non-NaN operand, so this never goes
                // negative for an `abs_seconds` before the start.
                return (i, (abs_seconds - cumulative).max(0.0));
            }
            cumulative = next;
        }
        (last, self.source_videos[last].duration_seconds)
    }

    /// True if any clip, match event or player highlight points at source
    /// `index`.
    ///
    /// The UI disables a source's remove button on this; [`remove_source`]
    /// re-checks it. macOS counted clips only, so a match event could be left
    /// pointing at the wrong file.
    ///
    /// [`remove_source`]: Project::remove_source
    pub fn source_is_referenced(&self, index: usize) -> bool {
        self.clips.iter().any(|c| c.source_index == index)
            || self.match_events.iter().any(|m| m.source_index == index)
            || self
                .player_highlights
                .iter()
                .any(|h| h.source_index == index)
            || self.slates.iter().any(|s| s.source_index == index)
    }

    /// Remove source `index`, keeping every clip, match event and player
    /// highlight on its own physical file.
    ///
    /// Refuses while the source is referenced. On success every higher
    /// `source_index` — in clips, match events **and** highlights (macOS
    /// remapped clips only) — drops by one, and `current` (the player's
    /// source) goes through the same remap: `None` means the current source
    /// was the one removed.
    ///
    /// # Panics
    ///
    /// If `index` is out of range, like `Vec::remove`.
    pub fn remove_source(
        &mut self,
        index: usize,
        current: usize,
    ) -> Result<Option<usize>, SourceReferenced> {
        if self.source_is_referenced(index) {
            return Err(SourceReferenced { index });
        }
        self.source_videos.remove(index);
        let shift = |i: &mut usize| {
            if *i > index {
                *i -= 1;
            }
        };
        self.clips
            .iter_mut()
            .for_each(|c| shift(&mut c.source_index));
        self.match_events
            .iter_mut()
            .for_each(|m| shift(&mut m.source_index));
        self.player_highlights
            .iter_mut()
            .for_each(|h| shift(&mut h.source_index));
        self.slates
            .iter_mut()
            .for_each(|s| shift(&mut s.source_index));
        Ok(match current.cmp(&index) {
            std::cmp::Ordering::Less => Some(current),
            std::cmp::Ordering::Equal => None,
            std::cmp::Ordering::Greater => Some(current - 1),
        })
    }

    /// Move source `from` to position `to` (the `Vec::remove` + `Vec::insert`
    /// convention) and remap clips, match events, player highlights and
    /// `current` through the same permutation, returning the new `current`.
    ///
    /// A move is always a valid permutation, so there is nothing to refuse.
    /// macOS remapped clips only.
    ///
    /// # Panics
    ///
    /// If `from` or `to` is out of range.
    pub fn move_source(&mut self, from: usize, to: usize, current: usize) -> usize {
        let src = self.source_videos.remove(from);
        self.source_videos.insert(to, src);
        let remap = |i: usize| {
            if i == from {
                to
            } else if from < i && i <= to {
                i - 1
            } else if to <= i && i < from {
                i + 1
            } else {
                i
            }
        };
        for c in &mut self.clips {
            c.source_index = remap(c.source_index);
        }
        for m in &mut self.match_events {
            m.source_index = remap(m.source_index);
        }
        for h in &mut self.player_highlights {
            h.source_index = remap(h.source_index);
        }
        for s in &mut self.slates {
            s.source_index = remap(s.source_index);
        }
        remap(current)
    }

    /// Gate a candidate source's display aspect against the project's.
    ///
    /// The reference is the stored aspect of the first source other than
    /// `excluding` (the one being relinked; `None` on add). With no such source
    /// there is no gate, so a sole source can be relinked to a new aspect —
    /// the intent of macOS's relink gate, which in practice never ran. Stored
    /// aspects are used, so the gate works while other sources are missing.
    ///
    /// Rule ([`aspects_match`], macOS `aspectsMatch`): both aspects > 0 and
    /// `|a − b| / max(a, b) < 0.005`. The 0.5% absorbs phone footage that lands
    /// a pixel off (1920×1078) without admitting a genuinely different aspect.
    /// A NaN aspect fails the `> 0` test and so mismatches.
    pub fn check_aspect(
        &self,
        candidate: f64,
        excluding: Option<usize>,
    ) -> Result<(), AspectMismatch> {
        let reference = self
            .source_videos
            .iter()
            .enumerate()
            .find(|&(i, _)| Some(i) != excluding);
        let Some((_, reference)) = reference else {
            return Ok(());
        };
        let (a, b) = (reference.display_aspect, candidate);
        if aspects_match(a, b) {
            Ok(())
        } else {
            Err(AspectMismatch {
                existing: a,
                attempted: b,
            })
        }
    }

    /// Appends the clip a finished recording produced and returns it.
    ///
    /// The name is macOS's `defaultClipName`: the 1-based source number and
    /// the floored start, `"2-01:02:05"`. `sort_index` is the clip count,
    /// which is the next position because clips are kept in order (see
    /// [`Project::apply_clip_order`]). `created_at` is passed in because core
    /// has no clock.
    pub fn add_recorded_clip(
        &mut self,
        pending: PendingClip,
        duration: f64,
        events: Vec<CommentaryEvent>,
        created_at: String,
    ) -> &mut Clip {
        // `as` truncates toward zero and saturates, so a negative or NaN start
        // names as 00:00:00, like macOS's `max(0, ...)`.
        let total = pending.start_source_seconds as u64;
        let name = format!(
            "{}-{:02}:{:02}:{:02}",
            pending.source_index + 1,
            total / 3600,
            total % 3600 / 60,
            total % 60
        );
        let sort_index = self.clips.len() as i64;
        self.clips.push(Clip {
            id: pending.id,
            name,
            notes: String::new(),
            tags: Vec::new(),
            source_index: pending.source_index,
            start_source_seconds: pending.start_source_seconds,
            recording_duration: duration,
            recording_filename: format!("{}.mkv", pending.id),
            events,
            show_pip: self.preferences.pip_for_new_recordings,
            inset: if self.avatar.is_some() {
                Inset::Avatar
            } else {
                Inset::Camera
            },
            // The size and corner the coach last set on a clip: there is no
            // clip during a take, so "pick before" can only be a preference
            // (spec I6). Seeded here, beside `show_pip`, because this is the
            // core function that holds the line — the bus's `finish_recording`
            // is only its caller.
            inset_size: self.preferences.last_inset_size,
            inset_corner: self.preferences.last_inset_corner,
            sort_index,
            created_at,
            transcript: String::new(),
            // A take shot from a slate is linked by the caller, which is also
            // where the slate's name and tags are applied: this stays a pure
            // function of the recording.
            slate_id: None,
        });
        self.clips.last_mut().expect("just pushed")
    }

    /// Open a range on `source_index` at `in_seconds`, returning its id.
    ///
    /// **The slate is stored on this press**, with no out point yet, which is
    /// what keeps a half-marked range from being UI state that vanishes when
    /// the app closes — and what saves marking from needing an in-progress
    /// source index to invalidate when the source list changes underneath it.
    pub fn mark_slate_in(&mut self, source_index: usize, in_seconds: f64) -> Uuid {
        let id = Uuid::new_v4();
        self.slates.push(Slate {
            id,
            source_index,
            in_seconds,
            out_seconds: None,
            name: String::new(),
            tags: Vec::new(),
        });
        id
    }

    /// Close the range most recently opened on `source_index`.
    ///
    /// **Most recently opened, which is why the stored order is the marked
    /// order** — [`slates_sorted`](Self::slates_sorted) is for reading and
    /// nothing re-sorts the `Vec`. A coach who opens two and closes one closes
    /// the one they just started.
    pub fn mark_slate_out(
        &mut self,
        source_index: usize,
        out_seconds: f64,
    ) -> Result<Uuid, SlateError> {
        let slate = self
            .slates
            .iter_mut()
            .rev()
            .find(|s| s.source_index == source_index && s.out_seconds.is_none())
            .ok_or(SlateError::NothingOpen)?;
        if out_seconds <= slate.in_seconds {
            return Err(SlateError::OutBeforeIn {
                in_seconds: slate.in_seconds,
            });
        }
        slate.out_seconds = Some(out_seconds);
        Ok(slate.id)
    }

    /// Applies `edit` to slate `id`, or does nothing when it is gone.
    ///
    /// **Infallible, and the mark edits do not change that** (BACKLOG #119).
    /// [`SlateEdit::In`] and [`SlateEdit::Out`] can make a range inverted, and
    /// the check for that lives in the **bus**, before the mutation — not here.
    /// Two reasons: `Bus::edit_slates` is where a refusal can become a
    /// `UserError::Slate` notice, which is the path `mark_slate_out`'s own
    /// refusal already takes; and making this `Result` would put a fallible
    /// setter in front of two existing infallible call sites for a rule neither
    /// of them can break. Use [`Slate::would_invert`] to ask first.
    pub fn edit_slate(&mut self, id: Uuid, edit: SlateEdit) {
        let Some(slate) = self.slates.iter_mut().find(|s| s.id == id) else {
            return;
        };
        match edit {
            SlateEdit::Name(name) => slate.name = name.trim().to_owned(),
            SlateEdit::Tags(tags) => slate.tags = tags,
            SlateEdit::In(seconds) => slate.in_seconds = seconds,
            SlateEdit::Out(seconds) => slate.out_seconds = Some(seconds),
        }
    }

    /// Removes slate `id`. The clips it was shot into keep their `slate_id`,
    /// which then names nothing — harmless, since "has it been shot?" is asked
    /// of the clips and never of the slate.
    pub fn delete_slate(&mut self, id: Uuid) {
        self.slates.retain(|s| s.id != id);
    }

    /// Slates in reading order: by source, then by where they start. There is
    /// no hand-order to respect — see [`Slate`] — so this is computed rather
    /// than stored, and nothing has to renumber after a mark or a delete.
    pub fn slates_sorted(&self) -> Vec<&Slate> {
        let mut out: Vec<&Slate> = self.slates.iter().collect();
        out.sort_by(|a, b| {
            a.source_index
                .cmp(&b.source_index)
                .then(a.in_seconds.total_cmp(&b.in_seconds))
        });
        out
    }

    // ------------------------------------------------------------ clip order
    //
    // `clips` is kept in order with `sort_index == position` (Phase 3 spec
    // C3): `store::read` normalizes it and every mutation renumbers. Each order
    // operation is then a `Vec` operation, and export never meets a tie or a
    // gap.

    /// Set every `sort_index` to its position.
    pub(crate) fn renumber(&mut self) {
        for (i, c) in self.clips.iter_mut().enumerate() {
            c.sort_index = i as i64;
        }
    }

    /// The clip ids in list order.
    pub fn clip_order(&self) -> Vec<Uuid> {
        self.clips.iter().map(|c| c.id).collect()
    }

    /// The one order mutation, shared by move, sort, undo and redo.
    ///
    /// Clips named in `order` come first, in that order; ids that no longer
    /// exist (or repeat) are skipped. The rest follow in their current order,
    /// so an order captured before an add or delete still applies.
    pub fn apply_clip_order(&mut self, order: &[Uuid]) {
        let mut rest = std::mem::take(&mut self.clips);
        for id in order {
            if let Some(i) = rest.iter().position(|c| c.id == *id) {
                self.clips.push(rest.remove(i));
            }
        }
        self.clips.append(&mut rest);
        self.renumber();
    }

    /// The order after moving the clip at `from` to position `to` (the
    /// `Vec::remove` + `Vec::insert` convention).
    ///
    /// # Panics
    ///
    /// If `from` or `to` is out of range.
    pub fn moved_order(&self, from: usize, to: usize) -> Vec<Uuid> {
        let mut order = self.clip_order();
        let id = order.remove(from);
        order.insert(to, id);
        order
    }

    /// The order sorted by position in the game: source, then start. Stable,
    /// so clips at the same instant keep their relative order.
    pub fn source_sorted_order(&self) -> Vec<Uuid> {
        let mut clips: Vec<&Clip> = self.clips.iter().collect();
        clips.sort_by(|a, b| {
            a.source_index
                .cmp(&b.source_index)
                .then(a.start_source_seconds.total_cmp(&b.start_source_seconds))
        });
        clips.into_iter().map(|c| c.id).collect()
    }

    /// Remove clip `id` and return it. Its `sort_index` still holds the
    /// position it was removed from, which [`Project::insert_clip`] restores.
    pub fn remove_clip(&mut self, id: Uuid) -> Option<Clip> {
        let i = self.clips.iter().position(|c| c.id == id)?;
        let clip = self.clips.remove(i);
        self.renumber();
        Some(clip)
    }

    /// Insert `clip` at its `sort_index`, clamped to the list. A no-op if a
    /// clip with its id is already present.
    pub fn insert_clip(&mut self, clip: Clip) {
        if self.clips.iter().any(|c| c.id == clip.id) {
            return;
        }
        let at = usize::try_from(clip.sort_index)
            .unwrap_or(0)
            .min(self.clips.len());
        self.clips.insert(at, clip);
        self.renumber();
    }

    /// Set one field of clip `id` (see [`Clip::set`]), or `None` if there is
    /// no such clip.
    pub fn apply_edit(&mut self, id: Uuid, edit: ClipEdit) -> Option<ClipEdit> {
        Some(self.clips.iter_mut().find(|c| c.id == id)?.set(edit))
    }
}
