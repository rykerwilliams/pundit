# Installing

pundit ships as a `.deb` for **Ubuntu 24.04 and Linux Mint 22** on x86-64.
Download it from the
[releases page](https://github.com/rykerwilliams/pundit/releases), then, in
the folder you downloaded it to:

```bash
sudo apt install ./pundit_*_amd64.deb
```

It appears in the applications menu as **pundit**. From a terminal it is
`pundit`, or `pundit <folder>` to open (or create) a project in that folder;
with no folder it reopens the last project you had open.

If you have **Coach Cuts** installed — the name this app went by until 0.8.0 —
the same command replaces it. Your settings, your basket and the speech model
you already downloaded are carried over the first time pundit runs. If you had
pinned Coach Cuts to your panel, pin **pundit** instead: the old launcher goes
with the old package.

## What your machine needs

- **GStreamer 1.24 or newer**, as Ubuntu 24.04 ships it. The package pulls in
  the plugins it needs.
- **An x86-64-v3 CPU** — Haswell (2013) or newer. The speech recognizer is built
  for that instruction set.
- **A VA-API driver** for hardware video decode and encode. The package
  recommends `intel-media-va-driver` (or `va-driver-all`) and apt installs it by
  default. Without one, everything still works, but playback, recording and
  export fall back to software and are much slower.
- **An OpenGL (EGL) capable display**, on X11 or Wayland.
- **A microphone**, and a webcam if you want to be on camera. You can record
  with [a photo instead](recording.md#a-photo-instead-of-the-webcam).

## Where the app keeps things

Nothing about a match is kept outside its project folder. What the app keeps for
itself is about **this machine**, not about any match, so it stays behind when
you carry a project to another computer:

| Where | What |
|---|---|
| `~/.config/pundit/state.json` | The projects you have had open, the pen you draw with, the speech model you chose, the window's size and the side panels' widths. |
| `~/.config/pundit/basket.json` | [The basket](basket.md) and its settings. |
| `~/.cache/pundit/models/` | The speech model, downloaded the first time you [transcribe](transcripts.md). |
| `~/Videos/pundit/` | Films made from the basket (your videos folder, in your language). |

If `state.json` is damaged, the app falls back to the default for whatever it
cannot read and keeps the rest — though one bad entry in the recent projects
costs the whole list. A damaged `basket.json` empties the basket.

## Building it yourself

To run a build newer than the last release, or to build the package yourself,
follow
[Build from source](https://github.com/rykerwilliams/pundit/blob/main/README.md#build-from-source)
in the README.
