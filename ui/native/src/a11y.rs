//! Pure mapping from persisted accessibility settings to native presentation.
//!
//! Host evidence only: this module neither reads nor writes the settings slot
//! and grants no capability. It translates an already validated
//! [`nieuw_os_settings_core::Settings`] value into the presentation inputs the
//! native renderer already supports.
//!
//! Text size: native renders only [`Scale::Percent100`] and
//! [`Scale::Percent200`]. A persisted text scale at or above 150 % maps to
//! 200 %, everything below to 100 %. 150 % is the midpoint between the two
//! rendered steps, so each request lands on the nearer step and a request for
//! at least 150 % never renders at half its requested size. This is a
//! fail-closed fallback, not conformance with the text-size axis.
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
use nieuw_os_settings_core::Settings;

/// Smallest persisted text scale that renders at 200 %.
pub const PERCENT200_THRESHOLD: u16 = 150;

/// Native presentation inputs derived from persisted settings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Presentation {
    pub scale: Scale,
    pub reduced_motion: bool,
}

/// Map a persisted text-scale percentage to the nearest rendered native scale.
pub const fn scale_for_text_percent(percent: u16) -> Scale {
    if percent >= PERCENT200_THRESHOLD {
        Scale::Percent200
    } else {
        Scale::Percent100
    }
}

/// Map validated settings to native presentation inputs.
pub const fn presentation(settings: Settings) -> Presentation {
    Presentation {
        scale: scale_for_text_percent(settings.text_scale_percent()),
        reduced_motion: settings.reduced_motion(),
    }
}
