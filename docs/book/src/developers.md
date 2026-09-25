# Developers

pundit is a Rust workspace under `crates/`, with a Slint user interface and
GStreamer doing the video work. The design of the Linux app is written up in the
[Linux port design spec](https://github.com/rykerwilliams/pundit/blob/main/docs/superpowers/specs/2026-09-19-linux-port-design.md),
and the conventions every change is held to — the zero-copy decode path, the
capture clock, the export graph, the crate dependency rules — are in
[`CLAUDE.md`](https://github.com/rykerwilliams/pundit/blob/main/CLAUDE.md).

## The crates

- **`crates/pundit-core`** — pure logic: the project format, the playback
  timeline, zoom, and stroke replay. It declares no media dependency at all, and
  CI runs its tests on a machine with no GStreamer installed.
- **`crates/pundit-media`** — everything GStreamer: the source player,
  capture, the export frame driver and the overlay rasterizer.
- **`crates/pundit-app`** — the Slint user interface, the command bus and
  the event layer.
- **`crates/pundit-harness`** — headless integration tests driven over the
  command bus.

## API documentation

The rustdoc for each crate, built with private items included:

- [`pundit_core`](api/pundit_core/index.html)
- [`pundit_media`](api/pundit_media/index.html)
- [`pundit_app`](api/pundit_app/index.html)

## Building and design history

- [Build from source](https://github.com/rykerwilliams/pundit/blob/main/README.md#build-from-source)
  — the packages to install, and how to run the tests and build the `.deb`.
- [`docs/superpowers/`](https://github.com/rykerwilliams/pundit/tree/main/docs/superpowers)
  — the specs, plans and measurement spikes behind every phase of the port, in
  date order.
