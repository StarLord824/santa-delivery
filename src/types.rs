use turbo::*;

// ============================================================================
// CONSTANTS
// ============================================================================

pub const SCREEN_W: f32 = 384.0;
pub const SCREEN_H: f32 = 216.0;
pub const PLAYER_X: f32 = 60.0;
pub const PLAYER_SPEED: f32 = 3.0;

// Base colors
pub const COLOR_SNOW: u32 = 0xf0f8ffff;
pub const COLOR_CHIMNEY: u32 = 0x8b4513ff;
pub const COLOR_GOLD: u32 = 0xffd700ff;

// Level-based sky colors (environment changes)
pub const SKY_COLORS: [u32; 5] = [
    0x1a2744ff,  // Level 1: Deep night
    0x0f1f3aff,  // Level 2: Darker midnight
    0x2a1a44ff,  // Level 3: Purple twilight
    0x0a1020ff,  // Level 4: Near black
    0x220000ff,  // Level 5: Blood moon red
];

// Game Modes
pub const MODE_TITLE: u8 = 0;
pub const MODE_DELIVERING: u8 = 1;
pub const MODE_KRAMPUS: u8 = 2;
pub const MODE_GAMEOVER: u8 = 3;

// ============================================================================
// STRUCTS
// ============================================================================

/// A chimney target where Santa needs to drop gifts

#[turbo::serialize]
pub struct Chimney {
    pub x: f32,        // World X position (scrolls left)
    pub y: f32,        // Fixed Y position (height)
    pub delivered: bool,
    pub style: u8,     // House style (0-2 for variation)
}

/// A gift that's been dropped and is falling

#[turbo::serialize]
pub struct FallingGift {
    pub x: f32,
    pub y: f32,
    pub vel_y: f32,    // Falling velocity
    pub target_chimney: usize, // Which chimney this is aimed at
    pub active: bool,
}

/// Krampus projectile

#[turbo::serialize]
pub struct Projectile {
    pub x: f32,
    pub y: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub active: bool,
}

/// Snowflake for atmosphere

#[turbo::serialize]
pub struct Snowflake {
    pub x: f32,
    pub y: f32,
    pub speed: f32,
    pub size: u32,
}
