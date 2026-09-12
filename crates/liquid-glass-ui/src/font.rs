//! UI font loading and the typography scale shared by all components.
//!
//! Fonts come from three sources, in priority order:
//!
//! 1. Files embedded at build time from `assets/fonts/` (see `build.rs`) —
//!    this is where an SF Pro download belongs when the app targets platforms
//!    without it.
//! 2. On macOS, the system UI font read at runtime (`SFNS.ttf`), whose family
//!    name is the obfuscated `"System Font"`.
//! 3. Iced's built-in default font.

use std::{borrow::Cow, sync::OnceLock};

use iced::{
    Font,
    font::{Family, Weight},
};

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/embedded_fonts.rs"));
}

/// The UI font payload an application should load at startup.
#[derive(Debug, Default)]
pub struct UiFonts {
    /// Font bytes to hand to `iced::font::load` / `Application::font`.
    pub bytes: Vec<Cow<'static, [u8]>>,
    /// The family name widgets should request, when a UI font was found.
    pub family: Option<&'static str>,
}

impl UiFonts {
    /// The [`Font`] widgets should use for interface text.
    #[must_use]
    pub fn font(&self) -> Option<Font> {
        self.family.map(|family| Font { family: Family::Name(family), ..Font::DEFAULT })
    }
}

/// Resolves the UI fonts for the current build and platform.
///
/// The result is cached: runtime fallbacks read font files from disk, and
/// widget code calls this per frame.
pub fn ui_fonts() -> &'static UiFonts {
    static FONTS: OnceLock<UiFonts> = OnceLock::new();
    FONTS.get_or_init(|| {
        if !embedded::EMBEDDED_FONTS.is_empty() {
            return UiFonts {
                bytes: embedded::EMBEDDED_FONTS
                    .iter()
                    .map(|(_, bytes)| Cow::Borrowed(*bytes))
                    .collect(),
                family: Some(embedded::EMBEDDED_FONTS[0].0),
            };
        }

        #[cfg(target_os = "macos")]
        if let Ok(bytes) = std::fs::read("/System/Library/Fonts/SFNS.ttf") {
            return UiFonts { bytes: vec![Cow::Owned(bytes)], family: Some("System Font") };
        }

        UiFonts::default()
    })
}

/// Returns the interface font with the given weight, falling back to Iced's
/// default family when no UI font is available.
#[must_use]
pub fn ui_font(weight: Weight) -> Font {
    let family = ui_fonts().family.map_or(Font::DEFAULT.family, Family::Name);
    Font { family, weight, ..Font::DEFAULT }
}

/// The font used by fused titlebars and toolbar titles.
///
/// This is intentionally a named preset instead of letting each application
/// combine a size and weight ad hoc. macOS toolbar titles need more optical
/// weight than ordinary body copy, especially over translucent surfaces.
#[must_use]
pub fn toolbar_title_font() -> Font {
    ui_font(Weight::Bold)
}

/// Interface text sizes following macOS settings typography.
pub mod size {
    /// Large in-page section titles ("General").
    pub const TITLE: f32 = 20.0;
    /// Prominent labels such as the toolbar title.
    pub const HEADLINE: f32 = 15.0;
    /// Fused titlebar/toolbar title. Kept separate so platform chrome can be
    /// calibrated without changing ordinary headline text.
    pub const TOOLBAR_TITLE: f32 = 15.0;
    /// Standard control and row labels.
    pub const BODY: f32 = 13.0;
    /// Supporting detail text under a row label.
    pub const CAPTION: f32 = 11.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_a_family_on_macos() {
        #[cfg(target_os = "macos")]
        assert!(ui_fonts().family.is_some());
    }

    #[test]
    fn typography_scale_decreases() {
        const {
            assert!(size::TITLE > size::HEADLINE);
            assert!(size::HEADLINE > size::BODY);
            assert!(size::BODY > size::CAPTION);
        }
    }
}
