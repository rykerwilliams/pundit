# Backlog

Deferred items from the scoreboard work (spec → plan → execution → review cycle).
Each entry: what, why deferred, when to revisit.

## What is open

Entry numbers are permanent and never reused: CLAUDE.md and code comments cite
them. **Nothing here is deleted.** An entry that says *why not yet* is what stops
the same ground being dug twice — see 57, which exists only to record that a
spec's premise was measurably wrong and that fixing it the obvious way would have
made things worse.

### Next, in order

- **85.** Recent projects, and a drawer to switch between them. The coach
- **78.** App settings for the things an export writes. The coach (2026-09-24):
- **96.** Every hot key should be reassignable. The coach (2026-09-25): "we need
- **77.** An export queue across projects. The coach (2026-09-24): "i open…
- **84.** Music under a goals reel — Openverse search, then the mixer (needs #78)
- **102.** Snap the scrubber to events, as an option. The coach (2026-09-28):
  "snap to events in the scrubber as an option" — which marks it covers is the
  open question (needs #78 for the control, and #100's per-field read)
- **104.** Double-clicking a slate should take the player to its in point, as a
  clip row's double-click does (and a click should stop toggling the selection)
- **105.** A slate's tag field should offer the tags already in use — the clip
  inspector's suggestion list, lifted into one shared `TagField`

### Waiting on the coach

A judgement no amount of code supplies — a licence, a piece of hardware, a
product call, or their own data.

- **79.** What the app does over a forwarded X11 display, and saying so. The…
- **82.** P4 (showing suggestions) is not justified and is not started. No
- **80.** The restarts — mostly done; match B's seven are the remainder
- **81.** Task 3.6 — the *model* spike (rten vs ort, D-FINE-N); P3's own spike is done
- **53.** 2160p export
- **103.** A possession tracker, as an analysis pass — which output (a share, a
  timeline, or the change moments) is the coach's call; built on P5/P6's
  detector and kit clustering (#81)

### Open, but gated on something happening

Each says in its own words what would make it worth doing: a coach reporting the
symptom, a format that scores in double digits, a source with real timestamp
gaps, a second person handed the app. Until then the fix costs more than the
problem — which is the entry, not an excuse for it.

- **20.** Chrome coordinate space for non-16:9 sources
- **25.** Wayland vs X11 for the drawing overlay
- **27.** macOS export bugs the port fixes but the Swift tree keeps
- **28.** Non-finite floats silently corrupt a project on save
- **29.** Long-GOP 4K scrubbing on low-power iGPUs
- **30.** Re-measure SkipCoordinator's burst window against real…
- **31.** GL re-setup after a window hide
- **32.** Recents list and a menu bar
- **33.** Pinch-to-zoom
- **34.** Rotated source videos
- **35.** Physical-key bindings for A/D and digits
- **36.** Decoding slows to ~0.1× when the display is off…
- **37.** Measure and correct the commentary A/V offset
- **38.** Orphaned recordings after a crash
- **39.** Fall back to x264 when a VA encoder is present but broken
- **40.** Camera format and audio-source fallbacks
- **41.** Live device list and global device preferences
- **42.** PiP checkbox, start flash, level-meter polish
- **44.** Clip-edit undo coalescing, tag-overview Duration sort,…
- **45.** Multi-select and bulk tag edits
- **46.** Export on machines without surfaceless EGL, and more…
- **48.** Live drawing overlay has no automated coverage; two…
- **49.** records_h264_and_opus_with_the_file_duration is…
- **50.** tests/recorder.rs fails when its tests run in parallel
- **51.** A video-less recording would hang the preview's pump on a…
- **52.** No UI for the source and commentary volumes
- **54.** Preview has no game audio
- **55.** An export run leaks about 19 dmabuf fds and never plateaus
- **56.** The score label overflows its cell at double-digit…
- **57.** A realistic club name is ellipsized to ~7 characters.…
- **58.** scan_abs can pair a new source's index with the old…
- **59.** Segment timestamps and click-a-line-to-seek. whisper…
- **60.** whisper-rs's set_abort_callback_safe is unsound in…
- **61.** All three whisper-rs *_safe callback setters leak their…
- **62.** Cache the WhisperContext across queued jobs. Phase 10…
- **63.** A truncated recording that EOSes cleanly still…
- **64.** A panic inside a transcription job would wedge the queue…
- **65.** A cancelled whisper run keeps eight threads busy for ~12…
- **68.** The volume slider still uses Slint's stock slider. A…
- **69.** The preview can starve under heavy load from other…
- **70.** A heap-corruption abort once, tearing down whisper in the…
- **71.** A possible thump at the start of every commentary…
- **72.** A source's first load at open once never settled, on CI.…
- **73.** , looks stuck across a timestamp gap longer than half a…
- **74.** A reel trim can be a silent no-op. A goal merged into the…
- **75.** A skip burst's replay margin failed once, by 54 ms. One…
- **76.** a_cancelled_copy_leaves_nothing's fixtures sit on the 30…
- **83.** a_pause_while_a_flushing_seek_recovers_playing_sticks…
- **93.** Delete the 0.8.0 rename shims. Three things exist only to…
- **97.** Typing slates in, as match events can be typed
- **98.** Export the slates as a silent breakdown film

### Closed, kept for the reasoning

- **95.** Fit window to video — shipped 2026-09-26; the splitter snap is still
  the coach's call (see the entry)
- **87.** Resizable panels — shipped 2026-09-27; panels grow but do not shrink,
  and there is no keyboard path (99)
- **88.** The inset's size and corner, per clip — shipped 2026-09-28; v13, three
  sizes and three corners, sticky; the two preferences get a control of their own
  with #78
- **107.** The docs site — shipped 2026-09-28; CI builds the book on every change
  and publishes it from `main` (filed as #104, renumbered for a collision)

- 21, 22, 23, 24, 26, 43, 47, 66, 67, 86, 89, 90, 91, 92, 94

### Archived

- **1–19**, the macOS app — titles only, at the end of this file.

## Archive — the macOS app (entries 1–19)

These describe the Swift app, which left the tree at 0.8.0 and lives at the
`macos-reference` tag. They are kept as **titles only**: the reasoning in them is
about code this repository no longer builds, and the full text of each is in git
history (`git log -p --follow BACKLOG.md`, or `git show 7f3037e:BACKLOG.md`).
Numbers are never reused — CLAUDE.md and code comments cite entries by number.

- **1.** Spec clock table uses `now ≤ tH1End`; code uses strict `now < tH1End` — RESOLVED
- **2.** Plan references `CoachCutups.xcodeproj` / scheme `CoachCutups` — RESOLVED
- **3.** Plan didn't note `xcodegen generate` is required after creating any new App-target file — RESOLVED
- **4.** `ScoreboardReplayOverlay.Coordinator.clip` is now refreshed on every `updateNSView`
- **5.** `MatchEventKind.isHalfTag` has one real call site — RESOLVED
- **6.** `MatchInspectorPanel` could reuse a "tag with keyboard hint" view helper
- **7.** `drawText` in `ScoreboardDraw.swift` does an extra context-flip + translate
- **8.** `CompilationInstruction` carries three correlated scoreboard fields — RESOLVED
- **9.** `matchLengthSeconds` UI bound through `* 60 / 60` Stepper — RESOLVED
- **10.** Stoppage time has no upper cap
- **11.** No "rapid undo coalescing" for match-event tagging
- **12.** Always-visible event picker vs `eventModeActive` toggle
- **13.** Audit `swiftLanguageModes: [.v5]` in `VideoCoachCore/Package.swift`
- **14.** `DeviceWiringModifier.body` chained `.onChange` modifier split
- **15.** Verify `SpeechAnalyzer` authorization flow on macOS 26
- **16.** Test coverage: `.transcribing` → `.summarizing` phase transition — OBSOLETED by the Linux port spec (summarization is dropped; see #22)
- **17.** Cmd-z while focused on transcript/summary field reverts AI write too
- **18.** No "queued" state in the inspector
- **19.** First-run speech model download UX

## Linux port (spec `docs/superpowers/specs/2026-09-19-linux-port-design.md`)

### 20. Chrome coordinate space for non-16:9 sources
- **Why deferred:** With the port letterboxing the base image, the text bar, PiP
  and scoreboard can lay out in output space (overlapping the letterbox bars,
  reading like broadcast furniture) or content space (staying inside the
  picture). Purely cosmetic, and it only differs for non-16:9 sources. There is
  no evidence either way in the macOS tree because its non-uniform stretch
  collapses the two spaces. Spec recommends output space.
- **When to revisit:** Phase 7 (clip preview), the first time a non-16:9 source
  is composited. Only the three chrome layers change; the stroke denormalization
  rule is unaffected either way.

### 21. `Project` ownership in the command bus — RESOLVED (Phase 2 spec D5: the bus thread owns it and publishes `Arc<Project>` snapshots)
- **Why deferred:** Whether the bus owns `Project` (every mutation is a
  `Command`) or the UI owns it and the bus owns media only. Spec recommends the
  bus owning it and emitting `Event::ProjectChanged(Arc<Project>)`, with the UI
  deriving Slint properties from the snapshot — it makes undo unambiguously
  bus-side and avoids partial-update bugs. Not locked because it depends on
  Slint's property model, which nobody has prototyped against.
- **When to revisit:** Phase 2 plan, after a Slint property-model spike.

### 22. whisper model distribution — RESOLVED
- **Why deferred:** Bundle (~140 MB package), download on first run, or require
  a user-supplied path. Spec recommends download-on-first-run with explicit
  prompt and progress. Note this does not newly break an offline guarantee —
  see #19, the macOS app already downloads a speech model inside `transcribe()`.
- **When to revisit:** Phase 10. Decide before the packaging phase, since it
  changes the artifact size.
- **Resolved 2026-09-21:** download on first use, implemented in Phase 11
  (spec `2026-09-21-linux-port-phase-11-design.md` S3). Note the "~140 MB"
  above is `base.en`; the default `small.en` is 487.6 MB, 3.3x that.

### 23. README accuracy: "no network calls" and "No FFmpeg" — RESOLVED
- **Why deferred:** Both claims on README line 5 are or will be false. "No
  network calls" is already inaccurate on macOS (#19). "No FFmpeg" becomes false
  the moment `gst-libav` ships, which it must for software decode of arbitrary
  match film. Not fixed now because the README describes the macOS app, which
  still ships.
- **When to revisit:** When the Linux build becomes the primary artifact
  (Phase 11), or sooner if the macOS README is touched for any other reason.
- **Resolved 2026-09-21:** the README is rewritten for the Linux port (Phase
  11 spec S7). It no longer makes either claim: it says the speech model
  downloads once, on first use, and that nothing leaves the machine after
  that; FFmpeg isn't mentioned (the package does depend on `gst-libav`).
- **Note 2026-09-22:** `ffmpeg` is now a **test-only** build dependency
  (`packaging/build-deps.txt`): `ffprobe` is the chapter test's independent
  reader of the `chpl` box (match vision spec C3). Nothing links or ships it,
  and the `.deb` still doesn't depend on it.

### 24. Linux packaging: AppImage vs Flatpak — RESOLVED (`.deb`)
- **Why deferred:** Flatpak sandboxing complicates camera, microphone and
  arbitrary-path file access — all three of which this app needs. Spec
  recommends AppImage first.
- **User expectation (2026-09-19):** the user pictured something like the macOS
  `.app` (one self-contained thing to download and run), which maps to
  AppImage, not Flatpak. The tension: GStreamer and the VA drivers live in the
  OS on Linux, so an AppImage must bundle GStreamer carefully and borrow the
  host's VA drivers. A `.deb` depending on Ubuntu's GStreamer is the simplest
  option for the user's own laptop. Weigh both against that expectation.
- **When to revisit:** Phase 11.
- **Resolved 2026-09-21:** a `.deb`, after the evidence overturned AppImage.
  The reasoning, and why Flatpak is a capture rewrite rather than a
  packaging choice, is in the Phase 11 spec's S0. AppImage and Flatpak stay
  open for a later phase if the app is ever handed to someone else.
- **Confirmed by the coach 2026-09-25:** *"we already solved packaging for now.
  only deb"*. Nothing here is a live question; it is kept as the record of why
  the two obvious alternatives were not chosen.

### 25. Wayland vs X11 for the drawing overlay
- **Why deferred:** Freehand telestration wants low input latency and the two
  display stacks differ. No measurement exists.
- **When to revisit:** Phase 6 spike, before stroke capture is built on either.

### 26. Fate of the `apple/` tree — RESOLVED (2026-09-25)
- Deleted from the working tree after 0.7.0, and kept whole at the annotated
  tag `macos-reference`. Nobody ran it, CI never built it, and the behaviour
  worth keeping is written down in the specs; the tag costs nothing and answers
  "what did the original do?" when a spec is silent.

### 27. macOS export bugs the port fixes but the Swift tree keeps
- **Why deferred:** The review found five live bugs in the macOS app: the export
  scoreboard clock ignores pauses and skips (`CompilationCompositor.swift:253`);
  non-16:9 sources are anamorphically distorted at fixed export resolutions;
  preview ignores `showPiP` while export honors it; the export Quality picker
  has no effect at all (`ExportSettings.bitrate` has zero production call
  sites); and unknown commentary events persist as empty-kind records that lose
  their payload. The port fixes all five by construction. Not fixed in Swift
  because that app was never maintained in parallel; it now lives only at the
  `macos-reference` tag (#26), so the file:line citations above are read with
  `git show macos-reference:<path>`.
- **When to revisit:** Only if the macOS app ships again. The scoreboard fix is
  small — pass `clip` instead of `clipStartAbsSeconds` into
  `CompilationInstruction` and call the preview formula.

### 28. Non-finite floats silently corrupt a project on save
- **Why deferred:** `serde_json` does not error on NaN or infinity — it writes
  `null`. A non-finite value in an *event* payload then re-decodes as
  `EventKind::Unknown`, so the event vanishes from replay, is re-emitted
  verbatim on the next save, and never produces a diagnostic. In a non-event
  field it becomes `null`, which no `f64` accepts, so the whole project reports
  `Malformed` and refuses to open. Either way the user loses work silently.
  Not fixed in `pundit-core` because a serializer-side validator that walks
  the document for nulls is more machinery than the problem warrants, and
  because the crate has no producer of NaN today — `Zoom::clamped` and
  `SkipCoordinator::request_skip` were the two panic-or-corrupt paths and both
  are fixed. `CommentaryEvent::new` now carries a `debug_assert` on finiteness
  so a producer bug is loud in development.
- **When to revisit:** Phase 2 and Phase 6, which introduce the first real
  producers (player positions, gesture coordinates normalized against a
  possibly-zero-height rect). The bus contract already localizes capture to one
  place, so sanitize there. Consider also surfacing a count of `Unknown` events
  from `store::read` so corruption is observable rather than quiet.

### 29. Long-GOP 4K scrubbing on low-power iGPUs
- **Why deferred:** On the reference laptop (15 W Comet Lake iGPU), accurate
  seek on synthetic 4K HEVC with a 2 s GOP measured 191 ms median / 336 ms
  worst — the only measured case over the 250 ms budget. The user's own footage
  (HEVC 1440p30, 0.5 s GOP) seeks in 10 / 22 ms, and a 2 s-GOP 1080p60 file in
  92 / 149 ms, so nothing the user actually shoots is affected. KEY_UNIT during
  drag (37 ms median on the 4K file) already covers live scrubbing; only the
  accurate settle on release is slow. Options if it matters: detect GOP length
  at import (the gate script already measures it) and offer short-GOP proxies,
  as NLEs do. Not worth building for a source type nobody has imported yet.
- **When to revisit:** When a user imports 4K long-GOP footage (common from
  some action cameras and broadcast downloads), or if Phase 2 targets a slower
  GPU than the reference laptop.

### 30. Re-measure SkipCoordinator's burst window against real seeks
- **Why deferred:** `DEFAULT_BURST_WINDOW` (150 ms) was tuned against mpv and
  VideoToolbox. On the reference laptop an accurate seek on the user's footage
  takes 10 ms median / 22 ms worst, so a leading exact seek lands long before
  150 ms and the coordinator will rarely enter burst mode at all. That is
  probably fine — the window then mostly just delays the settle after a burst
  — but it should be tuned by feel with the real transport, not by arithmetic.
- **When to revisit:** Phase 2, once skip keys drive a real player.

## Phase 2 deferrals (spec `docs/superpowers/specs/2026-09-19-linux-port-phase-2-design.md`)

### 31. GL re-setup after a window hide
- **Why deferred:** On Wayland, hiding a Slint window destroys it, and
  `RenderingSetup` later arrives with a new GL context. GStreamer's GL elements
  hold the old display/context from NULL→READY, so recovery means cycling the
  pipeline to NULL, re-wrapping, reloading and re-seeking. Phase 2's main
  window is never hidden, so teardown is handled (synchronous NULL with an
  acknowledgement) and re-setup is not.
- **When to revisit:** The first phase that hides a window (e.g. a preview or
  export window), or if a Wayland user reports a black player after
  minimize/restore.

### 32. Recents list and a menu bar
- **Why deferred:** macOS had neither; restore-last-project covers the common
  case, and a third entry point for Open/Add adds UI surface without adding
  capability.
- **When to revisit:** When the user works across several projects regularly,
  or if Linux users expect a File menu.

### 33. Pinch-to-zoom
- **Why deferred:** winit 0.30 delivers pinch gestures only on macOS/iOS, so
  Slint's `ScaleRotateGestureHandler` receives nothing on Linux. Scroll-zoom
  and drag-pan cover the interaction.
- **When to revisit:** When winit gains Linux touchpad gestures.

### 34. Rotated source videos
- **Why deferred:** Phase 2 rejects sources whose `image-orientation` tag isn't
  `rotate-0`, because the GL path doesn't rotate and none of the user's ~74
  files is rotated. Supporting it means swapping the stored aspect and adding
  `glvideoflip video-direction=auto` to the sink bin, and the Phase 6 stroke
  coordinates would need to follow.
- **When to revisit:** The first time a user needs portrait phone footage.

### 35. Physical-key bindings for A/D and digits
- **Why deferred:** Slint key events carry text, not scancodes, so A/D and the
  zoom digits follow the keyboard layout (on AZERTY the digits need Shift).
  Arrows are unaffected.
- **When to revisit:** Only if a non-QWERTY user reports it.

### 36. Decoding slows to ~0.1× when the display is off (vsync-blocked swap)
- **Why deferred:** In Phase 2 Task 5, with the laptop's monitor DPMS-off,
  playback ran at about 0.1× in both the app and the Task 0 spike (26 GL
  uploads in 8 s instead of ~240). With `vblank_mode=0` it ran at full speed.
  The likely cause is that Slint's vsync-blocked buffer swap on the shared EGL
  context also throttles GStreamer's GL upload work; the mechanism isn't
  confirmed. It doesn't affect normal use with the screen on, but it means a
  stalled UI thread can slow decoding, which will matter for export (Phase 8)
  if export ever shares the UI's GL context.
- **When to revisit:** Phase 8, when export runs its own GL pipeline — make
  sure it uses its own context, not the UI's. Or sooner if playback stutters
  when the window is occluded or on another workspace.

## Phase 4 deferrals (spec `docs/superpowers/specs/2026-09-19-linux-port-phase-4-design.md`)

### 37. Measure and correct the commentary A/V offset
- **Why deferred:** audio from `pipewiresrc` is stamped on arrival, while
  `v4l2src` video carries kernel capture times, so audio may sit ~20–40 ms
  late. Nothing measured it against a real sync source.
- **When to revisit:** Phase 8 (PiP export), with a clap test on the real
  camera; correct with a fixed audio `ts-offset` if it's over a frame.

### 38. Orphaned recordings after a crash
- **Why deferred:** a crash loses the event log, so the `.mkv` can't become a
  clip (R7). The file stays playable but unreferenced in `recordings/`.
- **When to revisit:** after Phase 3 (which deliberately doesn't auto-delete
  unreferenced media); with a user-visible "clean up unused recordings"
  action, or at packaging.
- **Re-deferred at Phase 11 (2026-09-21):** how the app is installed doesn't change what a crash leaves in `recordings/`. Revisit with the clean-up action, which is its own UX question.

### 39. Fall back to x264 when a VA encoder is present but broken
- **Why deferred:** R4 picks the encoder by element presence. A VA element
  that exists but fails fails the recording with an error.
- **When to revisit:** if a user's recording fails at start on a machine with
  VA elements, or at packaging (Phase 11) when hardware variety grows.
- **Re-deferred at Phase 11 (2026-09-21):** one `.deb` for one laptop grows hardware variety by nothing. Revisit on the first report from another machine.

### 40. Camera format and audio-source fallbacks
- **Why deferred:** R3 refuses cameras without a 16:9 ≤1280 30 fps mode
  (parent spec and macOS rule); R1 drops the `pulsesrc` fallback (its clock
  was measured ~473,000 s off monotonic, and the target runs PipeWire).
- **When to revisit:** Phase 11 packaging, or when a real camera or system
  hits the refusal.
- **Re-deferred at Phase 11 (2026-09-21):** packaging turned out not to touch capture — the `.deb` runs the same capture code as `cargo run`. Revisit when a real camera or system hits the refusal.

### 41. Live device list and global device preferences
- **Why deferred:** devices are enumerated when the popover opens, not
  watched; preferences are per project (macOS parity).
- **When to revisit:** if hot-plugging a camera while the popover is open
  proves annoying, or if re-picking devices per project does.

### 42. PiP checkbox, start flash, level-meter polish
- **Why deferred:** `show_pip` has no consumer until export (Phase 8), so it
  takes `pip_for_new_recordings` (default true) with no UI. The red start
  flash and the meter's 1 s peak hold and colour gradient are macOS polish.
- **When to revisit:** the checkbox shipped in the Phase 3 inspector
  (per clip); the polish at the end of the port.

### 43. `a_player_error_is_reported_and_play_recovers` — RESOLVED (2026-09-25)
- It failed in CI, which was this entry's own trigger, on a **docs-only**
  commit — so the cause was never the code under it. The mechanism: one
  unreadable file posts **two** errors (typefind's, then the stream error
  behind it), and the second can arrive after the reload has already started,
  pausing playback for a failure that is over.
- The test now presses play once per pause, exactly as a coach would, and
  asserts where playback *ends up* rather than that it was never interrupted.
  Proven both ways: with an unexpected pause injected it passes 3/3, and with
  the tolerance removed and the same injection it fails with the flake's own
  `timed out waiting for playing in b`.
- **What was deliberately not done:** filtering stale errors in
  `SourcePlayer`, the way it filters a stale `ASYNC_DONE`. There is no obvious
  correct filter — the *first*, real error also arrives before the load's
  `ASYNC_DONE`, so "ignore errors until the load settles" would throw away the
  detection this test exists for, and GStreamer's messages carry nothing that
  distinguishes the second error from the first. The app's own behaviour in the
  rare case is one spurious pause and one spurious error line after a file is
  repaired, which the coach answers with another press of play. Revisit only if
  that is seen in real use, with a load epoch on the player rather than a
  guess at the message.

## Phase 3 deferrals (spec `docs/superpowers/specs/2026-09-19-linux-port-phase-3-design.md`)

### 44. Clip-edit undo coalescing, tag-overview Duration sort, suggestion ↑/↓
- **Why deferred:** macOS parity items that add UI state or Slint key
  plumbing for small gains: one undo step per field session is already
  predictable; the overview sorts A–Z; Tab takes the top suggestion and
  typing narrows the list.
- **When to revisit:** if any is missed in real use.

### 45. Multi-select and bulk tag edits
- **Why deferred:** macOS had single selection only.
- **When to revisit:** if tagging many clips at once becomes a chore.

## Phase 5 deferrals (spec `docs/superpowers/specs/2026-09-19-linux-port-phase-5-design.md`)

### 46. Export on machines without surfaceless EGL, and more encoders
- **Why deferred:** the exporter uses `GLDisplayEGL::new_surfaceless()`
  (Mesa); proprietary NVIDIA drivers may lack the extension, and export then
  fails loudly. `vah264enc` and `nvh264enc` aren't on any test machine, so
  their settings would be untested.
- **When to revisit:** when someone runs the port on NVIDIA or an AMD/VA
  machine with `vah264enc`; add an EGL-device or GBM display path and the
  encoder entries then, measured.
- **Re-deferred at Phase 11 (2026-09-21):** the release pipeline proves export on Mesa (llvmpipe in CI, Intel on the laptop) and nothing else. Unchanged trigger.

### 47. Rare hang when a second bus shuts down while its player is prerolling — RESOLVED
- **Why deferred:** seen only in tests: open → shutdown → open → shutdown in
  one process hung in the second `shutdown()` 3 times in 200 runs under 4×
  parallel load; a single open/shutdown never hung (240 runs). The bus thread
  looked stuck while the player was prerolling a load. It may mean closing the
  window mid-load can freeze the app. User chose to keep moving on the phases
  (2026-09-19). An unfinished investigation's diff is in the session
  scratchpad (`shutdown-hang/wip.diff`), not committed.
- **When to revisit:** if closing the app ever hangs, or during the end-of-port
  hardening pass.
- **Seen on CI, 2026-09-21:** GitHub run 35668867690, attempt 2, hung in the
  harness test `a_cancel_clears_the_queue_and_leaves_no_failure` for over two
  hours before being cancelled — the test opens a project and shuts down
  within about a second, so the bus is still loading the source when the
  shutdown lands, which is this entry's shape. Both waits in
  `BusHandle::shutdown` (`acked.recv()`, `thread.join()`) are unbounded. The
  same test passed in the run before it and in attempt 1. CI jobs now carry
  `timeout-minutes`, so a recurrence fails in minutes instead of hanging. Its
  own revisit trigger — the end-of-port hardening pass — has now arrived. (spec `docs/superpowers/specs/2026-09-19-linux-port-phase-6-design.md`)
- **Resolved 2026-09-21:** a GStreamer 1.24.2 deadlock, not the bus. Taking
  `playbin3` down (READY or NULL) while a load's typefind thread is reporting
  the file's type deadlocks its `urisourcebin`: the state change holds the
  bin's state lock and waits for the typefind thread to stop, and that thread
  is waiting for the same lock to plug `parsebin` (gdb, every thread, in both
  the player and the bus; newer GStreamer no longer takes the lock). Any
  window close, source unload or reload inside a load's first milliseconds
  could hit it. `SourcePlayer::take_down` now lets a load still prerolling
  finish (bounded at 5 s) before any downward state change, and
  `taking_the_pipeline_down_mid_load_never_deadlocks` fails without it.

### 48. Live drawing overlay has no automated coverage; two accepted gaps
- **Why deferred:** `wire_drawing`, `show_strokes`, `clear_drawings` and the
  tick's expiry/rebuild live in `main.rs`, which no test binary links, so the
  live rule, the rebuild-on-change rule and "cleared on every recording
  transition" are covered only by the user's hands-on checks. Two behaviours
  are accepted rather than fixed: a mid-stroke window resize normalizes
  earlier points against the release rect, and while drawing is enabled the
  2/3 zoom keys pivot on the picture's centre (hover no longer reaches
  `zoom-area`).
- **When to revisit:** if the drawing overlay grows (a palette, shapes), move
  its state into a testable module; fix the hover pivot if it annoys in use.

### 49. `records_h264_and_opus_with_the_file_duration` is timing-flaky
- **Why deferred:** failed once at 2.033 s vs an expected 2.000 s during the
  Phase 6 review, and passed on re-run. Live test sources plus a loaded
  machine.
- **When to revisit:** if CI flakes; widen the tolerance to a few frames.

### 50. `tests/recorder.rs` fails when its tests run in parallel
- **Why deferred:** verified pre-existing (on a stashed tree): 1–2 of 3 fail
  under parallel execution, and pass serially. Live test sources contending
  for the VA encoder is the likely cause. Found during Phase 7.
- **When to revisit:** if CI flakes; mark the recorder tests serial, or give
  them their own encoder instance.

### 51. A video-less recording would hang the preview's pump on a frozen picture
- **Why deferred:** `composite::wait_for_room` waits without a deadline, which
  is exactly what makes PAUSED work: the pump stops because the appsrcs stop
  draining. If a clip's recording had no video stream while `show_pip` is
  true, the requested PiP pad would never produce and `glvideomixer` would
  wait on it forever, leaving the pump blocked on a picture that never moves.
  Not fixed: a deadline in `wait_for_room` would break pause, and the recorder
  never writes a recording without video — only a corrupt or hand-made file
  reaches this.
- **When to revisit:** if a preview is ever seen frozen with the transport
  still saying it is playing; the fix is to check the recording's streams when
  the job is built (a probe) and drop `show_pip` when there is no video.

## Phase 8 deferrals (spec `docs/superpowers/specs/2026-09-19-linux-port-phase-8-design.md`)

### 52. No UI for the source and commentary volumes
- **Why deferred:** `preview_source_volume` / `preview_commentary_volume` are read
  by preview and export and default to 1.0, which is what macOS shipped, but
  nothing sets them. Adding two sliders is easy; the question is where they
  belong (the export sheet, the preview transport, or preferences).
- **When to revisit:** the first time a commentary is drowned out by crowd noise.

### 53. 2160p export
- **Why deferred:** measured 0.56× realtime and it only upscales the user's
  1440p footage. `Resolution::R2160` stays in the project format.
- **When to revisit:** a 4K camera, or a machine that encodes 4K faster than
  realtime.

### 54. Preview has no game audio
- **Why deferred:** Phase 8 gives export the full mix, but preview still plays
  only the commentary. Phase 7 made the recording's native branch the pipeline
  clock and the only volume-controlled element, so a second pumped audio track
  needs an `audiomixer` pad that stalls the graph if unfed, breaks the pacing
  loop (the pump is paced by the clock it would feed), needs seek and EOS
  handling for a third appsrc, and needs the scrub mute to cover both tracks.
  Export is what gets shared, so it took the audio work first.
- **When to revisit:** when judging levels by ear matters, i.e. alongside the
  volume UI (#52).

### 55. An export run leaks about 19 dmabuf fds and never plateaus
- **Why deferred:** measured over eight consecutive export runs in one
  process: open file descriptors climbed 120 → 253 (~19 a run, `/proc/self/fd`
  dominated by `anon_inode:dmabuf`), with no plateau. Thread count stayed flat
  and RSS plateaued, so it is descriptors alone. The cause is
  `composite::Gl::shared`: the surfaceless `GLDisplayEGL` is process-wide and
  deliberately never finalized, so the display's buffer pools and the dmabufs
  imported through them outlive every run. Dropping the shared display is what
  the comment on `Gl::shared` says breaks concurrent exports — every
  surfaceless `GLDisplayEGL` wraps the same `EGLDisplay`, and finalizing one
  calls `eglTerminate` for all of them, which made side-by-side exports fail to
  import frames. So the obvious fix is the one thing that is known not to work,
  and 19 fds a run is ~50 runs inside a 1024 soft limit.
- **When to revisit:** if a long session ever hits `EMFILE`, or when GStreamer
  offers a way to release a display's imported buffers without terminating the
  `EGLDisplay`. Raising `RLIMIT_NOFILE` is the cheap stopgap.

## Phase 9 deferrals (spec `docs/superpowers/specs/2026-09-20-linux-port-phase-9-design.md`)

56. **The score label overflows its cell at double-digit scores.** Measured at
  1080p: the score cell is 138.2 px wide; `"3 - 1"` is 109.4 px, but
  `"12 - 9"` is 139.8 px and `"10 - 10"` is 170.3 px. The label is centred, so
  it spills symmetrically into the home and away cells rather than clipping.
- **Why deferred:** macOS behaved identically (it never fit the score), so
  this is not a regression, and soccer — the format the spec is written
  around — does not reach double digits. Fixing it means either a fourth memo
  slot keyed on size or a narrower column, and neither earns its place until a
  format that scores in double digits exists.
- **When to revisit:** when a basketball or hockey format lands, or the first
  time a real scoreboard reads `10 - 10`.

57. ~~**A realistic club name is ellipsized to ~7 characters.**~~ **Fixed in
  `babdea6`.** The spec's premise was measurably wrong: fitting
  `"Manchester United"` to the cell needs 16.7 px at 1080p, well above macOS's
  6 px floor, so "illegible anyway" did not hold and essentially no real club
  name rendered. Labels now shrink to a floor of a quarter of full size and
  ellipsize only below it. Left here as the record of why the spec said
  otherwise.
58. **`scan_abs` can pair a new source's index with the old source's offset.**
  `scan_abs` (`crates/pundit-app/src/main.rs`) falls back to
  `abs_seconds(ui.source_index, ui.last_secs)` when `ui.target_abs` is `None`.
  `last_secs` is written only by a *successful* `query_position()`, while
  `ui.source_index` is updated by `Event::Position` independently — so in the
  window after a cross-source seek has settled but before a query succeeds, the
  readout (and a tag taken in that instant) would pair the new index with the
  old source's seconds. That is exactly the pairing the function's doc comment
  says it prevents.
- **Why deferred:** not reproducible. A settled `Position` is only published
  once the flight is `Idle`, and the query works by then, so the window is
  empty in practice. Closing it properly needs either source seconds carried on
  `Event::Position` or `last_secs` keyed by source index — more machinery on
  the hot readout path than a window nobody has hit is worth.
- **When to revisit:** if a tag or the readout is ever seen a whole source's
  duration out, or when `Event::Position` gains a payload for another reason —
  add the source seconds to it then and the fallback becomes exact for free.

## Phase 10 deferrals (spec `docs/superpowers/specs/2026-09-20-linux-port-phase-10-design.md`)

59. **Segment timestamps and click-a-line-to-seek.** whisper returns per-segment
  `start_timestamp()`/`end_timestamp()` (centiseconds) for free, and storing them
  would let the coach click a transcript line to jump there — something the
  macOS app could never do.
- **Why deferred:** the transcript is editable. The moment the coach fixes a
  mangled player name, stored timings describe text that no longer exists, and
  every consumer needs a reconciliation story. Editable-plus-timestamped is a
  real design; editable-plus-timestamped with no reconciliation is a bug
  waiting to be found. It is also a format change (v7 → v8).
- **When to revisit:** if transcript search lands (which wants anchors anyway),
  or if the coach asks to navigate by what they said.

60. **whisper-rs's `set_abort_callback_safe` is unsound in 0.16.0.** It boxes the
  closure into a `Box<Box<dyn FnMut() -> bool>>` then installs
  `trampoline::<F>` with `F` the concrete closure type, so the trampoline
  reinterprets the fat pointer's data half. `set_progress_callback_safe`
  twelve lines above does it correctly. We work around it by passing an
  already-boxed trait object, so `F` is the boxed type and the cast is right.
- **Why deferred:** the workaround is free and local. Upstreaming it means a
  patch to a slow-moving crate (last commit 2026-03-14, now on Codeberg).
- **When to revisit:** when bumping whisper-rs — check whether the workaround
  is still needed, and whether it is still *safe*, since a fixed upstream would
  make the double box wrong in the other direction.

61. **All three whisper-rs `*_safe` callback setters leak their box.**
  `Box::into_raw` with no matching free; three small boxes per transcription
  job.
- **Why deferred:** a few dozen bytes per job on a job that allocates hundreds
  of megabytes. Recorded only so nobody spends an afternoon hunting it.
- **When to revisit:** never, unless a future caller sets callbacks in a loop.

62. **Cache the `WhisperContext` across queued jobs.** Phase 10 runs one
  worker thread per job, copying the `Exporter` precedent, so a six-clip
  queue loads `ggml-small.en.bin` six times.
- **Why deferred:** an earlier draft specified a long-lived worker holding the
  context, justified as saving "minutes" on a six-clip session. That was
  overstated — a warm `whisper_init_from_file` is sub-second — and it
  contradicted the spec's own "the bus thread owns the queue", needing a
  second channel and a drain protocol the design never named. It would also
  hold 466 MB resident through an entire recording session, beside the capture
  pipeline, on a 15 W laptop.
- **When to revisit:** if the Phase 10 closeout's throughput measurement shows
  model load is a material fraction of a job. The clean shape is a long-lived
  worker that drops the context whenever the queue empties *or* a blocker
  starts, which is what makes it more than a one-line change.


63. **A truncated recording that EOSes cleanly still transcribes as complete.**
  `Reader::rest` tells a cancel and a posted `ERROR` apart from EOF, but a
  Matroska/WebM file cut mid-stream — which `finish_recording` already knows it
  produces, since it emits `UserError::StopNotClean` — commonly EOSes with no
  bus error at all. `rest` then returns `Ok(partial)` and whisper transcribes
  ten seconds of a ninety-second take as the whole thing. S4's `""`-means-never
  -transcribed makes a short transcript indistinguishable from a right one, so
  nothing tells the coach.
- **Why deferred:** the check itself is cheap — `Clip::recording_duration` is
  already on the clip, and a run that read materially less than that is
  partial — but it means threading an expected duration down into media's
  `read_all`, which is a real interface change on a path shared with export,
  for a case that needs a crash or a kill mid-recording to reach.
- **When to revisit:** when the extraction interface is next opened anyway
  (Phase 11's model download touches neither, but a streaming or partial-
  transcript feature would), or if `StopNotClean` turns out to be common on
  real hardware rather than the backstop it is meant to be. The honest fix is
  for `read_all` to report how much sound it got and for the bus to mark a
  short read as `Finish::Failed("the recording is incomplete")`, which reuses
  the slot Phase 10 already has.

64. ~~**A panic inside a transcription job would wedge the queue for the
  session.**~~ **Fixed in P3 Task 3.2**, `pundit-media/src/job.rs`. The
  deferral said to revisit this "if the job thread grows a path that can panic
  on data", and vision is that path: an analysis indexes tensors and slices on
  what it decoded, and it shares the one running slot with transcription and
  tracking (spec B2), so a job that never finished would now stop all three.
  Every job thread starts through `job::spawn`, which sends the body's own
  terminal message or `JobMessage::panicked` carrying the panic's text, and
  then drops the sender. Left here as the record of why it waited.

65. **A cancelled whisper run keeps eight threads busy for ~12 s after the
  coach has moved on.** The abort callback is consulted once per encode and
  once per decode pass, so `Transcriber::drop` cancels and lets the thread go
  (spec S5). When a recording is what preempted it, that abandoned run
  overlaps the capture encoder at the start of the take; when the queue starts
  the *next* job immediately afterwards, two whisper contexts can be resident
  at once.
- **Why deferred:** the obvious guard — hold the `JoinHandle` and refuse to
  start a job while a cancelled one is still dying — stalls the queue
  silently, because `run_next_if_idle` only runs at the bottom of a bus turn
  and nothing wakes the bus when that thread finally exits. Making it correct
  needs a deadline, which is more machinery than a CPU spike deserves.
- **When to revisit:** if the closeout's throughput measurement shows the
  overlap costing real time on a take, or when a cheaper stop exists — a
  whisper.cpp whose abort is honoured per graph node would make the whole
  question go away, so check it when bumping whisper-rs (BACKLOG #60).

## Phase 11 deferrals (spec `docs/superpowers/specs/2026-09-21-linux-port-phase-11-design.md`)

66. ~~**`a_file_with_no_audio_track_is_a_failure` flaked once under a parallel
  run.**~~ **RESOLVED — it was not a flake.** In `pundit-media`'s
  transcribe tests, `matroskademux`'s "Internal data stream error" sometimes
  reached the test before the "no sound" answer it asserts. GitHub's runner
  then failed it every time, which made it reproducible: `Reader::start`
  checked its error slot before its stream-collection slot, so the demuxer's
  follow-on `not-linked` error could beat the "no audio track" answer it
  follows. Reading the collection first makes a video-only file always
  report "no sound" (commit `bb17011`).

67. ~~**A frame-accurate scrub on a full Trace file lands 0.2–0.3 s off
  target.**~~ **RESOLVED — it was never off: the targets were misprinted.**
  The readings came from a throwaway bus test that scrubbed to fractions of
  the file's duration (½, ⅕, ⅘, 0.35) and printed each *target* rounded to
  whole seconds (`{:.0}`), next to the position at full precision. On a
  1623.3955 s half, ½ is 811.69775 and 0.35 is 568.188425, and the player
  reported exactly those; the 70 s remux's "25 → 24.50" was 0.35 × 70.011
  = 24.50385, likewise exact. Every one of the eight readings equals its true
  target to the printed digit. Nothing on the UI path loses time either: the
  scrubber's value is a continuous `f32` (0.25 ms resolution at an hour), the
  release sends the last moved value, and the readout shows the seek's
  target until it settles.
  - **Standing proof:** the ignored
  `real_footage_scrubs_land_on_the_frame_export_picks` (20 targets per half,
  System and production sinks) — on two Trace halves, all 80 landings
  reported their target to <0.1 ms and showed the frame export picks — and
  the CI guard `a_paused_scrub_shows_the_frame_export_picks` on an
  edit-listed fixture.

68. **The volume slider still uses Slint's stock slider.** A volume drag that
  leaves the window on X11 ends with a cancel the stock `SliderBase` ignores
  (it returns early for any button but the left), so the volume applies but its
  `released` never fires and the value isn't persisted.
- **Why deferred:** the same Slint behaviour froze the scrubber, which got a
  custom `scrubber.slint`; the volume slider's failure is cosmetic by
  comparison — the level changes, it just isn't remembered.
- **When to revisit:** if the volume ever doesn't stick, or when the scrubber's
  wrapper is generalised.

69. **The preview can starve under heavy load from other programs.** While
  measuring A/V sync, many preview runs delivered 4–80 frames in 50 s instead
  of 1500, on both audio sinks and both source files, while another session's
  GPU/CPU-heavy process was running (and occasionally without it). The preview's
  Rust pump ran at the test's `nice 19` priority there.
- **Seen again 2026-09-26**, as a *test* failure rather than a measurement:
  `preview.rs::a_seek_lands_on_the_frame_it_asked_for` timed out on its frame
  poll during a full `cargo test --workspace` under `nice -n 19`, and passed on
  its own in 7.3 s. Same shape — the pump starved by everything else in the
  run — so this entry, not a new one.
- **Why deferred:** seen only in tests at lowest priority under deliberate
  contention; the clean runs were perfect (1500/1500 frames).
- **When to revisit:** if the coach's preview ever stutters or freezes while
  something else is running — check whether the pump needs a higher priority or
  the preview a way to drop frames gracefully.

70. **A heap-corruption abort once, tearing down whisper in the harness.** One
  run of `crates/pundit-harness/tests/transcribe.rs` died with glibc's
  `corrupted size vs. prev_size` (SIGABRT); three reruns were green. The
  Android branch (`claude/android-tablet-port`) records the same crash as its
  BACKLOG #69, a whisper teardown crash.
- **Why deferred:** seen once, not reproduced, and unrelated to the change it
  surfaced under (the `frame_at` boundary fix).
- **When to revisit:** soon if it recurs — heap corruption in a native library
  can crash the real app mid-transcription, not just a test. Start from the
  Android branch's findings, and try the harness under a memory checker
  (`MALLOC_CHECK_=3`, or valgrind on the single test) to catch it at the write
  rather than at the free.
- **Update (2026-09-22, match-vision Task 1.2):** not whisper-only. The GL
  export suites abort the same way (`corrupted size vs. prev_size`,
  `malloc(): mismatching next->prev_size`): 2 of 20 runs of the media
  `tests/export.rs` binary at 07312a2, 3 of 20 with Task 1.2, and once in the
  harness's `tests/export.rs`. None under gdb (25 runs). No unsafe code was
  added, so a race in GStreamer, Mesa or llvmpipe teardown is the lead. It is
  now recurring, so it is due: `MALLOC_CHECK_=3` or valgrind on the media
  export binary.
- **Update (2026-09-25):** `malloc(): mismatching next->prev_size` in the media
  `tests/export.rs` again, in 2 of 3 `cargo test --workspace` runs at 2430d39,
  both after 5 of its 29 tests — while the same binary run on its own passed 4
  times in a row. Whatever the trigger is, it is not the binary alone.
- **Update (2026-09-27, #87's close-out):** the media `tests/export.rs` binary
  again, `corrupted size vs. prev_size`, SIGABRT, **after 6 of its 29 tests** in
  a `cargo test --workspace --no-fail-fast` run under `nice -n 19`. The binary
  alone straight afterwards: **29/29 in 39.7 s.** Everything else in that
  workspace run passed (933 tests, 0 failures), and the change it surfaced under
  touched only `pundit-app`. So the 2026-09-25 reading holds exactly — it is not
  the binary alone, it is the binary under a loaded machine — and this is the
  fourth sighting of the pattern.
- **Investigated 2026-09-27 (the coach said go).** First real evidence, and it
  moves the entry on from guesswork. **Reproduced without a workspace run:**
  4 concurrent copies of the media `export` binary, `nice -n 19`, gave **1 crash
  in 16 runs** — so the trigger is CPU contention, not `cargo test --workspace`
  specifically, and a repro costs ~25 minutes rather than an hour. Two things
  that do **not** reproduce it, so nobody need retry them: the single suspect
  test (`a_vp8_counter_fixture_decodes_to_its_frame_numbers`) at 8-way
  concurrency, **0 in 64**; and one binary alone.
- **Two cores captured and symbolised** (`systemd-coredump` keeps them; no
  `ulimit` fiddling needed, and **gdb on a core works where gdb *around* the
  process never reproduced it**). Both abort identically — glibc
  `unlink_chunk` → `corrupted size vs. prev_size` inside `_int_malloc`, on a
  **non-main (per-thread) arena** — but they are discovered in completely
  different places:
  - `gst_bus_set_flushing` ← `gst_element_change_state` ×3 ← `set_state(Null)` in
    `fixtures::decode_each`, i.e. tearing a **fixture decode** pipeline down
    (`a_vp8_counter_fixture_decodes_to_its_frame_numbers`).
  - `gst_buffer_new_allocate` ← **`libgstvideoparsersbad.so`** on a streaming
    thread — a different codec and a different phase entirely.
- **So the backtrace does not name the culprit**, and reading it as if it did is
  the trap here: the corruption is already in the free list, and the crash lands
  on whichever `malloc` next walks it. That also explains the entry's three
  "faces" — they are one bug discovered in three arbitrary places, not three
  bugs.
- **What the heap says, which is more informative than the stack.** Read by hand
  from the first core: the chunk being unlinked has size `0x60`, while the
  following chunk's `prev_size` reads **0** instead of `0x60`, and the words
  around it are zeros. That is a **run of zeros written past the end of an
  allocation**, clobbering a chunk header — the signature of a clear with a
  miscomputed length, not a wild pointer. No `unsafe` exists in `fixtures.rs`,
  and its frame mapping uses `VideoFrameRef` plus a slice that would **panic**
  rather than corrupt, so the writer is in C.
- **CORRECTION (2026-09-27, later the same day): the libvorbis conclusion below
  is WRONG, and the way it is wrong is the whole lesson of this entry.** The
  fixtures were moved off Vorbis (see the spec
  `2026-09-27-unwanted-decoders-design.md`), which took `vorbisdec` constructions
  in the harness's `transcribe.rs` from 60 to **0** — and it **still aborted**,
  `corrupted size vs. prev_size`, on the very next workspace run. A fifth
  sighting, a fifth distinct discovery site, and this one with no Vorbis in the
  process at all:

      unlink_chunk → _int_malloc → g_malloc
        → gst_atomic_queue_new → gst_va_pool_new   (libgstva, VA-API)

  So `free(): invalid pointer` inside `vorbis_book_clear` was **itself a
  discovery site**: a legitimate pointer failing validation because the chunk
  header beside it was already clobbered. glibc's checker moved the abort
  *earlier*, not *to the culprit*. This entry had already written, in bold, that
  the backtrace does not name the culprit — and then I read the checker's
  backtrace as if it did. **The lesson generalises: no allocator-detected abort
  names the writer, however early you catch it.** Only a tool that watches the
  *write* can, which means ASan and nothing less.
- **The source of the corruption is therefore still unknown.** VA-API is now in
  the frame as a discovery site, and by the argument above that is not evidence
  against it either.
- **Superseded: libvorbis frees an invalid pointer when a Vorbis decoder
  is torn down.** `MALLOC_CHECK_=3` is **inert on glibc 2.34+** — the checks
  moved into `libc_malloc_debug.so`, which has to be preloaded with the tunable
  set, so the entry's own suggested command would have measured nothing. Run
  properly, `LD_PRELOAD=libc_malloc_debug.so.0
  GLIBC_TUNABLES=glibc.malloc.check=3` raised the hit rate from **1/16 to 4/24**
  and moved the abort from a bystander `malloc` to the culprit's own `free`:

      free_check → vorbis_book_clear → vorbis_info_clear   (libvorbis.so.0)
        → libgstvorbis.so → libgstaudio (the decoder base class)
        → gst_element_change_state        ← the decoder being torn down
        → libgstplayback.so (decodebin3)

  So the `corrupted size vs. prev_size` aborts, in all their places, are the
  damage being *discovered*; the write is libvorbis freeing a pointer that is
  not a live allocation while clearing its codebooks.
- **Why our tests and not everyone's.** Every audio-bearing fixture is
  `vorbisenc` (four sites in `fixtures.rs`), and `decode_each`'s video path
  selects only the video stream — "the other streams of the file are left
  alone". But **`decodebin3` decodes every stream regardless**: measured with
  `GST_DEBUG=GST_ELEMENT_FACTORY:4`, a video-only fixture decode still builds a
  `vorbisdec`, and **`caps=video/x-raw(ANY)` does not stop it**. So each fixture
  decode spins up a Vorbis decoder it never reads and then tears it down, which
  is the crashing path, ~8 at a time across the test threads.
- **Not reproducible with one stock pipeline**: `gst-launch-1.0 uridecodebin3`
  over a VP8+Vorbis file, 6-way, under the same heap checking — **0 in 60**. It
  wants several decoders coming down inside *one* process under load, which is
  what a test binary does and a single `gst-launch` never does.
- **The app is exposed too, but narrowly.** The player decodes audio, so a
  source whose audio is Vorbis would run the same teardown on every project
  close or source change. Football footage is MP4/MKV with AAC, so this is
  unlikely rather than impossible — and it is the honest answer to this entry's
  original worry ("can crash the real app"): yes, but only for Vorbis input.
- **The ASan recipe, and it does not need a rebuild** (2026-09-27). An
  instrumented build is **not available**: `RUSTFLAGS=-Zsanitizer=address` on the
  installed `nightly-2026-04-19` fails to compile `option-operations`, a
  gstreamer-rs dependency (36 errors). **Preloading works instead, and is the
  right tool for this signature anyway:**

      ASAN_OPTIONS=detect_leaks=0:verify_asan_link_order=0:halt_on_error=1:abort_on_error=1 \
      LD_PRELOAD=/usr/lib/x86_64-linux-gnu/libasan.so.8 <test binary>

  Uninstrumented code gets no per-access shadow checks, but ASan's **`memset` /
  `memcpy` / `strcpy` interceptors do bounds-check**, and the heap forensics in
  this entry point at a *run of zeros past an allocation* — i.e. exactly what a
  `memset` interceptor catches. **Proven live before trusting it**: a deliberate
  `memset(malloc(64), 0, 96)` under that exact command reports
  `heap-buffer-overflow … WRITE of size 96` with the writing frame named. A
  diagnostic nobody has proved can fail is the recurring mistake in this entry;
  that check takes ten seconds.
  The export suite runs clean under the preload (6.4 s for one test), so the
  harness is valid.
- **ASan does not fire, and that is a result: 0 crashes in 24 runs, 0 reports.**
  Against a base rate of ~21% (11 in 48 below), zero in 24 is a one-in-a-thousand
  coincidence, so ASan's allocator — redzones, quarantine, a different layout —
  **suppresses** the race rather than missing it. Do not spend more runs there.
  The same is true of an instrumented build if a working nightly ever appears:
  the allocator is the thing that changes, and the allocator is what the bug
  depends on.
- **THE SHARPEST CONSTRAINT, measured the same day: it needs parallel tests
  *inside* one process.** Four concurrent copies of the media `export` binary,
  all under glibc heap checking:

  | configuration | crashes |
  |---|---|
  | parallel tests per process, GPU | **6 / 24** |
  | parallel tests per process, `/dev/dri` hidden (software) | **5 / 24** |
  | **`--test-threads=1` per process** | **0 / 24** |
  | parallel tests per process, under ASan | 0 / 24 |

  A loaded machine is **not** sufficient. So this is concurrent use of something
  **process-wide**, not a decoder bug and not contention — which also retires
  three suspects at once: the GPU and VA-API (the software arm crashes at the
  same rate), the video decoder (the two arms use different ones), and libvorbis
  (already retired above).
- **The one documented piece of process-wide state is the prime suspect:**
  `composite::Gl::shared()` — **one surfaceless `GLDisplayEGL` and one
  `GLContext` per process, never dropped**, handed to every export and every
  GL-sink preview by `clone()`. Its own doc records that these wrap the *same*
  `EGLDisplay` and that finalizing one calls `eglTerminate` for all. A GL context
  is not safe for concurrent use, and llvmpipe reaches it through EGL exactly as
  the real driver does, which is why hiding the GPU changed nothing.
- **When to revisit — the experiment that would confirm or kill it:** run the
  4-way repro against a suite that uses **no** GL (the media `copy` tests are a
  stream copy) and against one that shares the GL context hard (`export`). If
  only the GL one crashes, it is `Gl::shared()`, and the fix is a real design
  question — a context per job costs the `eglTerminate` problem that doc comment
  exists for, so it would more likely be a mutex around the pipelines that use
  it. Note the harness's `transcribe.rs` and `reel` have also crashed, so check
  whether those paths touch a GL display before concluding.
- **Superseded plan (kept because its first item shipped for other reasons):**
  1. **Stop decoding audio we throw away** (fixes the flake, and is less wasted
     work regardless). The `caps` property does not do it, so it needs proper
     stream selection: take the `StreamCollection` message and send a
     `select-streams` event naming only the video stream, ~30 lines in
     `decode_each`. **Verify with the repro that now exists**: 4 concurrent
     copies of the media `export` binary under the preload above, 24 runs,
     expecting 0 where it is currently 4. Note the *audio* fixture path
     (`fixtures.rs:618`) legitimately decodes Vorbis and would still tear a
     decoder down, so confirm whether that path is a second source before
     calling it closed.
  2. **Upstream.** This is a libvorbis/gst-plugins-base bug, not ours. The
     material for a report is in this entry: the backtrace, the heap forensics
     (a chunk's `prev_size` zeroed while the neighbour's size reads `0x60`), the
     hit rates, and the fact that GStreamer 1.24.2 on Ubuntu 24.04 is the
     platform. A single-process C repro building and tearing down N vorbisdecs
     concurrently is what a maintainer would want, and does not exist yet.
- **Update (2026-09-25, the rename):** a **third** face of the same pattern, and
  the most informative one. A workspace run at 77cc357 died in the harness's
  `tests/transcribe.rs` after its first test with `gst_mini_object_copy:
  assertion 'mini_object != NULL' failed` followed by `Caught a segmentation
  fault while loading plugin file: …/libgstlevel.so` and exit 255; the binary
  alone passed 18/18 straight after. That is a crash **inside GStreamer's
  registry-loading fork**, not in whisper and not in Mesa, which makes a shared
  registry contended by several test binaries at once the strongest lead yet —
  and points at `GST_REGISTRY=<per-binary path>` as the experiment to run before
  any memory checker. The whole class is: one test binary of several, always
  under a parallel workspace run, never alone. 917 tests passed in that run and
  nothing asserted false.

71. **A possible thump at the start of every commentary recording.** The
  Android session's audio analyzer found that real Linux recordings open with
  a full-scale negative DC step decaying over ~350 ms (its BACKLOG #70 on
  `claude/android-tablet-port`). Suspects: `pipewiresrc` start-up, or
  `opusenc`. Export's audio ramp is only `RAMP_SAMPLES` = 240 samples (5 ms),
  far too short to hide it.
- **Why deferred:** not yet confirmed by ear, and it can't be reproduced with
  the test capture sources (`audiotestsrc` has no hardware start-up); the
  agent working here doesn't open the real microphone without a task that
  needs it. Added to the hands-on checklist instead.
- **When to revisit:** if the user hears a thump or click at the start of a
  take. Likely fixes, cheapest first: fade the first few hundred ms in, or put
  a DC-blocking high-pass (`audiocheblimit mode=high-pass cutoff=20`) before
  the encoder; find the true source first by recording a few seconds with and
  without each element.

72. **A source's first load at open once never settled, on CI.** GitHub run
  35697707647, attempt 1: `a_seek_in_the_final_second_stays_in_its_source`
  (`crates/pundit-harness/tests/transport.rs`) opened its project, the bus
  issued the load of source 0 at 0 s (`Position { target_abs: Some(0.0) }`), and
  no settled position followed within the harness's 15 s. Attempt 2 of the
  same commit passed, as did the runs before and after.
- **Why deferred:** roughly once in seven runs and not reproduced in ~6000 local
  opens, so no fix can be shown to work yet; what is written down below is the
  evidence and the discriminator, so the next occurrence settles it.
- **Recurred locally, 2026-09-24**, on the same symptom and a different test in
  the same file: `a_skip_burst_then_a_scrub_release_never_sticks` timed out
  "waiting for settled at 0 in source 0", during a full `cargo test --workspace`
  on a machine also running a release build. The same suite reran green. So it
  is the **open**, not the skip burst, and it is not CI-only.
- **Recurred on CI 2026-09-25**, failing the v0.7.0 release workflow:
  `a_skip_burst_across_a_source_boundary_lands_on_the_accumulated_target`, same
  "settled at 0 in source 0" — every failure so far is `Rig::open`'s settle,
  whose message is that string whatever the test goes on to do.
- **Two red herrings, both from the dump, both ruled out (2026-09-25).** The
  unconsumed-events dump listed `Basket` first and `ProjectOpened` second, which
  looked like the bus's new opening event being mishandled. It is an artifact:
  `Rig::open` waited with `poll_until`, which receives events without consuming
  them, and its condition reads only the *latest* `Position`, so **every**
  failure in these files dumped the whole session from its first event
  regardless of the cause. The rigs now wait step by step (`wait_opened`, then
  `wait_settled`), so the next dump names the step and holds only what came
  after it. The other red herring is this entry's own guess above: only one load
  is issued at open (`commit` → `ensure_loaded`, one `Position` target in the
  dump), and in every failure so far the bus was **fresh** — its `playbin3` had
  only ever gone NULL → READY, so `take_down` can have waited on nothing and no
  stale `ASYNC_DONE` can have existed.
- **What is left, and it is in the player.** `Flight::Loading`,
  `Flight::Seeking` and `Flight::Settling` each wait for an `ASYNC_DONE` with no
  bound, and in all three `Bus::publish_position` publishes nothing — the first
  two have a target to republish, and `Settling` has neither a target nor
  `is_idle`, so it publishes nothing at all. One lost `ASYNC_DONE` is therefore
  a permanent silent wedge, in the app as much as in a test: a project that
  opens on a black frame, a scrubber stuck at 0, no error, for ever. The symptom
  says a message was lost; it does not yet say which.
- **Ruled out by measurement** (a `playbin3` running the player's load sequence
  — READY, `uri`, PAUSED — on a WebM fixture, this laptop, GStreamer 1.24.2):
  READY → PAUSED returned `Async` 200/200, so `preroll` never silently misses
  the `ASYNC_DONE` it then waits for; and the pipeline had already committed to
  `(Success, Paused, VoidPending)` at the instant the load's `ASYNC_DONE` was
  *posted*, 200/200 — the commit precedes the post, so the staleness filter in
  the `Loading` arm cannot absorb the message it is waiting for, and the bus
  thread, which reads the state later still, can see it only more settled. The
  filter now logs when it absorbs one (`player: absorbed an ASYNC_DONE …`).
- **Not reproduced:** ~6000 first opens (8 threads, fresh bus each, 8 busy-loop
  processes for load) settled every time, the slowest in 348 ms — against a
  15 s bound, so the timeout is not merely too short *here*. Twelve rounds of
  the `transport` binary under the same load were clean too (its fixture
  encodes time out at that load, which is the noise you will see).
- **Fourth sighting, 2026-09-30 — and the discriminator below could not be
  applied, because CI throws the evidence away.** GitHub run 36655386922, on a
  **docs-only** pull request (two markdown files), so nothing in the change can be
  implicated: `a_skip_burst_across_a_source_boundary_lands_on_the_accumulated_target`
  again, again `Position { source_index: 0, target_abs: Some(0.0) }` with no settle
  inside the bound. **Measured, on this laptop:** that test's run prints
  `bus: loaded …` **three times under `--nocapture` and zero times without it** —
  libtest captures the bus thread's stderr, and `rust.yml` runs a plain
  `cargo test --workspace`. So every marker this entry's discriminator keys on is
  discarded before it reaches a log, and has been for all four sightings.
- **Fifth sighting, minutes after the fourth, and the rate estimate above is no
  longer true.** GitHub run 36657306149, the same docs-only branch one push later:
  `a_pause_right_after_a_skip_is_anchored_at_the_skip_target` this time, same panic
  site (`pundit-harness/src/lib.rs:164`), same settle timeout. So **two of three
  workspace runs** on a branch that changes only markdown, where this entry says
  "roughly once in seven runs" — and each recovered on a re-run of the failed job.
  The failing test is whichever one happens to open a project at the wrong moment,
  which is the entry's own conclusion ("every failure so far is `Rig::open`'s
  settle") holding across five sightings and three different test names.
- **What that means for the fix below:** the next step is not `GST_DEBUG`, it is
  making the failure **carry its own evidence**, exactly as #75's resolution did for
  the replay margin — have the rig collect the bus's diagnostic lines and include
  them in the timeout's panic message, rather than hoping a log survives. `--nocapture`
  in CI is the cheap alternative and a worse one: it interleaves every test's output
  in a parallel run, so the lines cannot be attributed to the test that failed.
- **When to revisit:** the next occurrence, which is now decidable from the
  failing test's captured stderr:
  - **`bus: loaded …` present** → the preroll finished and the *seek's*
    `ASYNC_DONE` was lost (`Flight::Seeking`), or `apply_playing`'s redundant
    PAUSED returned `Async` and left the slot in `Flight::Settling` waiting for
    an `ASYNC_DONE` nobody will post.
  - **absent** → the preroll itself never completed, and
    `GST_DEBUG=*:3,playbin3:5,urisourcebin:5` on the run is the next step.
  - **`player: absorbed an ASYNC_DONE …` present** → the staleness filter ate
    it after all, and it should be replaced by counting the stale messages
    `take_down` can leave behind rather than querying the state.
- **The fix nobody has earned yet:** a bound on a load and on a seek, armed in
  `Bus::load` and disarmed by `Loaded`/`SeekDone`, dispatched by the deadline
  the run loop already computes (`skip_deadline`, `start_deadline`), and on
  expiry an `Event::Error` and a `reset` — about thirty lines and one more
  deadline. It would turn the wedge into a reported error, which is right for
  the app whatever the cause; it would *not* make the test pass, and a bound
  loose enough to be safe on a loaded CI runner would fire well after the
  harness's own 15 s. So it waits for the diagnosis.


73. **`,` looks stuck across a timestamp gap longer than half a frame.** A back
  step seeks to half a nominal frame before the shown frame's start, nominal or
  its own, whichever is earlier (`step_target`, `player/mod.rs`). Where the file
  has a real gap there, the ACCURATE seek lands on the frame after the gap —
  the frame already shown — so the step does nothing and repeating it doesn't
  help. (A frame held long is left, since its own start is earlier; but after a
  scrub lands inside one, its start is the scrub's target, so `,` creeps back
  half a frame a press until it leaves it.)
- **Why deferred:** MP4 game footage has no such gaps (Trace's irregular
  29.997 fps drifts well under half a frame); only a VP8 WebM fixture does.
  A fix needs a retry further back, or knowing the frame boundaries.
- **When to revisit:** if a coach reports `,` stuck on some file, or when a
  source with real gaps (a screen capture, a dropped-frame phone clip) is used.

74. **A reel trim can be a silent no-op.** A goal merged into the previous reel
  entry (at or before its end on the same source, spec R2) has its lead-in
  ignored, and a lead-in that reaches back past the previous entry's end is
  clamped to it, so setting either changes nothing in the export. The Match
  panel still offers "Reel starts here" on that goal, and its row still shows
  the stored span ("−12 s / +6 s") rather than what the reel will play.
- **Why deferred:** disabling the button, or showing the clamped span, needs
  the row to know the reel's merges and clamps, which only the plan computes;
  the goals a coach tags are rarely close enough to merge. The stored trim is
  kept, so it takes effect if the earlier goal is deleted or trimmed.
- **When to revisit:** if a coach reports a trim that "does nothing", or when
  the Match panel grows a view of the reel's actual entries.

75. **A skip burst's replay margin failed once, by 54 ms.** One run of
  `a_skip_burst_while_playing_lands_where_replay_puts_it`
  (`crates/pundit-harness/tests/recording.rs`) failed with "replay reaches
  7.5019 before the pause anchored at 7.4479". The same suite reran 12/12
  green, and two full workspace runs either side were green. Seen while the
  match-event editor's commands landed, which touch no transport code.
- **Seen a second time, 2026-09-27**, during #87's close-out: same test, same
  suite, in a full `cargo test --workspace` under `nice -n 19` with three other
  builds contending for the machine. Passed alone on the next run (2.86 s). The
  failure message still does not carry the margin, so the second sighting tells
  us no more than the first did — which makes the diagnostic below the next step
  rather than a nicety.
- **Why deferred:** twice now, both times on a loaded machine, on a timing margin
  rather than a logic error. It is not BACKLOG #70 (that is the whisper
  teardown).
- **Done 2026-09-27 (the coach said go), and the measurement changes the
  reading.** The margin is now computed into a named `REPLAY_MARGIN` and
  **printed on every run**, not only on failure, so a sighting reports its own
  number. Then it was measured rather than guessed: **24 runs at 4-way
  contention gave 6.2–12.4 ms, median 8.9 ms** — the 0.05 tolerance is about
  **four times** the worst that load produces.
- **So the tolerance is not too tight, and widening it would be the wrong fix.**
  Both flakes came in at **54 ms**: six times the median and far outside the
  measured spread, i.e. an **outlier with its own cause** rather than the tail of
  normal variance. Something occasionally stalls by ~50 ms; this assertion is
  the only thing that notices, and silencing it would discard the evidence.
  (The entry previously said the message "does not carry the margin" — it did
  carry both positions, and subtracting them is where the 54 ms comes from. What
  was missing was the margin on the *passing* runs, which is what makes an
  outlier recognisable as one.)
- **When to revisit:** on the next sighting, which will now print its own
  margin. If it is ~54 ms again, the cause is a discrete stall worth chasing (a
  seek landing late, or the recording thread descheduled) rather than a
  tolerance to adjust. A coach reporting a take whose replay drifts from where
  they paused is what would make it urgent.

76. **`a_cancelled_copy_leaves_nothing`'s fixtures sit on the 30 s EOS bound.**
  That test generates two 1280×720 × 900-frame H.264 sources, and
  `fixtures.rs`'s pipeline bound is 30 s; unloaded the suite takes ~29 s, so on
  any busy machine (CI included) the test fails in fixture generation with
  "fixture pipeline did not reach EOS within 30 s" — nothing to do with the
  copy it tests. Verified pre-existing on unmodified HEAD under load, 3 of 3.
- **Why deferred:** it fails loudly in the fixture, not silently in the code,
  and the copy path itself is proven by the deterministic deadlock test.
- **When to revisit:** the first time CI fails on it, or with the next test
  touching that file — a smaller fixture (fewer frames, or 640×360) is the fix,
  not a wider bound.

77. **An export queue across projects.** The coach (2026-09-24): "i open project
  1, do stuff, enqueue. then project 2, do stuff, enqueue, then start the queue
  and walk away for a bit. other apps have this sort of thing, like mkvtoolnix's
  muxer." Today an export run belongs to the open project and starts at once
  (`bus/export.rs`'s `Active`/`ExportRun`), so exporting three matches means
  sitting through three of them.
- **The shape, agreed with the coach:** an `ExportJob` is already a
  self-contained value — the plan, the sources' paths, the scoreboard, the
  cues, the tags — so **enqueue is "build the jobs now, run them later"**, and
  the queue is a list of them. The work is: don't tie the run loop to the open
  project; a panel listing what is waiting, by project name, with Start and
  remove; and one job's failure (footage moved, disk full) failing only itself.
  A first version keeps the queue in memory — closing the app loses it — and
  doesn't let a queued job be edited.
- **Why deferred:** only by the coach's own order (2026-09-24): the detection
  measurement comes first. It is not blocked on anything.
- **When to revisit:** as soon as P3's measurement is done, or sooner if the
  coach asks.

78. **App settings for the things an export writes.** The coach (2026-09-24):
  "add to the backlog an appsettings? we don't need a screen for it yet. maybe we
  already have it. e.g. the srt file gen, the other chapter track, etc. these are
  general app config settings to be turned off or on."
- **What exists already, and where.** Two homes, deliberately: per-project in
  `Preferences` inside `project.json` (`scan_volume`, the preview volumes, the
  last export resolution, quality and scoreboard mode, `pip_for_new_recordings`)
  and machine-wide in `$XDG_CONFIG_HOME/pundit/state.json` (the last project,
  the speech model, the window size). **Machine-wide is the cheap one:** a field
  added to `Preferences` is a `formatVersion` bump every time, which is why the
  whisper model picker went to `state.json` in the first place.
- **What would become a setting:** whether an export writes the `.srt` sidecar,
  the `.chapters.txt` list and the embedded `tx3g` track; whether it writes the
  file tags; possibly the reel's default lead-in and tail (20 s / 6 s today,
  constants), and the avatar's pulse constants. All are "on" today with no way
  to say otherwise.
- **The decisions to take first:** which of those are a property of the *machine*
  (this coach never wants a sidecar) versus of the *project* (this match is for
  YouTube, that one is for the parents' TV); and whether a setting that changes
  what a file contains belongs in the export sheet next to its own row rather
  than in a settings screen at all.
- **Why deferred:** no screen is wanted yet, and every one of these is currently
  the right default. It becomes real the first time a default is wrong for a
  coach, and then it should arrive with its home already decided rather than as
  six checkboxes.
- **When to revisit:** the first "can I turn that off", or when a second coach
  uses the app.

79. **What the app does over a forwarded X11 display, and saying so.** The coach
  asked (2026-09-24) whether it runs over remote X11. Expected answer: **no, by
  construction** — `main.rs` selects Slint's Skia OpenGL renderer and fails
  loudly otherwise, and that renderer needs **EGL** (CLAUDE.md's decode rules:
  EGL is what lets GStreamer import decoded frames without a CPU copy; X11
  forwarding offers indirect GLX). Even if it started, forwarding raw 1080p30
  frames is gigabytes a minute with no hardware decode at the far end.
- **What to do:** run it once over `ssh -X` and record **exactly** what happens —
  a clear message naming the renderer, or a confusing crash. Then say it in the
  README beside the other runtime requirements: this is a local application, and
  remote use means streaming the screen (Sunshine/Moonlight, NoMachine, RustDesk
  or VNC on the real session), not forwarding the display. If the failure is
  confusing, make the message name EGL and point at that line.
- **Why deferred:** nothing is broken; the gap is that a coach who tries it gets
  no explanation.
- **When to revisit:** with the next README pass, or the first time anyone asks
  again. The docs session on `claude/docs` owns the README and is the natural
  place for the wording once the behaviour is known.

## P3 deferrals (spec `docs/superpowers/specs/2026-09-22-match-vision-design.md`, verdict `docs/superpowers/spikes/2026-09-24-match-vision-measurements.md`)

80. **The restarts — MOSTLY DONE (2026-09-25), and they already changed the
  answer.** The coach timed them the same week: **9 of 16**, being all of match
  A (6) and all of match C (3); match B's `kickoffs.txt` is still the blank
  template. This entry read "never been written down" long after they were
  written down and used — corrected here.
- **What they bought, from
  `docs/superpowers/spikes/2026-09-24-match-vision-measurements.md`:** `W` is
  **60 s, not the shipped 150** — the longest walk-back ever timed, rounded up,
  covering 9 of 9 with 13.9 s to spare. Suggestions fell from claiming **57% of
  a held-out match to 29%**, and lift over chance rose from **+0.15 to +0.48**.
  Given the nine real restarts — the picture's half done perfectly — the
  confirmation rule finds **9 of 9 goals with no false ones**, every cheer
  within 2.4 s of the tagged frame.
- **So the restarts did their job: they proved which half is broken.** It is not
  the sound. The picture offers about 21 candidate restarts a half where a half
  holds three or four, which is why suggestions are still not shown (#82) and
  why P5's formation check is now the whole question.
- **What is left:** match B's seven restarts, which would widen the sample but
  cannot change the conclusion — the two filled matches are the held-out pair,
  and B is the tuning match. Worth doing only alongside the next detection
  attempt, not before it.
- **Why deferred:** the remaining seven are the coach's own step, and nothing
  in the repo can do it.
- **When to revisit:** with P5, if it is ever attempted.

81. **Task 3.6 — the *model* spike (L3, V-2, V-7) — is deferred with P5 rather
  than run.**
- **Not the spike that was run, which is the confusion this line exists to
  stop** (the coach asked, 2026-09-25). P3's measurement pass *was* done and is
  `docs/superpowers/spikes/2026-09-24-match-vision-measurements.md`: it measured
  the **classical** analyzer — Goertzel bands, dB over a rolling median,
  stillness, thumbnails — produced the verdict, and cleared the throughput bar
  at **71–88 s per file, ~23× realtime** against ≤ 5 minutes. No model, no ONNX
  runtime, nothing downloaded.
- **This one is about a neural detector:** `rten` against `ort` on D-FINE-N at
  640 and 960 input, far-side person recall on 20 hand-checked kick-off frames,
  how often the stitch-seam ghost appears, a cold start end to end (V-7) — and
  V-2, whether the virtual camera frames both halves of the pitch at a kick-off
  at all.

  Its entry condition is met (the
  goal precision bars failed), but it measures `rten` against `ort` for a
  detector nothing has decided to build, and V-2 — does the virtual camera frame
  both halves of the pitch at a kick-off? — only matters once P5 is wanted.
- **Why deferred:** the verdict says the formation check is the **only** cue
  left with a chance of clearing the bars, and whether to build it is the user's
  call, not a measurement's. Running the runtime benchmark first would pin a
  dependency choice to a phase that may never start.
- **When to revisit:** when the user asks for P5, or for P6 (click-to-track),
  which needs the same runtime and the same V-7 latency figure.

82. **P4 (showing suggestions) is not justified and is not started.** No
  `MatchSuggestion` in the project format, no `Analyze` command, no Match-panel
  row, no scrubber mark. Held out, the rule finds 7 goals of 9 with 29 false
  ones, and the periods are worse.
- **What would change it:** #80's restarts, a learned audio tagger in place of
  the level cue (the spec defers YAMNet), or P5's formation check. Not another
  threshold sweep: 540 points were tried and the best of them is worth +0.15
  over chance on footage it had not seen.
- **Why deferred:** the analysis backend is built, measured and shelved exactly
  as P3 said it would be if the numbers came out this way.
- **When to revisit:** when one of the three inputs above lands.

83. **`a_pause_while_a_flushing_seek_recovers_playing_sticks` flaked once under
  the full workspace run.** `pundit-media`'s lib tests, 2026-09-24: one
  failure inside a `cargo test --workspace` (every other suite green), and the
  same suite reran 122/122 green on its own a minute later. The test races a
  pause against a flushing seek's recovery, so a loaded machine — the workspace
  run has several GStreamer suites alive at once — is the obvious suspect.
- **Why deferred:** one occurrence, green on rerun, and the assertion is about
  a state the app reaches constantly and no user has ever seen wrong.
- **When to revisit:** if it fails twice, or once on CI. Then print the states
  the recovery went through rather than only the final one.

84. **Music under a goals reel.** The coach (2026-09-24): "pull in soundtrack
  files that are licensed for use, e.g. rock, edm, etc. for the goals clips." A
  reel is the one export with no commentary — it is game sound alone — and a
  highlights reel is the one thing people expect music under.
- **The mixing is the easy half.** `core::audio` already builds the regions and
  the envelope the export mixes in Rust, and the reel's entries are the same
  plan every other target uses: a track becomes one more region under the whole
  compilation, ducked or replacing the game sound. `avenc_aac` already encodes
  the result.
- **The licence is the hard half, and it is the reason this is not a weekend.**
  The app is AGPL and runs on the coach's machine; the *output* is a video they
  may upload. Bundling audio means shipping files whose licence permits
  redistribution AND synchronisation AND the coach's own upload, and YouTube's
  Content ID will flag plenty of "royalty-free" music regardless of what its
  licence says. The workable shapes, roughly in order of how little they promise
  on someone else's behalf: (a) the coach points at their own files, and the app
  only mixes; (b) a curated list of links the coach downloads themselves; (c)
  the `.deb` ships tracks, which means clearing each one and carrying the
  licences in `packaging/copyright`.
- **It should feel semi-automatic** (the coach, same day): *"pick genre"* —
  choose **rock** or **EDM** on the export and the app does the rest, rather
  than picking a file and setting levels. That works with any of the three
  shapes above: a genre is a folder the coach filled once, a curated list
  grouped by genre, or the shipped set tagged. The app then picks a track that
  fits the reel's length, starts it at the top, ducks the game sound under it
  and fades it out at the end — no controls beyond the genre unless the coach
  opens something.
- **The decisions before any code:** which of the three shapes; whether the
  music ducks under the game sound or replaces it; whether the genre is
  remembered per project or per export; what happens when a reel is longer than
  the track (loop, fade, or refuse); and whether the same track is used every
  time or it rotates, since a coach exporting ten matches does not want ten
  identical soundtracks.
- **The coach chose, 2026-09-25:** not a folder they fill — *"what i'd rather
  have is like some interface to a free music search? like free music archive
  or similar"*. So the app searches a catalogue, the coach picks a genre, and
  the track arrives without them hunting for files. That answers the shape
  question above: **(a) is still the floor, but it is no longer the ceiling.**
- **Free Music Archive cannot be that catalogue: its API is shut down.** FMA's
  own app-developers page says they retired it because of server load, and asks
  that apps host any FMA audio themselves under each track's CC terms. So FMA is
  a place a human browses, not something software queries.
- **Jamendo is the one that actually fits**, and it is worth naming so the next
  session does not re-derive it: a documented v3.0 API over a Creative Commons
  catalogue, search by genre/mood/tempo (which is exactly "pick genre"),
  **35,000 requests a month free for a non-commercial app**, and — the part that
  matters most — **the licence of each track is a field in the response**, so
  the filter can be enforced by the app instead of trusted to the coach.
  Runners-up if it ever goes away: Openverse's API (a CC aggregator that
  includes audio) and the Internet Archive's audio collections.
- **The client ID is a real decision, not a config detail.** Jamendo's API wants
  one, and this program is AGPL: an ID compiled into public source is one shared
  quota that anyone can spend, and the first abuse takes the feature down for
  every coach. **Recommended default:** ship none, put a field in the settings
  screen (#78) where the coach pastes their own, and fall back to shape (a) when
  it is empty — the app is then useful with no account and better with one.
- **Attribution is an output, and the app already writes outputs.** Every CC
  track requires credit in some form, so a reel that used one writes it beside
  the video exactly as `.chapters.txt` is written — same `finish`, same name
  cleaning, same never-fatal rule — and the text is ready to paste into a
  YouTube description. Burning a credit over the last seconds is a second step,
  not the first one.
- **The network discipline is already written down; follow it.** Transcription's
  model download is the precedent: permission carried on the job type rather
  than inferred from a path, downloads only under `$XDG_CACHE_HOME/pundit/`,
  a checksum before use, and **no test may reach the network** — serve from
  `pundit_media::fixtures::serve`. A search adds a second network surface, and
  it must be as boring as the first.
- **The shape that keeps the licence question out of the audio path:** the
  mixer does not care where a file came from. Build the local folder first, make
  the search a *fetcher* that lands tracks in the same cache the mixer reads,
  and the export path never learns that Jamendo exists.
- **Why deferred:** it is a licensing question with a small piece of code
  attached, not the other way round — and the code now has a second half (a
  search UI and a fetcher) that wants #78's settings screen to exist first.
- **SoundCloud and Bandcamp were asked about (2026-09-25) and neither works —
  and the reason rules out the whole category.** SoundCloud's API is open only
  through a manual request form (public registration has been shut for years)
  and is a *streaming and embedding* API: oEmbed players and playback, nothing
  that grants the right to put a track under a video. Bandcamp has no public
  catalogue API at all — what exists is gated to labels and merch-fulfilment
  partners and reports sales; it is a shop, and buying a track grants no such
  right either. **The blocker was never the API, it is synchronisation rights**
  — the right to marry music to moving pictures, which streaming, embedding and
  purchase all leave untouched. Jamendo works because CC licences *do* grant it
  and the API says which licence each track carries. If the coach ever monetises
  a channel, Jamendo sells explicit sync licences over the same catalogue, which
  is the upgrade path rather than a different design.
- **Openverse is the better front door, and it was found by asking about the
  wrong URL.** `api.creativecommons.org/docs/` is CC's old *licensing-engine*
  API — the thing that helps an app choose and generate a licence, documented
  for versions 1.0/1.5 and last touched in 2009. CC's media search became
  **Openverse** (moved to WordPress in 2021, `api.openverse.org`), and it
  indexes over a million audio records from **Jamendo, Freesound, ccMixter,
  Wikimedia Commons and the Free Music Archive** — including, that is, the one
  catalogue whose own API is dead. **Anonymous queries need no key** (about
  20/min burst, 200/day sustained, which is far more than a coach picking a
  track occasionally), and registering an OAuth app later only raises the
  ceiling. That removes the client-ID decision from the critical path.
- **The tradeoff to know before building:** Openverse is an *index* — genre
  lives in tags, and the audio files are hosted by the source, so a download
  still goes to Jamendo or FMA. Jamendo's own API has explicit genre, mood and
  tempo filters, which is what *"pick rock or EDM"* actually wants.
- **Decided, 2026-09-25:** the local folder first (the mixer's only input), then
  **Openverse** as the search that fetches into the same cache — no key, several
  catalogues — with **Jamendo direct as the upgrade** if tag matching proves too
  coarse for genre, and the coach's own Jamendo key an option in #78 rather than
  a requirement.
- **When to revisit:** after #78, which is where the key field lives. Nothing
  further is needed from the coach.

85. **Recent projects, and a drawer to switch between them.** The coach
  (2026-09-24): "'recent projects' menu or similar? also could have a project
  drawer to switch between recents? good for working with several project and
  going back and forth." They have three tagged matches in two clubs and move
  between them; today every switch is **Open Project…** and a folder picker.
- **Most of it exists.** `bus/state.rs` already keeps `last_project` in
  `state.json` (machine-wide, no format bump) and `restore_last_project` opens
  it at launch. A recents list is that field grown into a short `Vec<PathBuf>`,
  written where it is written now — in `open_project`, which is the one place a
  project is opened.
- **The list needs a name per entry, and the folder is the wrong one.** Two of
  the coach's projects are called `20260917-canfield` and
  `2016B vs Hudson 2026-09-19`; a drawer wants the project's own `name` and
  ideally its teams. Either store the name beside the path when it is opened
  (cheap, can go stale) or `store::read` each entry when the drawer opens (a
  handful of small reads, always right — and it is how a missing project gets
  greyed out rather than failing on click).
- **The drawer is the bigger half:** where it lives (a panel beside the sources,
  or a sheet), what a row shows, and what happens to unsaved state on a switch —
  today a project change goes through `open_project`, which is already the
  all-or-nothing path, so the switch itself is not the risk.
- **Why deferred:** nothing is blocked; it is friction, not a gap. It is also
  the natural companion to the **New match…** flow
  (`docs/superpowers/specs/2026-09-24-new-match-flow-design.md`), which creates
  the projects this would switch between — build them in that order.
- **When to revisit:** with the New match… flow, or the first time the coach
  says the picker is slowing them down again.

86. **The basket: a cut that spans matches — RESOLVED, shipped in 0.7.0.** The
  first half described below is built: `bus/basket.rs`, its own `basket.json`,
  the Basket… sheet, `core::plan::basket_plan`, and one film whose pieces each
  carry their own match's scoreboard and clock. The second half — a library to
  go *looking* for pieces made months ago — is #85 grown a level and is still
  open. The original entry follows, as the record of how the shape was chosen.

  The coach (2026-09-24), on the
  back of #85, and then more precisely: *"so i would be in project a, do a
  corner kick clip and then enqueue it, then go to project 2"* — and pressing
  Start gives **one video of all the pieces**, in the order they were added,
  each carrying its own match's scoreboard and clock. That is what they picked
  over "one file per project", which is #77's queue. **The two are different
  features that share machinery: #77 runs several jobs, this builds one job from
  several projects.** Original framing: *"especially good to get clips of the 'same thing' across different game projects?"* — every corner of the season,
  one player's goals across three matches, every time a press worked. Today a
  clip belongs to a project and an export is built from one project's plan, so
  the answer is export three reels and join them elsewhere.
- **What already fits.** Tags are the existing "same thing" (`Clip.tags`, the
  per-tag export rows), and `ExportJob` is already a self-contained value —
  entries carrying their own source paths, recordings, cues and scoreboard. So
  an export whose entries come from several projects is not a new renderer; it
  is a new way to *build* the entry list. This is the same realisation as the
  export queue (#77), one step further: the queue runs several jobs, this builds
  one job from several projects.
- **What doesn't.** The match clock and the scoreboard are per project, so an
  entry from another match must carry its own — the plan currently derives them
  once per job. The text bar's numbering ("2 / 5") means nothing across matches.
  Clip ids are unique per project, not globally. And nothing in the app can
  currently *see* another project's clips, which is the real work: a library
  view over a set of project folders, which is #85's drawer grown a level.
- **The decisions before any code:** what the set of projects is (a folder the
  coach nominates, the recents list, something explicit like a "season"); whether
  the result is an export or a saved compilation that can be re-exported; and
  whether the scoreboard is drawn per entry from its own match or dropped.
- **What the coach's own workflow needs, which is less than the library.** They
  described adding pieces **as they work**: make the clip in the project they
  are in, put it in the basket, move on. That needs no view over other projects
  at all — the basket is a list the app holds while the coach moves between
  projects, and each entry is a self-contained piece captured when it was added.
  The library (#85 grown a level) is what you need to go *looking* for pieces
  you made months ago; it is the second half, not the first.
- **Why deferred:** only by order. The first half — a basket filled as you go,
  exported as one video — is buildable now and is the smaller feature.
- **When to revisit:** after #77's queue, whose machinery it shares, or sooner
  if the coach starts gathering corners before the queue exists.

87. **Resizable panels — RESOLVED (2026-09-27): a draggable splitter either side
  of the player, remembered in `state.json`.** The coach (2026-09-25): "resizing all the panels". Every
  column is a fixed width today — the left column is **280 px** of a window whose
  minimum is 1100×700 (`app.slint:638`, `:672`), and the lists inside it are
  capped in px too (the Match panel at `min(168px, lines × 28px)`, the
  highlights panel likewise). So a coach with a 4K screen gets the same 280 px
  of clip names as one on a laptop, and a long team name or clip name is
  ellipsized when there is room to spare.
- **What it means concretely:** a draggable splitter between the picture and each
  side column, a minimum per panel, and the widths remembered. `state.json` is
  the right home (machine-wide, no format bump, and it already keeps the window
  size), which also decides the question: a width is a property of the coach's
  screen, not of a project.
- **What to watch:** the picture's content rect is computed from the space left
  over, and three things are placed in it by the app — the live rings, the
  scoreboard image and the rubber band. They all follow the content rect
  already, so a resize is not a new class of bug, but the scoreboard is
  rasterized per device-pixel size (`main.rs`'s `show_board`) and would re-raster
  on every drag frame — it needs to raster on release, or on a coalesced size.
- **Shipped** (spec/plan `2026-09-26-panels-and-fit`): the columns' widths are
  layout constraints, not a `clamp` — measured, the obvious `clamp` form put the
  player 12px under its own minimum because each column read the other's raw
  width and neither knew about the splitters. **The panels grow but do not
  shrink**: 280px is the width the inspector's transcript row was fitted to, so
  today's widths are the floor. The scoreboard re-raster this entry asked to
  coalesce needed nothing — see the spec's W9.
- **What was checked and needed no change:** the raster cost above. **What is
  still open:** 99 (no keyboard path to the splitters).

88. **The inset's size and corner, per clip — RESOLVED** (2026-09-28). Three
  sizes (0.16 / **0.22** / 0.30 of the output width, the middle exactly today's)
  and three corners (**bottom-right** / bottom-left / top-right), stored on the
  clip and **sticky**: the last one the coach set seeds the next recording
  (`Preferences::last_inset_size` / `last_inset_corner`), which is what makes
  "pick before" reachable without the app-settings panel #78 will bring. Spec
  `docs/superpowers/specs/2026-09-28-inset-size-and-corner-design.md`, plan
  beside it, shipped in three commits (core, media, app).
- **Two corrections this entry needed.** It said "a `formatVersion` bump to 12":
  **v12 was taken** by slates, so this is **v13** — an implementation trusting
  the entry would have written a version every 0.9.0 project already claims. And
  its list of what must follow was missing the item that would have shipped a
  bug: **the export built one avatar rect per run**, so a compilation mixing a
  Small bottom-left avatar clip with a Large bottom-right one drew both in the
  same place, with no test failing. Each entry's rect is now built in
  `Pip::open`, pinned by
  `media/tests/export.rs::a_run_of_mixed_placements_draws_each_avatar_in_its_own_box`.
- **Top-left is not offered**, which answers the rule this entry asked for: the
  scoreboard is locked there and is drawn *over* the inset, so it would not
  corrupt anything — it would just be a half-hidden face the coach has to work
  out. A control that appears only when no scoreboard is configured is worse than
  one that is simply absent.
- **Steps, not a slider**, and not for the reason a first draft gave (that a free
  ratio would make the caption bar jitter — steps jitter too, and a per-clip
  corner moves the bar's whole edge). The real reasons: a continuous ratio is
  **unguessable** with no live preview of the composite, and three steps can each
  be **measured** against the longest line the app produces (≈74 / 68 / 61
  characters at 1080p) where a slider cannot.
- **`avatar_box` stopped being corner arithmetic**, and that was a fix rather
  than a tidy-up: shrinking a rect about its bottom-right corner drifts a
  bottom-left avatar **105.6px** off the left edge and hangs a top-right one the
  same 105.6px below the top at Medium/1080p — the same number on both axes,
  because the box is square. It is now a ratio on a ratio, exact to the pixel
  where Medium/BottomRight is concerned.
- **What was checked and needed no change:** the GL 1×1 filler (it feeds a pad
  whose rect is set in the PTS-keyed probe), and `shows_camera_pip` /
  `shows_avatar` / `shows_inset` (they answer *whether*; this is *where*).
  **What is still open:** nothing of this entry — but the two new preferences
  have no control of their own, which is #78's job, where
  `pip_for_new_recordings` would finally get one too.

89. **The app's live self-view sat under the drawings; the export's inset sat
  over them — RESOLVED** (2026-09-25 review, the other way round). The inset had
  been raised to the mixer's top layer so the caption bar's 60% black could not
  wash over the coach's face once the inset moved onto the bar. That hid every
  stroke and highlight pill drawn into ~422×152 px of picture, against the app's
  own rule that the coach's pen is what must never be hidden. The fix stops the
  **bar** at the inset's left edge instead (`layout::bar_rect`, which the line
  already did), and puts the overlay back on top
  (`composite::install_overlay_pad`): nothing washes the inset, nothing hides
  the pen, and `app.slint`'s "the export's layer order" comment is true again
  with no change to it. Pinned by `media/tests/export.rs`'s stacking test (a
  stroke into that corner survives) and `overlay.rs`'s bar test (the corner is
  untinted).

90. **The repository URLs and the docs site's base path — RESOLVED** (2026-09-25).
  All of them point at `rykerwilliams/pundit` now, and `book.toml`'s `site-url`
  is `/pundit/`, so the published site's assets and 404 page resolve under the
  new name — including the two dated docs that named the old URL, which were
  rewritten with the rest. What the historical specs and plans under
  `docs/superpowers/` and `rust/docs/` do keep is the old **app name**, on
  purpose: they are records of what was true when they were written.

91. **Stop being a fork, and the name — RESOLVED** (2026-09-25). The app is
  **pundit** (*pundit Understands Nothing, Discusses It Thoroughly*), lower case
  everywhere, and it lives at `rykerwilliams/pundit`, a repository created fresh
  and therefore **not a fork** — which took no GitHub Support request and left
  `rykerwilliams/coach-cutups` untouched, deliberately, along with its v0.7.0
  release. `tayl0r/coach-cutups` is credited in prose in `README.md`, which is
  what the shared AGPL history asks for.
- **Checked before committing to the name:** free on crates.io (all five
  package names), no Debian or Ubuntu package and no `pundit` binary, nothing on
  Flathub, the GitHub name available, no sports-video product using it, and the
  only live US trademark is `SOFTWAREPUNDIT` (reg. 7547190) — a software review
  site, different goods. `varvet/pundit`, the Rails authorization gem, owns the
  name in code search; the coach accepted that. `pundit.app`, `pundit.dev` and
  `getpundit.com` are registered but parked.
- **What carries an existing installation over:** `state::adopt_old_name` and
  the `.deb`'s `conflicts = "coach-cuts"` — two shims, both dated by #93.
  `$PUNDIT_WHISPER_MODEL` is **not** one of them: nothing reads the old variable,
  which is a clean break and is what the CHANGELOG's breaking note says.

92. **Slates — RESOLVED, shipped (2026-09-25).** `i` and `o` mark a range, the
  sidebar's Slates section holds it, Record on a row shoots it and the clip
  inherits the slate's name and tags. Spec:
  `docs/superpowers/specs/2026-09-25-slates-design.md`; plan:
  `docs/superpowers/plans/2026-09-25-slates.md`. Format v12.
- **What the reviews changed, which the spec's §S11 records in full:** the
  record lost five of its eleven fields, the link flipped to `Clip.slate_id` so
  nothing can dangle, `i` stores the slate on the first press (so a half-marked
  range survives the app closing), and the shoot moved *inside*
  `start_recording` — because `NoCamera` is raised after `can_record`, so
  sequencing from outside would move the player and then refuse.
- **What did not ship, deliberately:** the typed-line editor (#97) and
  exporting the slates as a silent breakdown film (#98).

  The original entry follows, as the record of how the shape was chosen.

  THE NEXT
  FEATURE, at the coach's direction (2026-09-25). Watching a game through, the
  coach wants to mark "here to here, corner routine, #corners" and move on,
  then come back and record the commentary over those ranges in a later pass.
- **A slate is not a clip, by construction.** A clip *is* a recording — `Clip`
  carries `recording_filename` and `recording_duration` as required fields, and
  "no commentary-less clips" is the model's own rule. So this is a new record on
  the project, `Project.slates: Vec<Slate>` (v12, additive, floor stays 7), and
  the clip list's invariants are untouched.
- **The name.** A slate is what identifies a take before it is shot, and this
  codebase already calls a recording session a *take* ("a camera take", "an
  avatar take", "a paused take" — `bus/mod.rs`). It is film-native like *reel*,
  *basket* and *film*, and it reads as a verb on a button: *shoot this slate*.
  Considered and rejected: *span* (precise, says nothing about intent),
  *draft clip* (implies a commentary-less clip, blurring the one rule above),
  *mark* (an instant, and the marking keys are the interaction, not the record),
  *segment* (taken — `timeline`'s play segments), *cue* (taken — `core::cues`),
  *highlight* (taken — `player_highlights`), *bookmark* (an instant).
- **Shape, following the format rules:** `id`, `source_index`, `in_seconds`,
  `out_seconds`, `name`, `notes`, `tags`, `created_at`, `sort_index` — a new
  struct, so no field-level defaults, and one source per slate (a range cannot
  span two files any more than a clip can).
- **The interaction is `i` / `o` while scanning** (in and out, the editing
  convention; `z`/`x`/`v` are match events, `,`/`.` frame steps, `J`/`L` scan
  speed). Marking an out before an in, or crossing a source boundary, is
  refused at the key, not stored and fixed later.
- **Typing them should reuse `core::match_entry`'s grammar**, which already has
  the refusals that matter: a time has a colon, a bare leading integer is a
  video number. A slate line adds a range — `2 14:05-14:40 corner routine
  #corners` — and the Edit events… sheet's row-plus-paste-box pattern is the
  editor, not a new kind of sheet.
- **Shooting one:** the slate's row offers Record, which seeks to `in_seconds`,
  arms the recording, and the resulting clip inherits the slate's name, notes
  and tags. That is the whole point of the feature — the tagging work is done
  once, live.
- **Open questions, all of them real:**
  - Does the slate survive its clip? Keeping it (with the clip's id) allows a
    second take and lets the list show what is still unshot; consuming it keeps
    one row per thing. Lean: keep it, because a coach re-records.
  - Is `out_seconds` binding or advisory? A clip's extent comes from its
    recording's length, so a take that runs past the out point already works.
    Advisory is the smaller change; binding needs a rule for the overrun.
  - "Watching the game live" has two readings: a first pass through the footage
    in the app (which is what this entry assumes, since a range is a source
    time), or at the pitch with no video loaded, which would need wall-clock
    times mapped through the match clock. Ask before building the second.
  - Slates in the same list as clips, greyed with a Record button, or a list of
    their own? The tag filter should cover both either way.
- **A natural follow-on, deliberately out of scope:** slates are already an edit
  decision list, so "export the slates" would be a silent breakdown film with
  the scoreboard burned in and no commentary — close to the whole-match copy
  path. Worth doing, worth not doing first.
- **Why deferred:** only by order; nothing is built yet.
- **When to revisit:** next, ahead of #88 (the per-clip inset) and #77 (the
  export queue). Starts with a spec, per the workflow in `CLAUDE.md`.

93. **Delete the 0.8.0 rename shims.** Three things exist only to carry a
  `coach-cuts` installation onto `pundit`, and they do nothing on a machine that
  has ever run 0.8.0: `state::adopt_old_name` / `adopt_in` / `OLD_APP_DIR` and
  their three tests (`crates/pundit-app/src/bus/state.rs`), the call at
  `main.rs`'s startup, and `conflicts = "coach-cuts"` in
  `crates/pundit-app/Cargo.toml`'s `[package.metadata.deb]`. Also then:
  `docs/hands-on-checklist.md`'s install step, which currently names
  `pundit_0.8.0_amd64.deb` and checks that apt says it is *removing*
  `coach-cuts` — true exactly once, so it should go back to the generic "It
  replaces the copy you have".
- **Why deferred:** a 0.7.x install that has not upgraded yet still needs all of
  it, and the coach's own laptop is the population.
- **When to revisit:** once 0.9.0 has shipped and the coach confirms every
  machine they use has run 0.8.0. The whole removal is one commit and should
  leave no `coach-cuts` string in `crates/` at all.

94. **Zoom and pan are undiscoverable — PARTLY RESOLVED (2026-09-25).** The
  drag that does nothing is now the moment the app says how: `DRAWING_HINT`
  reads "Ctrl+scroll to zoom, then drag to pan — or press R to draw", because
  that gesture has two readings and the app cannot tell which was meant. What
  is still true: `2`/`3`/`0` are written down nowhere, and a coach who never
  drags never sees the hint. The rest of this entry stands.

  The coach, using 0.8.0 for the first
  time (2026-09-25): "the panning doesn't work or i don't know how to do it".
  Nothing is broken — a left-drag over the picture pans, but only once the
  picture is zoomed in, because `zoom_input::panned` returns the zoom unchanged
  at `scale <= 1.0` (there is nothing to pan when the picture fits). So a coach
  who has not zoomed yet drags and sees nothing happen, and the way *in* —
  `2`/`3` to step the zoom, `0` or `1` to reset it, Ctrl+wheel to zoom about the
  pointer — is written down nowhere in the app.
- **What would fix it, cheapest first:** the `ZoomIndicator` already appears
  over the picture, so it is the natural place to say "Ctrl+scroll or 3 to zoom"
  while the zoom is at identity; or a notice on a drag that would have panned
  had the picture been zoomed, beside the existing `DRAWING_HINT`, which is the
  same shape of problem already solved once ("Drawing works while recording —
  press R"). A keyboard-shortcut sheet would cover the whole app but is a bigger
  piece and hides the answer behind a menu.
- **Why deferred:** the coach found it within a minute of asking, so it is a
  first-run problem rather than a blocker, and the honest fix is part of a
  wider "what can I press here?" pass.
- **When to revisit:** with #85 (recents drawer) or any other first-run polish,
  or immediately if a second person is ever handed the app.

95. **The player area should take the footage's aspect, so there are no black
  bars — RESOLVED (2026-09-26), as `F` / the Fit button.** The coach, on 0.8.0 (2026-09-25): "i'd like the preview window to be
  the right aspect ratio? i don't like the black bars in the view". Today
  `player` (`app.slint`) is a `Rectangle` that fills whatever the layout gives
  it, and `place-picture` letterboxes the frame inside it, so any window whose
  player area is not the footage's shape shows bars — in the screenshot,
  above and below a 16:9 source.
- **The catch, which the fix has to answer:** the leftover space does not
  disappear, it moves. Sizing the player to the source aspect turns black bars
  into either app background (the same bars in a different colour) or space the
  neighbouring panels take. So this is only a real win **together with panels
  that can use the slack** — #87 (resizable panels) and the inspector — or with
  the window itself offering to match the footage's shape.
- **Worth knowing before designing it:** the bars are not decoration. The
  clipped content rect is what strokes and highlights are normalized to and
  what the export crops to (spec D9, one `Zoom::transform`), so the picture
  rect must stay exactly what export produces — the fix belongs in how much
  *area* the player is given, never in how the frame is placed inside it.
- **Checked in the layout (2026-09-25), and it changes the answer.** The
  window is `sidebar (240) | player | inspector (280)` in a `HorizontalLayout`,
  and the bars in the coach's screenshot are **above and below** the picture.
  Panels are horizontal; the slack is vertical. **Widening or narrowing a panel
  cannot absorb it** — a narrower panel makes the player *wider*, which reduces
  letterboxing up to the point where the player area is exactly 16:9, and
  produces pillarboxing past it. So #87 alone does not deliver this, and the
  entry's original premise ("panels that can use the slack") is wrong for the
  bars the coach actually has.
- **What would deliver it:** a **"Fit window to video"** action — resize the
  *window* so the player area comes out at the footage's aspect exactly. One
  action, no bars, and nothing moves unless asked, which answers the "a window
  that moves on its own is startling" worry by making it explicit. Optionally,
  a snap while dragging a splitter, so the aspect is reachable by hand too.
- **Decided by the coach (2026-09-25): "Fit window to video".** An action, not
  an automatic resize — the window moves when asked and never on its own. To
  build with #87, and the splitter snap comes with it so the shape is reachable
  by hand too.
- **Specced 2026-09-26** (`docs/superpowers/specs/2026-09-26-panels-and-fit-design.md`),
  and the spec **declines to build the snap** — see its W11. Because the panels
  can only grow (their present widths are what their rows were fitted to), a
  splitter drag can only make the player *narrower*, which makes a letterboxed
  player worse. The snap could fire only on a **pillarboxed** player: for 16:9
  footage in a 1600px window, that is window heights in `[700, 715.5)`. It
  cannot touch the bars the coach complained about — those would need a player
  1514px wide inside 1600px, leaving 86px for two columns.
- **Shipped:** `F` fits the window, shrinking the dimension with the slack, so
  the picture is never re-fitted — see CLAUDE.md and the spec's W1. Everything
  here about the snap stands; nothing else in this entry is open.
- **Why deferred (the snap only):** it would cost a tint, a test and the only
  place a panel drag reads the footage, and buy nothing in the case that
  motivated it. **Needs the coach's nod**, since they asked for it.
- **When to revisit:** if the panels ever gain a minimum below today's widths,
  which is what would make the shape reachable by hand.

96. **Every hot key should be reassignable.** The coach (2026-09-25): "we need
  to have all the hot keys reassignable". Today there are **33 `event.text ==`
  branches** in one `FocusScope` in `app.slint`, each naming its key inline:
  `R` records, `Z`/`X`/`V` tag match events, `,`/`.` step, `J`/`L` set the scan
  speed, `0`–`3` the zoom, `I`/`O` will mark slates (#92), and so on. Nothing
  is data, so nothing can be changed without a rebuild, and nothing can be
  *listed* either — which is half of why #94 exists.
- **The real work is not the settings UI, it is turning the branches into a
  table.** One action enum in the app crate (`Action::{Record, TagHomeGoal,
  StepBack, ScanFaster, ZoomIn, MarkIn, …}`), one default binding per action,
  and one lookup the `FocusScope` consults. That refactor is worth doing on its
  own merits even if nothing is ever rebound: it puts every key in one readable
  place, makes "what can I press here?" answerable (#94), and stops the next
  feature inventing a key that is already taken — which is exactly the check
  #92 had to do by grep.
- **Where a keymap lives:** `state.json`, with the pen, the speech model and
  the window size. A binding is a property of the coach's hands, not of a
  match, so it must never be a `project.json` field — and `state.json` costs no
  format bump (`AppFiles`, `bus/state.rs`). Store it as action → binding by
  **name**, so an unknown action in a file from a later build is ignored rather
  than throwing the document away, exactly as `whisper_model` and `pen` do.
- **What makes this harder than it looks, and must be decided in the spec:**
  - **Slint gives `event.text`, not a scancode** — the same limitation already
    recorded in #35. A rebind UI that captures "the key the coach pressed"
    captures a *character*, so on AZERTY the digits arrive shifted and a dead
    key arrives as nothing. Either the feature is honestly character-based
    (and says so), or it waits for a Slint that exposes physical keys.
  - **Text entry must keep winning.** The window's `text-editing` fold is what
    stops a key firing while a `LineEdit` has focus; every rebindable action
    has to stay behind it, and a coach must not be able to bind a bare letter
    in a way that breaks typing in a sheet.
  - **Conflicts and reset.** Two actions on one binding, and a way back to the
    defaults, are the two things every remapper needs and the reason the table
    has to be the source of truth rather than a list of overrides.
  - **Modifiers.** `Ctrl+O`, `Ctrl+Z`, `Ctrl+Shift+Z` and the plain letters
    share the same handler; a binding is a (modifiers, key) pair, not a letter.
- **It wants #78's settings screen to land on**, and it should ship with a
  read-only view of the bindings first — that alone closes #94 and is most of
  the value.
- **Why deferred:** only by order; it is the natural companion to #78, and the
  table refactor should not be rushed into the same file four queued features
  are already editing (#87, #88, #92).
- **When to revisit:** with #78, or immediately if the coach's hands disagree
  with a default badly enough to be worth the detour. Related: #35 (physical
  keys), #94 (nothing says what the keys are), #78 (where the screen goes).

97. **Typing slates in, as match events can be typed.** Dropped from the slates
  spec during review (§S6) rather than built: the grammar reuse it rested on
  does not exist. `match_entry`'s `#` is a **comment to end of line**, so the
  obvious `2 14:05-14:40 corner routine #corners` would have had its tag
  silently discarded — and this codebase has no `#` tag sigil at all, tags
  being comma-separated through `normalize_tags`. The words after the time are
  a closed vocabulary (`parse_kind` refuses anything that is not an event word
  or a team name), and a slate's tail is free text.
- **What is genuinely reusable** if it is ever built: `parse_time`,
  `format_time`, and the rule that a leading bare integer is a video number —
  in a new `core::slate_entry`, not a second mode bolted onto `match_entry`.
  The editor sheet itself is ~300 lines of Slint plus its `editing` fold, the
  rebuild discipline and the focus-drop-on-commit rule.
- **Why deferred:** the coach asked to mark ranges while watching, which is
  what shipped. Nobody has asked to type them.
- **When to revisit:** if a coach arrives with times already written down —
  which is exactly how `kickoffs.txt` came about, so it is not far-fetched.

98. **Export the slates as a silent breakdown film.** Named out of scope in the
  slates spec (§S9) and worth doing on its own: the slates are already an edit
  decision list, so "export the slates" is a cut of the marked ranges with the
  scoreboard burned in and no commentary. `reel_plan` is the shape to follow —
  a list of ranges on sources becomes `PlanEntry { clip_id: None, segments:
  vec![one Play segment] }` plus chapters — so it is roughly one `ExportTarget`
  variant and thirty lines against that.
- **It is also the only thing that would make `out_seconds` load-bearing.**
  Today the out point is shown in the row and nothing reads it: a take runs as
  long as the coach talks. This is what would give it a job.
- **Why deferred:** a second feature, and the first one had to prove itself.
- **When to revisit:** when a coach wants the ranges without the talking —
  a walkthrough to send a player, or a silent cut to watch back.

99. **No keyboard path to the panel splitters.** #87 ships a 6px draggable grip
  either side of the player and nothing else: the only way to resize a panel is
  to grab it with the pointer. GtkPaned makes its handle focusable and moves it
  with the arrow keys (F6/F8 to cycle); QSplitterHandle does not, so this matches
  Qt and not GTK. In an app where every other control has a key and the coach
  works by keyboard, a 6px pointer target is a real gap.
- **Why deferred:** the obvious fix collides with the window's keyboard design.
  `AppWindow` has `forward-focus: keys` and every letter is a global binding, so
  a focusable grip would swallow `z` / `x` / `v` while it held focus. It needs a
  decision about how a focused control coexists with the global letters — which
  is #96's question (rebindable hot keys), not a splitter's.
- **When to revisit:** with #96, or if the coach asks to resize without the mouse.

100. **`state.json` is read all-or-nothing, so one bad value still costs the
  whole file.** #87 added container-level `#[serde(default)]` to `PanelWidths`
  and `WindowSize`, which rescues a **partial** object — but measured, every one
  of these still returns `State::default()` and so loses the last project, the
  pen and the speech model: `"panels":"wide"`, `"panels":{"sidebar":-5}`,
  `"panels":{"sidebar":1.5}`, `"panels":null`, `"window":{"height":-1}`.
  `AppFiles::read` gives up on any `serde_json` error for the document, and every
  setter rewrites it.
- **Why deferred:** the real fix is a per-field read (parse to a
  `serde_json::Map`, take each key independently, keep the defaults for whatever
  fails), about 15 lines in one place, which would retire the whole class rather
  than one shape of it. That is worth doing on its own merits and was not #87's
  to do. It is also why `CLAUDE.md` gives this exact hazard as the reason the
  basket lives in its own file.
- **When to revisit:** next time anything is added to `state.json`, or the first
  time a coach loses their last-project pointer for no visible reason.

101. **The export test binaries abort with `corrupted size vs. prev_size` under
  heavy concurrent load.** glibc heap corruption, `SIGABRT`, always mid-suite and
  always at a different test, seen three times in `pundit-media --test export`
  and once in `pundit-harness --test export` — never in a run of that suite
  alone. Observed on 2026-09-28 during #88's gates with three other sessions
  building on the machine (loadavg ~21); `cargo test --workspace` on a quiet
  machine that same day was clean, 971 passed. It aborts the test *process*, so
  `cargo test` exits 101 with **no failing test named**, which is the confusing
  part: the run looks broken rather than flaky.
- **Not #88's.** The aborting binary was `pundit-media`'s, built from sources
  byte-identical to the commit before it; that suite then ran 30/30 twice,
  cleanly, alone.
- **Where to look:** the export path's GStreamer graph under CPU starvation —
  llvmpipe (CI has no GPU either), the surfaceless GL display, or a buffer pool
  freed while a probe still holds it. `GST_DEBUG` plus a `valgrind` or ASan run
  of `--test export` under an artificial load is the experiment; a heap-corruption
  bug is not going to be read out of the source.
- **Why deferred:** it has never been seen on a quiet machine or in CI, and the
  rule here is not to chase rare flakes unless the fix is free. This one is a
  memory-safety bug somewhere in a C library boundary, which is the opposite of
  free.
- **When to revisit:** if it happens on a quiet machine or in CI even once — that
  would make it a real bug rather than a load artefact — or if anything else in
  the export path starts corrupting memory.

102. **Snap the scrubber to events, as an option.** The coach (2026-09-28):
  "snap to events in the scrubber as an option." Dragging the scrubber lands on
  whatever frame the pixel under the cursor works out to, so getting the playhead
  onto a goal you tagged means dragging and then nudging with the arrows.
- **What "events" covers is the first thing to settle**, and it should be settled
  with the coach rather than guessed: match events (`Project.match_events`) are
  the obvious ones, but slate in/out marks, player highlights and clip starts are
  all marks on the same footage, and snapping to all of them at once could make a
  busy match feel like the scrubber is fighting back. A defensible first cut is
  match events only, with the rest behind the same option if asked for.
- **The snap radius is in pixels, not seconds.** The scrubber's seconds-per-pixel
  changes with the window's width and the source's length, so a radius in seconds
  would snap from half a screen away on a short clip and never on a long one. The
  highlight-ring and slate work both key marks by the **displayed frame's** time,
  which is what a snap should land on too.
- **"As an option" means it has to be reachable and remembered**, which is
  `state.json` (a property of the machine, not the project) — and see #100: that
  file is read all-or-nothing, so a new key wants the per-field read that entry
  describes, or it is one more value that can cost the coach their last project.
  #78's settings panel is where the control would live.
- **It must not fight the frame-accurate paths.** `,` / `.` step exactly one
  frame and the arrows skip fixed amounts; both are promises about exact
  distances, so snapping belongs to the **drag** alone, not to any keyed
  movement.
- **Why deferred:** filed on the day it was asked for, with #88 closing out; it
  is a UX affordance with a real design question in it (which marks), not a bug.
- **When to revisit:** with #78, which brings both the settings panel it needs a
  control in and the per-field `state.json` read it should not go in without.

103. **A possession tracker, as an analysis pass.** The coach (2026-09-28): "a
  possession tracker analysis pass." Which team has the ball, over the match — so
  a coach can see how much of the game their side had, and where it changed.
- **What it produces is the first thing to settle with the coach**, because the
  shapes cost very different amounts: (a) a whole-match or per-period
  **percentage** for each side; (b) a **timeline** of spells, drawn on the
  scrubber beside the match-event marks; (c) **change-of-possession moments**,
  which is what the match-vision spec's Deferred list wanted for guessing where a
  goal's build-up starts (`2026-09-22-match-vision-design.md`, "An automatic
  guess at the move's start"). (a) tolerates noisy per-frame guesses because it
  averages them; (b) and (c) do not.
- **It needs the ball, and the spec already doubts the ball.** The same Deferred
  entry says change of possession "needs ball tracking, which the ball's few
  pixels in the wide framing make doubtful". On Trace's follow-the-play cut at
  1080p the ball is a handful of pixels on the far side, and the detectors the
  spec cleared for licence (D-FINE-N / DEIM-N, COCO-descended) are person-first.
  A proxy that skips the ball — the team with more players in the half the
  camera is framing, or the kit nearest the pan's centre — is cheaper and much
  weaker, and would have to be measured, not assumed.
- **It sits on the vision layer that is shelved.** Telling the teams apart is
  P5's kit clustering (torso colour, k = 2), finding the players is its detector,
  and following them is P6's tracker; #81 is the runtime spike none of that has
  had. So this is not a pass that can be built next to P3's sound-and-motion
  `Analyzer` — it is a consumer of P5/P6, and it inherits their download prompt,
  their queue rules (a job, preempted by recording) and their stay-on-the-machine
  promise.
- **The bar before anything is shown** follows P3's rule: measure on the coach's
  tagged matches first, and show nothing that fails. Truth would have to be
  hand-marked possession on a few stretches of matches A–C, which the coach does
  not have yet — itself a question for them. Nothing identifying goes in the repo
  (CLAUDE.md, match analysis).
- **Why deferred:** filed on the day it was asked for; it depends on a detector
  and kit clustering that have not been built, and on a product call about which
  of the three outputs is wanted.
- **When to revisit:** when #81's spike runs and P5 or P6 is built — the kit
  clustering and the detector are most of the work — or sooner if the coach
  settles on (a) and a ball-free proxy is worth measuring on its own.

104. **Double-clicking a slate should take the player to its in point.** The
  coach (2026-09-28): "when i select a slate, i expect to be taken to the
  beginning of it in the timeline? or like, if i double click it i guess?"
  Today a slate row's click only toggles the selection (`app.slint`, the slates
  `ListView`'s `TouchArea`) — nothing on the row moves the player, so going back
  to watch a marked range means finding it on the scrubber by eye.
- **Double-click, not click — the clip row's rule.** A clip row selects on click
  and jumps on double-click (`jump-to-clip` → `Command::JumpToClip` →
  `bus/clips.rs::jump_to_clip`); a slate row is the same kind of row in the same
  panel and should behave the same. A single-click seek would also move the
  picture every time the coach selects a slate only to rename or tag it. (Match
  rows seek on a single click, but they have no selection to fight with.)
- **The toggle has to change with it.** A slate row's click *toggles* —
  clicking the selected row deselects it — and a double-click delivers two
  clicks first, so as written a double-click would jump and leave the slate
  **deselected**. Selecting on click (as the clip row does), with deselection
  by Esc or by clicking empty space, removes that; toggling only when the
  second click is not a double is the fiddly alternative.
- **The bus side is `jump_to_clip`'s body with a slate's fields:**
  `reset_skip`, pause, then `load(slate.source_index, slate.in_seconds, …,
  Origin::Scrub)` if `seekable()` — which also switches video when the slate is
  on another source. A `Command::JumpToSlate(Uuid)` beside `JumpToClip`, or one
  command taking either kind of id; the first is the smaller change.
- **Refused where the row already is:** the row's `TouchArea` is disabled while
  recording, and `seekable()` covers a preview and a missing source. A
  half-marked slate (`out_seconds: None`) still has an in point, so it jumps
  like any other.
- **Why deferred:** filed while the coach was using the app; a small UX gap, not
  a bug, and the other session is mid-#88.
- **When to revisit:** any time — it is a row handler and one bus command, and
  it pairs naturally with the next slate work (#97 or #98).

105. **A slate's tag field should offer the tags already in use.** The coach
  (2026-09-28): "tags in slates should be saveable so i can select them or
  similar." The tags *are* saved — `Slate.tags`, v12, and `tag_vocabulary` is
  already clips ∪ slates — but nothing lets the coach **pick** one: the slate's
  field (`slate-tags-edit` in `app.slint`, under the slates list) is a bare
  `LineEdit`, so every tag is typed out in full, and a typo makes a second tag.
- **The clip inspector already has the picker** (C8): its tags field calls
  `suggest-tags` on every edit (`main.rs`, `tag_suggestions` over
  `tag_vocabulary`), shows the list as a `Rectangle` over the fields below —
  deliberately not a `PopupWindow`, whose `show()` would steal focus and commit
  the field — and takes the top one on Tab, `take-suggestion` splicing it into
  the comma-separated text. The slate field needs exactly that, not a second
  version of it.
- **So the fix is a component, not a copy.** The field, its suggestion state
  (`suggestions`, `suggestions-dismissed`, `suggesting`) and the overlay live
  inside `Inspector` today; lifting them into a `TagField` that both the
  inspector and the slate editor use leaves one implementation of the Tab / Esc
  rules and the focus trap. The overlay is positioned from the field's own `x`
  and `y` as a sibling in its layout, so the component has to carry the overlay
  with it — check the slate editor's `VerticalLayout` gives it room to draw over
  whatever sits below (the Shoot row).
- **Esc must still reach the field first.** The slate fields already fold into
  the window's `text-editing` (spec S6); the suggestion list's Esc-to-dismiss
  has to keep returning `accept` before the sheet or window sees it, as the
  inspector's does.
- **"Or similar" is worth asking about:** the coach may also mean clicking a
  tag to **filter** the slates list, as the clip list's `tag-filter` does (the
  tag overview is clips-only, by the slates spec's choice). That is a separate,
  larger change; this entry is the picker.
- **Why deferred:** filed while the coach was using the app; a UX gap, and the
  other session is mid-#88.
- **When to revisit:** with the next slate work (#104, #97, #98) — #104 touches
  the same rows.

106. **Every format field costs a hand edit in eighteen test files.** A full `Clip`
  struct literal is written out by hand in each of `pundit-core/tests/*.rs`,
  `pundit-harness/src/lib.rs`, and four places in `pundit-media`; `Clip` has no
  `Default`, so adding a field breaks all of them. #88 (v13) spent 14 of its 35
  changed files on that edit alone, setting both new fields to the values
  `Default` already gives; v12 (slates) paid the same toll, and v14 will.
- **The shape:** a `tests/common/mod.rs` in `pundit-core` with one
  `fn clip_stub() -> Clip`, and one `mod common;` per file — after which a new
  format field is a zero-test-file change.
- **Not a `Default for Clip` in production code**, which is the tempting
  shortcut: a document struct that can be built with a nil `Uuid` and an empty
  recording filename is a worse hazard than the toll, and `add_recorded_clip` is
  deliberately the one constructor.
- **Why deferred:** it is a test-infrastructure change with no user-visible
  effect, and doing it inside a format bump would have mixed ~20 mechanical
  edits into a change whose diff needed to be readable. Found by #88's
  adversarial simplification review.
- **When to revisit:** at the **start** of the next format bump, before its
  fields are added — that is the one moment the work pays for itself
  immediately.

107. **Nothing built the book — RESOLVED** (2026-09-28). **Numbered 104 when it
  was filed**, which collided with the slate double-click entry another session
  added the same day; commits from 2026-09-28 cite it as #104. `docs/book/` had a
  `book.toml`, a `SUMMARY.md` and four pages, and **nothing touched any of it** —
  no workflow built it, there was no `gh-pages` branch and no deploy action, so
  #90 had resolved `site-url = "/pundit/"` for a site that did not exist.
- **What shipped:** `.github/workflows/docs.yml`, in two jobs on purpose.
  **`build` runs on every pull request** and is the check that was missing:
  `create-missing = false` makes a `SUMMARY.md` entry with no file a hard error,
  and a broken relative link is a dead link on a page a coach is reading, neither
  visible from the markdown. **`deploy` runs only from `main`**, so a pull request
  verifies the book without publishing it — observed doing exactly that on PR #1
  (`build` pass in 9s, `publish` skipped), then deploying from `main` in 24s.
  The site is live at <https://rykerwilliams.github.io/pundit/>; Pages is set to
  `build_type: workflow`, and the coach enabled it.
- **mdbook is pinned to 0.4.40** and installed from its release tarball rather
  than `cargo install` — seconds against minutes, and no Rust toolchain on a job
  that needs none. The pin is deliberate: mdbook's HTML output and its
  `SUMMARY.md` strictness both move between releases. Bump it with a local
  `mdbook build` first.
- **What it caught immediately:** the keyboard-shortcuts page had been committed
  unbuilt, because nothing could build it. It is now verified on every change.
- **What is still open:** `lychee` for external links, the third tool the
  2026-09-22 docs plan names — worth it once the guide has outbound links worth
  checking, and not before.
