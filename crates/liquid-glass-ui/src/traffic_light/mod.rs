//! The traffic-light control: its semantic actions, its layout metrics, and its
//! Apple colourimetry and vector glyphs.
//!
//! These live in the widget layer because the widget needs them; the state
//! machine, its tuning, and the GPU scene builder stay with the shell.

pub mod action;
pub mod layout;
pub mod palette;

pub use action::{ControlAction, WindowControlAction, WindowExpandBehavior};
pub use layout::*;
pub use palette::*;
