# When something goes wrong

## Where the log is

pundit writes what it is doing to standard error. Started from the applications
menu on an X11 session (Linux Mint's default), that lands in
`~/.xsession-errors`; started from a terminal as `pundit`, you see it directly.
When you report a problem, the last few dozen lines are the most useful thing
you can send.

## Playback is slow or jerky

Every time a video loads, pundit logs one line saying how it is being decoded:

```bash
grep "bus: loaded" ~/.xsession-errors | tail -1
```

On a machine decoding in hardware it names a `vah264dec` or `vah265dec`
decoder, `memory:DMABuf` and `egl`. Anything else — `avdec_…`, `SystemMemory` or
`glx` — means the slow path: check a VA-API driver is installed
(`intel-media-va-driver` or `va-driver-all`).

If playback slows only while the monitor is off or asleep, start the app as
`vblank_mode=0 pundit`.

## Exports are slow

Without a VA-API driver, export falls back to the `x264enc` software encoder,
which is much slower. The whole match exports in about a minute when it can be
[copied rather than encoded](exporting.md#the-whole-match).

## Recording will not start

The message says what was missing. If it is the camera, check it is chosen in
**Devices…**, or record with [a photo instead](recording.md#a-photo-instead-of-the-webcam).

## A project will not open

- **"This project was made by a newer version of pundit"** — install that
  version again.
- **A missing video** shows as **Missing** under Sources; **Relink…** points the
  project at where it is now.
- **Going back to an older version** — the first save after an upgrade keeps the
  old file beside the new one, as `project.json.v<old version>`. Rename it back
  to `project.json` to use it with the older build, losing what changed since.

## Settings gone strange

The app's own files are listed under
[Installing](install.md#where-the-app-keeps-things). Moving one aside resets
just that: your projects are never stored there.
