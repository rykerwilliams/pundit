# Transcripts

**Transcribe**, on a clip's transcript row in the inspector, writes out what you
said, using [whisper.cpp](https://github.com/ggml-org/whisper.cpp) on your own
computer. The transcript is yours to edit.

## The speech model

The picker beside the button chooses the model:

| Model | Download | |
|---|---|---|
| `small.en` | 488 MB | The default. More accurate; about 0.7× real time on a typical laptop. |
| `base.en` | 148 MB | Faster, less accurate. |

The first time you transcribe with a model, pundit downloads it into
`~/.cache/pundit/models/` and keeps it — the button says so, as **Download
488 MB and transcribe**, before you press it. After that, nothing leaves your
machine. The model you choose is remembered for this computer, not for the
project.

## While it runs

Transcribing is always something you ask for; recording never starts it. Clips
you ask for queue behind each other, and the inspector shows how long the current
one has been going.

**Recording always wins.** Starting a recording stops a transcription in
progress and puts that clip back at the front of the queue, so the take gets the
whole machine.

## Using your own model file

If `$PUNDIT_WHISPER_MODEL` names a model file, pundit uses that instead of the
picker, which greys out and shows the file.
