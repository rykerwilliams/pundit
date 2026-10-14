//! A speech model, fetched the first time it is needed (Phase 11 spec S3).
//!
//! ```text
//! souphttpsrc location=<url> iradio-mode=false ! filesink location=<dest>.part
//! ```
//!
//! **GStreamer is the HTTP client and glib is the hash**, so this adds no
//! crate: `souphttpsrc` is `plugins-good`, whose `libsoup` brings TLS through
//! `glib-networking`, and `glib::Checksum` is already linked through
//! `gst::glib`. It follows redirects, which Hugging Face's CDN needs.
//!
//! **`.part`, verified, then renamed**, as export does: nothing at the final
//! path is ever less than the whole file, so "is the model there" stays one
//! `is_file`.
//!
//! [`get`] is the same transport into memory, for what has no digest to check
//! — a page of search results, a fetched song (BACKLOG #84 phase 2).
//!
//! **Network facts, measured for the spec and not handled here:**
//! `souphttpsrc` gives up on a read after 15 s and retries 3 times, and the
//! CDN's signed URL expires about an hour after issue — which only matters
//! below roughly 135 kB/s for `small.en`.

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use gstreamer as gst;
use gstreamer::glib;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use pundit_core::metadata::APP_NAME;

use crate::composite::{CompositeError, Stopper};

/// How often the download looks at the cancel flag and its own progress.
const POLL: gst::ClockTime = gst::ClockTime::from_mseconds(100);

/// How much of the file is hashed at a time: small enough that a cancel is
/// noticed within a few milliseconds, big enough that the loop costs nothing
/// beside the hash.
const HASH_CHUNK: usize = 1 << 20;

/// Held for the whole of a [`download`]: one at a time, process-wide.
///
/// **What it prevents:** the transcriber is never joined, so a cancelled job
/// can still be between its last cancel check and its rename when the next
/// job opens — and truncates — the same `.part`, and the rename would then
/// move a half-written file to the model's path. Rare (it takes a clip
/// trashed with another queued behind it), but the fix costs nothing: a
/// cancelled download lets go within [`POLL`], so a successor waits at most
/// that long.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// Where a file comes from and how to know it arrived whole: its URL, its
/// sha256 and its length.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fetch {
    pub url: String,
    /// Lower-case hex, as [`glib::Checksum::string`] writes it.
    pub sha256: String,
    /// The whole file's length: the progress denominator, and the first
    /// check on what arrived.
    pub bytes: u64,
}

/// Downloads `fetch` to `dest`, reporting whole percents to `progress` from
/// this thread, and returns once `dest` is the verified file.
///
/// `progress(0)` comes first, before anything can fail or wait, so a caller
/// showing a download has something to show even for one that fails at once
/// or waits its turn behind a cancelled one.
///
/// **Every failure deletes the `.part`, a cancel included** — a refused
/// request, a dropped connection, a full disk, a file that fails its check —
/// so nothing is left taking up the room the next recording needs. Only
/// quitting mid-download leaves one, and the next attempt truncates it.
///
/// A cancel used to leave its `.part` alone, because the transcriber is never
/// joined and a delete could land after the *next* job's `filesink` had opened
/// the same path. [`ONE_AT_A_TIME`] rules that out: the cancelled job still
/// holds it while it cleans up, so the next job can't open the file until it
/// is gone. The failure says what went wrong. No retry here: pressing
/// Transcribe again is the retry.
pub fn download(
    fetch: &Fetch,
    dest: &Path,
    progress: &mut dyn FnMut(u8),
    cancel: &AtomicBool,
) -> Result<(), CompositeError> {
    progress(0);
    // A download that panicked leaves nothing this guards to repair.
    let _turn = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    // `filesink` opens its file and nothing else, so a first run on a
    // machine with no cache directory yet would fail here without this.
    if let Some(dir) = dest.parent() {
        std::fs::create_dir_all(dir).map_err(|e| {
            CompositeError::Failed(format!("could not create {}: {e}", dir.display()))
        })?;
    }
    let part = part_path(dest);
    let outcome =
        fetch_to(fetch, &part, progress, cancel).and_then(|()| check(fetch, &part, dest, cancel));
    match outcome {
        // Whatever else happened, including a hash that passed just as the
        // cancel came in: a cancelled job never renames. Safe to delete under
        // `_turn`, which the next job is waiting on.
        _ if cancel.load(Ordering::SeqCst) => {
            let _ = std::fs::remove_file(&part);
            Err(CompositeError::Cancelled)
        }
        Err(failed) => {
            let _ = std::fs::remove_file(&part);
            Err(failed)
        }
        Ok(()) => std::fs::rename(&part, dest).map_err(|e| {
            CompositeError::Failed(format!(
                "could not move {} to {}: {e}",
                part.display(),
                dest.display()
            ))
        }),
    }
}

/// `dest` with `.part` on the end of its whole name, as export writes its
/// output: `ggml-small.en.bin.part`, never `ggml-small.en.part`.
fn part_path(dest: &Path) -> PathBuf {
    let mut name = dest.as_os_str().to_owned();
    name.push(".part");
    PathBuf::from(name)
}

/// The transfer: `fetch.url` into `part`, until EOS, an error or a cancel.
///
/// **Progress is polled, not probed:** `filesink` answers a byte-position
/// query with what it has written, which is the number the coach cares about
/// and needs no callback on the streaming thread.
fn fetch_to(
    fetch: &Fetch,
    part: &Path,
    progress: &mut dyn FnMut(u8),
    cancel: &AtomicBool,
) -> Result<(), CompositeError> {
    let make = |factory: &str| {
        gst::ElementFactory::make(factory)
            .build()
            .map_err(|e| CompositeError::Failed(format!("{factory} is missing: {e}")))
    };
    let src = make("souphttpsrc")?;
    src.set_property("location", &fetch.url);
    // Off, so a server that happens to send ICY headers can't turn this into
    // a radio stream with caps of its own.
    src.set_property("iradio-mode", false);
    let sink = make("filesink")?;
    sink.set_property("location", part);
    let pipeline = gst::Pipeline::new();
    pipeline
        .add_many([&src, &sink])
        .expect("add the download elements");
    src.link(&sink).expect("link souphttpsrc to filesink");
    let pipeline = Stopper(pipeline);
    let bus = pipeline.bus().expect("a pipeline has a bus");

    let failed = |err: Option<&gst::message::Error>| {
        CompositeError::Failed(match err {
            Some(err) => unreachable_or(&fetch.url, err),
            None => format!("could not download {}: it would not start", fetch.url),
        })
    };
    if pipeline.set_state(gst::State::Playing).is_err() {
        // The element's own reason, if it posted one before refusing.
        let msg = bus.pop_filtered(&[gst::MessageType::Error]);
        return Err(failed(msg.as_ref().and_then(|msg| match msg.view() {
            gst::MessageView::Error(err) => Some(err),
            _ => None,
        })));
    }
    let mut reported = 0;
    loop {
        if cancel.load(Ordering::SeqCst) {
            return Err(CompositeError::Cancelled);
        }
        if let Some(msg) =
            bus.timed_pop_filtered(POLL, &[gst::MessageType::Eos, gst::MessageType::Error])
        {
            match msg.view() {
                gst::MessageView::Error(err) => return Err(failed(Some(err))),
                _ => {
                    // All of it, which a fast transfer can reach between
                    // two polls.
                    if reported != 100 {
                        progress(100);
                    }
                    return Ok(());
                }
            }
        }
        if let Some(done) = pipeline.query_position::<gst::format::Bytes>() {
            let percent = (u64::from(done) * 100 / fetch.bytes.max(1)).min(100) as u8;
            if percent != reported {
                reported = percent;
                progress(percent);
            }
        }
    }
}

/// What to tell the coach about a transfer that failed with `err`.
///
/// **Offline is the likeliest failure at a field, and GStreamer's words for it
/// are the worst.** A refused connection or a name that won't resolve gets no
/// error of `souphttpsrc`'s own: `basesrc` posts its generic "Internal data
/// stream error" instead, which says nothing a coach can act on. Every error
/// the server or the disk causes — a 404, a 403, a full disk — is posted
/// first, with a reason of its own, and is passed through. The raw error goes
/// to stderr either way.
fn unreachable_or(url: &str, err: &gst::message::Error) -> String {
    let raw = crate::error_text(err);
    if !err.error().matches(gst::StreamError::Failed) {
        return format!("could not download {url}: {raw}");
    }
    eprintln!("download: {url}: {raw}");
    let host = url
        .split_once("://")
        .and_then(|(_, rest)| rest.split(['/', ':']).next())
        .unwrap_or(url);
    format!("could not reach {host} — check the internet connection")
}

/// How long [`get`] waits on any one read before giving up — `souphttpsrc`'s
/// own `timeout`, which bounds each read and not the whole transfer, so a slow
/// but moving one is never cut off. One second under test, where a stall is
/// the thing being waited out.
#[cfg(not(test))]
const GET_TIMEOUT_SECONDS: u32 = 20;
#[cfg(test)]
const GET_TIMEOUT_SECONDS: u32 = 1;

/// Why [`get`] returned no body.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GetError {
    /// The server answered with this HTTP status, 400 or more — read from the
    /// error's `http-status-code` detail, never from its text.
    #[error("the server answered {0}")]
    Status(u16),
    /// Anything else: offline, a refused connection, a body cut short, a body
    /// over the limit. Worded for the coach.
    #[error("{0}")]
    Failed(String),
    #[error("cancelled")]
    Cancelled,
}

/// The body at `url`, whole, into memory — never more than `limit` bytes.
///
/// ```text
/// souphttpsrc location=<url> user-agent=pundit/<version> iradio-mode=false
///             timeout=<GET_TIMEOUT_SECONDS> ! appsink
/// ```
///
/// [`download`]'s transport, for what has no length or digest to check it by:
/// a page of search results, a fetched song. The loop is the same bus-driven
/// one, with the cancel looked at between pulls and the bytes counted as they
/// arrive, so `limit` holds against a server that sends for ever.
///
/// **There is no length check of its own.** A body short of its
/// `Content-Length` is `souphttpsrc`'s to catch, and it does: it tries to
/// resume with a Range request, and a server that answers that with a `200`
/// gets *"Server does not support seeking."* (measured on 1.24.2), which is
/// [`GetError::Failed`]. **An HTTP status is read from the error's
/// structured details**, `http-status-code`, and only from 400 up: that short
/// body's error carries `200`.
pub fn get(url: &str, limit: usize, cancel: &AtomicBool) -> Result<Vec<u8>, GetError> {
    let make = |factory: &str| {
        gst::ElementFactory::make(factory)
            .build()
            .map_err(|e| GetError::Failed(format!("{factory} is missing: {e}")))
    };
    let src = make("souphttpsrc")?;
    src.set_property("location", url);
    src.set_property(
        "user-agent",
        format!("{APP_NAME}/{}", env!("CARGO_PKG_VERSION")),
    );
    src.set_property("iradio-mode", false);
    src.set_property("timeout", GET_TIMEOUT_SECONDS);
    let sink = gst_app::AppSink::builder()
        .sync(false)
        .max_buffers(16)
        .enable_last_sample(false)
        .build();
    let pipeline = gst::Pipeline::new();
    pipeline
        .add_many([&src, sink.upcast_ref()])
        .expect("add the fetch elements");
    src.link(&sink).expect("link souphttpsrc to appsink");
    let pipeline = Stopper(pipeline);
    let bus = pipeline.bus().expect("a pipeline has a bus");

    let failed = |err: &gst::message::Error| {
        let status = err
            .details()
            .and_then(|d| d.get::<u32>("http-status-code").ok())
            .and_then(|code| u16::try_from(code).ok())
            .filter(|&code| code >= 400);
        match status {
            Some(code) => GetError::Status(code),
            None => GetError::Failed(unreachable_or(url, err)),
        }
    };
    let first_error = |bus: &gst::Bus| {
        bus.pop_filtered(&[gst::MessageType::Error])
            .and_then(|msg| match msg.view() {
                gst::MessageView::Error(err) => Some(failed(err)),
                _ => None,
            })
    };
    if pipeline.set_state(gst::State::Playing).is_err() {
        return Err(first_error(&bus).unwrap_or_else(|| {
            GetError::Failed(format!("could not download {url}: it would not start"))
        }));
    }
    let mut body = Vec::new();
    loop {
        if cancel.load(Ordering::SeqCst) {
            return Err(GetError::Cancelled);
        }
        if let Some(sample) = sink.try_pull_sample(POLL) {
            if let Some(map) = sample.buffer().and_then(|b| b.map_readable().ok()) {
                body.extend_from_slice(&map);
            }
            if body.len() > limit {
                return Err(GetError::Failed(format!(
                    "{url} is larger than the {limit} bytes pundit will fetch"
                )));
            }
            continue;
        }
        // Starved for a whole poll: the end, a failure, or a slow server. An
        // `ERROR` is posted before the source's EOS reaches the sink, so it is
        // looked for first.
        if let Some(error) = first_error(&bus) {
            return Err(error);
        }
        if sink.is_eos() {
            return Ok(body);
        }
    }
}

/// Whether `part` is the file `fetch` describes, as it will be at `dest`:
/// `Failed` naming `dest` when it isn't, `Cancelled` if the cancel came in
/// while it was being hashed.
///
/// **The file is hashed, not the stream.** What is checked is then exactly
/// what gets renamed, whatever `filesink` did or didn't write. The cost is
/// glib's speed, not the disk's: about 90 MB/s on the reference laptop
/// (148 MB in 1.6 s), so `small.en` sits at 100% for some five seconds while
/// it is checked.
fn check(
    fetch: &Fetch,
    part: &Path,
    dest: &Path,
    cancel: &AtomicBool,
) -> Result<(), CompositeError> {
    let unreadable = |e: std::io::Error| {
        CompositeError::Failed(format!("could not read {}: {e}", part.display()))
    };
    let mismatch = |why: String| {
        CompositeError::Failed(format!(
            "the download of {} failed its check ({why}), and was deleted",
            dest.display()
        ))
    };
    let mut file = File::open(part).map_err(unreadable)?;
    // The cheap check first, and the one that says the most when it fails:
    // a transfer that stopped early.
    let len = file.metadata().map_err(unreadable)?.len();
    if len != fetch.bytes {
        return Err(mismatch(format!("{len} bytes of {}", fetch.bytes)));
    }
    let mut sum = glib::Checksum::new(glib::ChecksumType::Sha256).expect("glib has sha256");
    let mut chunk = vec![0; HASH_CHUNK];
    loop {
        if cancel.load(Ordering::SeqCst) {
            return Err(CompositeError::Cancelled);
        }
        let n = file.read(&mut chunk).map_err(unreadable)?;
        if n == 0 {
            break;
        }
        sum.update(&chunk[..n]);
    }
    let got = sum.string().expect("a sha256 has a hex form");
    if got == fetch.sha256 {
        Ok(())
    } else {
        Err(mismatch(format!("sha256 {got}, expected {}", fetch.sha256)))
    }
}

#[cfg(test)]
mod tests {
    use std::net::TcpListener;
    use std::sync::{mpsc, Arc};
    use std::time::Duration;

    use super::*;
    use crate::fixtures::{serve, served_body, Answer, SERVED_SHA256};

    /// [`serve`], answering at once.
    fn server(answer: Answer) -> String {
        serve(answer, Duration::ZERO)
    }

    fn fetch(url: String, sha256: &str) -> Fetch {
        Fetch {
            url,
            sha256: sha256.into(),
            bytes: served_body().len() as u64,
        }
    }

    fn dir() -> tempfile::TempDir {
        gst::init().unwrap();
        tempfile::tempdir().unwrap()
    }

    /// The file lands at `dest` — in a directory that didn't exist, as the
    /// cache's `models/` doesn't on a first run — byte for byte, with no
    /// `.part` beside it, and the percent climbs to 100.
    #[test]
    fn a_good_file_lands_verified() {
        let dir = dir();
        let dest = dir.path().join("models").join("ggml-test.bin");
        let mut percents = Vec::new();
        download(
            &fetch(server(Answer::Whole), SERVED_SHA256),
            &dest,
            &mut |p| percents.push(p),
            &AtomicBool::new(false),
        )
        .expect("the download succeeds");
        assert!(
            std::fs::read(&dest).unwrap() == served_body(),
            "the file is not the body"
        );
        assert!(!part_path(&dest).exists(), "the .part was left behind");
        assert_eq!(percents.first(), Some(&0), "{percents:?}");
        assert_eq!(percents.last(), Some(&100), "{percents:?}");
        assert!(percents.is_sorted(), "{percents:?}");
    }

    /// A file that arrives whole but isn't the one asked for is deleted, and
    /// the failure names where it was going — and nothing lands there.
    #[test]
    fn a_wrong_hash_deletes_the_part_and_names_the_path() {
        let dir = dir();
        let dest = dir.path().join("ggml-test.bin");
        let wrong = "0".repeat(64);
        let error = download(
            &fetch(server(Answer::Whole), &wrong),
            &dest,
            &mut |_| {},
            &AtomicBool::new(false),
        )
        .expect_err("the hash does not match");
        let CompositeError::Failed(message) = error else {
            panic!("expected a failure, got {error:?}");
        };
        assert!(
            message.contains(&dest.display().to_string()) && message.contains(SERVED_SHA256),
            "the message names neither the path nor what arrived: {message}"
        );
        assert!(!part_path(&dest).exists(), "the .part survived");
        assert!(!dest.exists(), "a file that failed its check was kept");
    }

    /// A cancel mid-transfer is [`CompositeError::Cancelled`], and **no file
    /// lands at `dest`** — the `.part` is left, on purpose (see
    /// [`download`]).
    #[test]
    fn a_cancel_leaves_no_final_file() {
        let dir = dir();
        let dest = dir.path().join("ggml-test.bin");
        let url = server(Answer::Stall);
        let cancel = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel();
        let run = std::thread::spawn({
            let (dest, cancel) = (dest.clone(), cancel.clone());
            move || {
                download(
                    &fetch(url, SERVED_SHA256),
                    &dest,
                    &mut |p| {
                        let _ = tx.send(p);
                    },
                    &cancel,
                )
            }
        });
        // Half the body has arrived, so the cancel lands mid-transfer rather
        // than before it starts.
        let deadline = std::time::Instant::now() + Duration::from_secs(15);
        while rx.recv_timeout(Duration::from_secs(15)).expect("progress") < 40 {
            assert!(std::time::Instant::now() < deadline, "no progress");
        }
        cancel.store(true, Ordering::SeqCst);
        assert_eq!(run.join().unwrap(), Err(CompositeError::Cancelled));
        assert!(!dest.exists(), "a cancelled download landed");
        assert!(
            !part_path(&dest).exists(),
            "a cancelled download left its .part taking up room"
        );
    }

    /// A server that refuses fails the download with the server's reason and
    /// the URL — not as a file that failed its check.
    #[test]
    fn a_server_error_fails() {
        let dir = dir();
        let dest = dir.path().join("ggml-test.bin");
        let url = server(Answer::Status("404 Not Found"));
        let error = download(
            &fetch(url.clone(), SERVED_SHA256),
            &dest,
            &mut |_| {},
            &AtomicBool::new(false),
        )
        .expect_err("there is nothing there");
        let CompositeError::Failed(message) = error else {
            panic!("expected a failure, got {error:?}");
        };
        assert!(
            message.contains(&format!("could not download {url}")),
            "{message}"
        );
        assert!(!dest.exists());
    }

    /// **A transfer that fails deletes its `.part`**, as a failed check
    /// does: a refused request once left 2 MB behind, and a disk that fills
    /// mid-download would leave nearly 488 MB where the next recording needs
    /// the room, with the coach unaware the file exists.
    #[test]
    fn a_failed_transfer_deletes_the_part() {
        let dir = dir();
        let dest = dir.path().join("ggml-test.bin");
        download(
            &fetch(server(Answer::Status("403 Forbidden")), SERVED_SHA256),
            &dest,
            &mut |_| {},
            &AtomicBool::new(false),
        )
        .expect_err("the server refuses");
        assert!(!part_path(&dest).exists(), "the .part survived");
    }

    /// **Offline says so.** A refused connection gets nothing of
    /// `souphttpsrc`'s own, only `basesrc`'s generic "Internal data stream
    /// error" — which is what the coach used to read, at the field, where
    /// being offline is the likeliest failure there is.
    #[test]
    fn an_unreachable_server_says_so() {
        let dir = dir();
        let dest = dir.path().join("ggml-test.bin");
        let port = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let error = download(
            &fetch(
                format!("http://127.0.0.1:{port}/ggml-test.bin"),
                SERVED_SHA256,
            ),
            &dest,
            &mut |_| {},
            &AtomicBool::new(false),
        )
        .expect_err("nothing is listening");
        let CompositeError::Failed(message) = error else {
            panic!("expected a failure, got {error:?}");
        };
        assert!(
            message.contains("could not reach 127.0.0.1")
                && message.contains("internet connection")
                && !message.contains("Internal data stream error"),
            "{message}"
        );
        assert!(!part_path(&dest).exists(), "the .part survived");
    }

    fn body(len: usize) -> Arc<[u8]> {
        (0..len).map(|i| (i % 253) as u8).collect()
    }

    #[test]
    fn get_returns_the_body() {
        gst::init().unwrap();
        let sent = body(300_000);
        let got = get(
            &server(Answer::Body(sent.clone())),
            1 << 20,
            &AtomicBool::new(false),
        )
        .expect("the body");
        assert!(*got == *sent, "{} bytes back of {}", got.len(), sent.len());
    }

    #[test]
    fn get_refuses_past_its_limit() {
        gst::init().unwrap();
        let error = get(&server(Answer::Whole), 512 << 10, &AtomicBool::new(false))
            .expect_err("a 1 MB body under a 512 KB limit");
        assert!(
            matches!(&error, GetError::Failed(why) if why.contains("larger than")),
            "{error:?}"
        );
    }

    /// A transfer that stops mid-way is given up on at `souphttpsrc`'s own
    /// read timeout — a second under test — and not left to hang. Measured on
    /// 1.24.2, 1.1 s: the timeout, then a Range request this server answers
    /// with a `200`, which is [`a_short_body_is_an_error`]'s error. Only the
    /// time is asserted here; what the error is belongs to that test.
    #[test]
    fn get_gives_up_on_a_stall() {
        gst::init().unwrap();
        let begun = std::time::Instant::now();
        get(&server(Answer::Stall), 4 << 20, &AtomicBool::new(false))
            .expect_err("a stalled transfer");
        let took = begun.elapsed();
        assert!(took < Duration::from_secs(10), "gave up after {took:?}");
    }

    /// **A body short of its `Content-Length` is an error**, and not an HTTP
    /// status: `souphttpsrc` tries to resume it with a Range request, gets a
    /// `200`, and says *"Server does not support seeking"* with
    /// `http-status-code=200` (measured on 1.24.2).
    #[test]
    fn a_short_body_is_an_error() {
        gst::init().unwrap();
        let sent = body(100_000);
        let result = get(
            &server(Answer::Cut(sent.clone(), sent.len() / 2)),
            1 << 20,
            &AtomicBool::new(false),
        );
        assert!(matches!(result, Err(GetError::Failed(_))), "{result:?}");
    }

    #[test]
    fn get_reads_the_status_from_the_details() {
        gst::init().unwrap();
        assert_eq!(
            get(
                &server(Answer::Status("429 Too Many Requests")),
                1 << 20,
                &AtomicBool::new(false),
            ),
            Err(GetError::Status(429))
        );
    }

    #[test]
    fn get_cancels() {
        gst::init().unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let url = server(Answer::Stall);
        let run = std::thread::spawn({
            let cancel = cancel.clone();
            move || {
                let begun = std::time::Instant::now();
                (get(&url, 4 << 20, &cancel), begun.elapsed())
            }
        });
        std::thread::sleep(Duration::from_millis(500));
        cancel.store(true, Ordering::SeqCst);
        let (result, took) = run.join().unwrap();
        assert_eq!(result, Err(GetError::Cancelled));
        assert!(took < Duration::from_secs(2), "cancelled after {took:?}");
    }

    /// **Downloads take turns, process-wide.** The second waits while the
    /// first holds the `.part` it would otherwise truncate, and lands once
    /// the first is cancelled — see [`ONE_AT_A_TIME`].
    #[test]
    fn downloads_take_turns() {
        let dir = dir();
        let dest = dir.path().join("ggml-test.bin");
        let cancel = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel();
        let first = std::thread::spawn({
            let (dest, cancel, url) = (dest.clone(), cancel.clone(), server(Answer::Stall));
            move || {
                download(
                    &fetch(url, SERVED_SHA256),
                    &dest,
                    &mut |p| {
                        let _ = tx.send(p);
                    },
                    &cancel,
                )
            }
        });
        while rx.recv_timeout(Duration::from_secs(15)).expect("progress") < 40 {}
        let (done_tx, done_rx) = mpsc::channel();
        let second = std::thread::spawn({
            let (dest, url) = (dest.clone(), server(Answer::Whole));
            move || {
                let result = download(
                    &fetch(url, SERVED_SHA256),
                    &dest,
                    &mut |_| {},
                    &AtomicBool::new(false),
                );
                let _ = done_tx.send(());
                result
            }
        });
        assert!(
            done_rx.recv_timeout(Duration::from_millis(1_000)).is_err(),
            "the second download ran while the first held the .part"
        );
        cancel.store(true, Ordering::SeqCst);
        assert_eq!(first.join().unwrap(), Err(CompositeError::Cancelled));
        second.join().unwrap().expect("the second download lands");
        assert!(std::fs::read(&dest).unwrap() == served_body());
    }
}
