//! The audio edit: which span of which file is heard at each emitted sample,
//! and the fades at its edges.
//!
//! Positions are **emitted** samples — the output timeline with the AAC
//! priming head dropped — so the numbers here are `frame · 1600 − 1024`.

mod common;

use uuid::Uuid;

use std::path::PathBuf;

use pundit_core::audio::{
    audio_regions, envelope, Region, Track, AUDIO_SAMPLE_RATE, MUSIC_DUCK, MUSIC_FADE_IN,
    MUSIC_FADE_OUT, MUSIC_GAIN, PRIMING_SAMPLES, RAMP_SAMPLES,
};
use pundit_core::event::{CommentaryEvent, EventKind};
use pundit_core::export::{compilation_schedule, Compilation, FrameSpec};
use pundit_core::plan::{CompilationPlan, ExportTarget, PlanEntry};
use pundit_core::project::{Clip, Project, SourceRef};
use pundit_core::timeline::{PlaybackSegment, SegmentKind};
use pundit_core::zoom::Zoom;

/// Audio samples per output frame: 48000 / 30.
const SPF: u64 = 1600;
const RATE: f64 = AUDIO_SAMPLE_RATE as f64;
const DUR: f64 = 1000.0;

fn clip(start: f64, duration: f64, events: Vec<CommentaryEvent>) -> Clip {
    Clip {
        id: Uuid::new_v4(),
        name: "c".into(),
        start_source_seconds: start,
        recording_duration: duration,
        recording_filename: "c.mkv".into(),
        events,
        created_at: "2026-09-19T00:00:00Z".into(),
        ..common::clip_stub()
    }
}

fn pause(t: f64, anchor: f64) -> CommentaryEvent {
    CommentaryEvent::new(
        t,
        EventKind::Pause {
            source_time: anchor,
        },
    )
}
fn play(t: f64, anchor: f64) -> CommentaryEvent {
    CommentaryEvent::new(
        t,
        EventKind::Play {
            source_time: anchor,
        },
    )
}

/// Every region of a compilation of `clips`, at the given volumes.
fn regions_at(clips: Vec<Clip>, source: f64, commentary: f64) -> Vec<Region> {
    let mut p = Project::new("p");
    p.source_videos.push(SourceRef {
        relative_path: "film.mp4".into(),
        display_name: "film".into(),
        duration_seconds: DUR,
        display_aspect: 16.0 / 9.0,
    });
    p.clips = clips;
    p.preferences.preview_commentary_volume = commentary;
    let c = compilation_schedule(&p, &ExportTarget::AllClips);
    // The source volume is the run's own, not the project's: core takes it as
    // a parameter so a basket can mute without faking a `Preferences`.
    audio_regions(&c, &p.preferences, source, None)
}

fn regions(clips: Vec<Clip>) -> Vec<Region> {
    regions_at(clips, 1.0, 1.0)
}

fn game(rs: &[Region]) -> Vec<Region> {
    rs.iter()
        .filter(|r| matches!(r.track, Track::Game { .. }))
        .cloned()
        .collect()
}

fn commentary(rs: &[Region]) -> Vec<Region> {
    rs.iter()
        .filter(|r| matches!(r.track, Track::Commentary { .. }))
        .cloned()
        .collect()
}

fn music(rs: &[Region]) -> Vec<Region> {
    rs.iter()
        .filter(|r| matches!(r.track, Track::Music(_)))
        .cloned()
        .collect()
}

fn approx(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// A region of `len` emitted samples at gain 1, for envelope tests.
fn span(len: u64) -> Region {
    Region {
        track: Track::Game { entry: 0 },
        out_samples: 0..len,
        source_offset: 0.0,
        gain: 1.0,
    }
}

#[test]
fn a_playing_clip_is_one_game_region_and_one_commentary_region() {
    let rs = regions(vec![clip(10.0, 2.0, vec![])]);
    assert_eq!(rs.len(), 2);

    let game = &rs[0];
    assert_eq!(game.track, Track::Game { entry: 0 });
    assert_eq!(game.out_samples, 0..60 * SPF - PRIMING_SAMPLES);
    // The dropped head is skipped in the source too, not replayed late.
    assert!(approx(
        game.source_offset,
        10.0 + PRIMING_SAMPLES as f64 / RATE
    ));

    let mic = &rs[1];
    assert_eq!(mic.track, Track::Commentary { entry: 0 });
    assert_eq!(mic.out_samples, game.out_samples);
    // The recording's own zero is the entry's record time zero.
    assert!(approx(mic.source_offset, PRIMING_SAMPLES as f64 / RATE));
}

/// One entry with no clip, two seconds of play: a goals-reel entry.
/// A one-goal reel of `frames` output frames, all game sound.
fn reel_of(frames: usize) -> Compilation {
    let entry = PlanEntry {
        clip_id: None,
        source_index: 0,
        segments: vec![PlaybackSegment {
            kind: SegmentKind::Play,
            source_start: 10.0,
            out_duration: frames as f64 / 30.0,
        }],
        start_frame: 0,
        frames,
        text: String::new(),
    };
    Compilation {
        frames: (0..frames as u32)
            .map(|n| FrameSpec {
                entry: 0,
                source_time: 10.0 + f64::from(n) / 30.0,
                zoom: Zoom::IDENTITY,
            })
            .collect(),
        plan: CompilationPlan {
            entries: vec![entry],
            chapters: Vec::new(),
        },
    }
}

/// An entry with no clip (a goals-reel entry) has no recording, so the game is
/// all it plays.
#[test]
fn an_entry_without_a_clip_is_game_audio_only() {
    let rs = audio_regions(&reel_of(60), &Project::new("p").preferences, 1.0, None);
    assert_eq!(commentary(&rs), vec![]);
    let game = game(&rs);
    assert_eq!(game.len(), 1);
    assert_eq!(game[0].out_samples, 0..60 * SPF - PRIMING_SAMPLES);
}

/// Muting the source leaves the commentary exactly as it was, and takes the
/// game away entirely rather than turning it down (spec M2).
///
/// **The commentary regions are compared, not the whole `Vec`**, which
/// interleaves the game's: the two runs cannot have the same length. What the
/// comparison pins is that muting touches one track — same spans, same
/// offsets, same gain — which holds because `region`'s priming drop is
/// computed per region from the entry's own frames, so entry 0's commentary
/// loses the same 1024 samples either way.
#[test]
fn a_muted_source_leaves_the_commentary_untouched_and_no_game_region() {
    let clips = || {
        vec![
            clip(10.0, 3.0, vec![pause(1.0, 11.0), play(2.0, 11.0)]),
            clip(40.0, 2.0, Vec::new()),
        ]
    };
    let loud = regions_at(clips(), 1.0, 1.0);
    let muted = regions_at(clips(), 0.0, 1.0);

    assert!(
        !game(&loud).is_empty(),
        "the unmuted run has to have game regions for this to mean anything"
    );
    // The untouched half first, so a run that only turned the game *down*
    // fails on the half that is about the game.
    assert_eq!(
        commentary(&muted),
        commentary(&loud),
        "the mute moved the commentary"
    );
    assert!(
        muted
            .iter()
            .all(|r| matches!(r.track, Track::Commentary { .. })),
        "a muted run carries a game region: {muted:#?}"
    );
}

/// A muted reel is silent: its entries have no clip, so there is no
/// commentary region either and the whole list is empty (spec M6).
#[test]
fn a_muted_reel_entry_has_no_regions_at_all() {
    let rs = audio_regions(&reel_of(60), &Project::new("p").preferences, 0.0, None);
    assert_eq!(rs, vec![], "a muted reel entry still carries sound");
}

#[test]
fn a_freeze_is_silent_on_the_game_track_and_not_on_the_commentary() {
    // Play 0–1 s, freeze 1–2 s on 11.0, play 2–3 s from 11.0.
    let c = clip(10.0, 3.0, vec![pause(1.0, 11.0), play(2.0, 11.0)]);
    let rs = regions(vec![c]);

    let game = game(&rs);
    assert_eq!(game.len(), 2);
    assert_eq!(game[0].out_samples, 0..30 * SPF - PRIMING_SAMPLES);
    assert_eq!(
        game[1].out_samples,
        60 * SPF - PRIMING_SAMPLES..90 * SPF - PRIMING_SAMPLES
    );
    assert!(approx(game[1].source_offset, 11.0));
    // Nothing covers the freeze.
    let mid = 45 * SPF - PRIMING_SAMPLES;
    assert!(game.iter().all(|r| !r.out_samples.contains(&mid)));

    let mic = commentary(&rs);
    assert_eq!(mic.len(), 1);
    assert!(mic[0].out_samples.contains(&mid));
}

#[test]
fn a_clip_that_never_plays_has_no_game_region() {
    let c = clip(10.0, 2.0, vec![pause(0.0, 10.0)]);
    let rs = regions(vec![c]);
    assert!(game(&rs).is_empty());
    assert_eq!(commentary(&rs).len(), 1);
}

#[test]
fn a_segment_boundary_lands_on_the_same_frame_as_the_picture() {
    // [0.03, 0.04) contains 1/30, so the freeze owns frame 1 and only frame 1
    // — `tests/export.rs` pins the same clip's frames.
    let c = clip(10.0, 1.0, vec![pause(0.03, 50.0), play(0.04, 20.0)]);
    let game = game(&regions(vec![c]));
    assert_eq!(game.len(), 2);
    assert_eq!(game[0].out_samples, 0..SPF - PRIMING_SAMPLES);
    assert_eq!(
        game[1].out_samples,
        2 * SPF - PRIMING_SAMPLES..30 * SPF - PRIMING_SAMPLES
    );
    assert!(approx(game[1].source_offset, 20.0));
}

#[test]
fn entries_abut_on_whole_frames_and_only_the_first_is_shortened() {
    // 0.55 s quantizes to 17 frames, so entry 1 starts at frame 17.
    let rs = regions(vec![clip(10.0, 0.55, vec![]), clip(20.0, 0.55, vec![])]);
    let game = game(&rs);
    assert_eq!(game.len(), 2);
    assert_eq!(game[0].track, Track::Game { entry: 0 });
    assert_eq!(game[1].track, Track::Game { entry: 1 });
    assert_eq!(game[0].out_samples, 0..17 * SPF - PRIMING_SAMPLES);
    assert_eq!(
        game[1].out_samples,
        17 * SPF - PRIMING_SAMPLES..34 * SPF - PRIMING_SAMPLES
    );
    // Only the first region loses samples to the priming drop.
    assert!(approx(game[1].source_offset, 20.0));

    let mic = commentary(&rs);
    assert_eq!(mic[1].out_samples, game[1].out_samples);
    assert!(approx(mic[1].source_offset, 0.0));
}

#[test]
fn an_empty_entry_contributes_nothing() {
    let rs = regions(vec![clip(10.0, 0.0, vec![]), clip(20.0, 1.0, vec![])]);
    assert!(rs.iter().all(|r| matches!(
        r.track,
        Track::Game { entry: 1 } | Track::Commentary { entry: 1 }
    )));
    assert_eq!(rs.len(), 2);
}

/// The two gains come from two different places on purpose: the game's is the
/// run's own argument, the commentary's is read off `Preferences`.
#[test]
fn gains_come_from_the_source_argument_and_the_commentary_preference() {
    let rs = regions_at(vec![clip(10.0, 1.0, vec![])], 0.25, 0.75);
    assert_eq!(game(&rs)[0].gain, 0.25);
    assert_eq!(commentary(&rs)[0].gain, 0.75);
}

#[test]
fn the_envelope_ramps_in_and_out_over_five_milliseconds() {
    let r = span(48_000);
    assert_eq!(envelope(&r, 0), 0.0);
    assert!(approx(envelope(&r, RAMP_SAMPLES / 2), 0.5));
    assert_eq!(envelope(&r, RAMP_SAMPLES), 1.0);
    assert_eq!(envelope(&r, 24_000), 1.0);
    assert_eq!(envelope(&r, 47_999), 0.0);
    assert!(approx(envelope(&r, 47_999 - RAMP_SAMPLES / 2), 0.5));
    assert_eq!(envelope(&r, 47_999 - RAMP_SAMPLES), 1.0);
}

#[test]
fn the_envelope_scales_by_the_regions_gain() {
    let mut r = span(48_000);
    r.gain = 0.5;
    assert_eq!(envelope(&r, 24_000), 0.5);
    assert!(approx(envelope(&r, RAMP_SAMPLES / 2), 0.25));
}

#[test]
fn a_region_shorter_than_two_ramps_never_reaches_full_gain() {
    // 100 samples, ~2 ms: it rises to 50/240 in the middle and back.
    let r = span(100);
    assert_eq!(envelope(&r, 0), 0.0);
    assert!(approx(envelope(&r, 50), 49.0 / RAMP_SAMPLES as f64));
    assert_eq!(envelope(&r, 99), 0.0);
    for s in 0..100 {
        assert!(envelope(&r, s) < 1.0, "sample {s}");
    }
    // Degenerate lengths stay finite rather than dividing by a zero span.
    assert_eq!(envelope(&span(1), 0), 0.0);
}

#[test]
fn the_envelope_is_zero_outside_the_region() {
    let r = Region {
        out_samples: 1_000..2_000,
        ..span(0)
    };
    assert_eq!(envelope(&r, 999), 0.0);
    assert_eq!(envelope(&r, 2_000), 0.0);
    assert_eq!(envelope(&r, 1_500), 1.0);
}

#[test]
fn the_first_region_fades_in_on_the_emitted_stream() {
    // The whole fade must survive the priming drop: it starts at emitted 0, not
    // part-way up a ramp the encoder never sees.
    let rs = regions(vec![clip(10.0, 2.0, vec![])]);
    for r in &rs {
        assert_eq!(r.out_samples.start, 0);
        assert_eq!(envelope(r, 0), 0.0);
        assert_eq!(envelope(r, RAMP_SAMPLES), 1.0);
    }
}

// --- Music under a goals reel (BACKLOG #84) ---

fn song() -> PathBuf {
    PathBuf::from("/music/rock/song.ogg")
}

/// One music region over the whole reel, carrying its file, at the music's
/// level — and the game under it, ducked.
#[test]
fn music_spans_the_reel_and_ducks_the_game() {
    let reel = reel_of(30 * 20);
    // A fractional source volume, so the duck is shown to multiply it.
    let rs = audio_regions(&reel, &Project::new("p").preferences, 0.5, Some(&song()));
    let music = music(&rs);
    assert_eq!(music.len(), 1, "{rs:?}");
    assert_eq!(music[0].track, Track::Music(song()));
    assert_eq!(music[0].gain, MUSIC_GAIN);
    // The whole output, on the emitted timeline: from 0 to the last frame.
    assert_eq!(music[0].out_samples, 0..(30 * 20) * SPF - PRIMING_SAMPLES);
    // The track is read from its own start, less the dropped priming head.
    assert!(approx(
        music[0].source_offset,
        PRIMING_SAMPLES as f64 / RATE
    ));

    let game = game(&rs);
    assert_eq!(game.len(), 1);
    assert!(approx(game[0].gain, MUSIC_DUCK * 0.5));
}

/// Muting the source mutes the game and leaves the music: a reel cut to a song.
#[test]
fn a_muted_reel_with_music_is_the_music_alone() {
    let rs = audio_regions(
        &reel_of(300),
        &Project::new("p").preferences,
        0.0,
        Some(&song()),
    );
    assert!(game(&rs).is_empty(), "{rs:?}");
    assert_eq!(music(&rs).len(), 1);
}

/// The music fades in over its first second and out over its last three.
#[test]
fn the_music_fades_in_over_a_second_and_out_over_three() {
    let len = 20 * AUDIO_SAMPLE_RATE as u64;
    let music = Region {
        track: Track::Music(song()),
        out_samples: 0..len,
        source_offset: 0.0,
        gain: 1.0,
    };
    assert!(approx(envelope(&music, MUSIC_FADE_IN / 2), 0.5));
    assert!(approx(envelope(&music, MUSIC_FADE_IN), 1.0));
    assert!(approx(envelope(&music, len / 2), 1.0));
    assert!(approx(envelope(&music, len - 1 - MUSIC_FADE_OUT / 2), 0.5));
    assert_eq!(envelope(&music, len - 1), 0.0);
}

/// A reel shorter than the two fades: they meet below full level.
#[test]
fn on_a_short_reel_the_music_fades_meet() {
    let rs = audio_regions(
        &reel_of(60),
        &Project::new("p").preferences,
        1.0,
        Some(&song()),
    );
    let music = &music(&rs)[0];
    let (start, end) = (music.out_samples.start, music.out_samples.end);
    let peak = (start..end).map(|s| envelope(music, s)).fold(0.0, f64::max);
    assert!(peak > 0.0 && peak < MUSIC_GAIN, "peak {peak}");
}
