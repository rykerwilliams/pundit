<!-- Source of truth: `handle-key` in crates/pundit-app/ui/app.slint. Every key
     below is one branch of that function, and the conditions here are its gates
     (`can-play`, `can-tag`, `can-draw`, `can-fit`, `can-scan-fast`) in plain
     words. If you add or change a key there, change it here in the same commit. -->

# Keyboard shortcuts

The keys work whenever you are not typing in a box. pundit catches them before
anything on screen can, so a slider you have just dragged will not quietly steal
your arrow keys — but a text field takes everything, including <kbd>Esc</kbd>,
until you leave it.

Almost nothing here needs a modifier: the four <kbd>Ctrl</kbd> combinations are
<kbd>Ctrl</kbd>+<kbd>0</kbd> under *Zooming in*, and the three under *The
project* at the bottom.

## Watching the game

| Key | What it does |
|---|---|
| <kbd>Space</kbd> | Play or pause |
| <kbd>←</kbd> <kbd>→</kbd> or <kbd>A</kbd> <kbd>D</kbd> | Skip back or forward 3 seconds |
| <kbd>Shift</kbd> + those | Skip 10 seconds instead |
| <kbd>,</kbd> <kbd>.</kbd> | One frame back or forward, while paused |
| <kbd>[</kbd> <kbd>]</kbd> | Jump to the previous or next match event |
| <kbd>J</kbd> <kbd>L</kbd> | While playing: slower or faster, 1× up to 32× |
| <kbd>F</kbd> | Shrink the window to fit the footage |

<kbd>J</kbd> and <kbd>L</kbd> only do something while the video is actually
playing, and any pause puts you back to 1×. Every frame is still decoded at 32×,
so what you see is honest at speed rather than key frames flashing past.

<kbd>F</kbd> only ever makes the window **smaller** — it closes the window up
around the picture so the black bars go, rather than redrawing the picture to
fill the window, so what you are looking at never changes size or shape. Leave
full screen first; there is no shape to change in full screen.

## Zooming in

| Key | What it does |
|---|---|
| <kbd>3</kbd> | Zoom in |
| <kbd>2</kbd> | Zoom out |
| <kbd>1</kbd> or <kbd>Ctrl</kbd>+<kbd>0</kbd> | Back to the whole picture |

If the pointer is over the picture, zooming moves towards where it is pointing.
Zoom is off while a preview is open, because a preview always plays at the whole
picture — a change would only show once you closed it.

## Recording

| Key | What it does |
|---|---|
| <kbd>R</kbd> | Start or stop recording |
| <kbd>Esc</kbd> | Stop recording |
| <kbd>C</kbd> | Clear what you have drawn |

<kbd>C</kbd> works while a take is actually running. Drawing itself is the mouse,
with the pens beside the video.

## Marking the match

| Key | What it does |
|---|---|
| <kbd>Z</kbd> | Home goal |
| <kbd>X</kbd> | Away goal |
| <kbd>V</kbd> | Start or stop a period |

These mark the game video at the playhead, so you can tag while you watch. They
work while you are recording too — a goal belongs to the footage, not to the take
you happen to be in the middle of.

You can also type events in as lines (**Edit events…** in the Match panel), which
is the way to fix a time you got wrong.

## Marking a slate

A slate is a range you mark now to record commentary over later.

| Key | What it does |
|---|---|
| <kbd>I</kbd> | Start of the range |
| <kbd>O</kbd> | End of it |

The slate is saved the moment you press <kbd>I</kbd>, so nothing is lost if you
stop there — you will see a row with an open end, which you can finish or delete.
<kbd>O</kbd> closes the most recent open range **on the video you are watching**.
Press <kbd>I</kbd> twice and you get two slates; delete the one you did not mean.

To shoot one, select it and press <kbd>R</kbd> — the transport button reads
**Record range** while a slate is selected — or choose **Record** from the row's
menu. At the end of the range the game **pauses on its last frame** while the
recording keeps going, so you can finish your sentence. See
[slates](slates.md).

## Ringing a player

| Key | What it does |
|---|---|
| <kbd>H</kbd> | Turn the highlight tool on or off |
| <kbd>Esc</kbd> | Drop the selected ring, then leave the tool |

A ring belongs to the footage, so it shows up wherever that footage does —
scanning, recording, a preview, and every export that crosses it.

## Clips

| Key | What it does |
|---|---|
| <kbd>Delete</kbd> | Delete the selected clip |
| <kbd>Esc</kbd> | Clear the selection |

A deleted clip goes to a trash folder inside the project and <kbd>Ctrl</kbd>+<kbd>Z</kbd>
puts it back where it was.

## The project

| Key | What it does |
|---|---|
| <kbd>Ctrl</kbd>+<kbd>O</kbd> | Open a project |
| <kbd>Ctrl</kbd>+<kbd>Z</kbd> | Undo |
| <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>Z</kbd> or <kbd>Ctrl</kbd>+<kbd>Y</kbd> | Redo |

Undo and redo are off while you are recording.

## What <kbd>Esc</kbd> does

<kbd>Esc</kbd> is the one key whose job depends on what is in front of you. It
takes the innermost thing first, so you never lose more than you meant to:

1. A message on screen — dismisses it. <kbd>Return</kbd> does this too, and
   nothing else.
2. A sheet that is open — closes it. If you are in one of its text boxes, the
   first <kbd>Esc</kbd> leaves the box and the next closes the sheet, so a
   half-typed paste does not vanish on one keystroke.
3. The highlight tool — drops the selected ring, then leaves the tool. Both
   before it will stop a recording, so ringing a player during a paused take
   cannot end the take by accident.
4. A recording — stops it.
5. A preview — closes it.
6. A selected clip — clears the selection.

<kbd>Home</kbd> and <kbd>End</kbd> do nothing on purpose: they are swallowed so a
slider you touched cannot jump the video to one end.

## Changing them

You cannot yet. Reassignable keys are on the list.
