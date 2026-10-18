# Slates

A slate is a range of the game you mark now and record commentary over later.
It is the way to watch a match through once without stopping to talk.

## Marking a range

Press <kbd>I</kbd> where the moment starts and <kbd>O</kbd> where it ends. The
range goes into the **Slates** list in the sidebar, where you can name it and
tag it. Both keys work while you are recording, because you spot the next moment
while talking over this one.

The slate is saved the moment you press <kbd>I</kbd>, so a range you never
finished is a row with an open end, there for you to finish or delete.
<kbd>O</kbd> closes the most recent open range on the video you are watching.

To move a mark, put the playhead where it belongs and press **Set in** or
**Set out** on the selected slate. The selected slate's range is drawn on the
scrubber, in the slate colour its row is selected in. To change that colour,
for the band and the row alike, open **Settings…** (or right-click the
scrubber and choose **Band colour…**). To watch it, double-click the row or choose **Preview slate** from its
menu: the game plays from the start of the range and stops at its end.

## Typing ranges in

If you already have the moments written down, press **Type…** in the Slates
header (it is there as soon as the project has a video) and paste them into the
box, one per line:

```text
2 14:05-14:40 corner routine [corners, set piece]
```

That is video 2, from 14:05 to 14:40, named *corner routine*, tagged `corners`
and `set piece`. The video number is optional — the box's picker says which
video a line without one is on — and so are the name and the tags. End a range
with a dash and nothing after it, `14:05-`, to leave it open. A time on its own
is refused, so a list of match events pasted into the wrong box adds nothing.
Each line is echoed back before you press **Add**, and the lines that can't be
added stay in the box for you to fix.

Click a row to retype it as a line. A slate you retype keeps its title, and the
clip you recorded from it is left as it was. The row's **→** selects the slate
and shows its first frame, ready for <kbd>R</kbd>.

## Recording over one

Select a slate and press <kbd>R</kbd> — the transport button reads **Record
range** while a slate is selected — or choose **Record** from the row's menu.
The game jumps to the range's start and the take begins. At the end of the range
the game pauses on its last frame while you finish your sentence; the recording
keeps going until you stop it.

The clip that comes out already carries the slate's name and tags. The slate
stays in the list after its take, because coaches re-record.

## Words over the range

Type a title in the field under the slate's name. Any export or preview that
shows that stretch of the game shows the words along the bottom, in place of the
clip's caption, which comes back when the range ends. If the clip has no caption
showing by then, the title fades out instead. The name is your filing; the title
is what a viewer reads. A slate with a title shows a **T** in the list — hover it
to read the words.

To move the words, re-mark the slate with **Set in** and **Set out**. To remove
them, clear the field. A range with no end yet shows nothing until you press
<kbd>O</kbd>. The live picture while you scan never shows a title; a straight copy
of the whole match cannot carry one either.

## Working through a theme

Filter the list by a tag — "corners", say — and the Slates list becomes a queue:
after each take it moves on to the next range with that tag that has not been
recorded yet, and parks the game there. Press <kbd>R</kbd>, talk, stop, and the
next one is waiting. Cancel a take and it stays on the same range.

Press <kbd>G</kbd> to start: it selects the first range in the filtered list
still waiting to be recorded and parks the game there, so the whole pass can run
from the keyboard. It is also the way back to a range you skipped — the pass only
moves forwards, but <kbd>G</kbd> always starts from the top.

When the last one is done the status bar says so — "All 8 'corners' ranges
shot", or "All 8 ranges shot" with no filter. It only says it when every range
in the list really is recorded; if you skipped one earlier, the pass just stops.
