//! Measured traffic-light geometry and standalone laboratory sample layout.
//!
//! Every metric derives from `bmol-designs::traffic_lights`; this module never
//! hard-codes an Apple measurement.

use liquid_glass_scene::GlassId;

pub use bmol_designs::traffic_lights::*;

/// AppKit's standard traffic-light circle measures 28 px on a 2x display.
/// Keep the cross-platform sample in logical points so its 1:1 reference is
/// independent of the backing scale factor.
pub const WINDOW_CONTROL_NATIVE_SIZE: f32 = DIAMETER;
/// Visual diameter of the enlarged inspection samples.
pub const WINDOW_CONTROL_LARGE_SIZE: f32 = 64.0;
/// Native spacing between two adjacent controls.
pub const WINDOW_CONTROL_GAP: f32 = SPACING;
/// Spacing used by the enlarged inspection samples.
pub const WINDOW_CONTROL_LARGE_GAP: f32 = 12.0;

/// Raw id numbers, in Apple's close / minimize / zoom order.
///
/// These are the single source of truth for the sample ids. Consumers whose
/// compositor sits on a different `liquid-glass-scene` version can rebuild
/// their own id type from these without copying the table.
pub const WINDOW_CONTROL_NATIVE_RAW_IDS: [u64; 3] = [100, 101, 102];
pub const WINDOW_CONTROL_REFERENCE_RAW_IDS: [u64; 3] = [110, 111, 112];
pub const WINDOW_CONTROL_LARGE_RAW_IDS: [u64; 3] = [120, 121, 122];
pub const WINDOW_CONTROL_INACTIVE_RAW_IDS: [u64; 3] = [130, 131, 132];
pub const WINDOW_CONTROL_DISABLED_RAW_IDS: [u64; 3] = [140, 141, 142];

/// Raw ids of every sample group, indexed by sample group.
pub const WINDOW_CONTROL_RAW_GROUPS: [[u64; 3]; 5] = [
    WINDOW_CONTROL_NATIVE_RAW_IDS,
    WINDOW_CONTROL_REFERENCE_RAW_IDS,
    WINDOW_CONTROL_LARGE_RAW_IDS,
    WINDOW_CONTROL_INACTIVE_RAW_IDS,
    WINDOW_CONTROL_DISABLED_RAW_IDS,
];

const fn typed_ids(raw: [u64; 3]) -> [GlassId; 3] {
    [GlassId(raw[0]), GlassId(raw[1]), GlassId(raw[2])]
}

/// Stable glass ids for the measured (1:1) sample.
pub const WINDOW_CONTROL_NATIVE_IDS: [GlassId; 3] = typed_ids(WINDOW_CONTROL_NATIVE_RAW_IDS);
/// Stable glass ids for the second 1:1 reference sample.
pub const WINDOW_CONTROL_REFERENCE_IDS: [GlassId; 3] = typed_ids(WINDOW_CONTROL_REFERENCE_RAW_IDS);
/// Stable glass ids for the enlarged active sample.
pub const WINDOW_CONTROL_LARGE_IDS: [GlassId; 3] = typed_ids(WINDOW_CONTROL_LARGE_RAW_IDS);
/// Stable glass ids for the enlarged inactive-window sample.
pub const WINDOW_CONTROL_INACTIVE_IDS: [GlassId; 3] = typed_ids(WINDOW_CONTROL_INACTIVE_RAW_IDS);
/// Stable glass ids for the enlarged unavailable / edited sample.
pub const WINDOW_CONTROL_DISABLED_IDS: [GlassId; 3] = typed_ids(WINDOW_CONTROL_DISABLED_RAW_IDS);

/// Leading offset of the measured sample inside the titlebar band.
pub const WINDOW_CONTROL_NATIVE_X: f32 = bmol_designs::traffic_lights::LEADING_MARGIN;
/// Top offset of the measured sample inside the titlebar band.
pub const WINDOW_CONTROL_NATIVE_Y: f32 = bmol_designs::traffic_lights::LEADING_MARGIN;

/// Reference sample origin.
pub const WINDOW_CONTROL_REFERENCE_X: f32 = 240.0;
/// Reference sample origin.
pub const WINDOW_CONTROL_REFERENCE_Y: f32 = 196.0;
/// Enlarged active sample origin.
pub const WINDOW_CONTROL_LARGE_X: f32 = 240.0;
/// Enlarged active sample origin.
pub const WINDOW_CONTROL_LARGE_Y: f32 = 292.0;
/// Enlarged inactive sample origin.
pub const WINDOW_CONTROL_INACTIVE_X: f32 = 240.0;
/// Enlarged inactive sample origin.
pub const WINDOW_CONTROL_INACTIVE_Y: f32 = 418.0;
/// Enlarged unavailable sample origin.
pub const WINDOW_CONTROL_DISABLED_X: f32 = 240.0;
/// Enlarged unavailable sample origin.
pub const WINDOW_CONTROL_DISABLED_Y: f32 = 544.0;

/// All sample groups in scene/z-order. Each group owns three controls.
pub const WINDOW_CONTROL_GROUPS: [(&str, [GlassId; 3]); 5] = [
    ("native", WINDOW_CONTROL_NATIVE_IDS),
    ("reference", WINDOW_CONTROL_REFERENCE_IDS),
    ("large", WINDOW_CONTROL_LARGE_IDS),
    ("inactive", WINDOW_CONTROL_INACTIVE_IDS),
    ("disabled", WINDOW_CONTROL_DISABLED_IDS),
];

/// Number of buttons in one traffic-light group.
pub const GROUP_LEN: usize = 3;
/// Number of sample groups shipped by the standalone laboratory.
pub const GROUP_COUNT: usize = WINDOW_CONTROL_RAW_GROUPS.len();
