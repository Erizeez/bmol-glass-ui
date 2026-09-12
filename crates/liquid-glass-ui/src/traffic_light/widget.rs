//! The Iced glyph layer and pointer handling for traffic lights.
//!
//! The widget is deliberately split in two:
//!
//! - the **visual circles** ([`window_control_circle`]) draw only Apple vector
//!   glyphs, an optional specular arc, and an optional self-contained sphere;
//! - [`TrafficLightGroup`] owns *all* pointer handling for the group: hover
//!   reveal, per-control hit testing with optical slop, and the press lifecycle.
//!
//! One widget owning the whole group is what makes the interaction correct: the
//! press follows the pointer across the group instead of being cancelled by a
//! one-pixel excursion out of a 14 pt box, and a release can never be lost
//! because there is a single owner for the armed control.

use std::rc::Rc;

use iced::advanced::widget::{tree, Tree, Widget};
use iced::advanced::{self, layout, renderer, svg as advanced_svg, Clipboard, Layout, Shell};
use iced::mouse;
use iced::widget::{container, row, space, stack, svg};
use iced::{Background, Color, Element, Event, Length, Padding, Point, Rectangle, Size, Vector};

use crate::GlassOverlay;
use crate::traffic_light::action::{WindowControlAction, WindowExpandBehavior};
use crate::traffic_light::event::TrafficLightsEvent;
use crate::traffic_light::layout::control_hover_slop;
use crate::traffic_light::palette::{
    SVG_CLOSE, SVG_MAXIMIZE, SVG_MINIMIZE, SVG_SPECULAR_ARC, SVG_ZOOM, blend_color,
    resolve_active_button_colors, resolve_inactive_button_colors, window_control_glyph_color,
    window_control_glyph_size,
};

/// When enabled, the traffic-light widget leaves the sphere body and specular
/// arc transparent so an external GPU glass compositor renders the physical
/// glass, while the widget contributes only the Apple vector glyph layer and
/// the full press/hover interaction.
static GLASS_PASSTHROUGH: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Enables or disables GPU glass passthrough for the traffic-light widget.
pub fn set_glass_passthrough(enabled: bool) {
    GLASS_PASSTHROUGH.store(enabled, std::sync::atomic::Ordering::Relaxed);
}

/// Returns whether GPU glass passthrough is currently enabled.
#[must_use]
pub fn glass_passthrough() -> bool {
    GLASS_PASSTHROUGH.load(std::sync::atomic::Ordering::Relaxed)
}

/// Mirrors AppKit's `NSWindow.isDocumentEdited`: when set, the close control
/// renders a filled status dot instead of the `✕` glyph.
static DOCUMENT_EDITED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Mirrors `NSWindow.setDocumentEdited(_:)`.
pub fn set_document_edited(edited: bool) {
    DOCUMENT_EDITED.store(edited, std::sync::atomic::Ordering::Relaxed);
}

/// Mirrors `NSWindow.isDocumentEdited`.
#[must_use]
pub fn is_document_edited() -> bool {
    DOCUMENT_EDITED.load(std::sync::atomic::Ordering::Relaxed)
}

/// Control group variants for the standalone showcase laboratory.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ControlGroup {
    Native,
    Reference,
    Active,
    Inactive,
    Disabled,
}

impl ControlGroup {
    /// Stable index of this sample group, matching the interaction store.
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Native => 0,
            Self::Reference => 1,
            Self::Active => 2,
            Self::Inactive => 3,
            Self::Disabled => 4,
        }
    }

    /// Whether this sample renders a window under an inactive (unfocused) key
    /// window.
    #[must_use]
    pub const fn is_inactive_window(self) -> bool {
        matches!(self, Self::Inactive)
    }
}

/// Status dot for disabled or unsaved close control.
#[must_use]
pub fn window_control_status_dot<
    'a,
    Message: 'a,
    Theme: 'a + container::Catalog,
    Renderer: advanced::Renderer + 'a,
>(
    color: Color,
    size: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    <Theme as container::Catalog>::Class<'a>: From<container::StyleFn<'a, Theme>>,
{
    let dot_size = (size * 0.24).max(3.0);
    container(space())
        .width(Length::Fixed(dot_size))
        .height(Length::Fixed(dot_size))
        .style(move |_theme| container::Style {
            background: Some(Background::Color(color)),
            border: iced::Border::default().rounded(dot_size * 0.5),
            ..Default::default()
        })
        .into()
}

/// Centered container helper for optical alignment.
#[must_use]
pub fn centered<
    'a,
    Message: 'a,
    Theme: 'a + container::Catalog,
    Renderer: advanced::Renderer + 'a,
>(
    content: Element<'a, Message, Theme, Renderer>,
    size: f32,
) -> Element<'a, Message, Theme, Renderer> {
    container(content)
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .center_x(Length::Fixed(size))
        .center_y(Length::Fixed(size))
        .into()
}

/// Everything the visual layer needs for one control circle.
#[derive(Debug, Clone, Copy)]
pub struct CircleStyle {
    pub action: WindowControlAction,
    pub size: f32,
    pub is_dark: bool,
    pub is_active: bool,
    pub hover: f32,
    pub scale: f32,
    /// Press tint, `0..=1`.
    pub press: f32,
    pub is_fullscreen_symbol: bool,
    /// Render the unsaved-document dot instead of the glyph.
    pub dot: bool,
}

/// Draws one traffic-light circle: sphere body, specular arc, and glyph.
///
/// Pure presentation — it never captures events. Interaction is owned by
/// [`TrafficLightGroup`].
#[allow(clippy::too_many_arguments)]
pub fn window_control_circle<'a, Message: 'a, Theme, Renderer>(
    style: CircleStyle,
) -> Element<'a, Message, Theme, Renderer>
where
    Theme: 'a + container::Catalog + iced::widget::svg::Catalog,
    <Theme as container::Catalog>::Class<'a>: From<container::StyleFn<'a, Theme>>,
    <Theme as iced::widget::svg::Catalog>::Class<'a>: From<iced::widget::svg::StyleFn<'a, Theme>>,
    Renderer: advanced::Renderer + advanced_svg::Renderer + 'a,
{
    let CircleStyle {
        action,
        size,
        is_dark,
        is_active,
        hover,
        scale,
        press,
        is_fullscreen_symbol,
        dot,
    } = style;

    let visual_size = size * scale;
    let hover = hover.clamp(0.0, 1.0);
    let press = press.clamp(0.0, 1.0);

    // Layer 1: sphere body. Hover only reveals the glyph and transitions
    // inactive→active colours; the brighter palette is reserved for press.
    let (active_fill, active_border) =
        resolve_active_button_colors(action, is_dark, press > 0.5, false);
    let (fill, border) = if is_active {
        (active_fill, active_border)
    } else if hover > 0.0 {
        let (inactive_fill, inactive_border) = resolve_inactive_button_colors(is_dark);
        (
            blend_color(inactive_fill, active_fill, hover),
            blend_color(inactive_border, active_border, hover),
        )
    } else {
        resolve_inactive_button_colors(is_dark)
    };
    let fill = to_iced(fill);
    let border = to_iced(border);

    let shadow = if is_dark {
        iced::Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
            offset: Vector::new(0.0, 0.5),
            blur_radius: 1.5,
        }
    } else {
        iced::Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.12),
            offset: Vector::new(0.0, 0.5),
            blur_radius: 1.0,
        }
    };

    let passthrough = glass_passthrough();
    let sphere_body: Element<'a, Message, Theme, Renderer> = container(space())
        .width(Length::Fixed(visual_size))
        .height(Length::Fixed(visual_size))
        .style(move |_theme| {
            if passthrough {
                container::Style::default()
            } else {
                container::Style {
                    background: Some(Background::Color(fill)),
                    border: iced::Border::default()
                        .rounded(visual_size * 0.5)
                        .width(0.5)
                        .color(border),
                    shadow,
                    ..Default::default()
                }
            }
        })
        .into();

    // Layer 2: specular crescent + subsurface rim. Skipped under GPU
    // passthrough because the physical-glass shader supplies its own.
    let specular_highlight: Element<'a, Message, Theme, Renderer> = if passthrough {
        space().width(Length::Fixed(visual_size)).height(Length::Fixed(visual_size)).into()
    } else {
        container(
            svg(svg::Handle::from_memory(SVG_SPECULAR_ARC.as_bytes()))
                .width(Length::Fixed(visual_size))
                .height(Length::Fixed(visual_size)),
        )
        .width(Length::Fixed(visual_size))
        .height(Length::Fixed(visual_size))
        .into()
    };

    // Layer 3: Apple vector glyph, or the unsaved-document dot.
    let glyph_element: Element<'a, Message, Theme, Renderer> = if dot {
        // The unsaved-document dot is part of the resting control: macOS shows
        // it whether or not the pointer is over the group. Only the vector
        // glyphs are hover-revealed.
        let dot_color = to_iced(window_control_glyph_color(is_dark, action, !is_active, 1.0));
        container(window_control_status_dot(dot_color, size * scale))
            .width(Length::Fixed(visual_size))
            .height(Length::Fixed(visual_size))
            .center_x(Length::Fixed(visual_size))
            .center_y(Length::Fixed(visual_size))
            .into()
    } else {
        let svg_source = match action {
            WindowControlAction::Close => SVG_CLOSE,
            WindowControlAction::Minimize => SVG_MINIMIZE,
            WindowControlAction::Zoom | WindowControlAction::Expand => {
                if is_fullscreen_symbol { SVG_ZOOM } else { SVG_MAXIMIZE }
            }
        };
        let glyph_size = window_control_glyph_size(action, size) * scale;
        let glyph_color = to_iced(window_control_glyph_color(is_dark, action, !is_active, hover));
        let color = Color { a: 1.0, ..glyph_color };
        container(
            svg(svg::Handle::from_memory(svg_source.as_bytes()))
                .width(Length::Fixed(glyph_size))
                .height(Length::Fixed(glyph_size))
                .opacity(hover.clamp(0.0, 1.0))
                .style(move |_theme, _status| svg::Style { color: Some(color) }),
        )
        .width(Length::Fixed(visual_size))
        .height(Length::Fixed(visual_size))
        .center_x(Length::Fixed(visual_size))
        .center_y(Length::Fixed(visual_size))
        .into()
    };

    centered(
        container(stack![sphere_body, specular_highlight, glyph_element])
            .width(Length::Fixed(visual_size))
            .height(Length::Fixed(visual_size))
            .center_x(Length::Fixed(visual_size))
            .center_y(Length::Fixed(visual_size))
            .into(),
        size,
    )
}

/// Converts the renderer-agnostic scene colour into Iced's colour type.
///
/// The palette and the scene are Iced-independent; this is the single
/// boundary where the two colour types meet.
#[must_use]
fn to_iced(color: liquid_glass_scene::Color) -> Color {
    Color::from_rgba(color.r, color.g, color.b, color.a)
}

/// Local interaction state of [`TrafficLightGroup`].
#[derive(Debug, Default)]
struct TrafficLightGroupState {
    hovered: bool,
    /// The control the pointer pressed, kept until the button is released.
    armed: Option<usize>,
    /// Whether the pointer is currently back over the armed control.
    on_armed: bool,
}

/// Interactive owner of one traffic-light group.
///
/// Publishes [`TrafficLightsEvent`] values; the caller routes them into its
/// own interaction state. Hit testing covers the circles plus optical slop,
/// and an armed press survives leaving and re-entering the control, so a click
/// can never be lost to hand tremor.
pub struct TrafficLightGroup<'a, Message, Theme, Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    size: f32,
    gap: f32,
    slop: f32,
    on_event: Option<Rc<dyn Fn(TrafficLightsEvent) -> Message + 'a>>,
}

impl<Message, Theme, Renderer> std::fmt::Debug for TrafficLightGroup<'_, Message, Theme, Renderer> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrafficLightGroup")
            .field("size", &self.size)
            .field("gap", &self.gap)
            .field("slop", &self.slop)
            .finish_non_exhaustive()
    }
}

impl<'a, Message, Theme, Renderer> TrafficLightGroup<'a, Message, Theme, Renderer> {
    /// Wraps the visual row in the group's pointer handling.
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        size: f32,
        gap: f32,
        slop: f32,
    ) -> Self {
        Self {
            content: content.into(),
            size,
            gap,
            slop,
            on_event: None,
        }
    }

    /// Sets the single event sink for this group.
    #[must_use]
    pub fn on_event(
        mut self,
        on_event: impl Fn(TrafficLightsEvent) -> Message + 'a,
    ) -> Self {
        self.on_event = Some(Rc::new(on_event));
        self
    }

    fn stride(&self) -> f32 {
        self.size + self.gap
    }

    fn inner_width(&self) -> f32 {
        self.size * 3.0 + self.gap * 2.0
    }

    fn publish(&self, shell: &mut Shell<'_, Message>, event: TrafficLightsEvent) {
        if let Some(on_event) = &self.on_event {
            shell.publish(on_event(event));
        }
    }

    /// Maps a point inside this widget's bounds to a control index.
    fn hit_index(&self, local: Point) -> Option<usize> {
        let x = local.x - self.slop;
        let y = local.y - self.slop;
        if y < 0.0 || y > self.size || x < 0.0 {
            return None;
        }
        let stride = self.stride();
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let index = (x / stride) as usize;
        if index >= 3 || x - index as f32 * stride > self.size {
            return None;
        }
        Some(index)
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for TrafficLightGroup<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: advanced::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<TrafficLightGroupState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(TrafficLightGroupState::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        Size::new(
            Length::Fixed(self.inner_width() + self.slop * 2.0),
            Length::Fixed(self.size + self.slop * 2.0),
        )
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let child_limits =
            limits.width(Length::Fixed(self.inner_width())).height(Length::Fixed(self.size));
        let child = self.content.as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &child_limits,
        );
        layout::Node::with_children(
            Size::new(self.inner_width() + self.slop * 2.0, self.size + self.slop * 2.0),
            vec![child.move_to(Point::new(self.slop, self.slop))],
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout.children().next().unwrap_or(layout),
            cursor,
            viewport,
        );
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let local = cursor.position_in(bounds);
        let hovered = local.is_some();
        let hit = local.and_then(|point| self.hit_index(point));

        let state = tree.state.downcast_mut::<TrafficLightGroupState>();
        if hovered != state.hovered {
            state.hovered = hovered;
            self.publish(shell, TrafficLightsEvent::GroupHover(hovered));
            shell.request_redraw();
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if hovered => {
                if let Some(index) = hit {
                    state.armed = Some(index);
                    state.on_armed = true;
                    self.publish(shell, TrafficLightsEvent::PressStart(index));
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if let Some(index) = state.armed.take() {
                    let committed = state.on_armed;
                    state.on_armed = false;
                    self.publish(
                        shell,
                        TrafficLightsEvent::PressEnd { index, committed },
                    );
                    shell.capture_event();
                }
            }
            _ => {}
        }

        // Continuous press tracking. The armed control keeps the button; leaving
        // it clears the tint only, and coming back restores it without
        // restarting the growth animation.
        if let Some(index) = state.armed {
            let on_armed = hit == Some(index);
            if on_armed != state.on_armed {
                state.on_armed = on_armed;
                let message = if on_armed {
                    TrafficLightsEvent::PressStart(index)
                } else {
                    TrafficLightsEvent::PressCancel(index)
                };
                self.publish(shell, message);
                shell.request_redraw();
            }
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap_or(layout),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let content_interaction = self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        );
        if content_interaction == mouse::Interaction::None && cursor.is_over(layout.bounds()) {
            mouse::Interaction::Idle
        } else {
            content_interaction
        }
    }
}

impl<'a, Message: Clone + 'a, Theme: 'a, Renderer: advanced::Renderer + 'a>
    From<TrafficLightGroup<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
{
    fn from(group: TrafficLightGroup<'a, Message, Theme, Renderer>) -> Self {
        Element::new(group)
    }
}

/// Configuration for one rendered group of three controls.
#[derive(Debug, Clone, Copy)]
pub struct ControlGroupStyle {
    /// Circle diameter in logical points.
    pub size: f32,
    /// Visual gap between two adjacent circles.
    pub gap: f32,
    pub is_dark: bool,
    /// Whether the window owning these controls is focused.
    pub is_active: bool,
    /// Whether the close control is unavailable.
    pub close_disabled: bool,
    /// Draw the unsaved-document dot on the close control.
    pub close_dot: bool,
    pub expand_behavior: WindowExpandBehavior,
    /// Group glyph reveal progress.
    pub hover: f32,
    /// Per-control press tint.
    pub press: [f32; 3],
    /// Per-control press scale.
    pub scales: [f32; 3],
}

/// Builds the three visual circles of one group.
fn control_row<'a, Id: Copy + 'a, Message: 'a, Theme, Renderer>(
    ids: [Id; 3],
    style: ControlGroupStyle,
) -> Element<'a, Message, Theme, Renderer>
where
    Theme: 'a + container::Catalog + iced::widget::svg::Catalog,
    <Theme as container::Catalog>::Class<'a>: From<container::StyleFn<'a, Theme>>,
    <Theme as iced::widget::svg::Catalog>::Class<'a>: From<iced::widget::svg::StyleFn<'a, Theme>>,
    Renderer: advanced::Renderer + advanced_svg::Renderer + 'a,
{
    let _ = ids;
    let actions = [
        WindowControlAction::Close,
        WindowControlAction::Minimize,
        if style.expand_behavior.is_fullscreen() {
            WindowControlAction::Expand
        } else {
            WindowControlAction::Zoom
        },
    ];
    let circles = actions.into_iter().enumerate().map(|(index, action)| {
        let circle = CircleStyle {
            action,
            size: style.size,
            is_dark: style.is_dark,
            is_active: style.is_active && !(index == 0 && style.close_disabled),
            // The whole group reveals its glyphs together, as macOS does.
            hover: style.hover,
            scale: style.scales[index],
            press: style.press[index],
            is_fullscreen_symbol: style.expand_behavior.is_fullscreen(),
            dot: index == 0 && style.close_dot,
        };
        window_control_circle::<Message, Theme, Renderer>(circle)
    });
    let mut iter = circles;
    row![iter.next().unwrap(), iter.next().unwrap(), iter.next().unwrap()]
        .spacing(style.gap)
        .align_y(iced::Alignment::Center)
        .into()
}

/// Builds one interactive traffic-light group with full press lifecycle.
///
/// The returned element owns hover tracking, per-control hit testing with
/// optical slop, and the press lifecycle. `on_event` receives
/// [`TrafficLightsEvent`] values already addressed to the group's control
/// index; route them into the application's interaction state.
///
/// # Compositing
///
/// The sphere material is drawn by the GPU glass compositor while the Apple
/// vector glyphs are drawn by Iced, so the glyphs must be composited *above*
/// the glass pass or the glass paints over them. This builder performs that
/// routing itself: the group comes back already wrapped in a [`GlassOverlay`],
/// and a caller that simply places it in a layout gets correctly layered
/// controls. Place it on the glass — anywhere in the widget tree, at any
/// depth — rather than trying to add the overlay yourself.
#[allow(clippy::too_many_arguments)]
pub fn window_control_group<'a, Id: Copy + 'a, Message: Clone + 'a, Theme, Renderer>(
    ids: [Id; 3],
    style: ControlGroupStyle,
    on_event: impl Fn(Id, TrafficLightsEvent) -> Message + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Theme: 'a + container::Catalog + iced::widget::svg::Catalog,
    <Theme as container::Catalog>::Class<'a>: From<container::StyleFn<'a, Theme>>,
    <Theme as iced::widget::svg::Catalog>::Class<'a>: From<iced::widget::svg::StyleFn<'a, Theme>>,
    Renderer: advanced::Renderer + advanced_svg::Renderer + crate::GlassForegroundRenderer + 'a,
{
    let slop = control_hover_slop(style.size);
    let row = control_row::<Id, Message, Theme, Renderer>(ids, style);
    let group: Element<'a, Message, Theme, Renderer> =
        TrafficLightGroup::new(row, style.size, style.gap, slop)
            .on_event(move |event| on_event(ids[event_index(&event)], event))
            .into();
    GlassOverlay::new(group).into()
}

fn event_index(event: &TrafficLightsEvent) -> usize {
    match *event {
        TrafficLightsEvent::GroupHover(_) => 0,
        TrafficLightsEvent::PressStart(index)
        | TrafficLightsEvent::PressCancel(index)
        | TrafficLightsEvent::PressEnd { index, .. } => index,
    }
}

/// Helper to position a control group with optical slop compensation.
#[must_use]
pub fn positioned_control_group<
    'a,
    Message: 'a,
    Theme: 'a + container::Catalog,
    Renderer: advanced::Renderer + 'a,
>(
    content: Element<'a, Message, Theme, Renderer>,
    x: f32,
    y: f32,
    size: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    <Theme as container::Catalog>::Class<'a>: From<container::StyleFn<'a, Theme>>,
{
    let slop = control_hover_slop(size);
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            top: y - slop,
            right: 0.0,
            bottom: 0.0,
            left: x - slop,
        })
        .into()
}

/// Backwards-compatible alias for [`window_control_circle`].
pub fn view_single_button<'a, Message: 'a, Theme, Renderer>(
    action: WindowControlAction,
    size: f32,
    is_dark: bool,
    is_active: bool,
    hover_progress: f32,
    scale: f32,
    is_fullscreen_symbol: bool,
) -> Element<'a, Message, Theme, Renderer>
where
    Theme: 'a + container::Catalog + iced::widget::svg::Catalog,
    <Theme as container::Catalog>::Class<'a>: From<container::StyleFn<'a, Theme>>,
    <Theme as iced::widget::svg::Catalog>::Class<'a>: From<iced::widget::svg::StyleFn<'a, Theme>>,
    Renderer: advanced::Renderer + advanced_svg::Renderer + 'a,
{
    window_control_circle(CircleStyle {
        action,
        size,
        is_dark,
        is_active,
        hover: hover_progress,
        scale,
        press: 0.0,
        is_fullscreen_symbol,
        dot: action == WindowControlAction::Close && is_document_edited(),
    })
}

/// Builds a single visual control circle with an explicit press tint.
#[allow(clippy::too_many_arguments)]
pub fn view_single_button_interactive<'a, Message: 'a, Theme, Renderer>(
    action: WindowControlAction,
    size: f32,
    is_dark: bool,
    is_active: bool,
    hover_progress: f32,
    scale: f32,
    is_pressed: bool,
    is_fullscreen_symbol: bool,
) -> Element<'a, Message, Theme, Renderer>
where
    Theme: 'a + container::Catalog + iced::widget::svg::Catalog,
    <Theme as container::Catalog>::Class<'a>: From<container::StyleFn<'a, Theme>>,
    <Theme as iced::widget::svg::Catalog>::Class<'a>: From<iced::widget::svg::StyleFn<'a, Theme>>,
    Renderer: advanced::Renderer + advanced_svg::Renderer + 'a,
{
    window_control_circle(CircleStyle {
        action,
        size,
        is_dark,
        is_active,
        hover: hover_progress,
        scale,
        press: if is_pressed { 1.0 } else { 0.0 },
        is_fullscreen_symbol,
        dot: action == WindowControlAction::Close && is_document_edited(),
    })
}

/// Convenience wrapper mirroring the historical `control_group` shape.
///
/// Kept so existing showcase code keeps reading naturally; it simply forwards
/// to [`window_control_group`].
#[allow(clippy::too_many_arguments)]
pub fn control_group<'a, Id: Copy + 'a, Message: Clone + 'a, Theme, Renderer>(
    ids: [Id; 3],
    size: f32,
    gap: f32,
    is_dark: bool,
    is_active: bool,
    close_disabled: bool,
    expand_behavior: WindowExpandBehavior,
    hover_amount: f32,
    press_scales: [f32; 3],
    press_tints: [f32; 3],
    on_event: impl Fn(Id, TrafficLightsEvent) -> Message + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Theme: 'a + container::Catalog + iced::widget::svg::Catalog,
    <Theme as container::Catalog>::Class<'a>: From<container::StyleFn<'a, Theme>>,
    <Theme as iced::widget::svg::Catalog>::Class<'a>: From<iced::widget::svg::StyleFn<'a, Theme>>,
    Renderer: advanced::Renderer + advanced_svg::Renderer + crate::GlassForegroundRenderer + 'a,
{
    let style = ControlGroupStyle {
        size,
        gap,
        is_dark,
        is_active,
        close_disabled,
        close_dot: close_disabled && is_document_edited(),
        expand_behavior,
        hover: hover_amount,
        press: press_tints,
        scales: press_scales,
    };
    window_control_group(ids, style, on_event)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traffic_light::layout::{DIAMETER, SPACING};

    fn style() -> ControlGroupStyle {
        ControlGroupStyle {
            size: DIAMETER,
            gap: SPACING,
            is_dark: false,
            is_active: true,
            close_disabled: false,
            close_dot: false,
            expand_behavior: WindowExpandBehavior::Fullscreen,
            hover: 0.0,
            press: [0.0; 3],
            scales: [1.0; 3],
        }
    }

    #[test]
    fn hit_testing_covers_circles_and_slop_but_not_the_gaps() {
        let group: TrafficLightGroup<'_, (), iced::Theme, iced::Renderer> =
            TrafficLightGroup::new(space(), DIAMETER, SPACING, 6.0);

        // Leading slop, then the centre of each circle.
        assert_eq!(group.hit_index(Point::new(2.0, 3.0 + DIAMETER * 0.5)), None);
        for index in 0..3 {
            let x = 6.0 + index as f32 * (DIAMETER + SPACING) + DIAMETER * 0.5;
            assert_eq!(group.hit_index(Point::new(x, 6.0 + DIAMETER * 0.5)), Some(index));
        }

        // The gap between two circles is not a hit.
        let in_gap = 6.0 + DIAMETER + SPACING * 0.5;
        assert_eq!(group.hit_index(Point::new(in_gap, 6.0 + DIAMETER * 0.5)), None);

        // Outside the vertical band is not a hit.
        assert_eq!(group.hit_index(Point::new(6.0, 1.0)), None);
    }

    #[test]
    fn group_size_includes_the_optical_slop() {
        let group: TrafficLightGroup<'_, (), iced::Theme, iced::Renderer> =
            TrafficLightGroup::new(space(), DIAMETER, SPACING, 6.0);
        let expected_width = DIAMETER * 3.0 + SPACING * 2.0 + 12.0;
        assert!((group.inner_width() + 12.0 - expected_width).abs() < f32::EPSILON);
    }

    #[test]
    fn event_index_matches_the_control_slot() {
        assert_eq!(event_index(&TrafficLightsEvent::GroupHover(true)), 0);
        assert_eq!(event_index(&TrafficLightsEvent::PressStart(2)), 2);
        assert_eq!(event_index(&TrafficLightsEvent::PressCancel(1)), 1);
        assert_eq!(event_index(&TrafficLightsEvent::PressEnd { index: 2, committed: true }), 2);
    }

    #[test]
    fn svg_views_construct_for_every_action() {
        for action in [
            WindowControlAction::Close,
            WindowControlAction::Minimize,
            WindowControlAction::Zoom,
        ] {
            let _: Element<'_, (), iced::Theme, iced::Renderer> =
                view_single_button(action, DIAMETER, false, true, 1.0, 1.0, true);
            let _: Element<'_, (), iced::Theme, iced::Renderer> =
                view_single_button_interactive(action, DIAMETER, false, true, 1.0, 1.0, true, false);
        }
        let _: Element<'_, (), iced::Theme, iced::Renderer> = control_row(
            [liquid_glass_scene::GlassId(0); 3],
            style(),
        );
    }
}
