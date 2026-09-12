//! Reusable window semantics for Iced applications with custom chrome.
//!
//! The visual titlebar and its controls should only emit a [`WindowCommand`].
//! This module keeps the platform-facing part in one place, so applications do
//! not need to know whether the current window is backed by X11, Wayland,
//! macOS, or another Iced backend.

use std::fmt;

pub use bmol_window_shell::{
    ChromeDrawPlan, ChromeLayoutMode, NativeWindowOptions, WindowAppearance, WindowChromeConfig,
    WindowChromeMetrics, WindowRimConfig, WindowShellController, is_system_dark_mode,
    loyal_drag_bar, setup_native_window, wrap_border_resizer, wrap_window_rim,
};

use iced::{Element, Subscription, Task, window};

/// Default radius of a borderless Liquid Glass window in logical points (`H / 2`).
pub const DEFAULT_WINDOW_CORNER_RADIUS: u16 = bmol_designs::DEFAULT_WINDOW_CORNER_RADIUS as u16;

/// A semantic action that can be emitted by a custom window chrome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WindowCommand {
    /// Close the target window.
    Close,
    /// Minimize the target window.
    Minimize,
    /// Toggle the target window between maximized and normal states.
    ToggleMaximize,
    /// Toggle the target window between windowed and fullscreen modes.
    ToggleFullscreen,
    /// Execute the window's configured green traffic-light behavior.
    ToggleExpand(WindowExpandBehavior),
    /// Start moving the target window while the left mouse button is held.
    BeginDrag,
}

impl WindowCommand {
    /// Converts this command into an Iced window task for `id`.
    pub fn task<Message>(self, id: window::Id) -> Task<Message>
    where
        Message: Send + 'static,
    {
        match self {
            Self::Close => window::close(id),
            Self::Minimize => window::minimize(id, true),
            Self::ToggleMaximize => window::toggle_maximize(id),
            Self::ToggleFullscreen => window::mode(id).then(move |mode| {
                let next_mode = match mode {
                    window::Mode::Fullscreen => window::Mode::Windowed,
                    window::Mode::Windowed | window::Mode::Hidden => window::Mode::Fullscreen,
                };
                window::set_mode(id, next_mode)
            }),
            Self::ToggleExpand(behavior) => behavior.command().task(id),
            Self::BeginDrag => window::drag(id),
        }
    }
}

/// The semantic action represented by a macOS-style green traffic light.
///
/// macOS exposes both window zoom (the work-area maximize behavior) and true
/// fullscreen. A custom titlebar must choose which one a particular window
/// wants; Iced does not infer that choice from the rendered button.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum WindowExpandBehavior {
    /// Enter true fullscreen and return to windowed mode on the next press.
    #[default]
    Fullscreen,
    /// Maximize into the current work area and restore the previous frame.
    Maximize,
}

impl WindowExpandBehavior {
    /// Returns the concrete window command for this behavior.
    #[must_use]
    pub const fn command(self) -> WindowCommand {
        match self {
            Self::Fullscreen => WindowCommand::ToggleFullscreen,
            Self::Maximize => WindowCommand::ToggleMaximize,
        }
    }

    /// Returns the matching icon for a custom green traffic-light control.
    #[must_use]
    pub const fn icon(self) -> crate::UiIcon {
        match self {
            Self::Fullscreen => crate::UiIcon::WindowZoom,
            Self::Maximize => crate::UiIcon::WindowMaximize,
        }
    }
}

/// Common Iced settings for a custom or native window shell.
///
/// [`IcedWindowPolicy::liquid_glass`] is the prepared configuration for this
/// repository's borderless transparent windows. Applications that use native
/// decorations can start from [`Default::default`] instead.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IcedWindowPolicy {
    /// Whether the window has native decorations.
    pub decorations: bool,
    /// Whether the window can be resized.
    pub resizable: bool,
    /// Whether the native close affordance is available when decorations are shown.
    pub closeable: bool,
    /// Whether the native minimize affordance is available when decorations are shown.
    pub minimizable: bool,
    /// Whether the surface is transparent.
    pub transparent: bool,
    /// Whether the platform compositor should blur the transparent surface.
    pub blur: bool,
    /// On macOS, whether content extends beneath the titlebar.
    pub extend_into_titlebar: bool,
    /// Whether a native close request closes the window automatically.
    pub exit_on_close_request: bool,
    /// The action and icon used by the custom green traffic-light control.
    /// Defaults to true fullscreen to match this project's macOS-style
    /// window chrome.
    pub expand_behavior: WindowExpandBehavior,
    /// Radius applied to the final compositor output of a transparent window.
    /// This is separate from Iced's settings because Iced does not expose a
    /// cross-platform window mask API.
    pub corner_radius: u16,
}

impl IcedWindowPolicy {
    /// Returns the prepared policy for a borderless Liquid Glass window.
    #[must_use]
    pub const fn liquid_glass() -> Self {
        Self {
            decorations: false,
            resizable: true,
            closeable: true,
            minimizable: true,
            transparent: true,
            blur: true,
            extend_into_titlebar: true,
            exit_on_close_request: true,
            expand_behavior: WindowExpandBehavior::Fullscreen,
            corner_radius: DEFAULT_WINDOW_CORNER_RADIUS,
        }
    }

    /// Returns a copy that lets the application handle close requests first.
    ///
    /// Use this when an application needs to ask about unsaved work or route
    /// the request through a document/session manager. The application must
    /// then subscribe to [`IcedWindowController::close_requests`] or window
    /// events and eventually issue [`WindowCommand::Close`].
    #[must_use]
    pub const fn manual_close(self) -> Self {
        Self { exit_on_close_request: false, ..self }
    }

    /// Returns a copy using `behavior` for the green traffic-light control.
    #[must_use]
    pub const fn with_expand_behavior(self, behavior: WindowExpandBehavior) -> Self {
        Self { expand_behavior: behavior, ..self }
    }

    /// Returns a copy with a custom compositor window radius.
    #[must_use]
    pub const fn with_corner_radius(self, radius: u16) -> Self {
        Self { corner_radius: radius, ..self }
    }

    /// Returns the compositor window radius in logical points.
    #[must_use]
    pub const fn corner_radius(self) -> u16 {
        self.corner_radius
    }

    /// Returns the command represented by this window's green control.
    #[must_use]
    pub const fn expand_command(self) -> WindowCommand {
        WindowCommand::ToggleExpand(self.expand_behavior)
    }

    /// Produces the corresponding [`NativeWindowOptions`] for native window hardening.
    #[must_use]
    pub const fn native_options(self) -> NativeWindowOptions {
        NativeWindowOptions::new()
            .with_corner_radius(self.corner_radius as f64)
            .with_shadow(false)
            .with_edr(true)
            .with_stage_manager_guard(true)
    }

    /// Applies this policy to an existing Iced settings value.
    #[must_use]
    pub fn apply(self, mut settings: window::Settings) -> window::Settings {
        settings.decorations = self.decorations;
        settings.resizable = self.resizable;
        settings.closeable = self.closeable;
        settings.minimizable = self.minimizable;
        settings.transparent = self.transparent;
        settings.blur = self.blur && (cfg!(target_os = "linux") || cfg!(target_os = "macos"));
        settings.exit_on_close_request = self.exit_on_close_request;

        #[cfg(target_os = "macos")]
        if self.decorations {
            settings.platform_specific.titlebar_transparent = self.extend_into_titlebar;
            settings.platform_specific.title_hidden = self.extend_into_titlebar;
            settings.platform_specific.fullsize_content_view = self.extend_into_titlebar;
        }

        settings
    }
}

impl Default for IcedWindowPolicy {
    fn default() -> Self {
        Self {
            decorations: true,
            resizable: true,
            closeable: true,
            minimizable: true,
            transparent: false,
            blur: false,
            extend_into_titlebar: false,
            exit_on_close_request: true,
            expand_behavior: WindowExpandBehavior::default(),
            corner_radius: 0,
        }
    }
}

/// Small reusable controller that owns the target window ID and lifecycle state.
///
/// The controller deliberately does not own application state or decide what
/// a close request means. It only tracks the Iced window and turns semantic
/// commands into tasks. This keeps it usable by settings apps, editors, and
/// other applications with different close-confirmation policies.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IcedWindowController {
    id: Option<window::Id>,
    focused: bool,
}

impl IcedWindowController {
    /// Creates a controller that is not attached to a window yet.
    #[must_use]
    pub const fn new() -> Self {
        Self { id: None, focused: false }
    }

    /// Creates a controller attached to an existing Iced window ID.
    #[must_use]
    pub const fn from_id(id: window::Id) -> Self {
        Self { id: Some(id), focused: false }
    }

    /// Returns the currently attached window ID, if the window is open.
    #[must_use]
    pub const fn id(self) -> Option<window::Id> {
        self.id
    }

    /// Returns whether the tracked window is currently focused.
    #[must_use]
    pub const fn is_focused(self) -> bool {
        self.focused
    }

    /// Attaches the controller to `id`.
    pub fn attach(&mut self, id: window::Id) {
        self.id = Some(id);
    }

    /// Applies one Iced window event to the controller.
    ///
    /// A controller ignores events belonging to another window. This makes it
    /// safe to use one controller per window when an application opens more
    /// than one document or preferences window.
    pub fn observe(&mut self, id: window::Id, event: &window::Event) {
        match event {
            window::Event::Opened { .. } if self.id.is_none() => self.id = Some(id),
            window::Event::Closed if self.id == Some(id) => {
                self.id = None;
                self.focused = false;
            }
            window::Event::Focused if self.id == Some(id) => self.focused = true,
            window::Event::Unfocused if self.id == Some(id) => self.focused = false,
            _ => {}
        }
    }

    /// Produces the task for a semantic command, or a no-op if no window is attached.
    pub fn task<Message>(self, command: WindowCommand) -> Task<Message>
    where
        Message: Send + 'static,
    {
        self.id.map_or_else(Task::none, |id| command.task(id))
    }

    /// Subscribes to all Iced window events.
    pub fn events() -> Subscription<(window::Id, window::Event)> {
        window::events()
    }

    /// Queries the latest open window ID.
    pub fn latest() -> Task<Option<window::Id>> {
        window::latest()
    }

    /// Subscribes to windows as they are opened.
    pub fn open_events() -> Subscription<window::Id> {
        window::open_events()
    }

    /// Subscribes to native close requests.
    pub fn close_requests() -> Subscription<window::Id> {
        window::close_requests()
    }
}

/// An invisible titlebar region that publishes a message on mouse press.
///
/// Uses Iced's high-performance [`mouse_area`](iced::widget::mouse_area) to publish
/// on press and optionally on double click, eliminating custom widget tree boilerplate.
pub struct WindowDragArea<'a, Message, Theme, Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    on_press: Message,
    on_double_click: Option<Message>,
}

impl<'a, Message, Theme, Renderer> WindowDragArea<'a, Message, Theme, Renderer> {
    /// Wraps `content` in a press-triggered window-drag hit area.
    #[must_use]
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        on_press: Message,
    ) -> Self {
        Self { content: content.into(), on_press, on_double_click: None }
    }

    /// Attaches an optional double-click action (e.g. toggle maximize).
    #[must_use]
    pub fn on_double_click(mut self, on_double_click: Message) -> Self {
        self.on_double_click = Some(on_double_click);
        self
    }

    /// Converts this drag area into an Iced element.
    #[must_use]
    pub fn into_element(self) -> Element<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Theme: 'a,
        Renderer: iced::advanced::Renderer + 'a,
    {
        let mut area = iced::widget::mouse_area(self.content).on_press(self.on_press);
        if let Some(double_click) = self.on_double_click {
            area = area.on_double_click(double_click);
        }
        area.into()
    }
}

impl<Message, Theme, Renderer> fmt::Debug for WindowDragArea<'_, Message, Theme, Renderer> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("WindowDragArea").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn liquid_glass_policy_is_borderless_and_auto_close() {
        let policy = IcedWindowPolicy::liquid_glass();
        assert!(!policy.decorations);
        assert!(policy.transparent);
        assert!(policy.exit_on_close_request);
        assert_eq!(policy.expand_behavior, WindowExpandBehavior::Fullscreen);
        assert_eq!(policy.corner_radius(), DEFAULT_WINDOW_CORNER_RADIUS);
        assert!(!policy.manual_close().exit_on_close_request);
    }

    #[test]
    fn window_corner_radius_is_configurable_without_changing_window_semantics() {
        let policy = IcedWindowPolicy::liquid_glass().with_corner_radius(22);

        assert_eq!(policy.corner_radius(), 22);
        assert!(!policy.decorations);
        assert!(policy.transparent);
    }

    #[test]
    fn expand_behavior_maps_to_the_matching_window_command() {
        assert_eq!(WindowExpandBehavior::Fullscreen.command(), WindowCommand::ToggleFullscreen);
        assert_eq!(WindowExpandBehavior::Maximize.command(), WindowCommand::ToggleMaximize);
        assert_eq!(
            IcedWindowPolicy::default()
                .with_expand_behavior(WindowExpandBehavior::Maximize)
                .expand_behavior,
            WindowExpandBehavior::Maximize
        );
    }

    #[test]
    fn controller_attaches_on_open_and_tracks_focus() {
        let id = window::Id::unique();
        let mut controller = IcedWindowController::new();

        controller.observe(
            id,
            &window::Event::Opened { position: None, size: iced::Size::new(100.0, 100.0) },
        );
        assert_eq!(controller.id(), Some(id));

        controller.observe(id, &window::Event::Focused);
        assert!(controller.is_focused());
        controller.observe(id, &window::Event::Unfocused);
        assert!(!controller.is_focused());

        controller.observe(id, &window::Event::Closed);
        assert_eq!(controller.id(), None);
    }

    #[test]
    fn controller_ignores_another_window() {
        let first = window::Id::unique();
        let second = window::Id::unique();
        let mut controller = IcedWindowController::from_id(first);

        controller.observe(second, &window::Event::Focused);
        assert!(!controller.is_focused());
        assert_eq!(controller.id(), Some(first));
    }
}
