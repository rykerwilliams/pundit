# pundit

*pundit Understands Nothing, Discusses It Thoroughly.*

A desktop app for breaking down game film, for Linux. Open the game video,
record your commentary over it with a webcam and microphone while you play,
pause, scrub, zoom and draw on the picture, and export each clip as a video
with your voice, your drawings and a picture-in-picture of you burned in.

## What it does

- **A match is one timeline.** Add one video or several — both halves, say —
  and play straight across the join. **New match…** makes the project from the
  video files in one step: it reads the date and the opponent out of the folder
  and file names, sets up the scoreboard from your last match, and names the
  project `<Home> v <Away>`. Your footage is never copied, moved or renamed.
- **Scanning that keeps up.** Skip, step a frame at a time, scan at up to 32×
  with **J** and **L**, scrub with the mouse wheel, zoom and pan about the
  pointer, and press **F** to close the window up around the picture.
- **Commentary clips.** Press **R** and talk over the game. Each clip is a
  recording of you — the webcam and microphone, plus everything you did to the
  picture while recording: play, pause, seek, zoom and pan, and freehand
  drawings. It all replays in step, in the preview and in the export.
- **No webcam? Use a photo.** Pick an image under **Inset** in **Devices…** and
  the camera is never opened; your picture sits in the corner as a circle that
  swells as you talk. Each clip chooses how big its inset is and which corner it
  sits in.
- **Clips you can find again.** Name them, tag them, add notes, filter by tag,
  and undo and redo with Ctrl+Z and Ctrl+Shift+Z.
- **Mark now, talk later.** Press **I** and **O** to mark a range while you
  watch; it goes into the **Slates** list, where you can name and tag it. Come
  back later and record over each one — filtered by a tag, the list walks you
  through them one take after another.
- **Ring a player.** Press **H** and drag a box around a player; a labelled ring
  follows them between the boxes you place, while you scan, in previews and in
  exports.
- **Scoreboard and match clock.** Set up the two teams, their colours and the
  match format, then tag goals and the start and end of each period (**Z**,
  **X** and **V**), or type them in as lines. The clock and score are drawn on
  the picture while you scan and burned into previews and exports. If your video
  starts after kick-off, the clock can still read correctly.
- **Transcripts.** Transcribe a clip's commentary on your own computer, with
  [whisper.cpp](https://github.com/ggml-org/whisper.cpp).
- **Export.** An H.264 `.mp4` at 720p or 1080p for every clip, the clips with
  one tag, a single clip, a goals reel (per team, or all goals), or the whole
  match — into the project's `exports/` folder. Clips carry a caption bar with
  their number, name and tags. Files with chapters get a `.chapters.txt` beside
  them, ready to paste into a YouTube description.
- **The whole match in a minute, not an hour.** Where the footage allows it, the
  whole match is a straight copy of your video, with the scoreboard riding along
  as a subtitle track and an `.srt` beside the file.
- **The basket.** Gather clips from as many matches as you like and press
  **Start** for one film of them all, each piece with its own match's
  scoreboard.

A project is a folder: `project.json`, your commentary in `recordings/`, and
your exports in `exports/`. The game video stays where it is.

The [user guide](docs/book/src/guide/index.md) walks through all of it task by
task, and [every keyboard shortcut](docs/book/src/guide/shortcuts.md) is listed
along with what stops it working.

## Install

pundit ships as a `.deb` for **Ubuntu 24.04 and Linux Mint 22** on
x86-64. Download it from the
[releases page](https://github.com/rykerwilliams/pundit/releases), then,
in the folder you downloaded it to:

```bash
sudo apt install ./pundit_*_amd64.deb
```

If you have **Coach Cuts** installed — the name this app went by until 0.8.0 —
that one command replaces it, and your settings, your basket and the speech
model you already downloaded are carried over on the first run.

Or [build the package yourself](#build-from-source), which is also how you get
a build newer than the last release.

It appears in the applications menu as **pundit**. From a terminal it is
`pundit`, or `pundit <folder>` to open (or create) a project there;
with no argument it reopens the last project.

### Requirements

- **GStreamer 1.24 or newer**, as Ubuntu 24.04 ships it. The package pulls in
  the plugin sets it needs.
- **An x86-64-v3 CPU** — Haswell (2013) or newer, with AVX2, FMA, F16C and
  BMI2. The speech recognizer is built for that instruction set.
- **A VA-API driver for hardware video decode and encode.** The package
  recommends `intel-media-va-driver` (or `va-driver-all`), and apt installs
  it by default. Without one, playback falls back to software decode, and
  recording and export to the `x264enc` software encoder, which is much
  slower.
- **An OpenGL (EGL) capable display**, X11 or Wayland.
- **A webcam and microphone** for recording: the camera is captured through
  V4L2 and the microphone through PipeWire.

### Transcription and the network

Transcription runs entirely on your computer. The first time you transcribe,
the app downloads the speech model you chose in the clip inspector —
`small.en` (488 MB, the default, more accurate) or `base.en` (148 MB, faster)
— into `~/.cache/pundit/models/`, and keeps it. The Transcribe button
names the download before you press it. After that, nothing leaves your
machine.

### When something goes wrong

The app writes its diagnostics to stderr. Started from the applications menu
on an X11 session (Linux Mint's default), that lands in `~/.xsession-errors`;
or start it from a terminal as `pundit` to see it directly.

## Build from source

On Ubuntu 24.04 / Linux Mint 22, with Rust 1.92 or newer from
[rustup](https://rustup.rs), install the packages listed (and explained) in
[`packaging/build-deps.txt`](packaging/build-deps.txt) — the same list CI
installs:

```bash
sudo apt install $(grep -o '^[^#]*' packaging/build-deps.txt)

cargo run --release -p pundit-app   # run it
cargo test --workspace              # test it
```

The first build compiles whisper.cpp and takes a few minutes.

To build the `.deb` (into `target/debian/`), also install `cargo-deb` and
`cargo-about`, then run the packaging script:

```bash
cargo install --locked cargo-deb
cargo install --locked cargo-about --features cli
packaging/build-deb.sh
```

The code is a Cargo workspace under `crates/`: `pundit-core` (pure logic,
no media dependencies), `pundit-media` (GStreamer), `pundit-app` (the
Slint UI) and `pundit-harness` (headless integration tests). The
developer conventions — the zero-copy decode path, the capture clock, the
export graph, how to run CI's GPU-less path locally — are in
[`CLAUDE.md`](CLAUDE.md), and the design is in
[`docs/superpowers/specs/`](docs/superpowers/specs/).

## Documentation

| Where | What |
|---|---|
| [`docs/book/`](docs/book/) | The user guide, the keyboard shortcuts, the changelog and a page for developers, as an [mdBook](https://rust-lang.github.io/mdBook/). `mdbook build docs/book` builds it; CI checks it on every pull request and publishes it from `main`. |
| [`CHANGELOG.md`](CHANGELOG.md) | What each release changed, written for the person using the app. |
| [`CLAUDE.md`](CLAUDE.md) | The developer conventions, and the measurements behind them. |
| [`docs/superpowers/`](docs/superpowers/) | The specs, plans and measurement spikes behind every feature, in date order. |
| [`docs/hands-on-checklist.md`](docs/hands-on-checklist.md) | A try-everything pass on real hardware, in the order of a normal session. |
| [`BACKLOG.md`](BACKLOG.md) | Deferred work, each item with why it waits and when to revisit it. |

## Where this came from

This began as a macOS app written in Swift, SwiftUI and AVFoundation, forked
from [tayl0r/coach-cutups](https://github.com/tayl0r/coach-cutups) — whose
author named it **Coach Cuts**, which is what this app was called up to 0.7.0.
The Linux rewrite was written against that Swift tree, which lived here under
`apple/`; it is no longer in the working tree and is kept whole at the tag
`macos-reference` (`git worktree add /tmp/macos-reference macos-reference`).
Linux is the only platform maintained here, and the name changed at 0.8.0
because the app is not football-only and should not carry one it inherited.

## Licence

AGPL-3.0-or-later — see [`LICENSE`](LICENSE). The package installs the
licence notices for the third-party code built into the binary under
`/usr/share/doc/pundit/`.
