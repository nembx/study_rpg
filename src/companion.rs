use crate::{CompanionMode, CompanionPreferences};

const COMPACT_WIDTH: f64 = 280.0;
const COMPACT_HEIGHT: f64 = 160.0;
const EXPANDED_WIDTH: f64 = 360.0;
const EXPANDED_MAX_HEIGHT: f64 = 620.0;
const EXPANDED_HEIGHT_RATIO: f64 = 0.7;
const VERTICAL_MARGIN: f64 = 16.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompanionDisplay {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompanionWindowBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

pub fn companion_window_bounds(
    display: CompanionDisplay,
    preferences: CompanionPreferences,
) -> CompanionWindowBounds {
    let scale_factor = display.scale_factor.max(1.0);
    let logical_display_height = f64::from(display.height) / scale_factor;
    let available_logical_height = (logical_display_height - VERTICAL_MARGIN * 2.0).max(1.0);

    let (logical_width, logical_height) = match preferences.mode {
        CompanionMode::Compact => (COMPACT_WIDTH, COMPACT_HEIGHT.min(available_logical_height)),
        CompanionMode::Expanded => (
            EXPANDED_WIDTH,
            (logical_display_height * EXPANDED_HEIGHT_RATIO)
                .min(EXPANDED_MAX_HEIGHT)
                .min(available_logical_height),
        ),
    };

    let width = (logical_width * scale_factor).round() as u32;
    let height = (logical_height * scale_factor).round() as u32;
    let x = display.x + display.width as i32 - width as i32;
    let minimum_y = display.y + (VERTICAL_MARGIN * scale_factor).round() as i32;
    let maximum_y = display.y + display.height as i32
        - height as i32
        - (VERTICAL_MARGIN * scale_factor).round() as i32;
    let centered_y = display.y + (display.height as i32 - height as i32) / 2;
    let requested_y = preferences.y_position.unwrap_or(centered_y);
    let y = requested_y.clamp(minimum_y, maximum_y.max(minimum_y));

    CompanionWindowBounds {
        x,
        y,
        width,
        height,
    }
}
