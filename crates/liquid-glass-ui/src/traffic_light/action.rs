//! Semantic actions and expand behaviour for window controls.

/// Semantic window control actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowControlAction {
    Close,
    Minimize,
    Zoom,
    Expand,
}

/// Backwards-compatible alias for [`WindowControlAction`].
pub type ControlAction = WindowControlAction;

impl WindowControlAction {
    /// Human-readable name used by laboratory status text.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Close => "Close",
            Self::Minimize => "Minimize",
            Self::Zoom | Self::Expand => "Zoom",
        }
    }
}

/// The semantic action represented by a macOS-style green traffic light.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub enum WindowExpandBehavior {
    /// Enter true fullscreen and return to windowed mode on the next press.
    #[default]
    Fullscreen,
    /// Maximize to screen visible frame (classic zoom).
    Zoom,
    /// Maximize into the current work area and restore the previous frame.
    Maximize,
}

impl WindowExpandBehavior {
    /// Returns `true` when this behaviour means true fullscreen.
    #[must_use]
    pub const fn is_fullscreen(self) -> bool {
        matches!(self, Self::Fullscreen)
    }
}
