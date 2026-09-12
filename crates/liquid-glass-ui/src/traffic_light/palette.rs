//! Apple colourimetry, vector glyphs, and the physical sphere palettes.
//!
//! Everything here is pure data and pure functions: no Iced, no GPU, no global
//! state, so it can be asserted in ordinary unit tests.

use crate::traffic_light::action::WindowControlAction;
use crate::traffic_light::layout::WINDOW_CONTROL_NATIVE_SIZE;
use liquid_glass_scene::Color;

// Authentic Apple Text Glyphs matching original high-clarity typography
pub const GLYPH_CLOSE: &str = "✕";
pub const GLYPH_MINIMIZE: &str = "─";
pub const GLYPH_ZOOM: &str = "⤢";
pub const GLYPH_EXPAND: &str = "⤢";
pub const GLYPH_MAXIMIZE: &str = "+";

// Authentic Apple Vector SVGs with bold stroke and dynamic currentColor fill/stroke
pub const SVG_CLOSE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2.6" d="M1.5 1.5 8.5 8.5M8.5 1.5 1.5 8.5"/></svg>"#;
pub const SVG_MINIMIZE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-width="2.6" d="M1.25 5h7.5"/></svg>"#;
pub const SVG_ZOOM: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="-0.250 114.188 46.031 46.062"><path fill="currentColor" fill-rule="nonzero" d="M 0.000 122.313 L 0.000 146.688 C 0.000 149.063 2.312 149.875 3.750 148.438 L 34.000 118.188 C 35.406 116.781 34.594 114.438 32.250 114.438 L 7.969 114.438 C 2.656 114.438 0.000 117.063 0.000 122.313 Z M 37.687 160.000 C 42.937 160.000 45.531 157.313 45.531 152.031 L 45.531 127.750 C 45.531 125.375 43.219 124.563 41.781 126.000 L 11.531 156.250 C 10.125 157.656 10.937 160.000 13.281 160.000 Z"/></svg>"#;
pub const SVG_MAXIMIZE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-width="2.2" d="M5 1.5v7M1.5 5h7"/></svg>"#;

/// Authentic Apple Specular Crescent Arc and Subsurface Sheen SVG.
///
/// Simulates 3D spherical lens curvature, Fresnel top reflection (incident 47°
/// angle), and subsurface translucent light collection. Only used when the
/// widget draws its own sphere (no GPU compositor passthrough).
pub const SVG_SPECULAR_ARC: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 14 14" fill="none">
  <defs>
    <linearGradient id="specGrad" x1="7" y1="0.4" x2="7" y2="3.2" gradientUnits="userSpaceOnUse">
      <stop offset="0%" stop-color="#FFFFFF" stop-opacity="0.95"/>
      <stop offset="55%" stop-color="#FFFFFF" stop-opacity="0.45"/>
      <stop offset="100%" stop-color="#FFFFFF" stop-opacity="0.0"/>
    </linearGradient>
    <linearGradient id="bottomRimGrad" x1="7" y1="13.5" x2="7" y2="11.2" gradientUnits="userSpaceOnUse">
      <stop offset="0%" stop-color="#FFFFFF" stop-opacity="0.30"/>
      <stop offset="100%" stop-color="#FFFFFF" stop-opacity="0.0"/>
    </linearGradient>
  </defs>
  <!-- Authentic Top Edge Specular Crescent Highlight Arc: snugly fits the upper curvature -->
  <path d="M 2.4 3.0 C 3.4 1.1 10.6 1.1 11.6 3.0 C 10.2 1.8 3.8 1.8 2.4 3.0 Z" fill="url(#specGrad)"/>
  <!-- Subsurface Optical Lens Bottom Rim Reflection -->
  <path d="M 3.8 11.8 C 4.8 13.1 9.2 13.1 10.2 11.8 C 9.0 12.5 5.0 12.5 3.8 11.8 Z" fill="url(#bottomRimGrad)"/>
</svg>"##;

/// Builds an opaque sRGB colour from 8-bit channels.
///
/// `liquid_glass_scene::Color` only exposes normalised constructors, so the
/// calibrated palettes below keep their byte-precise Apple values through this
/// single conversion point.
#[must_use]
pub const fn rgb8(r: u8, g: u8, b: u8) -> Color {
    Color::rgba(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
}

/// Returns the matching Apple vector SVG source for the action.
#[must_use]
pub const fn window_control_icon(
    action: WindowControlAction,
    expand_behavior: crate::traffic_light::action::WindowExpandBehavior,
) -> &'static str {
    match action {
        WindowControlAction::Close => SVG_CLOSE,
        WindowControlAction::Minimize => SVG_MINIMIZE,
        WindowControlAction::Zoom | WindowControlAction::Expand => {
            if expand_behavior.is_fullscreen() { SVG_ZOOM } else { SVG_MAXIMIZE }
        }
    }
}

/// Linearly blends two RGBA tuples.
///
/// Version-free so a compositor on any `liquid-glass-scene` release blends
/// identically; [`blend_color`] is the typed convenience wrapper.
#[must_use]
pub fn blend_rgba(from: [f32; 4], to: [f32; 4], amount: f32) -> [f32; 4] {
    let amount = amount.clamp(0.0, 1.0);
    [
        from[0] + (to[0] - from[0]) * amount,
        from[1] + (to[1] - from[1]) * amount,
        from[2] + (to[2] - from[2]) * amount,
        from[3] + (to[3] - from[3]) * amount,
    ]
}

/// Helper to blend two colors linearly.
#[must_use]
pub fn blend_color(from: Color, to: Color, amount: f32) -> Color {
    let [r, g, b, a] = blend_rgba([from.r, from.g, from.b, from.a], [to.r, to.g, to.b, to.a], amount);
    Color::rgba(r, g, b, a)
}

/// Glyph size inside a control circle of `size`.
#[must_use]
pub fn window_control_glyph_size(action: WindowControlAction, size: f32) -> f32 {
    let factor = if size <= WINDOW_CONTROL_NATIVE_SIZE {
        match action {
            WindowControlAction::Close => 7.0 / WINDOW_CONTROL_NATIVE_SIZE,
            WindowControlAction::Minimize => 8.0 / WINDOW_CONTROL_NATIVE_SIZE,
            WindowControlAction::Zoom | WindowControlAction::Expand => 0.42,
        }
    } else {
        0.42
    };
    (size * factor).max(4.0)
}

/// Color for the control glyph based on action, is_dark, inactive state, and focus amount.
#[must_use]
pub fn window_control_glyph_color(
    is_dark: bool,
    action: WindowControlAction,
    inactive: bool,
    focus_amount: f32,
) -> Color {
    let focus_amount = focus_amount.clamp(0.0, 1.0);
    let active = match action {
        WindowControlAction::Close => rgb8(0x69, 0x09, 0x05),
        WindowControlAction::Minimize => rgb8(0x7A, 0x50, 0x00),
        WindowControlAction::Zoom | WindowControlAction::Expand => rgb8(0x00, 0x55, 0x00),
    };
    let alpha_factor = if is_dark { 0.85 } else { 0.75 };
    if inactive {
        let inactive_color = if is_dark {
            Color::rgba(0.82, 0.83, 0.86, 0.76)
        } else {
            Color::rgba(0.52, 0.53, 0.56, 0.78)
        };
        let blended = blend_color(inactive_color, active, focus_amount);
        Color::rgba(blended.r, blended.g, blended.b, focus_amount * alpha_factor)
    } else {
        Color::rgba(active.r, active.g, active.b, focus_amount * alpha_factor)
    }
}

/// Palette defining the 3D physical liquid-glass appearance for a window control button.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrafficLightSpherePalette {
    pub top_color: Color,
    pub bottom_color: Color,
    pub border_color: Color,
    pub glow_color: Color,
}

/// Blends two sphere palettes linearly by `amount`.
#[must_use]
pub fn blend_sphere_palette(
    from: TrafficLightSpherePalette,
    to: TrafficLightSpherePalette,
    amount: f32,
) -> TrafficLightSpherePalette {
    let amount = amount.clamp(0.0, 1.0);
    TrafficLightSpherePalette {
        top_color: blend_color(from.top_color, to.top_color, amount),
        bottom_color: blend_color(from.bottom_color, to.bottom_color, amount),
        border_color: blend_color(from.border_color, to.border_color, amount),
        glow_color: blend_color(from.glow_color, to.glow_color, amount),
    }
}

/// Resolves the set of calibrated source colours for one control index.
///
/// These are deliberately saturated: the glass body contributes a neutral
/// substrate and transmission, so a palette matched only to the displayed
/// centre sample would look washed out. `press` deepens the pigment; hover only
/// drives the inactive→active transition, never a per-control highlight.
#[must_use]
pub fn traffic_light_source_colors(index: usize, _press: f32) -> (Color, Color) {
    let (base, pressed) = match index {
        0 => ((0.98, 0.34, 0.30), (1.00, 0.56, 0.50)),
        1 => ((0.98, 0.72, 0.14), (1.00, 0.84, 0.38)),
        _ => ((0.16, 0.77, 0.22), (0.34, 0.92, 0.42)),
    };
    (
        Color::rgba(base.0, base.1, base.2, 0.96),
        Color::rgba(pressed.0, pressed.1, pressed.2, 0.96),
    )
}

/// Resolves physical liquid-glass sphere palette for an active button.
#[must_use]
pub fn resolve_active_sphere_palette(
    action: WindowControlAction,
    is_dark: bool,
    is_pressed: bool,
    _is_hovered: bool,
) -> TrafficLightSpherePalette {
    match action {
        WindowControlAction::Close => {
            if is_pressed {
                TrafficLightSpherePalette {
                    top_color: rgb8(0xFF, 0x88, 0x80),
                    bottom_color: rgb8(0xFF, 0x9E, 0x96),
                    border_color: rgb8(0xDB, 0x35, 0x2C),
                    glow_color: Color::rgba(1.0, 0.35, 0.30, 0.38),
                }
            } else {
                TrafficLightSpherePalette {
                    top_color: rgb8(0xFE, 0x5C, 0x52),
                    bottom_color: rgb8(0xFE, 0x74, 0x6C),
                    border_color: if is_dark {
                        rgb8(0xB8, 0x32, 0x2B)
                    } else {
                        rgb8(0xE0, 0x44, 0x3E)
                    },
                    glow_color: Color::rgba(0.90, 0.22, 0.18, 0.22),
                }
            }
        }
        WindowControlAction::Minimize => {
            if is_pressed {
                TrafficLightSpherePalette {
                    top_color: rgb8(0xFF, 0xD0, 0x52),
                    bottom_color: rgb8(0xFF, 0xE0, 0x78),
                    border_color: rgb8(0xE0, 0xA2, 0x03),
                    glow_color: Color::rgba(1.0, 0.82, 0.22, 0.35),
                }
            } else {
                TrafficLightSpherePalette {
                    top_color: rgb8(0xFE, 0xBB, 0x2C),
                    bottom_color: rgb8(0xFE, 0xD2, 0x52),
                    border_color: if is_dark {
                        rgb8(0xC2, 0x82, 0x16)
                    } else {
                        rgb8(0xDE, 0xA1, 0x23)
                    },
                    glow_color: Color::rgba(0.95, 0.70, 0.12, 0.20),
                }
            }
        }
        WindowControlAction::Zoom | WindowControlAction::Expand => {
            if is_pressed {
                TrafficLightSpherePalette {
                    top_color: rgb8(0x50, 0xE8, 0x68),
                    bottom_color: rgb8(0x78, 0xF2, 0x8C),
                    border_color: rgb8(0x20, 0xBA, 0x38),
                    glow_color: Color::rgba(0.28, 0.92, 0.38, 0.35),
                }
            } else {
                TrafficLightSpherePalette {
                    top_color: rgb8(0x28, 0xC6, 0x3E),
                    bottom_color: rgb8(0x4C, 0xDE, 0x64),
                    border_color: if is_dark {
                        rgb8(0x14, 0x8C, 0x1C)
                    } else {
                        rgb8(0x1A, 0xAB, 0x29)
                    },
                    glow_color: Color::rgba(0.18, 0.78, 0.28, 0.20),
                }
            }
        }
    }
}

/// Resolves physical liquid-glass sphere palette for an inactive window control.
#[must_use]
pub fn resolve_inactive_sphere_palette(is_dark: bool) -> TrafficLightSpherePalette {
    if is_dark {
        TrafficLightSpherePalette {
            top_color: rgb8(0x4C, 0x4C, 0x50),
            bottom_color: rgb8(0x5E, 0x5E, 0x64),
            border_color: Color::rgba(0.20, 0.20, 0.22, 0.60),
            glow_color: Color::rgba(0.0, 0.0, 0.0, 0.20),
        }
    } else {
        TrafficLightSpherePalette {
            top_color: rgb8(0xD1, 0xD1, 0xD6),
            bottom_color: rgb8(0xE5, 0xE5, 0xEA),
            border_color: Color::rgba(0.70, 0.70, 0.73, 0.80),
            glow_color: Color::rgba(0.0, 0.0, 0.0, 0.05),
        }
    }
}

/// Resolves fill and border colors for an active button.
#[must_use]
pub fn resolve_active_button_colors(
    action: WindowControlAction,
    is_dark: bool,
    is_pressed: bool,
    is_hovered: bool,
) -> (Color, Color) {
    let pal = resolve_active_sphere_palette(action, is_dark, is_pressed, is_hovered);
    (pal.top_color, pal.border_color)
}

/// Resolves fill and border colors for an inactive button.
#[must_use]
pub fn resolve_inactive_button_colors(is_dark: bool) -> (Color, Color) {
    let pal = resolve_inactive_sphere_palette(is_dark);
    (pal.top_color, pal.border_color)
}

/// Resolves fill and border colors for a button action and state.
#[must_use]
pub fn resolve_button_colors(
    action: WindowControlAction,
    is_dark: bool,
    is_active: bool,
    is_pressed: bool,
    is_hovered: bool,
) -> (Color, Color) {
    if !is_active {
        resolve_inactive_button_colors(is_dark)
    } else {
        resolve_active_button_colors(action, is_dark, is_pressed, is_hovered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traffic_light::action::WindowExpandBehavior;

    #[test]
    fn close_and_minimize_glyphs_get_extra_optical_weight() {
        let close = window_control_glyph_size(WindowControlAction::Close, WINDOW_CONTROL_NATIVE_SIZE);
        let minimize =
            window_control_glyph_size(WindowControlAction::Minimize, WINDOW_CONTROL_NATIVE_SIZE);
        let expand =
            window_control_glyph_size(WindowControlAction::Expand, WINDOW_CONTROL_NATIVE_SIZE);

        assert!(minimize > close);
        assert!(close > expand);
        assert_eq!(close, 7.0);
        assert_eq!(minimize, 8.0);
        assert_eq!(
            window_control_glyph_size(WindowControlAction::Close, crate::traffic_light::layout::WINDOW_CONTROL_LARGE_SIZE),
            crate::traffic_light::layout::WINDOW_CONTROL_LARGE_SIZE * 0.42
        );
    }

    #[test]
    fn inactive_window_glyphs_use_a_separate_pale_tone() {
        let active_light = window_control_glyph_color(false, WindowControlAction::Close, false, 1.0);
        let inactive_light = window_control_glyph_color(false, WindowControlAction::Close, true, 0.0);
        let active_dark = window_control_glyph_color(true, WindowControlAction::Close, false, 1.0);
        let inactive_dark = window_control_glyph_color(true, WindowControlAction::Close, true, 0.0);

        assert!(inactive_light.r > active_light.r);
        assert!(inactive_light.g > active_light.g);
        assert!(inactive_dark.r > active_dark.r);
        assert!(inactive_dark.g > active_dark.g);
    }

    #[test]
    fn glyph_color_transparency_tracks_focus() {
        let no_focus = window_control_glyph_color(false, WindowControlAction::Close, false, 0.0);
        let full_light = window_control_glyph_color(false, WindowControlAction::Close, false, 1.0);
        let full_dark = window_control_glyph_color(true, WindowControlAction::Close, false, 1.0);

        assert_eq!(no_focus.a, 0.0);
        assert_eq!(full_light.a, 0.75);
        assert_eq!(full_dark.a, 0.85);
    }

    #[test]
    fn dark_mode_active_borders_are_less_luminous() {
        let (_dark_fill, dark_border) =
            resolve_active_button_colors(WindowControlAction::Close, true, false, false);
        let (_light_fill, light_border) =
            resolve_active_button_colors(WindowControlAction::Close, false, false, false);

        assert_eq!(dark_border, rgb8(0xB8, 0x32, 0x2B));
        assert_eq!(light_border, rgb8(0xE0, 0x44, 0x3E));
        assert!(light_border.r > dark_border.r);
    }

    #[test]
    fn inactive_palette_distinguishes_dark_and_light() {
        let (dark_fill, dark_border) = resolve_inactive_button_colors(true);
        let (light_fill, light_border) = resolve_inactive_button_colors(false);

        assert_eq!(dark_fill, rgb8(0x4C, 0x4C, 0x50));
        assert_eq!(light_fill, rgb8(0xD1, 0xD1, 0xD6));
        assert!(light_fill.r > dark_fill.r);
        assert_ne!(dark_border, light_border);
    }

    #[test]
    fn icon_selection_follows_expand_behavior() {
        assert_eq!(
            window_control_icon(WindowControlAction::Expand, WindowExpandBehavior::Fullscreen),
            SVG_ZOOM
        );
        assert_eq!(
            window_control_icon(WindowControlAction::Expand, WindowExpandBehavior::Maximize),
            SVG_MAXIMIZE
        );
    }

    #[test]
    fn svg_sources_are_dynamic_current_color() {
        for source in [SVG_CLOSE, SVG_MINIMIZE, SVG_ZOOM, SVG_MAXIMIZE] {
            assert!(source.contains("currentColor"));
        }
        assert!(SVG_SPECULAR_ARC.contains("specGrad"));
        assert!(SVG_SPECULAR_ARC.contains("bottomRimGrad"));
    }
}
