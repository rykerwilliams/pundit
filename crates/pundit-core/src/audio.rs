//! The audio edit: which span of which file is heard at each output sample,
//! how loud, and the fades at its edges.
//!
//! Core owns the splice; media owns the samples (spec E3). There are three
//! tracks: the game video's own audio — heard **only while the source is
//! playing**, so a freeze is silent — the commentary recording, heard
//! throughout its entry, and a goals reel's music (BACKLOG #84), heard under the
//! whole compilation with the game ducked beneath it.
//!
//! # The emitted timeline
//!
//! Every position here is an **emitted sample**: an index into the stream
//! handed to the AAC encoder, which is the output timeline with its first
//! [`PRIMING_SAMPLES`] samples dropped (spec E3, measured). So emitted sample
//! `s` is output time `(s + PRIMING_SAMPLES) / AUDIO_SAMPLE_RATE`, and output
//! frame `n` is emitted sample `n · 1600 − 1024`.
//!
//! That shift is not cosmetic: the ramps have to live on the emitted timeline
//! or the first region's fade-in is inside the dropped head and the export
//! opens at full gain — the click the ramp rule exists to remove. Expressing
//! the regions themselves in emitted samples is what makes that true by
//! construction, and it leaves the caller one mapping instead of two: emitted
//! sample `s` of a region reads its file at
//! `source_offset + (s − out_samples.start) / AUDIO_SAMPLE_RATE`.

use std::ops::Range;
use std::path::{Path, PathBuf};

use crate::export::{frame_count, Compilation, OUTPUT_FPS};
use crate::plan::PlanEntry;
use crate::project::Preferences;
use crate::timeline::SegmentKind;

/// The mixing rate. Media decodes every audio file to F32LE at this rate, in
/// two channels; a sample position here counts **frames of audio**, not
/// interleaved floats.
pub const AUDIO_SAMPLE_RATE: u32 = 48_000;

/// Audio samples per output video frame.
///
/// Exact by construction — the const assert below fails the build if a future
/// rate or fps makes it fractional, because a fractional value would put entry
/// boundaries between samples and let rounding drift across a long run.
pub const SAMPLES_PER_FRAME: u64 = (AUDIO_SAMPLE_RATE / OUTPUT_FPS) as u64;
const _: () = assert!(AUDIO_SAMPLE_RATE.is_multiple_of(OUTPUT_FPS));

/// Samples `avenc_aac` prepends as priming, and therefore the number media
/// drops from the head of the mixed stream so the sound lands where the picture
/// does (measured: 1024 samples, 21.3 ms — spec E3).
///
/// Core needs it because the ramps are computed on the post-drop timeline; the
/// drop itself happens in media.
pub const PRIMING_SAMPLES: u64 = 1024;

/// The click guard's length: 5 ms at [`AUDIO_SAMPLE_RATE`]. Every region of the
/// game and the commentary fades over this at both edges.
pub const RAMP_SAMPLES: u64 = AUDIO_SAMPLE_RATE as u64 / 200;

/// The music's fade in: 1 s from the reel's start (#84 spec §M4).
pub const MUSIC_FADE_IN: u64 = AUDIO_SAMPLE_RATE as u64;
/// The music's fade out: 3 s ending on the reel's last frame.
pub const MUSIC_FADE_OUT: u64 = 3 * AUDIO_SAMPLE_RATE as u64;
/// The music's level, under which the game is ducked to [`MUSIC_DUCK`].
pub const MUSIC_GAIN: f64 = 0.7;
/// The game's level under music, as a multiple of its source volume.
pub const MUSIC_DUCK: f64 = 0.3;
/// **Why the mixer needs no limiter**: a reel has no commentary, one track's
/// regions never overlap, and the source volume is 0.0 or 1.0 — so the loudest
/// a music run can sum to is the two levels together. Keep it at or under full
/// scale and nothing can clip.
const _: () = assert!(MUSIC_GAIN + MUSIC_DUCK <= 1.0);

/// Which file a region is heard from.
///
/// **The game and the commentary name an entry** — an index into
/// `CompilationPlan::entries`, the same index [`crate::export::FrameSpec`]
/// carries, rather than repeating a file name per region. **The music names its
/// file**: it spans every entry, so an index would be a lie, and a reel has one
/// music region, so nothing repeats.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Track {
    /// The entry's source video, at `source_offset` into it.
    Game { entry: usize },
    /// The entry's commentary recording, at `source_offset` into it.
    Commentary { entry: usize },
    /// A goals reel's music (BACKLOG #84), from its start.
    Music(PathBuf),
}

/// One contiguous stretch of one file, at one gain.
///
/// Regions of different tracks overlap; regions of the same track never do.
#[derive(Debug, Clone, PartialEq)]
pub struct Region {
    pub track: Track,
    /// The emitted samples this region covers (see the module docs).
    pub out_samples: Range<u64>,
    /// Time into the file at `out_samples.start`.
    pub source_offset: f64,
    pub gain: f64,
}

/// The multiplier for emitted `sample`: the region's gain, shaped by the fades
/// at its edges. Zero outside the region.
///
/// Both edges ramp linearly, from silence and back to it — over
/// [`RAMP_SAMPLES`] for the game and the commentary, and over
/// [`MUSIC_FADE_IN`] / [`MUSIC_FADE_OUT`] for the music. The two edges are
/// scaled **separately** (`rise / in`, `fall / out`) and the smaller taken,
/// which handles a region shorter than its two fades without a special case:
/// it stays quiet rather than jumping to full gain — a 3 ms fragment of game
/// sound, or a reel shorter than four seconds, whose two music fades meet
/// below full level.
///
/// The ramp windows are clamped to the region, so nothing is evaluated before
/// the emitted timeline's t = 0: the first region fades in over the first 5 ms
/// the encoder actually receives instead of starting part-way up a ramp that
/// the priming drop swallowed.
pub fn envelope(region: &Region, sample: u64) -> f64 {
    if !region.out_samples.contains(&sample) {
        return 0.0;
    }
    let (fade_in, fade_out) = match region.track {
        Track::Music(_) => (MUSIC_FADE_IN, MUSIC_FADE_OUT),
        Track::Game { .. } | Track::Commentary { .. } => (RAMP_SAMPLES, RAMP_SAMPLES),
    };
    let rise = (sample - region.out_samples.start) as f64 / fade_in as f64;
    let fall = (region.out_samples.end - 1 - sample) as f64 / fade_out as f64;
    region.gain * rise.min(fall).min(1.0)
}

/// Every audio region of `compilation`, in output order within each track (the
/// music, spanning everything, comes last; the mixer sorts by start).
///
/// **`source_volume` is the choice for this run, not a stored preference**
/// (spec M2): the export sheet's "Mute source audio" writes
/// [`Preferences::export_source_volume`] and the bus passes it here, while a
/// basket passes its own — which is what lets `bus/basket.rs` keep mixing at
/// `Preferences::default()` on purpose (basket spec J6) and still be mutable.
/// At `0.0` the game contributes **no region at all** rather than a gain of
/// zero, so nothing downstream opens a reader to decode samples it would
/// multiply away. `0.0` and `1.0` round-trip exactly through JSON and the
/// checkbox writes literals, so the equality is exact — a level (BACKLOG #126)
/// must quantize its bottom detent to exactly `0.0`.
///
/// The commentary's gain is still read from `prefs`, which has no writer
/// (BACKLOG #124's remaining half).
///
/// **`music` is a goals reel's track** (#84): with one, a single
/// [`Track::Music`] region spans the whole compilation at [`MUSIC_GAIN`], and
/// every game region is ducked to [`MUSIC_DUCK`] times the source volume; a
/// muted source still emits no game region, so a muted reel is the music alone.
/// **Pass music only for a compilation without commentary** — the assert beside
/// the constants assumes it, and nothing here enforces it.
///
/// Adjacent play segments stay separate regions even when the source is
/// continuous across them. A `Skip` or `Play` event is a source discontinuity,
/// and media seeks its audio pipeline per region regardless, so the fade pair
/// at the join is honest rather than an artefact of the splice.
pub fn audio_regions(
    compilation: &Compilation,
    prefs: &Preferences,
    source_volume: f64,
    music: Option<&Path>,
) -> Vec<Region> {
    let game_gain = match music {
        Some(_) => MUSIC_DUCK * source_volume,
        None => source_volume,
    };
    let mut regions = Vec::new();
    for (i, entry) in compilation.plan.entries.iter().enumerate() {
        let end = entry.start_frame + entry.frames;
        if source_volume != 0.0 {
            game_regions(entry, i, game_gain, &mut regions);
        }
        // One per entry with a clip: the recording runs from its own zero for
        // the whole entry, freezes included. An entry without one has no
        // recording, so its only sound is the game's.
        if entry.clip_id.is_none() {
            continue;
        }
        regions.extend(region(
            Track::Commentary { entry: i },
            entry.start_frame..end,
            0.0,
            prefs.preview_commentary_volume,
        ));
    }
    // Last, so the regions stay in output order within each track; the mixer
    // sorts by start regardless.
    if let Some(path) = music {
        regions.extend(region(
            Track::Music(path.to_path_buf()),
            0..compilation.plan.total_frames(),
            0.0,
            MUSIC_GAIN,
        ));
    }
    regions
}

/// Append `entry`'s play segments as game regions; freezes contribute nothing,
/// which is how they come out silent.
///
/// Segments are placed on the same whole frames the picture uses: a boundary at
/// `s` seconds into the entry belongs to the frame
/// [`frame_count`]`(s)` — the rule `export::walk` applies to the video, epsilon
/// and all, so sound and picture switch on the same frame. The last segment
/// runs to the entry's end, taking the up-to-one-frame of quantization with it
/// exactly as the picture does.
fn game_regions(entry: &PlanEntry, index: usize, gain: f64, out: &mut Vec<Region>) {
    let mut seconds = 0.0;
    let mut first = 0;
    for (n, seg) in entry.segments.iter().enumerate() {
        seconds += seg.out_duration;
        let last = if n + 1 == entry.segments.len() {
            entry.frames
        } else {
            frame_count(seconds).min(entry.frames)
        };
        if seg.kind == SegmentKind::Play {
            out.extend(region(
                Track::Game { entry: index },
                entry.start_frame + first..entry.start_frame + last,
                seg.source_start,
                gain,
            ));
        }
        first = last;
    }
}

/// A region over output frames `frames`, moved onto the emitted timeline.
///
/// `None` when nothing survives: an entry or a segment that got no frame, or —
/// only ever for the very first region — a span living entirely inside the
/// dropped head. What the drop removes from the head of a region is added to
/// its `source_offset`, so the region still reads the right audio: the export
/// loses its first 21 ms, it does not play them late.
fn region(track: Track, frames: Range<usize>, source_start: f64, gain: f64) -> Option<Region> {
    let start = frames.start as u64 * SAMPLES_PER_FRAME;
    let end = frames.end as u64 * SAMPLES_PER_FRAME;
    let dropped = PRIMING_SAMPLES.min(end).saturating_sub(start);
    let out_samples = start.saturating_sub(PRIMING_SAMPLES)..end.saturating_sub(PRIMING_SAMPLES);
    if out_samples.is_empty() {
        return None;
    }
    Some(Region {
        track,
        out_samples,
        source_offset: source_start + dropped as f64 / f64::from(AUDIO_SAMPLE_RATE),
        gain,
    })
}
