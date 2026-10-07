# Projects and matches

A project is one match, and it is a folder:

| In the folder | What it is |
|---|---|
| `project.json` | The match — its videos, clips, events, slates, highlights and settings. |
| `recordings/` | Your commentary, one file per take. |
| `exports/` | The videos you export. |

**The game video stays where it is.** A project points at your footage; it never
copies, moves or renames it.

## Starting a match

Press **New match…** — in the toolbar, or the big button on an empty window —
and pick the game's video files. pundit shows you what it worked out from them,
and every line can be changed before anything is made:

- **Projects folder.** pundit looks for a folder called `pundit` at or just above
  your videos, which is how most people already keep them. Failing that it
  offers the folder your last project was in, and failing that a `pundit` folder
  beside the footage — and says so. **Choose…** points it somewhere else.
- **Folder name.** The date, read out of the file or folder names, then the
  teams: `2026-09-21-city-athletic`. Putting the date first keeps a season in
  order. Type your own name and it stops following the teams.
- **Teams.** The opponent comes from the folder name and goes in **away**, your
  club in **home**. **⇄** swaps them, and the folder name follows as you watch.
  The scoreboard is set up with your last match's colours and format.
- **The videos**, in the order they will play.

Press **Create**. The project is named `<Home> v <Away>`.

Nothing is written until every video has been checked. If the second half is a
different shape from the first, you are told and no half-made project is left
behind. A folder that already holds a project is refused; an empty folder you
made yourself is used as it is.

## Opening one you made earlier

- **Recent ▾**, beside **Open Project…**, lists the last eight matches you had
  open, newest first and named after the match. One click switches. A match
  whose folder has gone — on a drive you have not plugged in, say — goes grey
  rather than vanishing, so you can plug the drive in and click it.
- **Open Project…** (<kbd>Ctrl</kbd>+<kbd>O</kbd>) opens any folder, and makes a
  new empty project in a folder that has none.
- The app reopens your last project on its own when it starts.

You cannot switch projects while an export is running or a preview is open; the
message says which.

## The game videos

**Add Source Video…** adds more footage to the match. The videos play as one
timeline, in the order listed under **Sources**. Drag them to reorder, and the
clock and every clip's place in the game follow. A video whose shape differs
from the others is refused.

The **✕** beside a video removes it, and is greyed out once a clip, a match
event, a slate or a highlight uses that video.

If you move or rename your footage, its video shows as **Missing**; **Relink…**
points the project at where it is now.

## Older projects

A project made by an older version of pundit is upgraded the first time you
save it. The old file is kept beside it as `project.json.v<old version>`, so you
can go back to the older version if you need to — losing whatever changed since.
