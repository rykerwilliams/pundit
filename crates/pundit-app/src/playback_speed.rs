//! Words a recorded playback rate for the inspector's review line (#140 task 7).
//!
//! **A lib module because `main.rs` is not testable** (the harness has no
//! window and a UI test links the library) — the `export_outcome`/`fit` pattern.
//! It exists so the rungs this names are **pinned by a test**: the slow-motion
//! ladder `1 → 0.75 → 0.5 → 0.25` is also tabulated in `app.slint`'s
//! `next-record-rate` (the cycle, pinned by
//! `record_rate::cycling_the_control_steps_the_recorded_rate`), and the two must
//! list the same rungs or a speed cycles to a value this shows as a bare
//! percentage. `the_labels_match_the_ladder` below pins this half; a fifth rung
//! added to one and not the other fails one of the two.

/// The named fractions match the speed control's ladder; anything off it — only
/// reachable from a hand-edited project — is shown as a percentage.
pub fn label(rate: f64) -> String {
    match rate {
        r if (r - 0.75).abs() < 1e-6 => "Three-quarter speed (0.75×)".to_owned(),
        r if (r - 0.5).abs() < 1e-6 => "Half speed (0.5×)".to_owned(),
        r if (r - 0.25).abs() < 1e-6 => "Quarter speed (0.25×)".to_owned(),
        r => format!("{:.0}% speed", r * 100.0),
    }
}

#[cfg(test)]
mod tests {
    use super::label;

    /// Every sub-unit rung of `app.slint`'s `next-record-rate` ladder gets a
    /// named label (not a percentage). If the ladder grows a rung this list
    /// must too, which is the whole point of pinning it here.
    #[test]
    fn the_labels_match_the_ladder() {
        for (rate, words) in [
            (0.75, "Three-quarter speed (0.75×)"),
            (0.5, "Half speed (0.5×)"),
            (0.25, "Quarter speed (0.25×)"),
        ] {
            assert_eq!(
                label(rate),
                words,
                "the ladder rung {rate}× has no named label"
            );
        }
    }

    /// A rate off the ladder (only a hand-edited project reaches it) is a
    /// percentage, never a panic or a wrong rung.
    #[test]
    fn an_off_ladder_rate_is_a_percentage() {
        assert_eq!(label(0.6), "60% speed");
        assert_eq!(label(1.0), "100% speed");
    }
}
