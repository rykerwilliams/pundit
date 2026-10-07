# Exporting

**Export…** opens the export sheet. Each thing you pick becomes one H.264 `.mp4`
in the project's `exports/` folder. A file of the same name already there is
never overwritten: the new one gets ` (2)` on the end.

## What to export

| Target | What you get |
|---|---|
| **All clips** | Every clip, in the clip list's order, as one video. |
| A **tag** | The clips carrying that tag. |
| A **clip** | That clip alone. |
| A **goals reel** | Each team that scored has one, and **All goals** appears when both did. |
| **The whole match** | The game with the clock and score, and no commentary. |

Every clip carries a caption bar along the bottom with its number, name and
tags, your inset in its corner, your drawings, any player highlights it crosses,
and the scoreboard in the top left.

## Resolution and quality

**Resolution** is 720p or 1080p. **Quality** is Low, Medium or High: higher
looks better and costs more space. There is no size target, so a busy passage
costs what it costs. As a guide, a 56-minute match at 1080p comes out at about
2.7 GB at Medium and 4.5 GB at High.

## Goals reels

A reel is every goal, in match order, each cut with its build-up: by default
from 20 seconds before the goal to 6 seconds after. Goals close together become
one piece. Trim any goal's piece from its row in the Match panel. A reel has the
game's sound and no commentary.

## The whole match

The whole match can be a **straight copy** of your footage instead of an
encode: it finishes in about a minute rather than an hour, at about a quarter of
the size, with not a pixel lost. The scoreboard then travels beside the picture
rather than on it — as a subtitle track inside the file, and as an `.srt` beside
it that VLC loads on its own and YouTube accepts as a caption file. Highlights
cannot ride a copy, and the sheet says so.

A copy needs footage it can carry: H.264 in an MP4 file, with AAC sound (or
none), and every video recorded the same way as the first. A phone or camera
usually records exactly that; Matroska or HEVC footage is encoded instead.

## The scoreboard picker

| Choice | Means |
|---|---|
| **Default** | The best available: the whole match is copied when its footage allows it, and the board is burned in everywhere else. |
| **Burned into the picture** | The board is drawn on the picture, always. |
| **Separate track** | Copy the whole match and carry the board as subtitles — and refuse, naming the file, if the footage cannot be copied. |

Only the whole match can carry the board beside the picture. A clip or a reel
asked for on a separate track has it burned in, so the board is never lost. The
line under the picker says what will actually happen.

## Mute source audio

Tick **Mute source audio** to leave the game's sound out and keep your
commentary only. A whole match or a goals reel has no commentary, so muted it
comes out silent. Muting can make a whole match copyable that was not — sound
the file will not carry cannot stand in the way of the copy.

The sheet remembers your choices for the project.

## The files beside the video

- **Chapters.** Every export with more than one piece has chapters, so a video
  player can jump straight to a clip or a goal.
- **`<name>.chapters.txt`**, the chapter list ready to paste into a YouTube
  description, which is the only way chapters reach a video on YouTube. It
  starts at `0:00`, leaves out any chapter within ten seconds of the one before
  (YouTube ignores the whole list otherwise), and is not written at all when
  fewer than three chapters are left.
- **`<name>.srt`**, the scoreboard as subtitles, beside a copied whole match.

Every file also says what it is in its own details: a title, the day the footage
was recorded, and — once the teams are set up and kick-off is tagged — the final
score and both team names.

## While it runs

The sheet shows each target's progress and time left. **Cancel export** stops
the run and keeps the files that already finished.
