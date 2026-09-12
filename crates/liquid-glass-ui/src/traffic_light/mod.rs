//! The traffic-light control: its semantic actions, its layout metrics, its
//! Apple colourimetry and vector glyphs, and the Iced widget that draws and
//! hit-tests them.
//!
//! # Why this lives in the widget layer
//!
//! The sphere material is drawn by the GPU Liquid Glass compositor while the
//! Apple vector glyphs are drawn by Iced. Those two live in different render
//! passes, so the glyphs have to be routed through the compositor's *overlay*
//! layer explicitly — a widget that skips that step renders a control with no
//! symbols at all, and the failure is silent.
//!
//! [`widget::window_control_group`] therefore performs the routing itself and
//! returns an element that is already correctly layered. Nothing above this
//! module has to remember to wrap it.
//!
//! What is *not* here: the interaction state machine (hover, press and the
//! press-scale spring), the material tuning knobs, and the
//! `liquid_glass_scene::GlassScene` builder that turns a frame of that state
//! into GPU nodes. Those belong to whichever application owns the window, and
//! they consume [`event::TrafficLightsEvent`] — the pointer facts this widget
//! emits — rather than being known to it.

pub mod action;
pub mod event;
pub mod layout;
pub mod palette;
pub mod widget;

pub use action::{ControlAction, WindowControlAction, WindowExpandBehavior};
pub use event::TrafficLightsEvent;
pub use layout::*;
pub use palette::*;
pub use widget::*;
