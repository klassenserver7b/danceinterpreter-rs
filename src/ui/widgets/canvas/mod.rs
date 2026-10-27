//! Canvas-drawn controls (power, restart, ripple toggle).
//!
//! These widgets paint their glyphs with `iced::widget::canvas` instead of
//! icon fonts so they can animate (ripple press feedback) beyond what a
//! static glyph offers.

pub mod canvas_toggle;
pub mod power_button;
pub mod restart_button;
