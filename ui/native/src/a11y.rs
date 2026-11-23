//! Pure mapping from persisted accessibility values to native presentation.
//!
//! Host evidence only: this module neither reads nor writes the settings slot
//! and grants no capability. It takes primitive values so that `ui/native`
//! keeps its audited, dependency-light graph. The app adapter (later
//! sdk/examples/materiaal or the settings service) passes the values from
//! `nieuw_os_settings_core::Settings`, which already rejects out-of-range text
//! scales before they get here. This mapping still fails closed on its own.
//!
//! Text size: native renders only [`Scale::Percent100`] and
//! [`Scale::Percent200`]. A text scale from 150 % to 300 % maps to 200 %;
//! 75 %-149 % maps to 100 %. 150 % is the midpoint between the two rendered
//! steps, so each request lands on the nearer step and a request for at least
//! 150 % never renders at half its requested size. A value outside the
//! settings-core range 75 %-300 % is invalid and fails closed to the 100 %
//! default. This is a fail-closed fallback, not conformance with the
//! text-size axis.
//!
//! Open items (steps defined by docs/design/accessibility/ACCESSIBILITY-MATRIX.md
//! that native does not render yet):
//! - 125 %: falls back to 100 % (smaller than requested).
//! - 150 %: falls back to 200 % (larger than requested).
//! - Larger than 200 % (settings-core allows up to 300 %): falls back to 200 %
//!   (smaller than requested).
//!
//! Settings-core also accepts 75 %-99 %, which the matrix does not define as a
//! step; these render at 100 %, never smaller than the default.
//!
//! Contrast: the matrix defines dark, light and contrast themes. Settings-core
//! has no contrast field, so contrast stays a session toggle and is not mapped
//! here; persistence remains an open item pending a project-owner decision.
//! The theme value is likewise not mapped here.
//!
//! Reduced motion: native currently renders no animation, so the flag is
//! passed through unchanged for consumers; it changes no pixels today.

use crate::Scale;

/// Smallest valid text scale (mirrors settings-core `MIN_TEXT_SCALE_PERCENT`).
pub const MIN_TEXT_SCALE_PERCENT: u16 = 75;
/// Largest valid text scale (mirrors settings-core `MAX_TEXT_SCALE_PERCENT`).
pub const MAX_TEXT_SCALE_PERCENT: u16 = 300;
/// Smallest text scale that renders at 200 %.
pub const PERCENT200_THRESHOLD: u16 = 150;

/// Native presentation inputs derived from persisted settings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Presentation {
    pub scale: Scale,
    pub reduced_motion: bool,
}

/// Map a persisted text-scale percentage to the nearest rendered native scale;
/// invalid values fail closed to 100 %.
pub const fn scale_for_text_percent(percent: u16) -> Scale {
    if percent >= PERCENT200_THRESHOLD && percent <= MAX_TEXT_SCALE_PERCENT {
        Scale::Percent200
    } else {
        Scale::Percent100
    }
}

/// Map persisted accessibility values to native presentation inputs.
pub const fn presentation(text_scale_percent: u16, reduced_motion: bool) -> Presentation {
    Presentation {
        scale: scale_for_text_percent(text_scale_percent),
        reduced_motion,
    }
}
