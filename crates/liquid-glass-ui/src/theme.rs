//! Iced-facing view of the design system's theme.
//!
//! The values live in `bmol-designs`, which carries no toolkit dependency. This
//! module is the single conversion point between them: the toolkit-free types are
//! re-exported unchanged, and only the palette is re-typed in Iced's colour so
//! widgets can use it directly.
//!
//! Converting per access site was measured and rejected: the palette is reached
//! through six `palette()` calls, but its fields are read seventy-two times. One
//! conversion here costs fifteen lines and leaves every access site alone.

use iced::{Color as IcedColor, Theme};
use liquid_glass_scene::{GlassMaterial, GlassShape};

pub use bmol_designs::{ClarityPolicy, GlassChrome, GlassRole, UiColorScheme, UiCornerStyle, to_iced};

/// The semantic palette in Iced's colour type.
///
/// Widgets hand these straight to `Background::Color`, `Border`, `Shadow` and
/// `Color::scale_alpha`, so the conversion happens once at construction instead
/// of at every read.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiPalette {
    pub window_background: IcedColor,
    pub sidebar_background: IcedColor,
    pub content_background: IcedColor,
    pub group_background: IcedColor,
    pub group_border: IcedColor,
    pub separator: IcedColor,
    pub text_primary: IcedColor,
    pub text_secondary: IcedColor,
    /// De-emphasized text such as disclosure chevrons and placeholders.
    pub text_tertiary: IcedColor,
    pub accent: IcedColor,
    /// Accent-tinted selection used inside lists and text fields.
    pub selection: IcedColor,
    pub sidebar_selection: IcedColor,
    pub hover: IcedColor,
    pub control_track_off: IcedColor,
    pub shadow: IcedColor,
}

impl From<bmol_designs::UiPalette> for UiPalette {
    fn from(palette: bmol_designs::UiPalette) -> Self {
        Self {
            window_background: to_iced(palette.window_background),
            sidebar_background: to_iced(palette.sidebar_background),
            content_background: to_iced(palette.content_background),
            group_background: to_iced(palette.group_background),
            group_border: to_iced(palette.group_border),
            separator: to_iced(palette.separator),
            text_primary: to_iced(palette.text_primary),
            text_secondary: to_iced(palette.text_secondary),
            text_tertiary: to_iced(palette.text_tertiary),
            accent: to_iced(palette.accent),
            selection: to_iced(palette.selection),
            sidebar_selection: to_iced(palette.sidebar_selection),
            hover: to_iced(palette.hover),
            control_track_off: to_iced(palette.control_track_off),
            shadow: to_iced(palette.shadow),
        }
    }
}

/// The design system's theme, with the palette in Iced's colour type.
///
/// Everything but [`Self::palette`] is a straight delegation to
/// [`bmol_designs::UiTheme`], so this type adds no policy of its own.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiTheme(bmol_designs::UiTheme);

impl UiTheme {
    #[must_use]
    pub const fn new(scheme: UiColorScheme) -> Self {
        Self(bmol_designs::UiTheme::new(scheme))
    }

    #[must_use]
    pub const fn light() -> Self {
        Self(bmol_designs::UiTheme::light())
    }

    #[must_use]
    pub const fn dark() -> Self {
        Self(bmol_designs::UiTheme::dark())
    }

    /// Derives the semantic theme from the Iced theme currently used to draw.
    #[must_use]
    pub fn from_iced(theme: &Theme) -> Self {
        Self(bmol_designs::UiTheme::from_iced(theme))
    }

    #[must_use]
    pub const fn scheme(self) -> UiColorScheme {
        self.0.scheme()
    }

    /// Returns the matching built-in Iced theme for standard controls.
    #[must_use]
    pub const fn iced_theme(self) -> Theme {
        self.0.iced_theme()
    }

    /// Returns semantic colours modelled after macOS split-view settings windows.
    #[must_use]
    pub fn palette(self) -> UiPalette {
        self.0.palette().into()
    }

    #[must_use]
    pub fn glass_material(self, role: GlassRole) -> GlassMaterial {
        self.0.glass_material(role)
    }

    #[must_use]
    pub const fn glass_shape(self, role: GlassRole) -> GlassShape {
        self.0.glass_shape(role)
    }

    #[must_use]
    pub fn glass_chrome(self, role: GlassRole) -> GlassChrome {
        self.0.glass_chrome(role)
    }

    #[must_use]
    pub fn compositor_chrome(self, role: GlassRole) -> GlassChrome {
        self.0.compositor_chrome(role)
    }
}

impl Default for UiTheme {
    fn default() -> Self {
        Self(bmol_designs::UiTheme::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The adapter exists to change the colour *type*, not the values. Every
    /// palette field must arrive with identical channels, for both schemes,
    /// because the seventy-two access sites downstream now read through it.
    #[test]
    fn the_palette_adapter_preserves_every_channel() {
        for source in [bmol_designs::UiTheme::light().palette(), bmol_designs::UiTheme::dark().palette()] {
            let adapted: UiPalette = source.into();
            let pairs = [
                (adapted.window_background, source.window_background),
                (adapted.sidebar_background, source.sidebar_background),
                (adapted.content_background, source.content_background),
                (adapted.group_background, source.group_background),
                (adapted.group_border, source.group_border),
                (adapted.separator, source.separator),
                (adapted.text_primary, source.text_primary),
                (adapted.text_secondary, source.text_secondary),
                (adapted.text_tertiary, source.text_tertiary),
                (adapted.accent, source.accent),
                (adapted.selection, source.selection),
                (adapted.sidebar_selection, source.sidebar_selection),
                (adapted.hover, source.hover),
                (adapted.control_track_off, source.control_track_off),
                (adapted.shadow, source.shadow),
            ];
            for (iced, scene) in pairs {
                assert_eq!(
                    (iced.r, iced.g, iced.b, iced.a),
                    (scene.r, scene.g, scene.b, scene.a)
                );
            }
        }
    }

    /// `UiTheme` adds no policy of its own; it must answer exactly what the
    /// design system answers, or the two would drift apart silently.
    #[test]
    fn the_theme_delegates_to_the_design_system() {
        assert_eq!(UiTheme::light().scheme(), UiColorScheme::Light);
        assert_eq!(UiTheme::dark().scheme(), UiColorScheme::Dark);
        assert_eq!(UiTheme::default().scheme(), bmol_designs::UiTheme::default().scheme());

        for role in [GlassRole::Sidebar, GlassRole::Toolbar, GlassRole::ContextMenu] {
            assert_eq!(
                UiTheme::dark().glass_shape(role),
                bmol_designs::UiTheme::dark().glass_shape(role)
            );
            assert_eq!(
                UiTheme::dark().glass_chrome(role),
                bmol_designs::UiTheme::dark().glass_chrome(role)
            );
            assert_eq!(
                UiTheme::light().glass_material(role),
                bmol_designs::UiTheme::light().glass_material(role)
            );
            assert_eq!(
                UiTheme::light().compositor_chrome(role),
                bmol_designs::UiTheme::light().compositor_chrome(role)
            );
        }
    }
}
