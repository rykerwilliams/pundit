# The scoreboard and match clock

Tell pundit who is playing and when the goals and periods happen, and it keeps
the clock and the score for you. Both are drawn on the picture while you scan,
and burned into every preview and export.

## Setting up the teams

**Set up teams…** in the Match panel opens the match setup:

- **Home team** and **Away team**: each one's name and kit colours. The swatches
  open a colour picker with a row of common kit colours, or take a hex code.
- **Format**: the number of **Periods**, the **Minutes each**, and any
  **Overtime periods**.
- **My video starts after kick-off**: tick it if your footage begins part-way
  into the game, and the clock still reads the right time.

A [new match](projects.md#starting-a-match) starts with your last match's colours
and format already filled in.

## Tagging as you watch

| Key | Marks |
|---|---|
| <kbd>Z</kbd> | A home goal |
| <kbd>X</kbd> | An away goal |
| <kbd>V</kbd> | The start or end of a period |

Each one marks the game video at the playhead, and shows on the scrubber — goals
in the scoring team's colour. <kbd>[</kbd> and <kbd>]</kbd> jump between them.
They work while you are recording too.

The clock follows the frame on screen, so it reads the same match time either
side of a pause in your commentary.

## Typing events in

**Edit events…** opens the events as lines, one per event:

```text
2 14:05 home goal
```

meaning the second video, 14 minutes 5 seconds in, a home goal. **Paste events**
takes a whole list at once; every line is echoed back as understood, already
there, or refused with the reason, and adding them all is one undo. A line with
no video number goes on the video you choose under **Lines with no number
are:**.

Two rules keep a typo from moving the clock:

- **A time always has a colon.** A number on its own at the start of a line is a
  video number, so `900 home goal` is refused ("there is no video 900") rather
  than read as 15 minutes.
- **Kick-off and restart words are refused.** Periods are counted in order, so
  one stray start or stop would shift every period after it. A list of restarts
  is not a list of periods.

To fix one event, click it in the list and it opens as a single line. Enter
applies the change and Esc leaves it alone.

## Goals and the reel

Each goal's row in the Match panel can trim where its piece of the
[goals reel](exporting.md#goals-reels) starts and ends. By default a piece
starts 20 seconds before the goal and runs 6 seconds after it.
