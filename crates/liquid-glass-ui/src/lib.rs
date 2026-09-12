//! UI-facing Liquid Glass components.
//!
//! The widgets implement Iced's advanced widget contract while keeping the
//! renderer-facing scene node independent from Iced.

#![deny(unsafe_code)]

use std::fmt;

pub mod components;
pub mod context_menu;
pub mod font;
pub mod icon;
pub mod popover;
pub mod scroll_view;
mod theme;
pub mod window;

use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_DYNAMIC_GLASS_ID: AtomicU64 = AtomicU64::new(50_000);

/// Generates a globally unique dynamic glass ID.
#[must_use]
pub fn next_dynamic_glass_id() -> GlassId {
    GlassId(NEXT_DYNAMIC_GLASS_ID.fetch_add(1, Ordering::Relaxed))
}

pub use context_menu::{ContextMenu, MenuItem, view_context_menu};
pub use icon::UiIcon;
pub use popover::{
    PopoverArrowConfig, PopoverArrowEdge, PopoverArrowPreset, PopoverArrowSide,
    align_arrow_to_target, build_popover_path, fill_popover, render_popover_shadow,
    stroke_popover_rim,
};
pub use scroll_view::{
    ScrollbarConfig, SpringScrollState, SpringScrollView, spring_scroll_view,
    spring_scroll_view_with_config,
};
pub use theme::{GlassChrome, GlassRole, UiColorScheme, UiCornerStyle, UiPalette, UiTheme};
pub use window::{
    DEFAULT_WINDOW_CORNER_RADIUS, IcedWindowController, IcedWindowPolicy, WindowCommand,
    WindowDragArea, WindowExpandBehavior,
};

use iced::advanced::text::Renderer as TextRenderer;
use iced::{
    Background, Border, Color as IcedColor, Event, Length, Padding, Pixels, Rectangle, Shadow,
    Size, Vector,
    advanced::{self, Clipboard, Layout, Shell, Widget, layout, mouse, renderer, widget::Tree},
};
use liquid_glass_scene::{
    CornerCurve, GlassId, GlassInteraction, GlassMaterial, GlassNode, GlassShape, GlassShapeLayer,
    PathCommand, Rect, squircle_path_commands,
};

/// Allows a compositor to route the visual contents of a glass surface into
/// a separate foreground renderer.
pub trait GlassForegroundRenderer {
    fn begin_glass_foreground(&mut self) {}
    fn end_glass_foreground(&mut self) {}
    fn begin_glass_overlay(&mut self) {}
    fn end_glass_overlay(&mut self) {}

    /// Publishes interaction state for the compositor-owned node with `id`.
    /// Renderers that do not own a Liquid Glass compositor may ignore it.
    fn update_glass_interaction(&self, _id: GlassId, _interaction: GlassInteraction) {}

    /// Registers a dynamic glass node for the current frame to be rendered by the compositor.
    fn register_glass_node(&mut self, _node: GlassNode) {}

    /// Clears and returns all registered dynamic glass nodes for the current frame.
    fn take_glass_nodes(&mut self) -> Vec<GlassNode> {
        Vec::new()
    }
}

impl GlassForegroundRenderer for () {}

/// Wraps a widget whose pixels belong above the glass composition pass.
pub struct GlassForeground<'a, Message, Theme, Renderer> {
    content: iced::Element<'a, Message, Theme, Renderer>,
}

impl<'a, Message, Theme, Renderer> GlassForeground<'a, Message, Theme, Renderer> {
    #[must_use]
    pub fn new(content: iced::Element<'a, Message, Theme, Renderer>) -> Self {
        Self { content }
    }
}

impl<Message, Theme, Renderer> fmt::Debug for GlassForeground<'_, Message, Theme, Renderer> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("GlassForeground").finish_non_exhaustive()
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for GlassForeground<'_, Message, Theme, Renderer>
where
    Renderer: advanced::Renderer + GlassForegroundRenderer,
{
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content.as_widget_mut().layout(&mut tree.children[0], renderer, limits)
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
        renderer.begin_glass_foreground();
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
        renderer.end_glass_foreground();
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(self.content.as_widget())]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn advanced::widget::Operation,
    ) {
        self.content.as_widget_mut().operate(&mut tree.children[0], layout, renderer, operation);
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
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
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
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<GlassForeground<'a, Message, Theme, Renderer>>
    for iced::Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: advanced::Renderer + GlassForegroundRenderer + 'a,
{
    fn from(foreground: GlassForeground<'a, Message, Theme, Renderer>) -> Self {
        iced::Element::new(foreground)
    }
}

/// Wraps a widget whose pixels belong above both the glass composition and
/// any post-composition masks applied to a foreground layer.
pub struct GlassOverlay<'a, Message, Theme, Renderer> {
    content: iced::Element<'a, Message, Theme, Renderer>,
}

impl<'a, Message, Theme, Renderer> GlassOverlay<'a, Message, Theme, Renderer> {
    #[must_use]
    pub fn new(content: iced::Element<'a, Message, Theme, Renderer>) -> Self {
        Self { content }
    }
}

impl<Message, Theme, Renderer> fmt::Debug for GlassOverlay<'_, Message, Theme, Renderer> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("GlassOverlay").finish_non_exhaustive()
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for GlassOverlay<'_, Message, Theme, Renderer>
where
    Renderer: advanced::Renderer + GlassForegroundRenderer,
{
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content.as_widget_mut().layout(&mut tree.children[0], renderer, limits)
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
        renderer.begin_glass_overlay();
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
        renderer.end_glass_overlay();
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(self.content.as_widget())]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn advanced::widget::Operation,
    ) {
        self.content.as_widget_mut().operate(&mut tree.children[0], layout, renderer, operation);
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
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
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
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<GlassOverlay<'a, Message, Theme, Renderer>>
    for iced::Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: advanced::Renderer + GlassForegroundRenderer + 'a,
{
    fn from(overlay: GlassOverlay<'a, Message, Theme, Renderer>) -> Self {
        iced::Element::new(overlay)
    }
}

/// A container that can later host any Iced widget tree.
#[derive(Clone, Debug)]
pub struct GlassContainer {
    node: GlassNode,
    chrome: GlassChrome,
    padding: f32,
    hovered: bool,
}

impl GlassContainer {
    #[must_use]
    pub fn new(id: GlassId, bounds: Rect) -> Self {
        Self {
            node: GlassNode::new(id, bounds),
            chrome: GlassChrome::default(),
            padding: 0.0,
            hovered: false,
        }
    }

    #[must_use]
    pub fn shape(mut self, shape: GlassShape) -> Self {
        self.node = self.node.shape(shape);
        self
    }

    #[must_use]
    pub fn corner_curve(mut self, corner_curve: CornerCurve) -> Self {
        self.node = self.node.corner_curve(corner_curve);
        self
    }

    #[must_use]
    pub fn material(mut self, material: GlassMaterial) -> Self {
        self.node = self.node.material(material);
        self
    }

    #[must_use]
    pub fn fuse_shape(mut self, shape: GlassShapeLayer) -> Self {
        self.node = self.node.fuse_shape(shape);
        self
    }

    #[must_use]
    pub const fn chrome(mut self, chrome: GlassChrome) -> Self {
        self.chrome = chrome;
        self
    }

    #[must_use]
    pub const fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    #[must_use]
    pub const fn node(&self) -> &GlassNode {
        &self.node
    }

    #[must_use]
    pub const fn padding_value(&self) -> f32 {
        self.padding
    }

    /// Returns a scene node using the final bounds computed by Iced layout.
    #[must_use]
    pub fn scene_node_for(&self, bounds: Rectangle) -> GlassNode {
        let mut node = GlassNode::new(
            self.node.id,
            Rect::new(bounds.x, bounds.y, bounds.width, bounds.height),
        )
        .shape(self.node.shape.clone())
        .corner_curve(self.node.corner_curve)
        .material(self.node.material)
        .interaction(self.node.interaction);
        for fused_shape in &self.node.fused_shapes {
            node = node.fuse_shape(fused_shape.clone());
        }
        node
    }

    /// Resolves this fixed-size widget into a renderer-independent scene node.
    ///
    /// `GlassContainer` exposes fixed bounds, so no concrete Iced renderer is
    /// needed to resolve its layout. Keeping this bridge renderer-independent
    /// also makes release builds work with Iced configurations that omit the
    /// debug-only null renderer.
    #[must_use]
    pub fn layout_scene_node(&mut self, _viewport: Size) -> GlassNode {
        self.scene_node_for(Rectangle {
            x: self.node.bounds.x,
            y: self.node.bounds.y,
            width: self.node.bounds.width,
            height: self.node.bounds.height,
        })
    }

    /// Converts this widget into an Iced element for any compatible renderer.
    #[must_use]
    pub fn into_element<Message, Theme, Renderer>(
        self,
    ) -> iced::Element<'static, Message, Theme, Renderer>
    where
        Message: 'static,
        Theme: 'static,
        Renderer: advanced::Renderer + GlassForegroundRenderer + 'static,
    {
        iced::Element::new(self)
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for GlassContainer
where
    Renderer: advanced::Renderer + GlassForegroundRenderer,
{
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(self.node.bounds.width), Length::Fixed(self.node.bounds.height))
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(
            limits,
            Length::Fixed(self.node.bounds.width),
            Length::Fixed(self.node.bounds.height),
        )
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        if bounds.intersection(viewport).is_none() {
            return;
        }

        // Register the dynamic glass node to the compositor renderer.
        renderer.register_glass_node(self.scene_node_for(bounds));

        // The compositor owns the material fill. Keeping the Iced-side base
        // transparent prevents a tinted rectangle from being composited a
        // second time on top of the shader result.
        let fill = if self.hovered {
            iced_color(self.chrome.hover_overlay)
        } else {
            IcedColor::TRANSPARENT
        };
        let border_color =
            iced_color(if self.hovered { self.chrome.hover_border } else { self.chrome.border });
        let radius = shape_radius(&self.node);

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border::default().rounded(radius).width(1.0).color(border_color),
                shadow: Shadow {
                    color: iced_color(self.chrome.shadow),
                    offset: Vector::new(0.0, self.chrome.shadow_offset_y),
                    blur_radius: self.chrome.shadow_blur,
                },
                snap: true,
            },
            Background::Color(fill),
        );
    }

    fn update(
        &mut self,
        _tree: &mut Tree,
        _event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let hovered = cursor.is_over(layout.bounds());
        renderer.update_glass_interaction(
            self.node.id,
            interaction_for_cursor(layout.bounds(), cursor, hovered, false),
        );
        if hovered != self.hovered {
            self.hovered = hovered;
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Message, Theme, Renderer> From<GlassContainer>
    for iced::Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: advanced::Renderer + GlassForegroundRenderer + 'a,
{
    fn from(container: GlassContainer) -> Self {
        iced::Element::new(container)
    }
}

pub use glass_panel::{GlassPanel, glass_panel};

mod glass_panel {
    use super::*;

    /// A Liquid Glass panel widget that wraps child content.
    ///
    /// The glass substrate, curvature, refraction, and optical highlights are
    /// rendered by the GPU compositor, while the child content is drawn in the
    /// foreground layer above the glass surface.
    pub struct GlassPanel<'a, Message, Theme, Renderer> {
        id: GlassId,
        content: iced::Element<'a, Message, Theme, Renderer>,
        shape: GlassShape,
        corner_curve: CornerCurve,
        material: GlassMaterial,
        padding: Padding,
        width: Length,
        height: Length,
    }

    impl<'a, Message, Theme, Renderer> GlassPanel<'a, Message, Theme, Renderer> {
        #[must_use]
        pub fn new(content: impl Into<iced::Element<'a, Message, Theme, Renderer>>) -> Self {
            Self {
                id: next_dynamic_glass_id(),
                content: content.into(),
                shape: GlassShape::RoundedRect { radius: 16.0 },
                corner_curve: CornerCurve::continuous(),
                material: GlassMaterial::regular(),
                padding: Padding::ZERO,
                width: Length::Shrink,
                height: Length::Shrink,
            }
        }

        #[must_use]
        pub const fn id(mut self, id: GlassId) -> Self {
            self.id = id;
            self
        }

        #[must_use]
        pub fn shape(mut self, shape: GlassShape) -> Self {
            self.shape = shape;
            self
        }

        #[must_use]
        pub fn squircle_radius(mut self, radius: f32) -> Self {
            self.shape = GlassShape::RoundedRect { radius };
            self.corner_curve = CornerCurve::continuous();
            self
        }

        #[must_use]
        pub const fn corner_curve(mut self, corner_curve: CornerCurve) -> Self {
            self.corner_curve = corner_curve;
            self
        }

        #[must_use]
        pub const fn material(mut self, material: GlassMaterial) -> Self {
            self.material = material;
            self
        }

        #[must_use]
        pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
            self.padding = padding.into();
            self
        }

        #[must_use]
        pub const fn width(mut self, width: Length) -> Self {
            self.width = width;
            self
        }

        #[must_use]
        pub const fn height(mut self, height: Length) -> Self {
            self.height = height;
            self
        }

        #[must_use]
        pub const fn node_id(&self) -> GlassId {
            self.id
        }

        #[must_use]
        pub fn node_shape(&self) -> &GlassShape {
            &self.shape
        }

        #[must_use]
        pub const fn node_corner_curve(&self) -> CornerCurve {
            self.corner_curve
        }

        #[must_use]
        pub const fn node_material(&self) -> GlassMaterial {
            self.material
        }

        #[must_use]
        pub const fn node_padding(&self) -> Padding {
            self.padding
        }
    }

    impl<Message, Theme, Renderer> fmt::Debug for GlassPanel<'_, Message, Theme, Renderer> {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter
                .debug_struct("GlassPanel")
                .field("id", &self.id)
                .field("shape", &self.shape)
                .field("material", &self.material)
                .finish_non_exhaustive()
        }
    }

    impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
        for GlassPanel<'_, Message, Theme, Renderer>
    where
        Renderer: advanced::Renderer + GlassForegroundRenderer,
    {
        fn size(&self) -> Size<Length> {
            Size::new(self.width, self.height)
        }

        fn size_hint(&self) -> Size<Length> {
            Size::new(self.width, self.height)
        }

        fn layout(
            &mut self,
            tree: &mut Tree,
            renderer: &Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            layout::padded(limits, self.width, self.height, self.padding, |limits| {
                self.content.as_widget_mut().layout(&mut tree.children[0], renderer, limits)
            })
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
            let bounds = layout.bounds();
            if bounds.intersection(viewport).is_none() {
                return;
            }

            // 1. Register dynamic GPU glass node for the compositor
            let node =
                GlassNode::new(self.id, Rect::new(bounds.x, bounds.y, bounds.width, bounds.height))
                    .shape(self.shape.clone())
                    .corner_curve(self.corner_curve)
                    .material(self.material);

            renderer.register_glass_node(node);

            // 2. Render children inside the Glass Foreground pass
            if let Some(child_layout) = layout.children().next() {
                renderer.begin_glass_foreground();
                self.content.as_widget().draw(
                    &tree.children[0],
                    renderer,
                    theme,
                    style,
                    child_layout,
                    cursor,
                    viewport,
                );
                renderer.end_glass_foreground();
            }
        }

        fn children(&self) -> Vec<Tree> {
            vec![Tree::new(self.content.as_widget())]
        }

        fn diff(&self, tree: &mut Tree) {
            tree.diff_children(&[self.content.as_widget()]);
        }

        fn operate(
            &mut self,
            tree: &mut Tree,
            layout: Layout<'_>,
            renderer: &Renderer,
            operation: &mut dyn advanced::widget::Operation,
        ) {
            if let Some(child_layout) = layout.children().next() {
                self.content.as_widget_mut().operate(
                    &mut tree.children[0],
                    child_layout,
                    renderer,
                    operation,
                );
            }
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
            if let Some(child_layout) = layout.children().next() {
                self.content.as_widget_mut().update(
                    &mut tree.children[0],
                    event,
                    child_layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                );
            }
        }

        fn mouse_interaction(
            &self,
            tree: &Tree,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
            renderer: &Renderer,
        ) -> mouse::Interaction {
            if let Some(child_layout) = layout.children().next() {
                self.content.as_widget().mouse_interaction(
                    &tree.children[0],
                    child_layout,
                    cursor,
                    viewport,
                    renderer,
                )
            } else {
                mouse::Interaction::default()
            }
        }

        fn overlay<'b>(
            &'b mut self,
            tree: &'b mut Tree,
            layout: Layout<'b>,
            renderer: &Renderer,
            viewport: &Rectangle,
            translation: Vector,
        ) -> Option<advanced::overlay::Element<'b, Message, Theme, Renderer>> {
            if let Some(child_layout) = layout.children().next() {
                self.content.as_widget_mut().overlay(
                    &mut tree.children[0],
                    child_layout,
                    renderer,
                    viewport,
                    translation,
                )
            } else {
                None
            }
        }
    }

    impl<'a, Message, Theme, Renderer> From<GlassPanel<'a, Message, Theme, Renderer>>
        for iced::Element<'a, Message, Theme, Renderer>
    where
        Message: 'a,
        Theme: 'a,
        Renderer: advanced::Renderer + GlassForegroundRenderer + 'a,
    {
        fn from(panel: GlassPanel<'a, Message, Theme, Renderer>) -> Self {
            iced::Element::new(panel)
        }
    }

    /// Helper constructor for [`GlassPanel`].
    pub fn glass_panel<'a, Message, Theme, Renderer>(
        content: impl Into<iced::Element<'a, Message, Theme, Renderer>>,
    ) -> GlassPanel<'a, Message, Theme, Renderer> {
        GlassPanel::new(content)
    }
}

fn shape_radius(node: &GlassNode) -> f32 {
    match node.shape {
        GlassShape::RoundedRect { radius } => radius,
        GlassShape::Superellipse { .. } => node.bounds.width.min(node.bounds.height) * 0.4,
        GlassShape::Capsule => node.bounds.height * 0.5,
        GlassShape::Circle => node.bounds.width.min(node.bounds.height) * 0.5,
        GlassShape::Ellipse => node.bounds.width.min(node.bounds.height) * 0.25,
    }
}

fn interaction_for_cursor(
    bounds: Rectangle,
    cursor: mouse::Cursor,
    hovered: bool,
    pressed: bool,
) -> GlassInteraction {
    let pointer = cursor.position_over(bounds).map_or([0.5, 0.5], |position| {
        [
            ((position.x - bounds.x) / bounds.width.max(1.0)).clamp(0.0, 1.0),
            ((position.y - bounds.y) / bounds.height.max(1.0)).clamp(0.0, 1.0),
        ]
    });
    GlassInteraction::normalized(
        pointer,
        f32::from(u8::from(hovered)),
        f32::from(u8::from(pressed)),
        0.0,
    )
}

/// Optional icon content for compact glass buttons.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GlassButtonIcon {
    Back,
    Forward,
}

/// The first interactive component contract.
#[derive(Clone, Debug)]
pub struct GlassButton {
    label: String,
    icon: Option<GlassButtonIcon>,
    node: GlassNode,
    chrome: GlassChrome,
}

impl GlassButton {
    #[must_use]
    pub fn new(id: GlassId, label: impl Into<String>, bounds: Rect) -> Self {
        Self {
            label: label.into(),
            icon: None,
            node: GlassNode::new(id, bounds)
                .shape(GlassShape::Capsule)
                .material(GlassMaterial::interactive()),
            chrome: GlassChrome::default(),
        }
    }

    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Replaces the material used by the Iced fallback paint and compositor
    /// scene node.
    #[must_use]
    pub fn material(mut self, material: GlassMaterial) -> Self {
        self.node = self.node.material(material);
        self
    }

    #[must_use]
    pub fn shape(mut self, shape: GlassShape) -> Self {
        self.node = self.node.shape(shape);
        self
    }

    #[must_use]
    pub fn corner_curve(mut self, corner_curve: CornerCurve) -> Self {
        self.node = self.node.corner_curve(corner_curve);
        self
    }

    #[must_use]
    pub const fn icon(mut self, icon: GlassButtonIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    #[must_use]
    pub const fn chrome(mut self, chrome: GlassChrome) -> Self {
        self.chrome = chrome;
        self
    }

    #[must_use]
    pub const fn node(&self) -> &GlassNode {
        &self.node
    }

    /// Converts this button into an Iced element that publishes `on_press`
    /// when the left mouse button is released inside its bounds.
    #[must_use]
    pub fn into_element<Message, Theme, Renderer>(
        self,
        on_press: Message,
    ) -> iced::Element<'static, Message, Theme, Renderer>
    where
        Message: Clone + 'static,
        Theme: 'static,
        Renderer: advanced::Renderer + TextRenderer + GlassForegroundRenderer + 'static,
    {
        self.into_element_with_press(on_press, None)
    }

    /// Converts this button into an Iced element and optionally publishes a
    /// second message at the beginning of a left-button press.
    ///
    /// The ordinary `on_press` message remains release-based. The additional
    /// callback is intended for transient visual state such as a material
    /// press light, while keeping activation semantics unchanged.
    #[must_use]
    pub fn into_element_with_press<Message, Theme, Renderer>(
        self,
        on_press: Message,
        on_press_start: Option<Message>,
    ) -> iced::Element<'static, Message, Theme, Renderer>
    where
        Message: Clone + 'static,
        Theme: 'static,
        Renderer: advanced::Renderer + TextRenderer + GlassForegroundRenderer + 'static,
    {
        self.into_element_with_press_callbacks(on_press, on_press_start, None, None)
    }

    /// Converts this button into an Iced element with callbacks for press
    /// start and release/cancellation.
    ///
    /// `on_press_end` is emitted for every captured mouse release, including
    /// releases outside the button. This is useful for resetting transient
    /// visual state without changing the ordinary click activation rule.
    #[must_use]
    pub fn into_element_with_press_end<Message, Theme, Renderer>(
        self,
        on_press: Message,
        on_press_start: Option<Message>,
        on_press_end: Option<Message>,
    ) -> iced::Element<'static, Message, Theme, Renderer>
    where
        Message: Clone + 'static,
        Theme: 'static,
        Renderer: advanced::Renderer + TextRenderer + GlassForegroundRenderer + 'static,
    {
        self.into_element_with_press_callbacks(on_press, on_press_start, None, on_press_end)
    }

    /// Converts this button into an Iced element with independent callbacks
    /// for press start, pointer cancellation, and release.
    ///
    /// Pointer cancellation is emitted when a captured press leaves the
    /// button. Release is emitted for every captured mouse release, including
    /// releases outside the button. The ordinary click callback is still only
    /// emitted when the release happens inside the button.
    #[must_use]
    pub fn into_element_with_press_callbacks<Message, Theme, Renderer>(
        self,
        on_press: Message,
        on_press_start: Option<Message>,
        on_press_cancel: Option<Message>,
        on_press_end: Option<Message>,
    ) -> iced::Element<'static, Message, Theme, Renderer>
    where
        Message: Clone + 'static,
        Theme: 'static,
        Renderer: advanced::Renderer + TextRenderer + GlassForegroundRenderer + 'static,
    {
        iced::Element::new(GlassButtonWidget {
            button: self,
            on_press,
            on_press_start,
            on_press_cancel,
            on_press_end,
        })
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct GlassButtonState {
    hovered: bool,
    pressed: bool,
}

struct GlassButtonWidget<Message> {
    button: GlassButton,
    on_press: Message,
    on_press_start: Option<Message>,
    on_press_cancel: Option<Message>,
    on_press_end: Option<Message>,
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for GlassButtonWidget<Message>
where
    Message: Clone,
    Renderer: advanced::Renderer + TextRenderer + GlassForegroundRenderer,
{
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<GlassButtonState>()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(GlassButtonState::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(
            Length::Fixed(self.button.node.bounds.width),
            Length::Fixed(self.button.node.bounds.height),
        )
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(
            limits,
            Length::Fixed(self.button.node.bounds.width),
            Length::Fixed(self.button.node.bounds.height),
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        if bounds.intersection(viewport).is_none() {
            return;
        }

        let state = tree.state.downcast_ref::<GlassButtonState>();
        let fill = if state.pressed {
            iced_color(self.button.chrome.pressed_overlay)
        } else if state.hovered {
            iced_color(self.button.chrome.hover_overlay)
        } else {
            IcedColor::TRANSPARENT
        };
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border::default()
                    .rounded(shape_radius(&self.button.node))
                    .width(1.0)
                    .color(iced_color(if state.hovered {
                        self.button.chrome.hover_border
                    } else {
                        self.button.chrome.border
                    })),
                shadow: Shadow {
                    color: iced_color(self.button.chrome.shadow),
                    offset: Vector::new(0.0, self.button.chrome.shadow_offset_y),
                    blur_radius: self.button.chrome.shadow_blur,
                },
                snap: true,
            },
            Background::Color(fill),
        );
        let (content, font, size) = match self.button.icon {
            Some(GlassButtonIcon::Back) => {
                (Renderer::SCROLL_LEFT_ICON.to_string(), Renderer::ICON_FONT, Pixels(14.0))
            }
            Some(GlassButtonIcon::Forward) => {
                (Renderer::SCROLL_RIGHT_ICON.to_string(), Renderer::ICON_FONT, Pixels(14.0))
            }
            None => (self.button.label.clone(), renderer.default_font(), Pixels(16.0)),
        };
        renderer.fill_text(
            advanced::Text {
                content,
                bounds: bounds.size(),
                size,
                line_height: advanced::text::LineHeight::default(),
                font,
                align_x: advanced::text::Alignment::Center,
                align_y: iced::alignment::Vertical::Center,
                shaping: advanced::text::Shaping::Auto,
                wrapping: advanced::text::Wrapping::None,
            },
            bounds.center(),
            iced_color(self.button.chrome.text),
            *viewport,
        );
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let hovered = cursor.is_over(layout.bounds());
        let state = tree.state.downcast_mut::<GlassButtonState>();
        let was_hovered = state.hovered;
        let was_pressed = state.pressed;
        state.hovered = hovered;

        if was_pressed && was_hovered && !hovered {
            if let Some(message) = self.on_press_cancel.clone() {
                shell.publish(message);
            }
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if hovered => {
                state.pressed = true;
                shell.capture_event();
                if let Some(message) = self.on_press_start.clone() {
                    shell.publish(message);
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if state.pressed => {
                state.pressed = false;
                shell.capture_event();
                if let Some(message) = self.on_press_end.clone() {
                    shell.publish(message);
                }
                if hovered {
                    shell.publish(self.on_press.clone());
                }
            }
            _ => {}
        }

        renderer.update_glass_interaction(
            self.button.node.id,
            interaction_for_cursor(
                layout.bounds(),
                cursor,
                state.hovered,
                state.pressed && state.hovered,
            ),
        );

        if was_hovered != state.hovered || was_pressed != state.pressed {
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

/// Content rendered in one equal-width glass control segment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GlassSegmentContent {
    ChevronLeft,
    ChevronRight,
    Glyph(String),
}

impl GlassSegmentContent {
    #[must_use]
    pub fn glyph(value: impl Into<String>) -> Self {
        Self::Glyph(value.into())
    }
}

/// One independently enabled item in a [`GlassSegmentedControl`].
#[derive(Clone, Debug)]
pub struct GlassSegment<Message> {
    content: GlassSegmentContent,
    on_press: Message,
    enabled: bool,
}

impl<Message> GlassSegment<Message> {
    #[must_use]
    pub const fn new(content: GlassSegmentContent, on_press: Message) -> Self {
        Self { content, on_press, enabled: true }
    }

    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    #[must_use]
    pub const fn content(&self) -> &GlassSegmentContent {
        &self.content
    }
}

/// A reusable equal-width segmented control backed by one glass surface.
#[derive(Clone, Debug)]
pub struct GlassSegmentedControl<Message> {
    node: GlassNode,
    chrome: GlassChrome,
    segments: Vec<GlassSegment<Message>>,
}

impl<Message> GlassSegmentedControl<Message> {
    /// Builds a control from one or more equal-width segments.
    ///
    /// # Panics
    ///
    /// Panics when `segments` is empty or contains more than 65,535 items.
    #[must_use]
    pub fn new(
        id: GlassId,
        bounds: Rect,
        segments: impl IntoIterator<Item = GlassSegment<Message>>,
    ) -> Self {
        let segments = segments.into_iter().collect::<Vec<_>>();
        assert!(!segments.is_empty(), "a segmented control needs at least one segment");
        assert!(
            u16::try_from(segments.len()).is_ok(),
            "a segmented control supports at most 65535 segments"
        );
        Self {
            node: GlassNode::new(id, bounds)
                .shape(GlassShape::Capsule)
                .material(GlassMaterial::interactive()),
            chrome: GlassChrome::default(),
            segments,
        }
    }

    #[must_use]
    pub fn material(mut self, material: GlassMaterial) -> Self {
        self.node = self.node.material(material);
        self
    }

    #[must_use]
    pub const fn chrome(mut self, chrome: GlassChrome) -> Self {
        self.chrome = chrome;
        self
    }

    #[must_use]
    pub const fn node(&self) -> &GlassNode {
        &self.node
    }

    #[must_use]
    pub fn segments(&self) -> &[GlassSegment<Message>] {
        &self.segments
    }

    #[must_use]
    pub fn into_element<Theme, Renderer>(self) -> iced::Element<'static, Message, Theme, Renderer>
    where
        Message: Clone + 'static,
        Theme: 'static,
        Renderer: advanced::Renderer
            + advanced::graphics::geometry::Renderer
            + TextRenderer
            + GlassForegroundRenderer
            + 'static,
    {
        iced::Element::new(GlassSegmentedWidget { control: self, hovered: None, pressed: None })
    }
}

/// Convenience wrapper for the common two-segment back/forward control.
#[derive(Clone, Debug)]
pub struct GlassNavigationControl {
    node: GlassNode,
    chrome: GlassChrome,
    back_enabled: bool,
    forward_enabled: bool,
}

impl GlassNavigationControl {
    #[must_use]
    pub fn new(id: GlassId, bounds: Rect) -> Self {
        Self {
            node: GlassNode::new(id, bounds)
                .shape(GlassShape::Capsule)
                .material(GlassMaterial::interactive()),
            chrome: GlassChrome::default(),
            back_enabled: true,
            forward_enabled: true,
        }
    }

    #[must_use]
    pub fn material(mut self, material: GlassMaterial) -> Self {
        self.node = self.node.material(material);
        self
    }

    #[must_use]
    pub const fn chrome(mut self, chrome: GlassChrome) -> Self {
        self.chrome = chrome;
        self
    }

    #[must_use]
    pub const fn back_enabled(mut self, enabled: bool) -> Self {
        self.back_enabled = enabled;
        self
    }

    #[must_use]
    pub const fn forward_enabled(mut self, enabled: bool) -> Self {
        self.forward_enabled = enabled;
        self
    }

    #[must_use]
    pub const fn node(&self) -> &GlassNode {
        &self.node
    }

    #[must_use]
    pub fn into_element<Message, Theme, Renderer>(
        self,
        on_back: Message,
        on_forward: Message,
    ) -> iced::Element<'static, Message, Theme, Renderer>
    where
        Message: Clone + 'static,
        Theme: 'static,
        Renderer: advanced::Renderer
            + advanced::graphics::geometry::Renderer
            + TextRenderer
            + GlassForegroundRenderer
            + 'static,
    {
        GlassSegmentedControl {
            node: self.node,
            chrome: self.chrome,
            segments: vec![
                GlassSegment::new(GlassSegmentContent::ChevronLeft, on_back)
                    .enabled(self.back_enabled),
                GlassSegment::new(GlassSegmentContent::ChevronRight, on_forward)
                    .enabled(self.forward_enabled),
            ],
        }
        .into_element()
    }
}

struct GlassSegmentedWidget<Message> {
    control: GlassSegmentedControl<Message>,
    hovered: Option<usize>,
    pressed: Option<usize>,
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for GlassSegmentedWidget<Message>
where
    Message: Clone,
    Renderer: advanced::Renderer
        + advanced::graphics::geometry::Renderer
        + TextRenderer
        + GlassForegroundRenderer,
{
    fn size(&self) -> Size<Length> {
        Size::new(
            Length::Fixed(self.control.node.bounds.width),
            Length::Fixed(self.control.node.bounds.height),
        )
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(
            limits,
            Length::Fixed(self.control.node.bounds.width),
            Length::Fixed(self.control.node.bounds.height),
        )
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        if bounds.intersection(viewport).is_none() {
            return;
        }

        let segment_count = self.control.segments.len();
        let segment_width = bounds.width / count_as_f32(segment_count);
        draw_segment_chrome(renderer, &self.control, bounds, self.hovered, self.pressed);

        for divider_index in 1..segment_count {
            if divider_touches_hovered(divider_index, self.hovered) {
                continue;
            }
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: bounds.x + segment_width * count_as_f32(divider_index) - 0.5,
                        y: bounds.y + 8.0,
                        width: 1.0,
                        height: bounds.height - 16.0,
                    },
                    border: Border::default(),
                    shadow: Shadow::default(),
                    snap: true,
                },
                Background::Color(iced_color(self.control.chrome.divider)),
            );
        }

        draw_segment_content(
            renderer,
            &self.control.segments,
            self.control.chrome,
            bounds,
            *viewport,
        );
    }

    fn update(
        &mut self,
        _tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let hovered = enabled_segment_at(&self.control.segments, layout.bounds(), cursor);
        let previous_hovered = self.hovered;
        let previous_pressed = self.pressed;
        self.hovered = hovered;

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if hovered.is_some() => {
                self.pressed = hovered;
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if self.pressed.is_some() =>
            {
                let pressed = self.pressed.take();
                shell.capture_event();
                if pressed == hovered
                    && let Some(index) = pressed
                {
                    shell.publish(self.control.segments[index].on_press.clone());
                }
            }
            _ => {}
        }

        renderer.update_glass_interaction(
            self.control.node.id,
            interaction_for_cursor(
                layout.bounds(),
                cursor,
                self.hovered.is_some(),
                self.pressed.is_some(),
            ),
        );

        if previous_hovered != self.hovered || previous_pressed != self.pressed {
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if enabled_segment_at(&self.control.segments, layout.bounds(), cursor).is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

fn draw_segment_content<Message, Renderer>(
    renderer: &mut Renderer,
    segments: &[GlassSegment<Message>],
    chrome: GlassChrome,
    bounds: Rectangle,
    viewport: Rectangle,
) where
    Renderer: advanced::Renderer + advanced::graphics::geometry::Renderer + TextRenderer,
{
    use advanced::graphics::geometry::{Frame, LineCap, LineJoin, Path, Stroke};

    let segment_width = bounds.width / count_as_f32(segments.len());
    renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
        let mut frame = Frame::new(renderer, bounds.size());
        let center_y = bounds.height * 0.5;

        for (index, segment) in segments.iter().enumerate() {
            let center_x = segment_width * (count_as_f32(index) + 0.5);
            let icon_color = iced_color(segment_text_color(segment, chrome));
            let stroke = Stroke::default()
                .with_color(icon_color)
                .with_width(1.8)
                .with_line_cap(LineCap::Round)
                .with_line_join(LineJoin::Round);
            let path = match segment.content {
                GlassSegmentContent::ChevronLeft => Some(Path::new(|path| {
                    path.move_to(iced::Point::new(center_x + 2.5, center_y - 5.0));
                    path.line_to(iced::Point::new(center_x - 2.5, center_y));
                    path.line_to(iced::Point::new(center_x + 2.5, center_y + 5.0));
                })),
                GlassSegmentContent::ChevronRight => Some(Path::new(|path| {
                    path.move_to(iced::Point::new(center_x - 2.5, center_y - 5.0));
                    path.line_to(iced::Point::new(center_x + 2.5, center_y));
                    path.line_to(iced::Point::new(center_x - 2.5, center_y + 5.0));
                })),
                GlassSegmentContent::Glyph(_) => None,
            };
            if let Some(path) = path {
                frame.stroke(&path, stroke);
            }
        }
        renderer.draw_geometry(frame.into_geometry());
    });

    for (index, segment) in segments.iter().enumerate() {
        let GlassSegmentContent::Glyph(content) = &segment.content else {
            continue;
        };
        renderer.fill_text(
            advanced::Text {
                content: content.clone(),
                bounds: Size::new(segment_width, bounds.height),
                size: Pixels(14.0),
                line_height: advanced::text::LineHeight::default(),
                font: renderer.default_font(),
                align_x: advanced::text::Alignment::Center,
                align_y: iced::alignment::Vertical::Center,
                shaping: advanced::text::Shaping::Auto,
                wrapping: advanced::text::Wrapping::None,
            },
            iced::Point::new(
                bounds.x + segment_width * (count_as_f32(index) + 0.5),
                bounds.center_y(),
            ),
            iced_color(segment_text_color(segment, chrome)),
            viewport,
        );
    }
}

fn draw_segment_chrome<Message, Renderer>(
    renderer: &mut Renderer,
    control: &GlassSegmentedControl<Message>,
    bounds: Rectangle,
    hovered: Option<usize>,
    pressed: Option<usize>,
) where
    Renderer: advanced::Renderer + advanced::graphics::geometry::Renderer,
{
    use advanced::graphics::geometry::{Frame, Stroke};

    let outer_style = crate::UiCornerStyle::CONTROL.with_radius(shape_radius(&control.node));
    renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
        let mut frame = Frame::new(renderer, bounds.size());
        let outer = squircle_path(bounds.size(), iced::Point::ORIGIN, outer_style);
        if control.chrome.border.a > 0.0 {
            frame.stroke(
                &outer,
                Stroke::default().with_width(1.0).with_color(iced_color(control.chrome.border)),
            );
        }
        if let Some(index) = hovered {
            let hover_bounds = segment_hover_bounds(
                Rectangle { x: 0.0, y: 0.0, ..bounds },
                control.segments.len(),
                index,
            );
            let overlay = if pressed == Some(index) {
                control.chrome.pressed_overlay
            } else {
                control.chrome.hover_overlay
            };
            let hover = squircle_path(
                hover_bounds.size(),
                iced::Point::new(hover_bounds.x, hover_bounds.y),
                crate::UiCornerStyle::CONTROL.with_radius(hover_bounds.height * 0.5),
            );
            frame.fill(&hover, iced_color(overlay));
        }
        renderer.draw_geometry(frame.into_geometry());
    });
}

fn squircle_path(
    size: Size,
    offset: iced::Point,
    style: crate::UiCornerStyle,
) -> advanced::graphics::geometry::Path {
    let commands = squircle_path_commands(&style.params(size.width, size.height));
    advanced::graphics::geometry::Path::new(move |path| {
        for command in commands.iter().copied() {
            match command {
                PathCommand::MoveTo(point) => {
                    path.move_to(iced::Point::new(offset.x + point.x, offset.y + point.y));
                }
                PathCommand::LineTo(point) => {
                    path.line_to(iced::Point::new(offset.x + point.x, offset.y + point.y));
                }
                PathCommand::CubicTo { c0, c1, to } => path.bezier_curve_to(
                    iced::Point::new(offset.x + c0.x, offset.y + c0.y),
                    iced::Point::new(offset.x + c1.x, offset.y + c1.y),
                    iced::Point::new(offset.x + to.x, offset.y + to.y),
                ),
                PathCommand::Close => path.close(),
            }
        }
    })
}

fn segment_text_color<Message>(
    segment: &GlassSegment<Message>,
    chrome: GlassChrome,
) -> liquid_glass_scene::Color {
    if segment.enabled { chrome.text } else { chrome.disabled_text }
}

fn count_as_f32(value: usize) -> f32 {
    f32::from(u16::try_from(value).expect("segment count was validated during construction"))
}

fn segment_index_at(
    bounds: Rectangle,
    segment_count: usize,
    cursor: mouse::Cursor,
) -> Option<usize> {
    let position = cursor.position_over(bounds)?;
    let segment_width = bounds.width / count_as_f32(segment_count);
    (0..segment_count)
        .find(|index| position.x < bounds.x + segment_width * count_as_f32(index.saturating_add(1)))
}

fn enabled_segment_at<Message>(
    segments: &[GlassSegment<Message>],
    bounds: Rectangle,
    cursor: mouse::Cursor,
) -> Option<usize> {
    segment_index_at(bounds, segments.len(), cursor).filter(|index| segments[*index].enabled)
}

fn divider_touches_hovered(divider_index: usize, hovered: Option<usize>) -> bool {
    hovered.is_some_and(|index| divider_index == index || divider_index == index.saturating_add(1))
}

const SEGMENT_HOVER_INSET: f32 = 4.0;

fn segment_hover_bounds(bounds: Rectangle, segment_count: usize, index: usize) -> Rectangle {
    let segment_width = bounds.width / count_as_f32(segment_count);
    let diameter = (segment_width.min(bounds.height) - SEGMENT_HOVER_INSET * 2.0).max(0.0);
    let segment_center_x = bounds.x + segment_width * (count_as_f32(index) + 0.5);
    Rectangle {
        x: segment_center_x - diameter * 0.5,
        y: bounds.center_y() - diameter * 0.5,
        width: diameter,
        height: diameter,
    }
}

fn iced_color(color: liquid_glass_scene::Color) -> IcedColor {
    IcedColor::from_rgba(color.r, color.g, color.b, color.a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_bridge_uses_widget_layout_size_and_origin() {
        let mut container = GlassContainer::new(GlassId(7), Rect::new(40.0, 24.0, 320.0, 180.0))
            .corner_curve(CornerCurve::Circular);

        let node = container.layout_scene_node(Size::new(960.0, 640.0));

        assert_eq!(node.id, GlassId(7));
        assert_eq!(node.bounds, Rect::new(40.0, 24.0, 320.0, 180.0));
        assert_eq!(node.corner_curve, CornerCurve::Circular);
    }

    #[test]
    fn button_publishes_message_on_click_release() {
        let mut button = GlassButtonWidget {
            button: GlassButton::new(GlassId(8), "Apply", Rect::new(0.0, 0.0, 120.0, 48.0)),
            on_press: 42_u8,
            on_press_start: Some(7_u8),
            on_press_cancel: None,
            on_press_end: Some(8_u8),
        };
        let mut tree = Tree::new(&button as &dyn Widget<u8, (), ()>);
        let limits = layout::Limits::new(Size::ZERO, Size::new(200.0, 100.0));
        let layout_node = <GlassButtonWidget<u8> as Widget<u8, (), ()>>::layout(
            &mut button,
            &mut tree,
            &(),
            &limits,
        );
        let layout = Layout::new(&layout_node);
        let viewport = Rectangle { x: 0.0, y: 0.0, width: 200.0, height: 100.0 };
        let cursor = mouse::Cursor::Available(iced::Point::new(12.0, 12.0));
        let mut messages = Vec::new();
        {
            let mut shell = Shell::new(&mut messages);
            let mut clipboard = iced::advanced::clipboard::Null;
            let press = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));
            <GlassButtonWidget<u8> as Widget<u8, (), ()>>::update(
                &mut button,
                &mut tree,
                &press,
                layout,
                cursor,
                &(),
                &mut clipboard,
                &mut shell,
                &viewport,
            );

            // A published press-start message causes the application view to
            // be rebuilt. The state tree must keep the capture state across
            // that reconciliation so release still reaches this widget.
            let replacement = GlassButtonWidget {
                button: GlassButton::new(GlassId(8), "Apply", Rect::new(0.0, 0.0, 120.0, 48.0)),
                on_press: 42_u8,
                on_press_start: Some(7_u8),
                on_press_cancel: None,
                on_press_end: Some(8_u8),
            };
            tree.diff(&replacement as &dyn Widget<u8, (), ()>);

            let release = Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left));
            <GlassButtonWidget<u8> as Widget<u8, (), ()>>::update(
                &mut button,
                &mut tree,
                &release,
                layout,
                cursor,
                &(),
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }

        assert_eq!(messages, vec![7, 8, 42]);
        assert!(!tree.state.downcast_ref::<GlassButtonState>().pressed);
    }

    #[test]
    fn compact_button_can_use_a_circle_and_builtin_icon() {
        let button = GlassButton::new(GlassId(9), "Back", Rect::new(0.0, 0.0, 36.0, 36.0))
            .shape(GlassShape::Circle)
            .icon(GlassButtonIcon::Back);

        assert_eq!(button.node().shape, GlassShape::Circle);
        assert_eq!(button.icon, Some(GlassButtonIcon::Back));
    }

    #[test]
    fn navigation_control_uses_one_capsule_glass_node() {
        let control = GlassNavigationControl::new(GlassId(12), Rect::new(0.0, 0.0, 72.0, 36.0));

        assert_eq!(control.node().id, GlassId(12));
        assert_eq!(control.node().shape, GlassShape::Capsule);
        assert!((control.node().bounds.width - 72.0).abs() < f32::EPSILON);
    }

    #[test]
    fn navigation_hover_feedback_is_a_centered_circle() {
        let bounds = Rectangle { x: 10.0, y: 20.0, width: 72.0, height: 36.0 };
        let hover = segment_hover_bounds(bounds, 2, 1);

        assert!((hover.width - hover.height).abs() < f32::EPSILON);
        assert!((hover.width - 28.0).abs() < f32::EPSILON);
        assert!((hover.y - bounds.y - SEGMENT_HOVER_INSET).abs() < f32::EPSILON);
        assert!((hover.center_x() - (bounds.x + bounds.width * 0.75)).abs() < f32::EPSILON);
        assert!((hover.center_y() - bounds.center_y()).abs() < f32::EPSILON);
    }

    #[test]
    fn segmented_control_accepts_more_than_two_items() {
        let control = GlassSegmentedControl::new(
            GlassId(13),
            Rect::new(0.0, 0.0, 108.0, 36.0),
            [
                GlassSegment::new(GlassSegmentContent::ChevronLeft, 1_u8),
                GlassSegment::new(GlassSegmentContent::glyph("⌂"), 2_u8),
                GlassSegment::new(GlassSegmentContent::ChevronRight, 3_u8),
            ],
        );

        assert_eq!(control.segments().len(), 3);
        assert_eq!(control.segments()[1].content(), &GlassSegmentContent::glyph("⌂"));
    }

    #[test]
    fn disabled_segment_does_not_hover_or_hide_dividers() {
        let segments = [
            GlassSegment::new(GlassSegmentContent::ChevronLeft, 1_u8),
            GlassSegment::new(GlassSegmentContent::ChevronRight, 2_u8).enabled(false),
        ];
        let bounds = Rectangle { x: 0.0, y: 0.0, width: 72.0, height: 36.0 };
        let cursor = mouse::Cursor::Available(iced::Point::new(54.0, 18.0));
        let hovered = enabled_segment_at(&segments, bounds, cursor);

        assert_eq!(hovered, None);
        assert!(!divider_touches_hovered(1, hovered));
    }

    #[test]
    fn glass_panel_creates_with_unique_ids_and_configuration() {
        let panel1 = GlassPanel::<(), (), ()>::new(iced::Element::new(GlassContainer::new(
            GlassId(1),
            Rect::new(0.0, 0.0, 100.0, 50.0),
        )))
        .squircle_radius(24.0)
        .padding(12.0);
        let panel2 = GlassPanel::<(), (), ()>::new(iced::Element::new(GlassContainer::new(
            GlassId(2),
            Rect::new(0.0, 0.0, 100.0, 50.0),
        )))
        .squircle_radius(18.0);

        assert_ne!(panel1.node_id(), panel2.node_id());
        assert_eq!(panel1.node_shape(), &GlassShape::RoundedRect { radius: 24.0 });
        assert_eq!(panel1.node_corner_curve(), CornerCurve::continuous());
        assert_eq!(panel1.node_padding(), Padding::from(12.0));
        assert_eq!(panel2.node_shape(), &GlassShape::RoundedRect { radius: 18.0 });
    }
}
