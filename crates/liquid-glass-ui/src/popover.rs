//! Authentic Apple Continuous Curvature Popover & Beak (触角) Primitives.
//!
//! Provides unopinionated, flexible building blocks for popovers, tooltips, and context menus.
//! Callers (such as Dock popups, Menu Bar dropdowns, or custom card widgets) can easily
//! compose these primitives to fit their specific layout and interaction requirements:
//!
//! - **Dock Integration**: Position arrow towards Dock icon centers with native 21×9 pt flare
//! - **Menu Bar Integration**: Project downward from top status bar items
//! - **Custom Floating Cards**: Freely position and style squircle popovers with multi-tier soft shadows

#![allow(clippy::cast_precision_loss)]

use iced::{
    Color, Point, Rectangle,
    widget::canvas::{self, Frame, Path},
};
use squircle_rs::{PathCommand, SquircleParams, squircle_popover_path_commands};

pub use bmol_designs::popover_metrics::{
    DOCK_MENU_ARROW_CENTER_OFFSET, PopoverArrowConfig, PopoverArrowEdge, PopoverArrowPreset,
    popover_arrow_profile_height, popover_arrow_profile_height_with_spline,
};
pub use squircle_rs::{
    APPLE_CORNER_SMOOTHING, MENU_WIDE_SPLINE, PopoverArrowParams, PopoverArrowSide, PopoverSpline,
    TOOLTIP_NARROW_SPLINE,
};

/// Computes a [`PopoverArrowConfig`] pointing directly at the center of a target anchor rectangle
/// (e.g. a Dock icon, Menu Bar status item, or clicked button).
///
/// This simplifies custom component encapsulation for Dock, Menu Bar, and contextual tooltips.
#[must_use]
pub fn align_arrow_to_target(
    card_bounds: Rectangle,
    target_bounds: Rectangle,
    edge: PopoverArrowEdge,
    preset: PopoverArrowPreset,
) -> PopoverArrowConfig {
    let offset = match edge {
        PopoverArrowEdge::Top | PopoverArrowEdge::Bottom => {
            let target_center_x = target_bounds.x + target_bounds.width * 0.5;
            let rel_x = target_center_x - card_bounds.x;
            rel_x / card_bounds.width.max(1.0)
        }
        PopoverArrowEdge::Left | PopoverArrowEdge::Right => {
            let target_center_y = target_bounds.y + target_bounds.height * 0.5;
            let rel_y = target_center_y - card_bounds.y;
            rel_y / card_bounds.height.max(1.0)
        }
        PopoverArrowEdge::None => 0.5,
    };

    PopoverArrowConfig::from_preset(edge, preset).with_offset(offset.clamp(0.0, 1.0))
}

/// Builds an authentic Apple continuous curvature squircle path with an integrated smooth
/// popover arrow / beak (触角) on the designated edge.
#[must_use]
pub fn build_popover_path(rect: Rectangle, radius: f32, arrow: PopoverArrowConfig) -> Path {
    let r = radius.min(rect.width * 0.5).min(rect.height * 0.5);
    let params =
        SquircleParams::new(rect.width, rect.height, r).with_smoothing(APPLE_CORNER_SMOOTHING);

    let arrow_side = match arrow.edge {
        PopoverArrowEdge::Top => PopoverArrowSide::Top,
        PopoverArrowEdge::Bottom => PopoverArrowSide::Bottom,
        PopoverArrowEdge::Left => PopoverArrowSide::Left,
        PopoverArrowEdge::Right => PopoverArrowSide::Right,
        PopoverArrowEdge::None => PopoverArrowSide::None,
    };

    let arrow_params = PopoverArrowParams {
        side: arrow_side,
        base_width: arrow.base_width,
        height: arrow.height,
        offset: arrow.offset,
        spline: PopoverSpline {
            apex_ctrl_u: arrow.spline.apex_ctrl_u,
            upper_flank_u: arrow.spline.upper_flank_u,
            upper_flank_v: arrow.spline.upper_flank_v,
            inflection_u: arrow.spline.inflection_u,
            inflection_v: arrow.spline.inflection_v,
            lower_flank_u: arrow.spline.lower_flank_u,
            lower_flank_v: arrow.spline.lower_flank_v,
            base_ctrl_u: arrow.spline.base_ctrl_u,
        },
    };

    let commands = squircle_popover_path_commands(&params, &arrow_params);
    let rx = rect.x;
    let ry = rect.y;

    Path::new(move |b| {
        for cmd in &commands {
            match *cmd {
                PathCommand::MoveTo(pt) => b.move_to(Point::new(rx + pt.x, ry + pt.y)),
                PathCommand::LineTo(pt) => b.line_to(Point::new(rx + pt.x, ry + pt.y)),
                PathCommand::CubicTo { c0, c1, to } => b.bezier_curve_to(
                    Point::new(rx + c0.x, ry + c0.y),
                    Point::new(rx + c1.x, ry + c1.y),
                    Point::new(rx + to.x, ry + to.y),
                ),
                PathCommand::Close => b.close(),
            }
        }
    })
}

/// Fills an authentic Apple squircle/popover on an Iced frame using continuous curvature.
pub fn fill_popover(
    frame: &mut Frame,
    rect: Rectangle,
    radius: f32,
    arrow: PopoverArrowConfig,
    color: Color,
) {
    if rect.width <= 0.0 || rect.height <= 0.0 || color.a <= 0.001 {
        return;
    }
    let path = build_popover_path(rect, radius, arrow);
    frame.fill(&path, color);
}

/// Strokes a continuous 1px fine edge highlight around the entire popover squircle + arrow rim.
pub fn stroke_popover_rim(
    frame: &mut Frame,
    rect: Rectangle,
    radius: f32,
    arrow: PopoverArrowConfig,
    color: Color,
    width: f32,
) {
    if rect.width <= 0.0 || rect.height <= 0.0 || color.a <= 0.001 {
        return;
    }
    let path = build_popover_path(rect, radius, arrow);
    frame.stroke(&path, canvas::Stroke::default().with_color(color).with_width(width));
}

/// Renders multi-layer soft drop shadows matching macOS Popover Window shadow geometry,
/// continuously wrapping both the squircle body and the popover arrow (触角).
pub fn render_popover_shadow(
    frame: &mut Frame,
    rect: Rectangle,
    corner_radius: f32,
    arrow: PopoverArrowConfig,
    is_dark: bool,
) {
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }

    // 1. Ambient Contact Shadow: Tight, soft ground contact (macOS HIG elevation = 4pt)
    let ambient_tiers = 8;
    let ambient_max_spread = 10.0f32;
    let base_ambient_alpha = if is_dark { 0.048 } else { 0.038 };

    for i in (0..ambient_tiers).rev() {
        let t = (i + 1) as f32 / ambient_tiers as f32;
        let spread = ambient_max_spread * t;
        let alpha = base_ambient_alpha * (1.0 - t).powi(2);
        let shadow_rect = Rectangle {
            x: rect.x - spread * 0.5,
            y: rect.y - spread * 0.25,
            width: rect.width + spread,
            height: rect.height + spread * 1.1,
        };
        let shadow_arrow = if arrow.is_visible() {
            arrow.with_size(arrow.base_width + spread * 1.2, arrow.height + spread * 0.8)
        } else {
            arrow
        };
        fill_popover(
            frame,
            shadow_rect,
            corner_radius + spread,
            shadow_arrow,
            Color::from_rgba(0.0, 0.0, 0.0, alpha),
        );
    }

    // 2. Key Elevation Drop Shadow: Deep spatial projection (macOS HIG elevation = 16pt)
    let key_tiers = 14;
    let key_max_spread = 30.0f32;
    let key_offset_y = 14.0f32;
    let base_key_alpha = if is_dark { 0.036 } else { 0.026 };

    for i in (0..key_tiers).rev() {
        let t = (i + 1) as f32 / key_tiers as f32;
        let spread = key_max_spread * t;
        let offset_y = key_offset_y * t;
        let alpha = base_key_alpha * (1.0 - t).powi(2);
        let shadow_rect = Rectangle {
            x: rect.x - spread * 0.75,
            y: rect.y + offset_y - spread * 0.35,
            width: rect.width + spread * 1.5,
            height: rect.height + spread * 1.35,
        };
        let shadow_arrow = if arrow.is_visible() {
            arrow.with_size(arrow.base_width + spread * 1.2, arrow.height + spread * 0.8)
        } else {
            arrow
        };
        fill_popover(
            frame,
            shadow_rect,
            corner_radius + spread,
            shadow_arrow,
            Color::from_rgba(0.0, 0.0, 0.0, alpha),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_arrow_to_target_dock_bottom() {
        let card = Rectangle { x: 100.0, y: 200.0, width: 154.0, height: 121.0 };
        let target_icon = Rectangle { x: 120.0, y: 330.0, width: 48.0, height: 48.0 };

        // Target center = 120 + 24 = 144. Rel x = 144 - 100 = 44. Offset = 44 / 154 ≈ 0.2857
        let arrow = align_arrow_to_target(
            card,
            target_icon,
            PopoverArrowEdge::Bottom,
            PopoverArrowPreset::MenuWide,
        );
        assert_eq!(arrow.edge, PopoverArrowEdge::Bottom);
        assert!((arrow.offset - (44.0 / 154.0)).abs() < 1e-4);
        assert_eq!(arrow.base_width, 21.0);
        assert_eq!(arrow.height, 9.0);
    }

    #[test]
    fn test_align_arrow_to_target_menu_bar_top() {
        let card = Rectangle { x: 50.0, y: 30.0, width: 200.0, height: 250.0 };
        let status_item = Rectangle { x: 70.0, y: 0.0, width: 20.0, height: 24.0 };

        // Target center = 70 + 10 = 80. Rel x = 80 - 50 = 30. Offset = 30 / 200 = 0.15
        let arrow = align_arrow_to_target(
            card,
            status_item,
            PopoverArrowEdge::Top,
            PopoverArrowPreset::AppKitStandard,
        );
        assert_eq!(arrow.edge, PopoverArrowEdge::Top);
        assert!((arrow.offset - 0.15).abs() < 1e-4);
        assert_eq!(arrow.base_width, 27.5);
        assert_eq!(arrow.height, 13.0);
    }

    #[test]
    fn test_build_popover_path_all_presets_and_edges() {
        let rect = Rectangle { x: 10.0, y: 10.0, width: 154.0, height: 121.0 };
        let r = 10.0;

        let edges = [
            PopoverArrowEdge::None,
            PopoverArrowEdge::Top,
            PopoverArrowEdge::Bottom,
            PopoverArrowEdge::Left,
            PopoverArrowEdge::Right,
        ];

        let presets = [
            PopoverArrowPreset::MenuWide,
            PopoverArrowPreset::AppKitStandard,
            PopoverArrowPreset::TooltipNarrow,
            PopoverArrowPreset::SubtleCompact,
        ];

        for &edge in &edges {
            for &preset in &presets {
                let config = PopoverArrowConfig::from_preset(edge, preset).with_offset(0.25);
                let path = build_popover_path(rect, r, config);
                drop(path);
            }
        }
    }
}
