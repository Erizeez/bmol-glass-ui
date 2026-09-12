//! The pointer facts the traffic-light widget reports.
//!
//! The widget owns this type because the widget is what emits it: it is the
//! widget's output vocabulary, not a state-machine concept. An application
//! routes these values into whatever interaction state it keeps.

/// Unified interaction event emitted by the traffic-light widget.
///
/// The widget reports *facts*: what the pointer did. The application's state
/// machine decides what they mean, and returns a window-control action only
/// when a press was committed to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrafficLightsEvent {
    /// The pointer entered or left the whole group. Drives the glyph reveal.
    GroupHover(bool),
    /// The pointer pressed this control.
    PressStart(usize),
    /// The pointer left this control while still holding the button.
    PressCancel(usize),
    /// The button was released. `committed` is `true` when the pointer was over
    /// the same control it pressed.
    PressEnd { index: usize, committed: bool },
}

impl TrafficLightsEvent {
    /// The control index this event addresses, if any.
    #[must_use]
    pub const fn index(self) -> Option<usize> {
        match self {
            Self::GroupHover(_) => None,
            Self::PressStart(index)
            | Self::PressCancel(index)
            | Self::PressEnd { index, .. } => Some(index),
        }
    }
}
