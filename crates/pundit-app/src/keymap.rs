//! Every key the window binds, in one table the coach can read and rebind
//! (BACKLOG #96; spec `docs/superpowers/specs/2026-10-02-rebindable-keys-design.md`,
//! plan `docs/superpowers/plans/2026-10-07-rebindable-keys.md`).
//!
//! **The convention is VS Code's `keybindings.json`**, which is what the coach
//! asked for ("that is generally a solved space; use a library or known
//! pattern/convention"): the **defaults live in code** ([`Action::default_keys`]),
//! the file holds **only the overrides** ([`Keymap::overrides`]) keyed by the
//! action's name, and a conflict is **last-wins** with the displaced binding
//! shown unbound. No crate fits — the plan's §0a records what `keybinds`,
//! `keymap-rs` and `keyboard-types` each buy and the one documented fact that
//! rules out the closest (`keybinds` forbids `Shift` on a character key, which
//! is exactly `shift+a`, and is case-sensitive, which throws away the Caps-Lock
//! rule below).
//!
//! **The stored map is a diff, not a copy**, which is the half of the
//! convention the spec left open: writing all 29 rows out would pin every
//! default against every future change, for a coach who touched one key. So
//! [`Keymap::overrides`] emits only the rows that differ, an absent row keeps
//! its default, and deleting the whole `keys` key is the reset path.
//!
//! **What a key event carries, and what it therefore cannot express.** Slint's
//! `KeyEvent` has three fields — `text`, `modifiers`, `repeat` — and the winit
//! backend reads `logical_key` only; `physical_key` never appears in it, and a
//! key that produces no text never reaches the app. So a **layout-independent**
//! binding is not expressible here and no plumbing on our side could recover
//! one (BACKLOG #35 is retired by this table rather than waiting for a Slint
//! that may never expose scancodes: a coach on another layout binds the keys
//! where they are, once, and `state.json` remembers).
//!
//! Two consequences of the text being the **shifted** reading:
//!
//! - **The matcher lowercases the text and compares the three modifiers
//!   exactly**, which is Slint's own rule for its internal matcher and what
//!   keeps Caps Lock free: under Caps Lock the backend reports `"R"` with
//!   `shift: false`, so `r` still records.
//! - **`shift+<punctuation>` is only bindable as the character the key
//!   produces.** A shifted letter still reports the letter (`"A"`, `shift:
//!   true`), which is what makes `shift+a` work; a shifted digit reports the
//!   punctuation, so Shift+1 arrives as `!` and is bound by writing `!`.
//!
//! **No `when` clause, and that is not an omission.** In this app a bound key
//! is swallowed *whether or not* its guard passes — every branch of
//! `handle-key` is `if (text == …) { if (pressed && <guard>) { … } return
//! accept; }`, so that a touched slider never turns the arrows into volume. A
//! `when` clause makes a binding **not match**, which would hand the key to
//! the focused child in exactly the case the guard refuses it. So the lookup is
//! `(key, modifiers) -> action` and nothing else, and [`Action::when`] is one
//! prose sentence for the coach to read rather than a second machine-readable
//! copy of a Slint guard.

use std::collections::BTreeMap;

/// How an unbound action's keys read in the listing.
const UNBOUND: &str = "—";

/// One thing a key does.
///
/// The variants are the window's own bindings, with two additions the table
/// made possible — `skip`'s Shift argument is lifted into [`Action::SkipBackFar`]
/// / [`Action::SkipForwardFar`], so the far skip is listable and rebindable
/// rather than a modifier read inside the action — and two the table made
/// necessary: [`Action::ShowKeys`] and [`Action::ShowRecents`].
///
/// **Escape, Tab, Shift+Tab and Return are not here, deliberately** (spec G4).
/// Escape is not an action with a key; it is a key with a position — "back out
/// of the innermost thing on screen" — in seven jobs, one of which is Slint's
/// own popover dismissal, after both dispatch phases, where nothing we write
/// runs. Rebinding it would let a coach make a sheet unleavable. Tab is focus
/// traversal and Return confirms the error dialog: platform conventions, not app
/// actions. Home and End are swallowed and bound to nothing, which is a row in
/// the sheet rather than a variant here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    OpenProject,
    Undo,
    Redo,
    ClearDrawings,
    MarkIn,
    MarkOut,
    TagHomeGoal,
    TagAwayGoal,
    TagPeriod,
    HighlightTool,
    ToggleRecording,
    DeleteClip,
    TogglePlay,
    SkipBack,
    SkipForward,
    SkipBackFar,
    SkipForwardFar,
    StepBack,
    StepForward,
    PrevEvent,
    NextEvent,
    ScanSlower,
    ScanFaster,
    FitWindow,
    ZoomReset,
    ZoomOut,
    ZoomIn,
    ShowKeys,
    ShowRecents,
}

impl Action {
    /// Every action, in the order the sheet lists them — which is also the
    /// order the load rule resolves a collision in ([`Keymap::with_overrides`]),
    /// so it is fixed rather than cosmetic: of two stored rows claiming one
    /// key, the one listed first keeps it.
    ///
    /// On [`crate::drawing::Pen::ALL`]'s pattern, and with its one hazard: a
    /// variant added here and left out of the list is an action with no row.
    /// `the_list_holds_every_action_in_declaration_order` is what catches it.
    pub const ALL: [Action; 29] = [
        Action::OpenProject,
        Action::Undo,
        Action::Redo,
        Action::ClearDrawings,
        Action::MarkIn,
        Action::MarkOut,
        Action::TagHomeGoal,
        Action::TagAwayGoal,
        Action::TagPeriod,
        Action::HighlightTool,
        Action::ToggleRecording,
        Action::DeleteClip,
        Action::TogglePlay,
        Action::SkipBack,
        Action::SkipForward,
        Action::SkipBackFar,
        Action::SkipForwardFar,
        Action::StepBack,
        Action::StepForward,
        Action::PrevEvent,
        Action::NextEvent,
        Action::ScanSlower,
        Action::ScanFaster,
        Action::FitWindow,
        Action::ZoomReset,
        Action::ZoomOut,
        Action::ZoomIn,
        Action::ShowKeys,
        Action::ShowRecents,
    ];

    /// Its key in `state.json`, on [`crate::drawing::Pen::label`]'s rule: a
    /// name, so the file is hand-readable and a name **this build does not
    /// know is ignored** rather than costing the document. That is the whole
    /// reason the stored map is keyed by name and not a positional list.
    pub const fn name(self) -> &'static str {
        match self {
            Action::OpenProject => "openProject",
            Action::Undo => "undo",
            Action::Redo => "redo",
            Action::ClearDrawings => "clearDrawings",
            Action::MarkIn => "markIn",
            Action::MarkOut => "markOut",
            Action::TagHomeGoal => "tagHomeGoal",
            Action::TagAwayGoal => "tagAwayGoal",
            Action::TagPeriod => "tagPeriod",
            Action::HighlightTool => "highlightTool",
            Action::ToggleRecording => "toggleRecording",
            Action::DeleteClip => "deleteClip",
            Action::TogglePlay => "togglePlay",
            Action::SkipBack => "skipBack",
            Action::SkipForward => "skipForward",
            Action::SkipBackFar => "skipBackFar",
            Action::SkipForwardFar => "skipForwardFar",
            Action::StepBack => "stepBack",
            Action::StepForward => "stepForward",
            Action::PrevEvent => "prevEvent",
            Action::NextEvent => "nextEvent",
            Action::ScanSlower => "scanSlower",
            Action::ScanFaster => "scanFaster",
            Action::FitWindow => "fitWindow",
            Action::ZoomReset => "zoomReset",
            Action::ZoomOut => "zoomOut",
            Action::ZoomIn => "zoomIn",
            Action::ShowKeys => "showKeys",
            Action::ShowRecents => "showRecents",
        }
    }

    pub fn from_name(name: &str) -> Option<Action> {
        Action::ALL.into_iter().find(|a| a.name() == name)
    }

    /// What it does, for the sheet's first column.
    pub const fn what(self) -> &'static str {
        match self {
            Action::OpenProject => "Open a project",
            Action::Undo => "Undo",
            Action::Redo => "Redo",
            Action::ClearDrawings => "Clear the drawings",
            Action::MarkIn => "Mark a slate's in point",
            Action::MarkOut => "Mark a slate's out point",
            Action::TagHomeGoal => "Tag a home goal",
            Action::TagAwayGoal => "Tag an away goal",
            Action::TagPeriod => "Start or end a period",
            Action::HighlightTool => "Ring a player",
            Action::ToggleRecording => "Start or stop recording",
            Action::DeleteClip => "Delete the selected clip",
            Action::TogglePlay => "Play or pause",
            Action::SkipBack => "Skip back three seconds",
            Action::SkipForward => "Skip forward three seconds",
            Action::SkipBackFar => "Skip back ten seconds",
            Action::SkipForwardFar => "Skip forward ten seconds",
            Action::StepBack => "Step back one frame",
            Action::StepForward => "Step forward one frame",
            Action::PrevEvent => "Go to the previous match event",
            Action::NextEvent => "Go to the next match event",
            Action::ScanSlower => "Scan slower",
            Action::ScanFaster => "Scan faster",
            Action::FitWindow => "Fit the window to the footage",
            Action::ZoomReset => "Reset the zoom",
            Action::ZoomOut => "Zoom out",
            Action::ZoomIn => "Zoom in",
            Action::ShowKeys => "Show the keys",
            Action::ShowRecents => "Open the recent projects list",
        }
    }

    /// When it fires, in prose — the sheet's third column, and the **only**
    /// reading of the Slint guard that belongs here. A machine-readable copy of
    /// `can-tag` would be a second source of truth with nothing keeping the two
    /// in step; a sentence is read by the coach and by nothing else.
    pub const fn when(self) -> &'static str {
        match self {
            Action::OpenProject | Action::Undo | Action::Redo => {
                "Any time, unless a recording is running"
            }
            Action::ClearDrawings => "While a recording is running",
            Action::MarkIn
            | Action::MarkOut
            | Action::TagHomeGoal
            | Action::TagAwayGoal
            | Action::HighlightTool => "While the game video is on screen",
            Action::TagPeriod => "While the game video is on screen, until the periods are full",
            Action::ToggleRecording | Action::TogglePlay => "Any time",
            Action::DeleteClip => "With a clip selected, unless a recording is running",
            Action::SkipBack
            | Action::SkipForward
            | Action::SkipBackFar
            | Action::SkipForwardFar => "Any time",
            Action::StepBack | Action::StepForward => "While paused, with the game video on screen",
            Action::PrevEvent | Action::NextEvent => {
                "With the game video on screen, unless recording or previewing"
            }
            Action::ScanSlower | Action::ScanFaster => "While the game video is playing",
            Action::FitWindow => "When the window is bigger than the picture",
            Action::ZoomReset | Action::ZoomOut | Action::ZoomIn => "Any time but during a preview",
            Action::ShowKeys | Action::ShowRecents => "Any time",
        }
    }

    /// The keys it answers to out of the box, as labels (spec D1): 29 actions,
    /// **34 bindings**, which is the window's own chain with the far skips
    /// lifted out and nothing else moved.
    ///
    /// **Labels rather than [`Binding`] values**, so this table reads as the
    /// sheet will show it and the spec's own table can be checked against it by
    /// eye. The cost is that a typo would be a silently dropped default, which
    /// `every_default_label_parses` is what stops.
    ///
    /// [`Action::ShowRecents`] ships **unbound**, which is what closes #85's
    /// deferral at the cost of one row: that spec shipped the popover with no
    /// keyboard path on the explicit grounds that #96 owns the question, and the
    /// coach picks the key from the sheet.
    pub const fn default_keys(self) -> &'static [&'static str] {
        match self {
            Action::OpenProject => &["ctrl+o"],
            Action::Undo => &["ctrl+z"],
            Action::Redo => &["ctrl+shift+z", "ctrl+y"],
            Action::ClearDrawings => &["c"],
            Action::MarkIn => &["i"],
            Action::MarkOut => &["o"],
            Action::TagHomeGoal => &["z"],
            Action::TagAwayGoal => &["x"],
            Action::TagPeriod => &["v"],
            Action::HighlightTool => &["h"],
            Action::ToggleRecording => &["r"],
            Action::DeleteClip => &["Delete"],
            Action::TogglePlay => &["Space"],
            Action::SkipBack => &["LeftArrow", "a"],
            Action::SkipForward => &["RightArrow", "d"],
            Action::SkipBackFar => &["shift+LeftArrow", "shift+a"],
            Action::SkipForwardFar => &["shift+RightArrow", "shift+d"],
            Action::StepBack => &[","],
            Action::StepForward => &["."],
            Action::PrevEvent => &["["],
            Action::NextEvent => &["]"],
            Action::ScanSlower => &["j"],
            Action::ScanFaster => &["l"],
            Action::FitWindow => &["f"],
            Action::ZoomReset => &["1", "ctrl+0"],
            Action::ZoomOut => &["2"],
            Action::ZoomIn => &["3"],
            Action::ShowKeys => &["F1"],
            Action::ShowRecents => &[],
        }
    }
}

/// A key with a name rather than a character, because the character Slint
/// delivers for one is a private-use code point.
///
/// **Each one's `char` comes from `slint::platform::Key`** ([`NamedKey::ch`]),
/// never from a literal: the enum is public and `impl From<Key> for char` is
/// generated beside it, so there is no code point in this file to copy wrongly.
/// The list is closed — the keys the app binds or could sensibly bind — rather
/// than a copy of Slint's ninety.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamedKey {
    UpArrow,
    DownArrow,
    LeftArrow,
    RightArrow,
    Escape,
    Return,
    Tab,
    Space,
    Backspace,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
}

impl NamedKey {
    pub const ALL: [NamedKey; 26] = [
        NamedKey::UpArrow,
        NamedKey::DownArrow,
        NamedKey::LeftArrow,
        NamedKey::RightArrow,
        NamedKey::Escape,
        NamedKey::Return,
        NamedKey::Tab,
        NamedKey::Space,
        NamedKey::Backspace,
        NamedKey::Delete,
        NamedKey::Home,
        NamedKey::End,
        NamedKey::PageUp,
        NamedKey::PageDown,
        NamedKey::F1,
        NamedKey::F2,
        NamedKey::F3,
        NamedKey::F4,
        NamedKey::F5,
        NamedKey::F6,
        NamedKey::F7,
        NamedKey::F8,
        NamedKey::F9,
        NamedKey::F10,
        NamedKey::F11,
        NamedKey::F12,
    ];

    /// How it is spelled in a label, which is Slint's own name for it.
    pub const fn name(self) -> &'static str {
        match self {
            NamedKey::UpArrow => "UpArrow",
            NamedKey::DownArrow => "DownArrow",
            NamedKey::LeftArrow => "LeftArrow",
            NamedKey::RightArrow => "RightArrow",
            NamedKey::Escape => "Escape",
            NamedKey::Return => "Return",
            NamedKey::Tab => "Tab",
            NamedKey::Space => "Space",
            NamedKey::Backspace => "Backspace",
            NamedKey::Delete => "Delete",
            NamedKey::Home => "Home",
            NamedKey::End => "End",
            NamedKey::PageUp => "PageUp",
            NamedKey::PageDown => "PageDown",
            NamedKey::F1 => "F1",
            NamedKey::F2 => "F2",
            NamedKey::F3 => "F3",
            NamedKey::F4 => "F4",
            NamedKey::F5 => "F5",
            NamedKey::F6 => "F6",
            NamedKey::F7 => "F7",
            NamedKey::F8 => "F8",
            NamedKey::F9 => "F9",
            NamedKey::F10 => "F10",
            NamedKey::F11 => "F11",
            NamedKey::F12 => "F12",
        }
    }

    /// The character a key event carries for it, **from Slint's own enum**.
    /// This is the whole of the mapping, and it is why no code point appears in
    /// this file.
    pub fn ch(self) -> char {
        use slint::platform::Key;
        char::from(match self {
            NamedKey::UpArrow => Key::UpArrow,
            NamedKey::DownArrow => Key::DownArrow,
            NamedKey::LeftArrow => Key::LeftArrow,
            NamedKey::RightArrow => Key::RightArrow,
            NamedKey::Escape => Key::Escape,
            NamedKey::Return => Key::Return,
            NamedKey::Tab => Key::Tab,
            NamedKey::Space => Key::Space,
            NamedKey::Backspace => Key::Backspace,
            NamedKey::Delete => Key::Delete,
            NamedKey::Home => Key::Home,
            NamedKey::End => Key::End,
            NamedKey::PageUp => Key::PageUp,
            NamedKey::PageDown => Key::PageDown,
            NamedKey::F1 => Key::F1,
            NamedKey::F2 => Key::F2,
            NamedKey::F3 => Key::F3,
            NamedKey::F4 => Key::F4,
            NamedKey::F5 => Key::F5,
            NamedKey::F6 => Key::F6,
            NamedKey::F7 => Key::F7,
            NamedKey::F8 => Key::F8,
            NamedKey::F9 => Key::F9,
            NamedKey::F10 => Key::F10,
            NamedKey::F11 => Key::F11,
            NamedKey::F12 => Key::F12,
        })
    }
}

/// The key half of a [`Binding`]: a character the coach typed, or one of the
/// named keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Named(NamedKey),
}

impl Key {
    /// The character a key event carries for it.
    pub fn ch(self) -> char {
        match self {
            Key::Char(c) => c,
            Key::Named(named) => named.ch(),
        }
    }

    /// How it is spelled in a label: the character itself, or the name.
    pub fn label(self) -> String {
        match self {
            Key::Char(c) => c.to_string(),
            Key::Named(named) => named.name().to_owned(),
        }
    }

    /// **One spelling reaches one key**, which is why this is not two
    /// independent parsers: a character a [`NamedKey`] owns is spelled by that
    /// name and only by it, so `" "` is not a label and `"Space"` is. Two
    /// spellings of one key would be two rows free to disagree.
    fn from_label(label: &str) -> Option<Key> {
        if let Some(named) = NamedKey::ALL
            .into_iter()
            .find(|n| n.name().eq_ignore_ascii_case(label))
        {
            return Some(Key::Named(named));
        }
        let c = one_char(label)?;
        if NamedKey::ALL.into_iter().any(|n| n.ch() == c) {
            return None;
        }
        Some(Key::Char(c))
    }
}

/// One key and its three modifiers — what a coach presses, and what the table
/// maps to an [`Action`].
///
/// Its spelling is its label, on [`crate::drawing::Pen`]'s pattern:
/// [`Binding::label`] emits canonically (`Ctrl+Shift+Alt+` in that order, then
/// the key) and [`Binding::from_label`] reads it back, returning `None` rather
/// than an error for anything it cannot — the rule every stored label in this
/// app follows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Binding {
    key: Key,
    ctrl: bool,
    shift: bool,
    alt: bool,
}

impl Binding {
    /// `"r"`, `"ctrl+z"`, `"ctrl+shift+z"`, `"shift+LeftArrow"`, `","`,
    /// `"Delete"`, `"F1"`.
    pub fn label(self) -> String {
        let mut label = String::new();
        for (on, word) in [
            (self.ctrl, "ctrl"),
            (self.shift, "shift"),
            (self.alt, "alt"),
        ] {
            if on {
                label.push_str(word);
                label.push('+');
            }
        }
        label.push_str(&self.key.label());
        label
    }

    /// Case-insensitive on the modifier words and on a named key's name, and
    /// **order-insensitive on the modifiers**, because the file is hand-edited;
    /// [`Binding::label`] is what is canonical. A modifier named twice, a
    /// missing key and a key that is not one are each `None`.
    ///
    /// A character is folded to lower case here, so `"R"` and `"r"` are one
    /// binding — the matcher lowercases the event text, so a stored `"R"` that
    /// kept its case would be a binding no key could ever match.
    pub fn from_label(label: &str) -> Option<Binding> {
        let (mut ctrl, mut shift, mut alt) = (false, false, false);
        let mut rest = label;
        loop {
            if let Some(tail) = strip_modifier(rest, "ctrl") {
                if ctrl {
                    return None;
                }
                (ctrl, rest) = (true, tail);
            } else if let Some(tail) = strip_modifier(rest, "shift") {
                if shift {
                    return None;
                }
                (shift, rest) = (true, tail);
            } else if let Some(tail) = strip_modifier(rest, "alt") {
                if alt {
                    return None;
                }
                (alt, rest) = (true, tail);
            } else {
                break;
            }
        }
        Some(Binding {
            key: Key::from_label(rest)?,
            ctrl,
            shift,
            alt,
        })
    }

    /// The key it is on, for the sheet and for a rebind.
    pub fn key(self) -> Key {
        self.key
    }

    /// The binding a key event **is** — the reverse of [`Binding::matches`],
    /// and what a capture in the Keys sheet turns a press into (plan task 5).
    ///
    /// `None` for anything that is not a key the table can hold: text that is
    /// not one character, and a **modifier pressed on its own**. The second is
    /// not an edge case — a coach reaching for `Ctrl+R` presses Ctrl first, and
    /// Slint delivers a key event for it — so a capture that took the first
    /// press would bind Ctrl and never see the `R`. The caller keeps waiting
    /// instead.
    ///
    /// The named keys are matched by **character**, through [`NamedKey::ch`],
    /// so a press of the left arrow comes back as `Key::Named(LeftArrow)` and
    /// spells itself `"LeftArrow"` rather than as the private-use code point
    /// Slint delivered. That is the same single mapping [`Key::from_label`]
    /// refuses a second spelling of.
    pub fn from_event(text: &str, ctrl: bool, shift: bool, alt: bool) -> Option<Binding> {
        let ch = one_char(text)?;
        if is_modifier(ch) {
            return None;
        }
        let key = match NamedKey::ALL.into_iter().find(|named| named.ch() == ch) {
            Some(named) => Key::Named(named),
            None => Key::Char(ch),
        };
        Some(Binding {
            key,
            ctrl,
            shift,
            alt,
        })
    }

    /// Whether this is one of the keys the app keeps for itself — the five the
    /// Keys sheet lists as reserved (spec G4), with or without modifiers.
    ///
    /// **A capture that lands on one is refused rather than stored**, because
    /// storing it would make a listed binding that can never fire: `handle-key`
    /// tests Escape, Home and End *ahead* of the lookup, and Tab and Return are
    /// the platform's — Tab especially, since a coach who bound it away could
    /// not reach a control by keyboard to put it back. The modifiers are
    /// ignored on purpose: the window's own tests of those three read
    /// `event.text` alone, so `Ctrl+Home` is swallowed there too.
    ///
    /// It is only the **capture** this refuses. A hand-edited `state.json` may
    /// still name one, and costs itself a row that does nothing — which is the
    /// bargain every other unreadable thing in that file strikes.
    pub fn reserved(self) -> bool {
        matches!(
            self.key,
            Key::Named(
                NamedKey::Escape
                    | NamedKey::Tab
                    | NamedKey::Return
                    | NamedKey::Home
                    | NamedKey::End
            )
        )
    }

    /// Whether a key event is this binding: the character, lowercased by the
    /// caller, and the three modifiers **exactly**. Exactly is what makes
    /// `shift+a` a different binding from `a` and leaves Caps Lock free.
    fn matches(self, ch: char, ctrl: bool, shift: bool, alt: bool) -> bool {
        self.key.ch() == ch && self.ctrl == ctrl && self.shift == shift && self.alt == alt
    }
}

/// `"<word>+"` off the front of `label`, case-insensitively.
fn strip_modifier<'a>(label: &'a str, word: &str) -> Option<&'a str> {
    let tail = label.get(word.len() + 1..)?;
    let head = label.get(..word.len())?;
    (head.eq_ignore_ascii_case(word) && label.as_bytes()[word.len()] == b'+').then_some(tail)
}

/// Whether `ch` is a **modifier key pressed on its own** rather than a key.
///
/// From `slint::platform::Key` for [`NamedKey::ch`]'s reason: no code point in
/// this file to copy wrongly. All nine of them, because the right-hand Ctrl and
/// Shift are their own members and a coach has two hands; Caps Lock is here too,
/// since it reports a press and binding it would be a key with no release.
fn is_modifier(ch: char) -> bool {
    use slint::platform::Key;
    [
        Key::Shift,
        Key::ShiftR,
        Key::Control,
        Key::ControlR,
        Key::Alt,
        Key::AltGr,
        Key::Meta,
        Key::MetaR,
        Key::CapsLock,
    ]
    .into_iter()
    .any(|key| char::from(key) == ch)
}

/// A key event's text as one lowercased character, or `None` for anything that
/// is not one character — which no key the app can bind ever is.
///
/// Lowercasing a character that lowercases to **several** (`"İ"`) leaves it
/// alone: a one-character key is what the rest of the table is about, and a
/// multi-character fold has no single char to compare.
fn one_char(text: &str) -> Option<char> {
    let mut chars = text.chars();
    let c = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    let mut folded = c.to_lowercase();
    match (folded.next(), folded.next()) {
        (Some(lower), None) => Some(lower),
        // One that folds to several is left as it is: there is no single
        // character to compare, and no key the app binds is one.
        _ => Some(c),
    }
}

/// One row of the listing the Keys sheet shows (spec D2). Generated from the
/// same table the matcher reads, so **a listed row is a row the matcher
/// holds**: the list cannot go stale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyRow {
    pub what: &'static str,
    pub keys: String,
    pub when: &'static str,
    /// The name the row is stored under — `"openProject"` — which is what the
    /// sheet's *Set…* and *✕* carry back (plan task 5). The row's identity, so
    /// nothing has to match on the words in `what`.
    pub action: &'static str,
    /// Whether it answers to any key at all. **The ✕ is enabled by this and
    /// not by comparing the `keys` cell against an em dash**, which is this
    /// file's spelling of "nothing" and no sheet's business.
    pub bound: bool,
}

/// Which keys do what, right now: the defaults with the coach's overrides
/// applied.
///
/// Read **once at startup** into the window's lookup callback, never per key
/// event — which would be a file read per keypress.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keymap {
    /// One entry per [`Action::ALL`] entry, in that order. An empty list is
    /// **unbound**, which is a real state rather than a degenerate one: it is
    /// how a coach frees a key, and it is what an action with no default ships
    /// as.
    rows: Vec<(Action, Vec<Binding>)>,
}

impl Keymap {
    /// The table as it ships (spec D1).
    pub fn defaults() -> Keymap {
        Keymap {
            rows: Action::ALL
                .into_iter()
                .map(|action| (action, parse_row(action.default_keys().iter().copied())))
                .collect(),
        }
    }

    /// The defaults with the stored overrides applied (spec G3's load rule).
    ///
    /// **Five failure shapes, four of which cost nothing**, and they are the
    /// reason this is a map keyed by name rather than a stored copy of the
    /// table:
    ///
    /// - an action name this build does not know is **ignored**;
    /// - a spelling that does not parse is **dropped** and its siblings
    ///   survive;
    /// - an action present with `[]` is **unbound**, deliberately;
    /// - an action absent **keeps its default**, which is what makes deleting
    ///   the `keys` key the reset path;
    /// - and a key claimed twice is resolved rather than refused, below.
    ///
    /// **The overrides are placed first and the defaults fill in around them**,
    /// which is the convention's last-wins read the other way up: a coach who
    /// moves `o` to another action by hand means it, so `markOut` loses the key
    /// and the sheet shows it unbound — the alternative drops the coach's own
    /// edit in favour of a default they were editing *away* from. Among the
    /// stored rows themselves the tie-break is [`Action::ALL`]'s order, and the
    /// loser keeps whichever of its own bindings survive, possibly none. Every
    /// drop is logged by name, and the result is always consistent: one binding,
    /// one action.
    ///
    /// The load is a **fixpoint** — feeding [`Keymap::overrides`] of the result
    /// back in gives the same map — which is what lets a later write of the
    /// whole diff be the same keymap the coach is already using.
    pub fn with_overrides(stored: &BTreeMap<String, Vec<String>>) -> Keymap {
        for name in stored.keys() {
            if Action::from_name(name).is_none() {
                eprintln!("keys: ignoring {name:?}, which is not an action this build has");
            }
        }
        let mut taken: Vec<(Binding, Action)> = Vec::new();

        // The stored rows first, in `ALL` order.
        let stored_rows: Vec<(Action, Vec<Binding>)> = Action::ALL
            .into_iter()
            .filter_map(|action| {
                let labels = stored.get(action.name())?;
                let parsed = labels.iter().filter_map(|label| {
                    Binding::from_label(label).or_else(|| {
                        eprintln!(
                            "keys: ignoring {label:?}, which is not a key, for {}",
                            action.name()
                        );
                        None
                    })
                });
                let kept = parsed
                    .filter(|binding| claim(action, *binding, &mut taken))
                    .collect::<Vec<_>>();
                Some((action, kept))
            })
            .collect();

        // Then every action the file does not mention, keeping the defaults a
        // stored row has not taken.
        let rows = Action::ALL
            .into_iter()
            .map(|action| {
                if let Some(row) = stored_rows.iter().find(|(a, _)| *a == action) {
                    return row.clone();
                }
                let kept = parse_row(action.default_keys().iter().copied())
                    .into_iter()
                    .filter(|binding| claim(action, *binding, &mut taken))
                    .collect();
                (action, kept)
            })
            .collect();
        Keymap { rows }
    }

    /// **Only what differs from the defaults**, which is what `state.json`
    /// stores — a diff, not a copy. Writing the whole table out would pin all
    /// 29 rows for ever, so a later version that retunes a default would never
    /// reach a coach who had rebound one unrelated key.
    ///
    /// `BTreeMap`, not `HashMap`: every `AppFiles` setter rewrites the whole
    /// document, so a hash order would shuffle `state.json`'s keys on every pen
    /// change.
    pub fn overrides(&self) -> BTreeMap<String, Vec<String>> {
        let defaults = Keymap::defaults();
        self.rows
            .iter()
            .filter(|(action, bindings)| bindings.as_slice() != defaults.bindings(*action))
            .map(|(action, bindings)| {
                (
                    action.name().to_owned(),
                    bindings.iter().map(|b| b.label()).collect(),
                )
            })
            .collect()
    }

    /// The keys an action answers to, in the order the sheet shows them.
    pub fn bindings(&self, action: Action) -> &[Binding] {
        match self.rows.iter().find(|(a, _)| *a == action) {
            Some((_, bindings)) => bindings,
            None => &[],
        }
    }

    /// The one lookup, called once per key event: the event's text and its
    /// three modifiers in, an action or nothing out.
    ///
    /// A linear scan over at most a few dozen rows. It is called from the UI
    /// thread once per press and once per release, so there is nothing here to
    /// index.
    pub fn action_for(&self, text: &str, ctrl: bool, shift: bool, alt: bool) -> Option<Action> {
        let ch = one_char(text)?;
        self.rows
            .iter()
            .find(|(_, bindings)| bindings.iter().any(|b| b.matches(ch, ctrl, shift, alt)))
            .map(|(action, _)| *action)
    }

    /// Every action, in [`Action::ALL`]'s order, as the sheet shows it.
    pub fn listing(&self) -> Vec<KeyRow> {
        self.rows
            .iter()
            .map(|(action, bindings)| KeyRow {
                what: action.what(),
                keys: if bindings.is_empty() {
                    UNBOUND.to_owned()
                } else {
                    bindings
                        .iter()
                        .map(|b| b.label())
                        .collect::<Vec<_>>()
                        .join(", ")
                },
                when: action.when(),
                action: action.name(),
                bound: !bindings.is_empty(),
            })
            .collect()
    }

    /// Puts `binding` on `action`, taking it from whatever action held it, and
    /// answers which that was (spec G3's last-wins).
    ///
    /// **It replaces `action`'s bindings rather than adding to them**, which is
    /// the rule the sheet's one *Set…* button can state and a coach can
    /// predict. The two-key defaults (`LeftArrow`/`a`, `1`/`Ctrl+0`) keep their
    /// pairs until a *Set…* touches the row; a second affordance for adding a
    /// binding beside an existing one is deferred (BACKLOG #135).
    ///
    /// **The displaced action loses only that one key** and keeps whatever else
    /// it had, so taking `a` off the short skip leaves `LeftArrow` on it. What
    /// comes out is always consistent — one binding, one action — which is what
    /// lets [`Keymap::overrides`] be written whole and read back unchanged, and
    /// it is why the writer takes the map rather than a row: last-wins changes
    /// **two** rows, and a one-row write would leave the displaced binding in
    /// the file for the next load to resurrect as a collision.
    pub fn rebind(&mut self, action: Action, binding: Binding) -> Option<Action> {
        let displaced = self
            .rows
            .iter_mut()
            .find(|(holder, bindings)| *holder != action && bindings.contains(&binding))
            .map(|(holder, bindings)| {
                bindings.retain(|held| *held != binding);
                *holder
            });
        if let Some((_, bindings)) = self.rows.iter_mut().find(|(a, _)| *a == action) {
            *bindings = vec![binding];
        }
        displaced
    }

    /// Leaves `action` answering to nothing — a real state rather than a
    /// degenerate one, and how a coach frees a key without having to give it to
    /// something else first.
    pub fn unbind(&mut self, action: Action) {
        if let Some((_, bindings)) = self.rows.iter_mut().find(|(a, _)| *a == action) {
            bindings.clear();
        }
    }
}

/// Labels into bindings, dropping what does not parse. Used for the defaults,
/// where a drop is a typo `every_default_label_parses` catches.
fn parse_row<'a>(labels: impl Iterator<Item = &'a str>) -> Vec<Binding> {
    labels.filter_map(Binding::from_label).collect()
}

/// Gives `binding` to `action` unless another action already holds it, in
/// which case the drop is logged by name and the caller moves on: "one
/// binding, one action" is what the load rule guarantees, and this is the one
/// place it is enforced.
fn claim(action: Action, binding: Binding, taken: &mut Vec<(Binding, Action)>) -> bool {
    if let Some((_, holder)) = taken.iter().find(|(b, _)| *b == binding) {
        eprintln!(
            "keys: {} is {}'s, so {} does not get it",
            binding.label(),
            holder.name(),
            action.name()
        );
        return false;
    }
    taken.push((binding, action));
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use slint::SharedString;

    fn overrides(rows: &[(&str, &[&str])]) -> BTreeMap<String, Vec<String>> {
        rows.iter()
            .map(|(name, labels)| {
                (
                    (*name).to_owned(),
                    labels.iter().map(|l| (*l).to_owned()).collect(),
                )
            })
            .collect()
    }

    /// **The one property the whole conflict rule rests on** (spec T1): no key
    /// is two actions', so a coach's first keypress is never ambiguous and the
    /// load rule has nothing to resolve until they edit the file.
    #[test]
    fn the_defaults_are_conflict_free() {
        let keymap = Keymap::defaults();
        let mut seen: Vec<(Binding, Action)> = Vec::new();
        for action in Action::ALL {
            for binding in keymap.bindings(action) {
                if let Some((_, other)) = seen.iter().find(|(b, _)| b == binding) {
                    panic!(
                        "{} is both {}'s and {}'s",
                        binding.label(),
                        other.name(),
                        action.name()
                    );
                }
                seen.push((*binding, action));
            }
        }
        assert_eq!(seen.len(), 34, "spec D1: 29 actions, 34 bindings");
    }

    /// A default that does not parse is a default that silently is not there,
    /// which is the one cost of writing the table as labels.
    #[test]
    fn every_default_label_parses() {
        for action in Action::ALL {
            assert_eq!(
                Keymap::defaults().bindings(action).len(),
                action.default_keys().len(),
                "{}: {:?}",
                action.name(),
                action.default_keys()
            );
        }
    }

    /// `ALL` is the sheet's order and the collision tie-break, so a variant
    /// left out of it is an action with no row and no place in the rule.
    #[test]
    fn the_list_holds_every_action_in_declaration_order() {
        for (i, action) in Action::ALL.into_iter().enumerate() {
            assert_eq!(action as usize, i, "{}", action.name());
        }
    }

    #[test]
    fn every_action_round_trips_through_its_name_and_says_what_it_does() {
        for action in Action::ALL {
            assert_eq!(Action::from_name(action.name()), Some(action));
            assert!(!action.what().is_empty(), "{}", action.name());
            assert!(!action.when().is_empty(), "{}", action.name());
        }
        assert_eq!(Action::from_name("nonesuch"), None);
    }

    /// Every spelling the coach can read in the sheet reads back as the same
    /// binding.
    #[test]
    fn every_default_binding_round_trips_through_its_label() {
        let keymap = Keymap::defaults();
        for action in Action::ALL {
            for binding in keymap.bindings(action) {
                let label = binding.label();
                assert_eq!(
                    Binding::from_label(&label),
                    Some(*binding),
                    "{label} ({})",
                    action.name()
                );
            }
        }
        assert_eq!(
            Keymap::defaults()
                .bindings(Action::Redo)
                .iter()
                .map(|b| b.label())
                .collect::<Vec<_>>(),
            ["ctrl+shift+z", "ctrl+y"],
            "the canonical order is Ctrl+Shift+Alt, then the key"
        );
    }

    #[test]
    fn a_label_that_is_not_a_key_is_none_rather_than_an_error() {
        for label in [
            "",
            "ctrl+",
            "shift",
            "ctrl+shift+",
            "Escapé",
            "ctrl+LeftArrow+a",
            "ab",
            "ctrl+ctrl+z",
            "F13",
        ] {
            assert_eq!(Binding::from_label(label), None, "{label:?}");
        }
    }

    /// **One spelling reaches one key.** A character a named key owns is
    /// spelled by the name, so the space bar has exactly one label.
    #[test]
    fn a_named_keys_character_is_not_also_a_label() {
        assert_eq!(Binding::from_label(" "), None);
        assert_eq!(Binding::from_label("\u{1b}"), None, "Escape's character");
        assert_eq!(
            Binding::from_label("Space").map(|b| b.label()),
            Some("Space".to_owned())
        );
        assert_eq!(
            Binding::from_label("space").map(|b| b.label()),
            Some("Space".to_owned()),
            "a name is read case-insensitively and emitted canonically"
        );
    }

    /// A hand-edit may spell the modifiers in any order and any case; what the
    /// sheet shows is canonical.
    #[test]
    fn a_hand_edited_label_is_read_loosely_and_emitted_canonically() {
        for label in ["Ctrl+Shift+Z", "shift+ctrl+z", "SHIFT+CTRL+Z"] {
            assert_eq!(
                Binding::from_label(label).map(|b| b.label()),
                Some("ctrl+shift+z".to_owned()),
                "{label:?}"
            );
        }
        assert_eq!(
            Binding::from_label("R").map(|b| b.label()),
            Some("r".to_owned()),
            "a character is folded, because the matcher lowercases the event"
        );
    }

    /// **The one failure mode a copied code point would have**, now a unit test
    /// with no window in it: every named key is a different key.
    #[test]
    fn every_named_key_maps_to_a_distinct_character() {
        let mut seen: Vec<(char, NamedKey)> = Vec::new();
        for named in NamedKey::ALL {
            let ch = named.ch();
            if let Some((_, other)) = seen.iter().find(|(c, _)| *c == ch) {
                panic!("{} and {} are the same key", named.name(), other.name());
            }
            seen.push((ch, named));
        }
        assert_eq!(seen.len(), NamedKey::ALL.len());
    }

    #[test]
    fn the_lookup_reads_the_modifiers_exactly() {
        let keymap = Keymap::defaults();
        let at = |text: &str, ctrl, shift, alt| keymap.action_for(text, ctrl, shift, alt);
        assert_eq!(at("r", false, false, false), Some(Action::ToggleRecording));
        assert_eq!(
            at("R", false, false, false),
            Some(Action::ToggleRecording),
            "Caps Lock reports uppercase with shift false, and must still record"
        );
        assert_eq!(
            at("r", false, true, false),
            None,
            "Shift is part of the binding (G2): Shift+R is not R"
        );
        assert_eq!(at("o", true, false, false), Some(Action::OpenProject));
        assert_eq!(
            at("o", false, false, false),
            Some(Action::MarkOut),
            "the branch order that used to carry this is gone (B5)"
        );
        assert_eq!(at("z", true, true, false), Some(Action::Redo));
        assert_eq!(at("y", true, false, false), Some(Action::Redo));
        assert_eq!(at("y", true, true, false), None, "Ctrl+Shift+Y is nothing");
        assert_eq!(at("a", false, true, false), Some(Action::SkipBackFar));
        assert_eq!(
            at(&NamedKey::LeftArrow.ch().to_string(), false, false, false),
            Some(Action::SkipBack),
            "a named key resolves through Slint's own character"
        );
        assert_eq!(at(" ", false, false, false), Some(Action::TogglePlay));
        assert_eq!(at("q", false, false, false), None);
        assert_eq!(at("", false, false, false), None);
    }

    /// Spec G3's load rule, one case per sentence of it.
    #[test]
    fn the_stored_rows_are_applied_over_the_defaults() {
        // An absent action keeps its default; `[]` unbinds.
        let keymap = Keymap::with_overrides(&overrides(&[("scanFaster", &[])]));
        assert!(keymap.bindings(Action::ScanFaster).is_empty());
        assert_eq!(
            keymap.action_for("j", false, false, false),
            Some(Action::ScanSlower)
        );

        // An unknown action name costs that row and nothing beside it.
        let keymap =
            Keymap::with_overrides(&overrides(&[("tagTheRef", &["t"]), ("fitWindow", &["g"])]));
        assert_eq!(
            keymap.action_for("g", false, false, false),
            Some(Action::FitWindow)
        );
        assert_eq!(keymap.action_for("t", false, false, false), None);

        // An unparseable spelling costs that label; its siblings survive.
        let keymap = Keymap::with_overrides(&overrides(&[("fitWindow", &["Fit", "g"])]));
        assert_eq!(
            keymap
                .bindings(Action::FitWindow)
                .iter()
                .map(|b| b.label())
                .collect::<Vec<_>>(),
            ["g"]
        );

        // And an override takes the key from whichever action held it by
        // default: last wins, and the loser is left unbound.
        let keymap = Keymap::with_overrides(&overrides(&[("tagHomeGoal", &["o"])]));
        assert_eq!(
            keymap.action_for("o", false, false, false),
            Some(Action::TagHomeGoal)
        );
        assert!(
            keymap.bindings(Action::MarkOut).is_empty(),
            "markOut lost its only key, which the sheet shows as unbound"
        );
        assert_eq!(
            keymap.bindings(Action::OpenProject),
            Keymap::defaults().bindings(Action::OpenProject),
            "ctrl+o is a different binding and is untouched"
        );
    }

    /// Two stored rows claiming one key: `ALL`'s order decides, and the loser
    /// keeps whichever of its own bindings survive.
    #[test]
    fn a_key_claimed_twice_goes_to_the_action_listed_first() {
        let keymap = Keymap::with_overrides(&overrides(&[
            ("markIn", &["q"]),
            ("fitWindow", &["q", "g"]),
        ]));
        assert_eq!(
            keymap.action_for("q", false, false, false),
            Some(Action::MarkIn)
        );
        assert_eq!(
            keymap
                .bindings(Action::FitWindow)
                .iter()
                .map(|b| b.label())
                .collect::<Vec<_>>(),
            ["g"],
            "it loses the contested key and keeps its own"
        );
    }

    /// **A diff, not a copy**: an untouched table stores nothing, so a later
    /// version that retunes a default reaches the coach.
    #[test]
    fn what_is_stored_is_only_what_differs() {
        assert!(Keymap::defaults().overrides().is_empty());

        let stored = overrides(&[("fitWindow", &["g"])]);
        let keymap = Keymap::with_overrides(&stored);
        assert_eq!(
            keymap.overrides(),
            stored,
            "one changed action is one stored key"
        );
        assert_eq!(
            Keymap::with_overrides(&keymap.overrides()),
            keymap,
            "and reading the diff back is the identity"
        );
    }

    /// The load rule's own result is stable: writing the whole diff out and
    /// reading it back is the same keymap, displaced rows included.
    #[test]
    fn a_displaced_default_survives_the_round_trip() {
        let keymap = Keymap::with_overrides(&overrides(&[("tagHomeGoal", &["o"])]));
        let stored = keymap.overrides();
        assert_eq!(
            stored.get("markOut").map(Vec::as_slice),
            Some(&[] as &[String]),
            "the action that lost the key is part of the diff"
        );
        assert_eq!(Keymap::with_overrides(&stored), keymap);
    }

    #[test]
    fn the_listing_has_a_row_for_every_action_and_says_when_it_is_unbound() {
        let listing = Keymap::defaults().listing();
        assert_eq!(listing.len(), Action::ALL.len());
        for row in &listing {
            assert!(!row.what.is_empty());
            assert!(!row.keys.is_empty());
            assert!(!row.when.is_empty());
        }
        let index = |action: Action| {
            Action::ALL
                .into_iter()
                .position(|a| a == action)
                .expect("in ALL")
        };
        assert_eq!(listing[index(Action::SkipBack)].keys, "LeftArrow, a");
        assert_eq!(listing[index(Action::ShowRecents)].keys, UNBOUND);
        // The two cells the sheet's controls read: the row's identity, and
        // whether there is anything for the ✕ to take away.
        assert_eq!(listing[index(Action::SkipBack)].action, "skipBack");
        assert!(listing[index(Action::SkipBack)].bound);
        assert!(!listing[index(Action::ShowRecents)].bound);
    }

    /// **A key event is a binding, and a bare modifier is not one.** The
    /// capture's whole input, and the one case that would otherwise bind Ctrl
    /// the instant a coach reached for `Ctrl+R`.
    #[test]
    fn a_key_event_reads_back_as_the_binding_it_is() {
        let label = |text: &str, ctrl, shift| {
            Binding::from_event(text, ctrl, shift, false).map(Binding::label)
        };
        assert_eq!(label("r", false, false).as_deref(), Some("r"));
        // Shift reaches the matcher as an upper-case character *and* a flag,
        // and `from_event` has to agree with `matches` about both halves.
        assert_eq!(label("R", false, true).as_deref(), Some("shift+r"));
        assert_eq!(label("z", true, false).as_deref(), Some("ctrl+z"));
        // A named key comes back by name rather than as the private-use code
        // point Slint delivered, which is the one spelling `from_label` reads.
        let arrow = SharedString::from(slint::platform::Key::LeftArrow);
        assert_eq!(
            label(&arrow, false, true).as_deref(),
            Some("shift+LeftArrow")
        );
        assert_eq!(
            Binding::from_label("shift+LeftArrow"),
            Binding::from_event(&arrow, false, true, false),
            "the two ways in agree, or a stored row is not the key pressed"
        );

        for modifier in [
            slint::platform::Key::Shift,
            slint::platform::Key::ShiftR,
            slint::platform::Key::Control,
            slint::platform::Key::ControlR,
            slint::platform::Key::Alt,
            slint::platform::Key::AltGr,
            slint::platform::Key::Meta,
            slint::platform::Key::MetaR,
            slint::platform::Key::CapsLock,
        ] {
            let text = SharedString::from(modifier);
            assert_eq!(
                Binding::from_event(&text, false, false, false),
                None,
                "{modifier:?} on its own is not a key to bind"
            );
        }
        assert_eq!(label("", false, false), None);
        assert_eq!(label("ab", false, false), None, "not one character");
    }

    /// **The five reserved keys are refused a capture**, modifiers or not: the
    /// window tests three of them ahead of the lookup and the other two are the
    /// platform's, so a stored row claiming one would be listed and dead.
    #[test]
    fn the_reserved_keys_refuse_a_capture() {
        for name in ["Escape", "Tab", "Return", "Home", "End"] {
            let bare = Binding::from_label(name).expect("a key");
            assert!(bare.reserved(), "{name} is capturable");
            let shifted = Binding::from_label(&format!("ctrl+shift+{name}")).expect("a key");
            assert!(shifted.reserved(), "ctrl+shift+{name} is capturable");
        }
        // And `Delete` is **not** one of them, which is the reason a capture
        // takes it rather than reading it as "unbind": it is `deleteClip`'s own
        // default, so a coach who moved that row has to be able to move it
        // back. Unbinding is the sheet's ✕ instead.
        for name in ["Delete", "Backspace", "Space", "F1", "r", "1"] {
            assert!(
                !Binding::from_label(name).expect("a key").reserved(),
                "{name} cannot be bound"
            );
        }
    }

    /// **A rebind is last-wins, and it answers who lost** (spec G3): the key
    /// moves, the loser keeps its other keys, and the winner's row becomes the
    /// one key the coach pressed.
    #[test]
    fn a_rebind_takes_the_key_and_says_where_from() {
        let mut keymap = Keymap::defaults();
        let a = Binding::from_label("a").expect("a key");

        assert_eq!(
            keymap.rebind(Action::ShowRecents, a),
            Some(Action::SkipBack),
            "the key was the short skip's and nothing said so"
        );
        assert_eq!(labels(&keymap, Action::ShowRecents), ["a"]);
        assert_eq!(
            labels(&keymap, Action::SkipBack),
            ["LeftArrow"],
            "the displaced action lost one key, not its row"
        );
        assert_eq!(
            keymap.action_for("a", false, false, false),
            Some(Action::ShowRecents),
            "one binding, one action"
        );

        // Replacing, not adding: the row becomes the one key pressed, which is
        // what the sheet's single *Set…* promises.
        let q = Binding::from_label("q").expect("a key");
        assert_eq!(
            keymap.rebind(Action::SkipBack, q),
            None,
            "nothing held q, so nothing was displaced"
        );
        assert_eq!(labels(&keymap, Action::SkipBack), ["q"]);

        // A key an action already holds displaces nobody and leaves the row as
        // that one key.
        assert_eq!(keymap.rebind(Action::SkipBack, q), None);
        assert_eq!(labels(&keymap, Action::SkipBack), ["q"]);
    }

    /// **Unbinding empties the row and nothing else**, and the diff says so —
    /// which is what makes it survive a reload rather than reverting.
    #[test]
    fn unbinding_leaves_the_row_empty_and_stored() {
        let mut keymap = Keymap::defaults();
        keymap.unbind(Action::ToggleRecording);
        assert_eq!(labels(&keymap, Action::ToggleRecording), [] as [String; 0]);
        assert_eq!(
            keymap.action_for("r", false, false, false),
            None,
            "the freed key still fires something"
        );
        assert_eq!(
            keymap.overrides().get("toggleRecording").map(Vec::as_slice),
            Some(&[] as &[String]),
            "an empty row has to be *in* the diff, or the next load restores r"
        );
        assert_eq!(
            Keymap::with_overrides(&keymap.overrides()),
            keymap,
            "and the whole map round-trips"
        );
    }

    /// Every key of one action, by label, for the rebind tests.
    fn labels(keymap: &Keymap, action: Action) -> Vec<String> {
        keymap
            .bindings(action)
            .iter()
            .map(|b| b.label())
            .collect::<Vec<_>>()
    }
}
