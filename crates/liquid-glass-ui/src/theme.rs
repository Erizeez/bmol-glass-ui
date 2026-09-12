//! Semantic colors and glass materials shared by Iced integrations.

use iced::{Color as IcedColor, Theme};
use liquid_glass_scene::{
    APPLE_CORNER_SMOOTHING, Color as GlassColor, GlareStyle, GlassMaterial, GlassShape,
    GlassVariant, ShadowStyle, SquircleParams,
};

/// The one corner model used by the UI library.
///
/// Iced's built-in `Border` can only rasterize circular corners, so controls
/// that are still supplied by Iced use [`Self::radius`] as their fallback
/// border radius. Custom Liquid Glass surfaces and widgets use
/// [`Self::params`] and therefore receive the exact `squircle-rs` continuous
/// curvature path.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiCornerStyle {
    radius: f32,
    smoothing: f32,
}

impl UiCornerStyle {
    pub const GROUP: Self = Self::new(10.0);
    pub const CONTROL: Self = Self::new(8.0);
    pub const PICKER: Self = Self::new(6.0);
    pub const MENU: Self = Self::new(8.0);
    pub const SCROLLBAR: Self = Self::new(8.0);
    pub const ICON_CHIP: Self = Self::new(8.0);
    pub const WINDOW: Self = Self::new(14.0);

    #[must_use]
    pub const fn new(radius: f32) -> Self {
        Self { radius, smoothing: APPLE_CORNER_SMOOTHING }
    }

    #[must_use]
    pub const fn with_smoothing(self, smoothing: f32) -> Self {
        Self { smoothing, ..self }
    }

    #[must_use]
    pub const fn with_radius(self, radius: f32) -> Self {
        Self { radius, ..self }
    }

    #[must_use]
    pub const fn radius(self) -> f32 {
        self.radius
    }

    #[must_use]
    pub const fn smoothing(self) -> f32 {
        self.smoothing
    }

    #[must_use]
    pub fn params(self, width: f32, height: f32) -> SquircleParams {
        SquircleParams::new(width, height, self.radius).with_smoothing(self.smoothing)
    }
}

/// The resolved light or dark tone used to render an application window.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum UiColorScheme {
    Light,
    #[default]
    Dark,
}

impl UiColorScheme {
    /// Resolves an Iced system theme mode to a concrete color scheme.
    #[must_use]
    pub const fn from_mode(mode: iced::theme::Mode) -> Self {
        match mode {
            iced::theme::Mode::Light => Self::Light,
            iced::theme::Mode::Dark | iced::theme::Mode::None => Self::Dark,
        }
    }
}

/// A semantic role whose material can vary with the active color scheme.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GlassRole {
    /// A full-height split-view surface that reveals and heavily blurs the
    /// desktop backdrop.
    Sidebar,
    Toolbar,
    InputField,
    /// Compatibility role for search-specific input fields.
    SearchField,
    FloatingControl,
    /// Floating context menus and popovers with heavy backdrop blur (64pt).
    ContextMenu,
}

/// Iced-side chrome drawn above a compositor-provided glass surface.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlassChrome {
    pub border: GlassColor,
    pub hover_border: GlassColor,
    pub divider: GlassColor,
    pub shadow: GlassColor,
    pub text: GlassColor,
    pub disabled_text: GlassColor,
    pub hover_overlay: GlassColor,
    pub pressed_overlay: GlassColor,
    pub shadow_offset_y: f32,
    pub shadow_blur: f32,
}

/// Semantic colors for regular, non-glass application UI.
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
    /// Neutral pill drawn behind the selected sidebar item.
    pub sidebar_selection: IcedColor,
    pub hover: IcedColor,
    /// The track of an off toggle switch.
    pub control_track_off: IcedColor,
    pub shadow: IcedColor,
}

/// Theme object used by the UI library and compositor adapters.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiTheme {
    scheme: UiColorScheme,
}

impl UiTheme {
    #[must_use]
    pub const fn new(scheme: UiColorScheme) -> Self {
        Self { scheme }
    }

    #[must_use]
    pub const fn light() -> Self {
        Self::new(UiColorScheme::Light)
    }

    #[must_use]
    pub const fn dark() -> Self {
        Self::new(UiColorScheme::Dark)
    }

    #[must_use]
    pub const fn scheme(self) -> UiColorScheme {
        self.scheme
    }

    /// Derives the semantic theme from the Iced theme currently used to draw.
    #[must_use]
    pub fn from_iced(theme: &Theme) -> Self {
        if theme.extended_palette().is_dark { Self::dark() } else { Self::light() }
    }

    /// Returns the matching built-in Iced theme for standard controls.
    #[must_use]
    pub const fn iced_theme(self) -> Theme {
        match self.scheme {
            UiColorScheme::Light => Theme::Light,
            UiColorScheme::Dark => Theme::Dark,
        }
    }

    /// Returns semantic colors modelled after macOS split-view settings windows.
    #[must_use]
    pub fn palette(self) -> UiPalette {
        match self.scheme {
            UiColorScheme::Light => UiPalette {
                window_background: rgba(0.950, 0.950, 0.965, 1.0),
                sidebar_background: rgba(0.900, 0.900, 0.920, 0.72),
                // The light settings content pane and fused right titlebar
                // are sampled as neutral white on macOS.
                content_background: rgba(1.0, 1.0, 1.0, 1.0),
                group_background: rgba(1.0, 1.0, 1.0, 0.94),
                group_border: rgba(0.0, 0.0, 0.0, 0.10),
                separator: rgba(0.0, 0.0, 0.0, 0.10),
                text_primary: rgba(0.08, 0.08, 0.09, 1.0),
                text_secondary: rgba(0.32, 0.32, 0.35, 1.0),
                text_tertiary: rgba(0.0, 0.0, 0.0, 0.26),
                accent: rgba(0.04, 0.42, 0.95, 1.0),
                selection: rgba(0.12, 0.46, 0.95, 0.18),
                sidebar_selection: rgba(0.0, 0.0, 0.0, 0.085),
                hover: rgba(0.0, 0.0, 0.0, 0.055),
                control_track_off: rgba(0.0, 0.0, 0.0, 0.10),
                shadow: rgba(0.0, 0.0, 0.0, 0.16),
            },
            UiColorScheme::Dark => UiPalette {
                window_background: rgba(0.105, 0.105, 0.115, 1.0),
                sidebar_background: rgba(0.145, 0.145, 0.155, 0.72),
                content_background: rgba(0.105, 0.105, 0.115, 1.0),
                group_background: rgba(0.175, 0.175, 0.190, 0.96),
                group_border: rgba(70.0 / 255.0, 70.0 / 255.0, 70.0 / 255.0, 1.0),
                separator: rgba(70.0 / 255.0, 70.0 / 255.0, 70.0 / 255.0, 1.0),
                text_primary: rgba(0.94, 0.94, 0.96, 1.0),
                text_secondary: rgba(0.66, 0.66, 0.69, 1.0),
                text_tertiary: rgba(1.0, 1.0, 1.0, 0.25),
                accent: rgba(0.24, 0.55, 1.0, 1.0),
                selection: rgba(0.20, 0.48, 0.95, 0.30),
                sidebar_selection: rgba(1.0, 1.0, 1.0, 0.12),
                hover: rgba(1.0, 1.0, 1.0, 0.065),
                control_track_off: rgba(1.0, 1.0, 1.0, 0.17),
                shadow: rgba(0.0, 0.0, 0.0, 0.34),
            },
        }
    }

    fn role_optical_params(self, role: GlassRole) -> (f32, GlassColor, f32) {
        match (self.scheme, role) {
            (UiColorScheme::Light, GlassRole::Sidebar) => {
                // Settings uses a very broad, neutral frosted medium here:
                // desktop color survives only as a soft low-frequency tint.
                (48.0, GlassColor::rgba(1.0, 1.0, 1.0, 0.22), 0.68)
            }
            (UiColorScheme::Light, GlassRole::Toolbar) => {
                (8.0, GlassColor::rgba(1.0, 1.0, 1.0, 0.04), 0.04)
            }
            (UiColorScheme::Light, GlassRole::InputField) => {
                (7.0, GlassColor::rgba(1.0, 1.0, 1.0, 0.06), 0.14)
            }
            (UiColorScheme::Light, GlassRole::SearchField) => {
                (5.0, GlassColor::rgba(1.0, 1.0, 1.0, 0.12), 0.24)
            }
            (UiColorScheme::Light, GlassRole::FloatingControl) => {
                // Navigation content is drawn inside this capsule. Keep its
                // interior nearly sharp while retaining the refractive edge.
                (2.0, GlassColor::rgba(1.0, 1.0, 1.0, 0.06), 0.06)
            }
            (UiColorScheme::Light, GlassRole::ContextMenu) => {
                // Apple context menus: calibrated white (255, 255, 255) with 185/255 opacity
                let (r, g, b, a) = bmol_designs::menu_metrics::LIGHT_MENU_BASE_RGBA_F32;
                (64.0, GlassColor::rgba(r, g, b, a), 1.0)
            }
            (UiColorScheme::Dark, GlassRole::Sidebar) => {
                (32.0, GlassColor::rgba(0.12, 0.16, 0.25, 0.14), 0.36)
            }
            (UiColorScheme::Dark, GlassRole::Toolbar) => {
                (8.0, GlassColor::rgba(0.10, 0.13, 0.20, 0.08), 0.025)
            }
            (UiColorScheme::Dark, GlassRole::InputField) => {
                (7.0, GlassColor::rgba(0.14, 0.18, 0.27, 0.09), 0.10)
            }
            (UiColorScheme::Dark, GlassRole::SearchField) => {
                (5.0, GlassColor::rgba(0.14, 0.18, 0.27, 0.15), 0.18)
            }
            (UiColorScheme::Dark, GlassRole::FloatingControl) => {
                (2.0, GlassColor::rgba(0.17, 0.22, 0.34, 0.10), 0.05)
            }
            (UiColorScheme::Dark, GlassRole::ContextMenu) => {
                // Apple context menus: calibrated base (42, 42, 42) with 202/255 opacity
                let (r, g, b, a) = bmol_designs::menu_metrics::DARK_MENU_BASE_RGBA_F32;
                (56.0, GlassColor::rgba(r, g, b, a), 42.0 / 255.0)
            }
        }
    }

    /// Builds a scheme-aware material for a selective glass surface.
    #[must_use]
    pub fn glass_material(self, role: GlassRole) -> GlassMaterial {
        let (blur_radius, tint, whiteness) = self.role_optical_params(role);

        let mut material = GlassMaterial::clear();
        material.variant = GlassVariant::Regular;
        material.blur.radius = blur_radius;
        material.tint = tint;
        material.whiteness = whiteness;
        material.refraction.thickness = 0.20;
        material.refraction.index = 1.40;
        material.refraction.strength = 0.70;
        material.dispersion.strength = 0.07;
        material.fresnel.range = 0.75;
        material.fresnel.hardness = 0.20;
        material.fresnel.strength = 0.20;
        if role == GlassRole::SearchField {
            // Give the floating search field the pronounced system-control
            // treatment: brighter vertical specular rims from the material
            // response, backed by the renderer's analytic SDF shadow pass.
            material.refraction.strength = 0.82;
            material.fresnel.range = 0.60;
            material.fresnel.hardness = 0.20;
            material.fresnel.strength = 0.50;
            material.glare = GlareStyle {
                range: 22.0,
                hardness: 0.14,
                convergence: 0.42,
                opposite_factor: 0.84,
                factor: 0.64,
            };
        }
        if role == GlassRole::FloatingControl {
            // Compact floating chrome uses the same thin clear-coat as the
            // search field, but with slightly less contrast so icon strokes
            // remain dominant over the material edge.
            material.fresnel.range = 0.56;
            material.fresnel.hardness = 0.22;
            material.fresnel.strength = 0.46;
            material.glare = GlareStyle {
                range: 20.0,
                hardness: 0.16,
                convergence: 0.46,
                opposite_factor: 0.84,
                factor: 0.59,
            };
        }
        if role == GlassRole::Sidebar || role == GlassRole::ContextMenu {
            // Broad/heavy frosted surfaces suppress specular glare and refraction
            // distortion so content and typography remain razor sharp.
            material.refraction.strength = 0.0;
            material.dispersion.strength = 0.0;
            material.fresnel.strength = 0.0;
        }
        material.opacity = match role {
            GlassRole::Sidebar => match self.scheme {
                UiColorScheme::Light => 0.88,
                UiColorScheme::Dark => 0.76,
            },
            GlassRole::ContextMenu => match self.scheme {
                UiColorScheme::Light => bmol_designs::menu_metrics::LIGHT_MENU_OPACITY,
                UiColorScheme::Dark => bmol_designs::menu_metrics::DARK_MENU_OPACITY,
            },
            // Keep the neutral layer present, but leave enough of the
            // compositor backdrop visible to read as glass on a transparent
            // desktop surface. The input remains the whitest control; the
            // navigation capsule is lighter and more transparent.
            GlassRole::Toolbar => 0.54,
            GlassRole::InputField => 0.64,
            GlassRole::SearchField | GlassRole::FloatingControl => 0.72,
        };
        material.shadow = match role {
            GlassRole::FloatingControl | GlassRole::ContextMenu => ShadowStyle::elevated(),
            GlassRole::SearchField => ShadowStyle::control(),
            GlassRole::Sidebar | GlassRole::Toolbar | GlassRole::InputField => {
                ShadowStyle::subtle()
            }
        };
        material
    }

    /// Returns the default geometry associated with a semantic glass role.
    #[must_use]
    pub const fn glass_shape(self, role: GlassRole) -> GlassShape {
        match role {
            GlassRole::Sidebar | GlassRole::Toolbar => GlassShape::RoundedRect { radius: 0.0 },
            GlassRole::ContextMenu => GlassShape::RoundedRect {
                radius: bmol_designs::menu_metrics::CONTAINER_CORNER_RADIUS,
            },
            GlassRole::InputField | GlassRole::SearchField | GlassRole::FloatingControl => {
                GlassShape::Capsule
            }
        }
    }

    /// Builds the Iced-side border, text, state overlay, and shadow colors.
    #[must_use]
    pub fn glass_chrome(self, role: GlassRole) -> GlassChrome {
        let (
            border,
            hover_border,
            divider,
            shadow,
            text,
            disabled_text,
            hover_overlay,
            pressed_overlay,
        ) = match self.scheme {
            UiColorScheme::Light => (
                GlassColor::rgba(0.0, 0.0, 0.0, 0.10),
                GlassColor::rgba(0.0, 0.0, 0.0, 0.20),
                GlassColor::rgba(0.0, 0.0, 0.0, 0.14),
                GlassColor::rgba(0.0, 0.0, 0.0, 0.14),
                GlassColor::rgba(0.08, 0.08, 0.09, 1.0),
                GlassColor::rgba(0.08, 0.08, 0.09, 0.34),
                GlassColor::rgba(0.0, 0.0, 0.0, 0.04),
                GlassColor::rgba(0.0, 0.0, 0.0, 0.09),
            ),
            UiColorScheme::Dark => (
                GlassColor::rgba(1.0, 1.0, 1.0, 0.16),
                GlassColor::rgba(1.0, 1.0, 1.0, 0.30),
                GlassColor::rgba(1.0, 1.0, 1.0, 0.18),
                GlassColor::rgba(0.0, 0.0, 0.0, 0.30),
                GlassColor::rgba(0.95, 0.95, 0.97, 1.0),
                GlassColor::rgba(0.95, 0.95, 0.97, 0.34),
                GlassColor::rgba(1.0, 1.0, 1.0, 0.055),
                GlassColor::rgba(1.0, 1.0, 1.0, 0.11),
            ),
        };
        let (shadow_offset_y, shadow_blur) = match role {
            GlassRole::Sidebar => (1.0, 14.0),
            GlassRole::Toolbar => (2.0, 10.0),
            GlassRole::InputField => (3.0, 9.0),
            GlassRole::SearchField => (3.0, 12.0),
            GlassRole::FloatingControl => (4.0, 10.0),
            GlassRole::ContextMenu => (8.0, 28.0),
        };
        GlassChrome {
            border,
            hover_border,
            divider,
            shadow,
            text,
            disabled_text,
            hover_overlay,
            pressed_overlay,
            shadow_offset_y,
            shadow_blur,
        }
    }

    /// Returns chrome for an Iced widget layered over a real shader surface.
    /// The shader owns the edge highlight and shadow, so the overlay only
    /// keeps text and interaction-state fills.
    #[must_use]
    pub fn compositor_chrome(self, role: GlassRole) -> GlassChrome {
        let mut chrome = self.glass_chrome(role);
        chrome.border = GlassColor::transparent();
        chrome.hover_border = GlassColor::transparent();
        chrome.shadow = GlassColor::transparent();
        if role == GlassRole::Toolbar {
            chrome.hover_overlay = GlassColor::transparent();
            chrome.pressed_overlay = GlassColor::transparent();
        }
        chrome
    }
}

impl GlassChrome {
    #[must_use]
    pub const fn transparent() -> Self {
        Self {
            border: GlassColor::transparent(),
            hover_border: GlassColor::transparent(),
            divider: GlassColor::transparent(),
            shadow: GlassColor::transparent(),
            shadow_offset_y: 0.0,
            shadow_blur: 0.0,
            text: GlassColor::transparent(),
            disabled_text: GlassColor::transparent(),
            hover_overlay: GlassColor::transparent(),
            pressed_overlay: GlassColor::transparent(),
        }
    }
}

impl Default for GlassChrome {
    fn default() -> Self {
        UiTheme::dark().glass_chrome(GlassRole::FloatingControl)
    }
}

fn rgba(red: f32, green: f32, blue: f32, alpha: f32) -> IcedColor {
    IcedColor::from_rgba(red, green, blue, alpha)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_and_dark_palettes_have_opposite_tones() {
        let light = UiTheme::light().palette();
        let dark = UiTheme::dark().palette();

        assert!(light.content_background.r > dark.content_background.r);
        assert!(light.text_primary.r < dark.text_primary.r);
    }

    #[test]
    fn glass_roles_use_different_blur_radii() {
        let theme = UiTheme::dark();
        let input = theme.glass_material(GlassRole::InputField);
        let button = theme.glass_material(GlassRole::FloatingControl);

        assert!(theme.glass_material(GlassRole::Toolbar).blur.radius > button.blur.radius);
        assert!(input.blur.radius > button.blur.radius);
        assert!(input.whiteness > button.whiteness);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn sidebar_is_whiter_and_more_blurred_than_toolbar() {
        let theme = UiTheme::light();
        let sidebar = theme.glass_material(GlassRole::Sidebar);
        let toolbar = theme.glass_material(GlassRole::Toolbar);

        assert!(sidebar.blur.radius > toolbar.blur.radius);
        assert!(sidebar.whiteness > toolbar.whiteness);
        assert!(sidebar.opacity > toolbar.opacity);
        assert!(sidebar.shadow.factor > 0.0);
        assert_eq!(sidebar.refraction.strength, 0.0);
        assert_eq!(sidebar.dispersion.strength, 0.0);
        assert_eq!(sidebar.fresnel.strength, 0.0);
    }

    #[test]
    fn input_fields_default_to_continuous_capsules() {
        let theme = UiTheme::light();

        assert_eq!(theme.glass_shape(GlassRole::InputField), GlassShape::Capsule);
    }

    #[test]
    fn search_field_uses_material_edge_light_and_sdf_shadow() {
        let theme = UiTheme::light();
        let search = theme.glass_material(GlassRole::SearchField);
        let input = theme.glass_material(GlassRole::InputField);

        assert!(search.shadow.factor > 0.0);
        assert!(search.fresnel.strength > input.fresnel.strength);
        assert!(search.glare.factor > input.glare.factor);
    }

    #[test]
    fn corner_styles_share_the_apple_squircle_smoothing() {
        let style = UiCornerStyle::CONTROL.with_radius(12.0);
        let params = style.params(120.0, 40.0);

        assert!((params.radius() - 12.0).abs() < f32::EPSILON);
        assert!((params.smoothing - APPLE_CORNER_SMOOTHING).abs() < f32::EPSILON);
    }

    #[test]
    fn compositor_chrome_leaves_edges_to_the_shader() {
        let chrome = UiTheme::light().compositor_chrome(GlassRole::SearchField);

        assert!(chrome.border.a.abs() < f32::EPSILON);
        assert!(chrome.hover_border.a.abs() < f32::EPSILON);
        assert!(chrome.shadow.a.abs() < f32::EPSILON);
    }

    #[test]
    fn disabled_glass_content_is_visually_muted() {
        let light = UiTheme::light().glass_chrome(GlassRole::FloatingControl);
        let dark = UiTheme::dark().glass_chrome(GlassRole::FloatingControl);

        assert!(light.disabled_text.a < light.text.a);
        assert!(dark.disabled_text.a < dark.text.a);
    }
}
