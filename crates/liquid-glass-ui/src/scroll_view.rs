//! Fluid Apple-style spring-driven scroll container and custom overlay scrollbar.
//!
//! Features Apple logarithmic rubber-banding when dragged beyond bounds,
//! zero-overshoot critically damped bounce-back, interactive gesture hand-off,
//! hover-expansion, and glass overlay compositing.

#![deny(unsafe_code)]

use std::time::{Duration, Instant};

use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::{self, Clipboard, Layout, Shell, Widget, layout, mouse, renderer};
use iced::{
    Background, Border, Color, Element, Event, Length, Point, Rectangle, Shadow, Size, Vector,
    keyboard, mouse::ScrollDelta, window,
};
use spring_rs::{
    APPLE_RUBBER_BAND_COEFFICIENT, Spring, SpringMotion, apple_rubber_band, inverse_rubber_band,
};

use crate::{GlassForegroundRenderer, UiColorScheme, UiCornerStyle};

/// Configuration parameters for the scrollbar geometry and timing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollbarConfig {
    /// Distance from the container's right edge to the scrollbar rail.
    pub edge_inset: f32,
    /// Distance from the top of the container before the scrollbar starts.
    pub top_inset: f32,
    /// Distance from the bottom of the container where the scrollbar ends.
    pub bottom_inset: f32,
    /// Resting width of the scrollbar thumb in pixels (macOS default is 6px).
    pub thumb_width: f32,
    /// Expanded width of the scrollbar thumb and track when hovered or dragged (macOS measured is 11px).
    pub hover_width: f32,
    /// Hit-test interactive slot width along the edge in pixels.
    pub slot_width: f32,
    /// Multiplier for wheel/trackpad scroll delta (default is 1.0 for authentic native 1:1 pixel mapping).
    pub scroll_factor: f32,
    /// Minimum height of the thumb when content is very long.
    pub min_thumb_height: f32,
    /// Minimum compressed height when stretched into overscroll.
    pub min_compressed_height: f32,
    /// How long the scrollbar remains fully visible before fading out after activity.
    pub hold_duration: Duration,
    /// Duration of the fade-out alpha transition.
    pub fade_duration: Duration,
}

impl Default for ScrollbarConfig {
    fn default() -> Self {
        Self {
            edge_inset: 3.0,
            top_inset: 0.0,
            bottom_inset: 0.0,
            thumb_width: 6.0,
            hover_width: 11.0,
            slot_width: 15.0,
            scroll_factor: 1.0,
            min_thumb_height: 36.0,
            min_compressed_height: 18.0,
            hold_duration: Duration::from_millis(650),
            fade_duration: Duration::from_millis(250),
        }
    }
}

impl ScrollbarConfig {
    /// Creates a sidebar-specific scrollbar configuration that accounts for search overlays
    /// and window insets.
    #[must_use]
    pub const fn sidebar(top_inset: f32, bottom_inset: f32, edge_inset: f32) -> Self {
        Self {
            edge_inset,
            top_inset,
            bottom_inset,
            thumb_width: 6.0,
            hover_width: 11.0,
            slot_width: 15.0,
            scroll_factor: 1.0,
            min_thumb_height: 36.0,
            min_compressed_height: 18.0,
            hold_duration: Duration::from_millis(650),
            fade_duration: Duration::from_millis(250),
        }
    }

    /// Sets the scroll delta multiplier.
    #[must_use]
    pub const fn with_scroll_factor(mut self, scroll_factor: f32) -> Self {
        self.scroll_factor = scroll_factor;
        self
    }

    /// Sets the expanded width of the scrollbar thumb and track when hovered or dragged.
    #[must_use]
    pub const fn with_hover_width(mut self, hover_width: f32) -> Self {
        self.hover_width = hover_width;
        self
    }

    /// Sets the top edge inset.
    #[must_use]
    pub const fn with_top_inset(mut self, top_inset: f32) -> Self {
        self.top_inset = top_inset;
        self
    }

    /// Sets the bottom edge inset.
    #[must_use]
    pub const fn with_bottom_inset(mut self, bottom_inset: f32) -> Self {
        self.bottom_inset = bottom_inset;
        self
    }

    /// Sets the right edge inset.
    #[must_use]
    pub const fn with_edge_inset(mut self, edge_inset: f32) -> Self {
        self.edge_inset = edge_inset;
        self
    }
}

// Measured composited light-mode target values for macOS native contrast:
const LIGHT_THUMB_TARGET: f32 = 58.0 / 255.0;
const LIGHT_TRACK_TARGET: f32 = 134.0 / 255.0;

/// Calculates scrollbar opacity based on activity timestamp and fade curve.
#[must_use]
pub fn scrollbar_opacity(
    last_activity: Option<Instant>,
    now: Instant,
    hold: Duration,
    fade: Duration,
) -> f32 {
    let Some(last_activity) = last_activity else {
        return 0.0;
    };
    let elapsed = now.saturating_duration_since(last_activity);
    if elapsed <= hold {
        1.0
    } else {
        1.0 - elapsed.saturating_sub(hold).as_secs_f32() / fade.as_secs_f32()
    }
    .clamp(0.0, 1.0)
}

/// The maximum asymptotic stretch limit for Apple logarithmic rubber-banding.
/// Keeps the scroll elastic resistance firm, compact, and responsive (44px - 64px),
/// preventing over-stretching or loose, floppy sensations.
#[must_use]
fn overscroll_dimension(viewport_height: f32) -> f32 {
    (viewport_height * 0.10).clamp(44.0, 64.0)
}

/// Computes Apple-standard logarithmic rubber-banded target displacement.
fn calculate_overscroll_target(raw_offset: f32, range: f32, viewport_height: f32) -> f32 {
    let dimension = overscroll_dimension(viewport_height);
    if raw_offset < 0.0 {
        // Dragging down past the top edge: raw_offset is negative.
        // apple_rubber_band handles sign preservation.
        apple_rubber_band(raw_offset, dimension)
    } else if raw_offset > range {
        // Dragging up past the bottom edge: raw_offset exceeds range.
        range + apple_rubber_band(raw_offset - range, dimension)
    } else {
        raw_offset
    }
}

/// Converts a visual displacement back to raw unconstrained gesture coordinate.
fn visual_to_raw_offset(offset: f32, range: f32, viewport_height: f32) -> f32 {
    let dimension = overscroll_dimension(viewport_height);
    if offset < 0.0 {
        inverse_rubber_band(offset, dimension, APPLE_RUBBER_BAND_COEFFICIENT)
    } else if offset > range {
        range + inverse_rubber_band(offset - range, dimension, APPLE_RUBBER_BAND_COEFFICIENT)
    } else {
        offset
    }
}

/// Current operational mode of the spring scroll state machine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollMode {
    /// Resting at equilibrium within bounds; animation loop is dormant.
    Idle,
    /// Actively tracked user input (wheel, trackpad, touch): direct 1:1 manipulation
    /// with zero latency. Rubber-band resistance is computed immediately on drag.
    Interacting,
    /// Inertial coasting / momentum flick within content bounds.
    Decelerating {
        /// Current velocity in pixels per second.
        velocity: f32,
    },
    /// Critically damped Apple spring snapping back from overscroll.
    Bouncing,
}

/// Internal state for spring-driven scrolling and scrollbar fading.
#[derive(Debug)]
pub struct SpringScrollState {
    /// Visual content displacement in pixels (rendered position).
    pub offset: f32,
    /// Unconstrained raw gesture coordinate before rubber-band resistance.
    pub raw_offset: f32,
    /// Instantaneous gesture velocity in pixels per second.
    pub velocity: f32,
    /// Current mode of the scroll physics engine.
    pub mode: ScrollMode,
    /// Critically damped spring for boundary snap-back.
    pub offset_spring: SpringMotion,
    /// Timestamp of the last processed animation frame.
    pub last_frame: Option<Instant>,
    /// Timestamp of the last received user gesture input.
    pub last_input_time: Option<Instant>,
    /// Timestamp of the initial outward push event when entering overscroll.
    pub overscroll_entry_time: Option<Instant>,
    pub viewport_height: f32,
    pub content_height: f32,
    pub grabbed_at: Option<f32>,
    pub opacity: f32,
    pub last_activity: Option<Instant>,
    pub keyboard_modifiers: keyboard::Modifiers,
}

impl Default for SpringScrollState {
    fn default() -> Self {
        Self {
            offset: 0.0,
            raw_offset: 0.0,
            velocity: 0.0,
            mode: ScrollMode::Idle,
            offset_spring: SpringMotion::new(0.0, 0.0, Spring::smooth_custom(0.35, 0.0)),
            last_frame: None,
            last_input_time: None,
            overscroll_entry_time: None,
            viewport_height: 0.0,
            content_height: 0.0,
            grabbed_at: None,
            opacity: 0.0,
            last_activity: None,
            keyboard_modifiers: keyboard::Modifiers::default(),
        }
    }
}

impl SpringScrollState {
    /// Returns the maximum scrollable travel range.
    #[must_use]
    pub fn scroll_range(&self) -> f32 {
        (self.content_height - self.viewport_height).max(0.0)
    }

    /// Directly sets the offset clamped within `[0, scroll_range]`.
    pub fn set_offset(&mut self, offset: f32) -> bool {
        let next = offset.clamp(0.0, self.scroll_range());
        if (next - self.offset).abs() <= f32::EPSILON {
            false
        } else {
            self.offset = next;
            self.raw_offset = next;
            self.velocity = 0.0;
            self.mode = ScrollMode::Idle;
            self.overscroll_entry_time = None;
            self.offset_spring.set_value(next);
            self.offset_spring.retarget(next);
            true
        }
    }

    /// Immediately applies a user scroll delta with 1:1 direct tracking and zero latency.
    ///
    /// If beyond boundaries, Apple's logarithmic rubber-banding is calculated instantly
    /// without spring lag. Also measures gesture velocity for subsequent fluid hand-off.
    pub fn apply_scroll_delta(&mut self, delta_offset: f32, now: Instant) -> bool {
        let range = self.scroll_range();
        let is_overscrolled = self.offset < 0.0 || self.offset > range;
        let pushing_further_out = (self.offset < 0.0 && delta_offset < 0.0)
            || (self.offset > range && delta_offset > 0.0);

        // 1. ABSOLUTE FUSE FOR MACOS RESIDUAL MOMENTUM IN BOUNCING MODE:
        // When already in Bouncing mode, any incoming wheel/trackpad events that continue
        // pushing further into overscroll MUST NOT interrupt the bounce-back spring!
        // macOS trackpad synthesizes ~1.0s of momentum events after user lifts fingers;
        // allowing them to reset mode to Interacting holds the view hostage at the edge for 1 full second.
        if self.mode == ScrollMode::Bouncing && is_overscrolled && pushing_further_out {
            return false;
        }

        // 2. Measure instantaneous velocity (px/s)
        if let Some(last_time) = self.last_input_time {
            let dt = now.saturating_duration_since(last_time).as_secs_f32();
            if dt > 0.001 && dt < 0.08 {
                let instant_vel = (delta_offset / dt).clamp(-8000.0, 8000.0);
                self.velocity = 0.6 * instant_vel + 0.4 * self.velocity;
            } else if dt >= 0.08 {
                // New gesture stroke after brief pause
                self.velocity = (delta_offset / dt.min(0.04)).clamp(-8000.0, 8000.0);
            }
        } else {
            self.velocity = (delta_offset / 0.016).clamp(-8000.0, 8000.0);
        }

        // 3. OVERSCROLL MOMENTUM FUSE & AUTONOMOUS BOUNCE HAND-OFF:
        // If in overscroll and incoming gesture continues pushing further outward,
        // measure how long this outward pushing streak has lasted.
        // A direct physical finger drag reaches Apple rubber-band saturation in ~25-30ms;
        // any continuous outward stream beyond that is macOS-synthesized momentum runoff
        // after finger release. We fuse it immediately and initiate snap-back!
        if is_overscrolled && pushing_further_out {
            let entry = *self.overscroll_entry_time.get_or_insert(now);
            let outward_duration = now.saturating_duration_since(entry);

            let next_raw = self.raw_offset + delta_offset;
            let projected_offset =
                calculate_overscroll_target(next_raw, range, self.viewport_height);
            let stretch_increment = (projected_offset - self.offset).abs();

            if outward_duration >= Duration::from_millis(25) || stretch_increment < 0.25 {
                // Instantly hand off to critically damped bounce-back spring!
                // Do NOT refresh last_input_time or allow raw_offset to stretch further!
                if self.mode != ScrollMode::Bouncing {
                    self.transition_from_interacting();
                }
                return false;
            }
            self.last_input_time = Some(now);
        } else {
            self.overscroll_entry_time = None;
            self.last_input_time = Some(now);
        }

        // 4. Seamlessly sync raw_offset from current visual offset if we were in autonomous mode
        if self.mode != ScrollMode::Interacting {
            self.raw_offset = visual_to_raw_offset(self.offset, range, self.viewport_height);
            self.mode = ScrollMode::Interacting;
        }

        // 5. Accumulate raw unconstrained displacement and compute rubber-banded target immediately
        self.raw_offset += delta_offset;
        let new_offset = calculate_overscroll_target(self.raw_offset, range, self.viewport_height);
        let changed = (new_offset - self.offset).abs() > f32::EPSILON;
        self.offset = new_offset;

        // Keep spring position and velocity synced for zero-glitch handoff
        self.offset_spring.set_value(self.offset);
        self.offset_spring.set_velocity(self.velocity);

        changed
    }

    /// Retargets the scroll position with Apple rubber-banding if dragging past boundaries.
    ///
    /// Preserved for API compatibility; delegates directly to `apply_scroll_delta`.
    pub fn retarget_scroll(&mut self, target_offset: f32, now: Instant) -> bool {
        let delta = target_offset - self.offset;
        self.apply_scroll_delta(delta, now)
    }

    /// Transitions from active user interaction to autonomous physics
    /// (either overscroll spring snap-back or momentum coasting).
    fn transition_from_interacting(&mut self) {
        let range = self.scroll_range();
        let release_velocity = self.velocity;

        if self.offset < 0.0 {
            // Overscroll top: snap back with firm, crisp critical damping (0.24s).
            // Zero out outward-directed momentum so the bounce starts INSTANTLY with no delay or backward drift.
            let initial_velocity = release_velocity.clamp(0.0, 250.0);
            self.offset_spring = SpringMotion::with_initial_velocity(
                self.offset,
                0.0,
                initial_velocity,
                Spring::smooth_custom(0.24, 0.0),
            );
            self.mode = ScrollMode::Bouncing;
        } else if self.offset > range {
            // Overscroll bottom: snap back with firm, crisp critical damping (0.24s).
            // Zero out outward-directed momentum so the bounce starts INSTANTLY with no delay or backward drift.
            let initial_velocity = release_velocity.clamp(-250.0, 0.0);
            self.offset_spring = SpringMotion::with_initial_velocity(
                self.offset,
                range,
                initial_velocity,
                Spring::smooth_custom(0.24, 0.0),
            );
            self.mode = ScrollMode::Bouncing;
        } else if release_velocity.abs() >= 60.0 {
            // In bounds with significant momentum: initiate inertial deceleration
            self.mode = ScrollMode::Decelerating { velocity: release_velocity };
        } else {
            // Settled at rest within bounds
            self.velocity = 0.0;
            self.mode = ScrollMode::Idle;
        }
    }

    /// Advances the spring physics simulation for one frame delta.
    /// Returns `true` if the simulation is still active and another frame must be scheduled.
    pub fn advance_spring(&mut self, now: Instant) -> bool {
        let dt = self
            .last_frame
            .map_or(1.0 / 60.0, |last| now.saturating_duration_since(last).as_secs_f32())
            .clamp(1.0 / 240.0, 0.05);
        self.last_frame = Some(now);

        let range = self.scroll_range();

        match self.mode {
            ScrollMode::Interacting => {
                let is_overscrolled = self.offset < 0.0 || self.offset > range;
                // In overscroll, the elastic tension is high; hand off to bounce-back spring
                // within 1 frame (16ms) after input ceases instead of lagging behind.
                let idle_threshold = if is_overscrolled {
                    Duration::from_millis(16)
                } else {
                    Duration::from_millis(45)
                };

                let is_idle = self
                    .last_input_time
                    .is_none_or(|t| now.saturating_duration_since(t) >= idle_threshold);
                if is_idle {
                    self.transition_from_interacting();
                    self.mode != ScrollMode::Idle
                } else {
                    true
                }
            }
            ScrollMode::Decelerating { mut velocity } => {
                // iOS-standard deceleration rate (0.997 per millisecond)
                let friction: f32 = 0.997;
                let decay = friction.powf(dt * 1000.0);
                velocity *= decay;

                let next = self.offset + velocity * dt;
                let dim = overscroll_dimension(self.viewport_height);

                if next < 0.0 {
                    // Coasted into top boundary: dissipate kinetic energy on impact
                    let start_offset = apple_rubber_band(next, dim);
                    self.offset = start_offset;
                    self.offset_spring = SpringMotion::with_initial_velocity(
                        start_offset,
                        0.0,
                        0.0,
                        Spring::smooth_custom(0.24, 0.0),
                    );
                    self.mode = ScrollMode::Bouncing;
                    true
                } else if next > range {
                    // Coasted into bottom boundary: dissipate kinetic energy on impact
                    let start_offset = range + apple_rubber_band(next - range, dim);
                    self.offset = start_offset;
                    self.offset_spring = SpringMotion::with_initial_velocity(
                        start_offset,
                        range,
                        0.0,
                        Spring::smooth_custom(0.24, 0.0),
                    );
                    self.mode = ScrollMode::Bouncing;
                    true
                } else {
                    self.offset = next;
                    self.raw_offset = next;
                    if velocity.abs() < 15.0 {
                        self.velocity = 0.0;
                        self.mode = ScrollMode::Idle;
                        false
                    } else {
                        self.mode = ScrollMode::Decelerating { velocity };
                        true
                    }
                }
            }
            ScrollMode::Bouncing => {
                let next = self.offset_spring.step(dt);
                // Clamp within maximum physical boundary limits
                let max_stretch = overscroll_dimension(self.viewport_height) * 1.2;
                self.offset = next.clamp(-max_stretch, range + max_stretch);

                if self.offset_spring.is_settled(0.1, 1.0) {
                    self.offset = self.offset_spring.target();
                    self.raw_offset = self.offset;
                    self.velocity = 0.0;
                    self.mode = ScrollMode::Idle;
                    self.overscroll_entry_time = None;
                    false
                } else {
                    true
                }
            }
            ScrollMode::Idle => false,
        }
    }
}

/// A container widget that provides Apple-style spring physics scrolling,
/// logarithmic rubber-banding, and an overlay scrollbar.
pub struct SpringScrollView<'a, Message, Theme, Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    color_scheme: UiColorScheme,
    config: ScrollbarConfig,
}

impl<Message, Theme, Renderer> std::fmt::Debug for SpringScrollView<'_, Message, Theme, Renderer> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SpringScrollView")
            .field("color_scheme", &self.color_scheme)
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl<'a, Message, Theme, Renderer> SpringScrollView<'a, Message, Theme, Renderer> {
    /// Creates a new `SpringScrollView` wrapping the given content.
    #[must_use]
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        color_scheme: UiColorScheme,
    ) -> Self {
        Self { content: content.into(), color_scheme, config: ScrollbarConfig::default() }
    }

    /// Customizes the scrollbar geometry configuration.
    #[must_use]
    pub fn config(mut self, config: ScrollbarConfig) -> Self {
        self.config = config;
        self
    }

    /// Computes the scrollbar bounding box within the container.
    #[must_use]
    pub fn scrollbar_bounds(&self, bounds: Rectangle) -> Rectangle {
        let top = self.config.top_inset.min(bounds.height);
        Rectangle::new(
            Point::new(0.0, top),
            Size::new(
                (bounds.width - self.config.edge_inset).max(0.0),
                (bounds.height - top - self.config.bottom_inset).max(0.0),
            ),
        )
    }

    /// Computes thumb bounds using the configured resting thumb width.
    #[must_use]
    pub fn thumb_bounds(&self, state: &SpringScrollState, bounds: Rectangle) -> Option<Rectangle> {
        self.thumb_bounds_with_width(state, bounds, self.config.thumb_width)
    }

    /// Computes thumb bounds for a given thumb width.
    #[must_use]
    pub fn thumb_bounds_with_width(
        &self,
        state: &SpringScrollState,
        bounds: Rectangle,
        thumb_width: f32,
    ) -> Option<Rectangle> {
        if state.scroll_range() <= f32::EPSILON || bounds.height <= 0.0 {
            return None;
        }

        let scrollbar = self.scrollbar_bounds(bounds);
        let visible_ratio = (state.viewport_height / state.content_height).clamp(0.0, 1.0);
        let base_height = (scrollbar.height * visible_ratio)
            .max(self.config.min_thumb_height)
            .min(scrollbar.height);

        let overscroll = if state.offset < 0.0 {
            -state.offset
        } else {
            (state.offset - state.scroll_range()).max(0.0)
        };

        // Smoothly squish the thumb when rubber-banding past the boundaries
        let max_squish = overscroll_dimension(state.viewport_height);
        let squish_progress = (overscroll / max_squish).clamp(0.0, 1.0);
        let compression = 1.0 - 0.70 * squish_progress;
        let height = (base_height * compression).max(self.config.min_compressed_height);

        let travel = (scrollbar.height - height).max(0.0);
        let relative_offset = (state.offset / state.scroll_range()).clamp(0.0, 1.0);
        let y = if state.offset < 0.0 {
            scrollbar.y
        } else if state.offset > state.scroll_range() {
            scrollbar.y + scrollbar.height - height
        } else {
            scrollbar.y + travel * relative_offset
        };

        Some(Rectangle::new(
            Point::new(scrollbar.x + scrollbar.width - thumb_width, y),
            Size::new(thumb_width, height),
        ))
    }

    /// Calculates the content offset requested by dragging or clicking at `pointer_y`.
    #[must_use]
    pub fn requested_offset(
        &self,
        state: &SpringScrollState,
        bounds: Rectangle,
        pointer_y: f32,
        grabbed_at: f32,
    ) -> f32 {
        let Some(thumb) = self.thumb_bounds_with_width(state, bounds, self.config.thumb_width)
        else {
            return 0.0;
        };
        let scrollbar = self.scrollbar_bounds(bounds);
        let travel = scrollbar.height - thumb.height;
        if travel <= f32::EPSILON {
            0.0
        } else {
            ((pointer_y - scrollbar.y - grabbed_at) / travel).clamp(0.0, 1.0) * state.scroll_range()
        }
    }

    fn mark_activity(state: &mut SpringScrollState, shell: &mut Shell<'_, Message>) {
        state.last_activity = Some(Instant::now());
        state.opacity = 1.0;
        shell.request_redraw();
    }

    fn update_fade(
        &self,
        state: &mut SpringScrollState,
        now: Instant,
        shell: &mut Shell<'_, Message>,
    ) {
        let opacity = scrollbar_opacity(
            state.last_activity,
            now,
            self.config.hold_duration,
            self.config.fade_duration,
        );
        if (opacity - state.opacity).abs() > f32::EPSILON {
            state.opacity = opacity;
        }
        if opacity > 0.0 {
            shell.request_redraw();
        } else {
            state.last_activity = None;
        }
    }

    fn content_cursor(cursor: mouse::Cursor, bounds: Rectangle, offset: f32) -> mouse::Cursor {
        if cursor.position_over(bounds).is_some() {
            cursor + Vector::new(0.0, offset)
        } else {
            cursor.levitate() + Vector::new(0.0, offset)
        }
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for SpringScrollView<'_, Message, Theme, Renderer>
where
    Renderer: advanced::Renderer + GlassForegroundRenderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<SpringScrollState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(SpringScrollState::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(self.content.as_widget())]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits.resolve(Length::Fill, Length::Fill, Size::ZERO);
        let content_limits =
            layout::Limits::new(Size::new(size.width, 0.0), Size::new(size.width, f32::INFINITY));
        let content =
            self.content.as_widget_mut().layout(&mut tree.children[0], renderer, &content_limits);
        let state = tree.state.downcast_mut::<SpringScrollState>();
        state.viewport_height = size.height;
        state.content_height = content.size().height;

        let max_displacement = overscroll_dimension(state.viewport_height) * 1.5;
        state.offset =
            state.offset.clamp(-max_displacement, state.scroll_range() + max_displacement);
        layout::Node::with_children(size, vec![content])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn advanced::widget::Operation,
    ) {
        let content_layout = layout.child(0);
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            content_layout,
            renderer,
            operation,
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
        let state = tree.state.downcast_mut::<SpringScrollState>();
        let bounds = layout.bounds();

        match event {
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                state.keyboard_modifiers = *modifiers;
            }
            Event::Window(window::Event::RedrawRequested(now)) => {
                if state.advance_spring(*now) {
                    shell.request_redraw();
                }
                self.update_fade(state, *now, shell);
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta })
                if cursor.position_over(bounds).is_some() =>
            {
                let factor = self.config.scroll_factor;
                let (x, y) = match delta {
                    ScrollDelta::Lines { x, y } => (*x * 60.0 * factor, *y * 60.0 * factor),
                    ScrollDelta::Pixels { x, y } => (*x * factor, *y * factor),
                };
                let movement = if state.keyboard_modifiers.shift() { x } else { y };
                state.apply_scroll_delta(-movement, Instant::now());
                Self::mark_activity(state, shell);
                shell.capture_event();
                return;
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if state.opacity > 0.0 || state.scroll_range() > f32::EPSILON =>
            {
                if let Some(pointer) = cursor.position_in(bounds) {
                    let scrollbar = self.scrollbar_bounds(bounds);
                    let in_slot = pointer.x >= bounds.width - self.config.slot_width
                        && pointer.y >= scrollbar.y
                        && pointer.y <= scrollbar.y + scrollbar.height;
                    if in_slot {
                        if let Some(thumb) =
                            self.thumb_bounds_with_width(state, bounds, self.config.thumb_width)
                        {
                            let grabbed_at = if thumb.contains(pointer) {
                                pointer.y - thumb.y
                            } else {
                                thumb.height * 0.5
                            };
                            state.grabbed_at = Some(grabbed_at);
                            state.set_offset(
                                self.requested_offset(state, bounds, pointer.y, grabbed_at),
                            );
                            Self::mark_activity(state, shell);
                            shell.capture_event();
                            return;
                        }
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(grabbed_at) = state.grabbed_at {
                    if let Some(pointer) = cursor.position_in(bounds) {
                        let next = self.requested_offset(state, bounds, pointer.y, grabbed_at);
                        if state.set_offset(next) {
                            Self::mark_activity(state, shell);
                        }
                        shell.capture_event();
                        return;
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.grabbed_at.take().is_some() =>
            {
                state.mode = ScrollMode::Idle;
                state.velocity = 0.0;
                shell.capture_event();
                return;
            }
            _ => {}
        }

        let content_layout = layout.child(0);
        let content_cursor = Self::content_cursor(cursor, bounds, state.offset);
        let content_viewport = Rectangle {
            x: bounds.x,
            y: bounds.y + state.offset,
            width: bounds.width,
            height: bounds.height,
        };
        let child_viewport = content_viewport.intersection(viewport).unwrap_or_default();
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            content_layout,
            content_cursor,
            renderer,
            clipboard,
            shell,
            &child_viewport,
        );
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
        let state = tree.state.downcast_ref::<SpringScrollState>();
        let bounds = layout.bounds();
        let Some(visible_bounds) = bounds.intersection(viewport) else {
            return;
        };
        let content_layout = layout.child(0);
        let content_cursor = Self::content_cursor(cursor, bounds, state.offset);
        let content_viewport = Rectangle {
            x: visible_bounds.x,
            y: visible_bounds.y + state.offset,
            width: visible_bounds.width,
            height: visible_bounds.height,
        };

        // Render content translated with clipping layer
        renderer.with_layer(visible_bounds, |renderer| {
            renderer.with_translation(Vector::new(0.0, -state.offset), |renderer| {
                self.content.as_widget().draw(
                    &tree.children[0],
                    renderer,
                    theme,
                    style,
                    content_layout,
                    content_cursor,
                    &content_viewport,
                );
            });
        });

        let scrollbar = self.scrollbar_bounds(bounds);
        let pointer = cursor.position_in(bounds);
        let scrollbar_hovered = state.scroll_range() > f32::EPSILON
            && pointer.is_some_and(|position| {
                position.x >= bounds.width - self.config.slot_width
                    && position.y >= scrollbar.y
                    && position.y <= scrollbar.y + scrollbar.height
            });
        let expanded = scrollbar_hovered || state.grabbed_at.is_some();
        let thumb_width = if expanded { self.config.hover_width } else { self.config.thumb_width };
        let track_width = thumb_width;
        let visual_opacity = if expanded { 1.0 } else { state.opacity };

        if visual_opacity <= 0.0 {
            return;
        }
        let Some(thumb) = self.thumb_bounds_with_width(state, bounds, thumb_width) else {
            return;
        };

        // The scrollbar is drawn onto the compositor's post-glass overlay layer
        renderer.begin_glass_overlay();
        if scrollbar_hovered || state.grabbed_at.is_some() {
            let track = Rectangle::new(
                Point::new(
                    bounds.x + scrollbar.x + scrollbar.width - track_width,
                    bounds.y + scrollbar.y,
                ),
                Size::new(track_width, scrollbar.height),
            );
            let track_color = match self.color_scheme {
                UiColorScheme::Light => Color::from_rgb8(0, 0, 0)
                    .scale_alpha((1.0 - LIGHT_TRACK_TARGET) * visual_opacity),
                UiColorScheme::Dark => Color::from_rgba(0.92, 0.92, 0.94, 0.20 * visual_opacity),
            };
            renderer.fill_quad(
                renderer::Quad {
                    bounds: track,
                    border: Border::default().rounded(track_width * 0.5),
                    shadow: Shadow::default(),
                    snap: false,
                },
                Background::Color(track_color),
            );
        }

        let hovered = pointer.is_some_and(|position| thumb.contains(position));
        let emphasis = if state.grabbed_at.is_some() {
            0.62
        } else if hovered {
            0.48
        } else {
            0.34
        };
        let color = match self.color_scheme {
            UiColorScheme::Light => {
                Color::from_rgb8(0, 0, 0).scale_alpha((1.0 - LIGHT_THUMB_TARGET) * visual_opacity)
            }
            UiColorScheme::Dark => {
                Color::from_rgba(0.92, 0.92, 0.94, (emphasis + 0.08) * visual_opacity)
            }
        };
        let absolute_thumb =
            Rectangle::new(Point::new(bounds.x + thumb.x, bounds.y + thumb.y), thumb.size());
        renderer.fill_quad(
            renderer::Quad {
                bounds: absolute_thumb,
                border: Border::default()
                    .rounded(UiCornerStyle::SCROLLBAR.with_radius(thumb.width * 0.5).radius()),
                shadow: Shadow::default(),
                snap: false,
            },
            Background::Color(color),
        );
        renderer.end_glass_overlay();
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<SpringScrollState>();
        if state.grabbed_at.is_some() {
            return mouse::Interaction::Grabbing;
        }
        let bounds = layout.bounds();
        if state.opacity > 0.0 {
            if let Some(pointer) = cursor.position_in(bounds) {
                let scrollbar = self.scrollbar_bounds(bounds);
                if pointer.x >= bounds.width - self.config.slot_width
                    && pointer.y >= scrollbar.y
                    && pointer.y <= scrollbar.y + scrollbar.height
                {
                    return mouse::Interaction::Grab;
                }
            }
        }
        let content_cursor = Self::content_cursor(cursor, bounds, state.offset);
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.child(0),
            content_cursor,
            viewport,
            renderer,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<SpringScrollView<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: advanced::Renderer + GlassForegroundRenderer + 'a,
{
    fn from(scroll_view: SpringScrollView<'a, Message, Theme, Renderer>) -> Self {
        Element::new(scroll_view)
    }
}

/// Helper function to create an Apple-style spring-driven scroll view.
pub fn spring_scroll_view<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    color_scheme: UiColorScheme,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: advanced::Renderer + GlassForegroundRenderer + 'a,
{
    Element::new(SpringScrollView::new(content, color_scheme))
}

/// Helper function to create an Apple-style spring-driven scroll view with custom scrollbar configuration.
pub fn spring_scroll_view_with_config<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    color_scheme: UiColorScheme,
    config: ScrollbarConfig,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: advanced::Renderer + GlassForegroundRenderer + 'a,
{
    Element::new(SpringScrollView::new(content, color_scheme).config(config))
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use iced::widget::space;

    #[test]
    fn scrollbar_width_resting_is_6px_and_expanded_is_11px() {
        let config = ScrollbarConfig::default();
        assert_eq!(config.thumb_width, 6.0);
        assert_eq!(config.hover_width, 11.0);

        let sidebar_config = ScrollbarConfig::sidebar(30.0, 3.0, 3.0);
        assert_eq!(sidebar_config.thumb_width, 6.0);
        assert_eq!(sidebar_config.hover_width, 11.0);

        let view = SpringScrollView::<(), iced::Theme, ()>::new(space(), UiColorScheme::Light);
        let state = SpringScrollState {
            offset: 0.0,
            viewport_height: 500.0,
            content_height: 1000.0,
            ..SpringScrollState::default()
        };
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(200.0, 500.0));

        let resting_thumb = view.thumb_bounds(&state, bounds).expect("has thumb");
        assert_eq!(resting_thumb.width, 6.0);

        let expanded_thumb = view
            .thumb_bounds_with_width(&state, bounds, view.config.hover_width)
            .expect("has thumb");
        assert_eq!(expanded_thumb.width, 11.0);
    }

    #[test]
    fn direct_tracking_in_bounds_is_one_to_one_zero_latency() {
        let mut state = SpringScrollState {
            viewport_height: 400.0,
            content_height: 1200.0,
            ..SpringScrollState::default()
        };
        assert_eq!(state.scroll_range(), 800.0);

        let now = Instant::now();
        // Move downwards by 120px: content moves upwards, offset increases by 120px
        state.apply_scroll_delta(120.0, now);
        assert_eq!(state.offset, 120.0);
        assert_eq!(state.mode, ScrollMode::Interacting);

        // Another move of 80px immediately sets offset to 200px (100% 1:1 zero lag)
        state.apply_scroll_delta(80.0, now + Duration::from_millis(16));
        assert_eq!(state.offset, 200.0);
    }

    #[test]
    fn overscroll_applies_instant_apple_rubber_band_resistance() {
        let mut state = SpringScrollState {
            viewport_height: 500.0,
            content_height: 1000.0,
            ..SpringScrollState::default()
        };

        let now = Instant::now();
        // Drag up past top boundary: raw_offset becomes negative
        state.apply_scroll_delta(-100.0, now);
        // Instant visual displacement is dampened by apple_rubber_band, strictly < 0 and > -100
        assert!(state.offset < 0.0);
        assert!(state.offset > -100.0);
        // Expected value: apple_rubber_band(-100, overscroll_dimension(500.0))
        let dim = overscroll_dimension(500.0);
        let expected = apple_rubber_band(-100.0, dim);
        assert!((state.offset - expected).abs() < 1e-3);
    }

    #[test]
    fn overscroll_release_triggers_spring_bounce_back() {
        let mut state = SpringScrollState {
            viewport_height: 500.0,
            content_height: 1000.0,
            ..SpringScrollState::default()
        };

        let now = Instant::now();
        // Drag 100px past the top edge
        state.apply_scroll_delta(-100.0, now);
        assert!(state.offset < 0.0);
        assert_eq!(state.mode, ScrollMode::Interacting);

        // Simulate frame after 35ms (exceeds 28ms crisp overscroll idle threshold)
        let frame1 = now + Duration::from_millis(35);
        let active = state.advance_spring(frame1);
        assert!(active);
        assert_eq!(state.mode, ScrollMode::Bouncing);

        // Step simulation forward until critically damped spring settles back to 0.0
        let mut t = frame1;
        for _ in 0..100 {
            t += Duration::from_millis(16);
            if !state.advance_spring(t) {
                break;
            }
        }
        assert_eq!(state.mode, ScrollMode::Idle);
        assert_eq!(state.offset, 0.0);
    }

    #[test]
    fn high_velocity_coasting_into_boundary_is_safely_absorbed_and_bounces_crisply() {
        let mut state = SpringScrollState {
            offset: 20.0,
            raw_offset: 20.0,
            viewport_height: 500.0,
            content_height: 1000.0,
            mode: ScrollMode::Decelerating { velocity: -2000.0 },
            ..SpringScrollState::default()
        };

        let now = Instant::now();
        // Advance frame: coasting at -2000px/s will slam past top boundary (0.0)
        let frame1 = now + Duration::from_millis(16);
        let active = state.advance_spring(frame1);
        assert!(active);
        assert_eq!(state.mode, ScrollMode::Bouncing);
        // Verify displacement was firmly absorbed and never flew wildly past reasonable limits
        assert!(state.offset < 0.0);
        assert!(state.offset > -overscroll_dimension(500.0));

        // Advance simulation: quickly and crisply settles to 0.0 within ~200ms
        let mut t = frame1;
        for _ in 0..30 {
            t += Duration::from_millis(16);
            if !state.advance_spring(t) {
                break;
            }
        }
        assert_eq!(state.mode, ScrollMode::Idle);
        assert_eq!(state.offset, 0.0);
    }

    #[test]
    fn continuous_macos_momentum_stream_cannot_hold_overscroll_hostage() {
        let mut state = SpringScrollState {
            viewport_height: 500.0,
            content_height: 1000.0,
            ..SpringScrollState::default()
        };

        let start = Instant::now();
        // User pulls into overscroll by -40px
        state.apply_scroll_delta(-40.0, start);
        assert!(state.offset < 0.0);

        // Frame after 25ms: transitions into Bouncing mode
        let frame1 = start + Duration::from_millis(25);
        let active = state.advance_spring(frame1);
        assert!(active);
        assert_eq!(state.mode, ScrollMode::Bouncing);

        // Simulate macOS trackpad pumping momentum events for 60 consecutive frames (1 full second)
        let mut t = frame1;
        for _ in 0..60 {
            t += Duration::from_millis(16);
            // System continues pumping outward delta (e.g. -2.0px)
            state.apply_scroll_delta(-2.0, t);
            // Verify mode remained Bouncing and was not kicked back to Interacting!
            assert_ne!(state.mode, ScrollMode::Interacting);
            if !state.advance_spring(t) {
                break;
            }
        }

        // Successfully snapped back to 0.0 without waiting for the 1.0s momentum stream to end
        assert_eq!(state.offset, 0.0);
        assert_eq!(state.mode, ScrollMode::Idle);
    }

    #[test]
    fn high_velocity_in_bounds_flick_triggers_momentum_deceleration() {
        let mut state = SpringScrollState {
            viewport_height: 500.0,
            content_height: 2000.0,
            ..SpringScrollState::default()
        };

        let start = Instant::now();
        // Simulate two rapid scroll events within 16ms (e.g. 100px in 16ms -> ~6000px/s)
        state.apply_scroll_delta(50.0, start);
        state.apply_scroll_delta(50.0, start + Duration::from_millis(16));
        assert!(state.velocity > 500.0);

        // Frame after 75ms of idle: transitions into Decelerating mode
        let frame1 = start + Duration::from_millis(16) + Duration::from_millis(75);
        let active = state.advance_spring(frame1);
        assert!(active);
        match state.mode {
            ScrollMode::Decelerating { velocity } => {
                assert!(velocity > 0.0);
            }
            _ => panic!("Expected Decelerating mode, got {:?}", state.mode),
        }

        // Advance frames until velocity decays and settles
        let mut t = frame1;
        for _ in 0..200 {
            t += Duration::from_millis(16);
            if !state.advance_spring(t) {
                break;
            }
        }
        assert_eq!(state.mode, ScrollMode::Idle);
        assert!(state.offset > 100.0);
    }

    #[test]
    fn continuous_uninterrupted_16ms_momentum_runoff_is_fused_and_bounces_immediately() {
        let mut state = SpringScrollState {
            viewport_height: 500.0,
            content_height: 1000.0,
            ..SpringScrollState::default()
        };

        let start = Instant::now();
        // User pulls into overscroll with an aggressive initial stroke
        state.apply_scroll_delta(-50.0, start);
        assert!(state.offset < 0.0);

        // macOS synthesized momentum stream starts immediately at frame intervals (16ms)
        // without ANY artificial idle pauses!
        let mut t = start;
        let mut frames_until_idle = 0;
        // Pumping 60 frames of decreasing momentum deltas from -30px down to -1px (1 full second)
        for i in 0..60 {
            t += Duration::from_millis(16);
            let simulated_delta = -30.0 * (0.95_f32).powi(i);
            state.apply_scroll_delta(simulated_delta, t);
            let active = state.advance_spring(t);
            if !active {
                frames_until_idle = i;
                break;
            }
        }

        // Must settle to 0.0 in well under 25 frames (~400ms), NOT waiting for the 60 frames (1.0s)!
        assert_eq!(state.offset, 0.0);
        assert_eq!(state.mode, ScrollMode::Idle);
        assert!(frames_until_idle > 0 && frames_until_idle < 30);
    }

    #[test]
    fn continuous_uninterrupted_bottom_overscroll_momentum_is_fused_and_bounces_immediately() {
        let mut state = SpringScrollState {
            viewport_height: 500.0,
            content_height: 1000.0, // range = 500.0
            ..SpringScrollState::default()
        };
        state.set_offset(500.0);
        assert_eq!(state.offset, 500.0);

        let start = Instant::now();
        // User pulls beyond bottom boundary
        state.apply_scroll_delta(50.0, start);
        assert!(state.offset > 500.0);

        // Continuous 16ms momentum stream pumping outward deltas at the bottom edge
        let mut t = start;
        let mut frames_until_idle = 0;
        for i in 0..60 {
            t += Duration::from_millis(16);
            let simulated_delta = 30.0 * (0.95_f32).powi(i);
            state.apply_scroll_delta(simulated_delta, t);
            let active = state.advance_spring(t);
            if !active {
                frames_until_idle = i;
                break;
            }
        }

        // Must settle back to range (500.0) crisply without 1.0s stall
        assert_eq!(state.offset, 500.0);
        assert_eq!(state.mode, ScrollMode::Idle);
        assert!(frames_until_idle > 0 && frames_until_idle < 30);
    }
}
