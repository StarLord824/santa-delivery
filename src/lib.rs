use turbo::*;

// ============================================================================
// STRUCTS
// ============================================================================

/// A chimney target where Santa needs to drop gifts
#[turbo::serialize]
struct Chimney {
    x: f32,        // World X position (scrolls left)
    y: f32,        // Fixed Y position (height)
    delivered: bool,
}

/// A gift that's been dropped and is falling
#[turbo::serialize]
struct FallingGift {
    x: f32,
    y: f32,
    vel_y: f32,    // Falling velocity
    target_chimney: usize, // Which chimney this is aimed at
    active: bool,
}

/// Krampus projectile
#[turbo::serialize]
struct Projectile {
    x: f32,
    y: f32,
    vel_x: f32,
    vel_y: f32,
    active: bool,
}

/// Snowflake for atmosphere
#[turbo::serialize]
struct Snowflake {
    x: f32,
    y: f32,
    speed: f32,
    size: u32,
}

// ============================================================================
// GAME STATE
// ============================================================================

#[turbo::game]
struct GameState {
    frame: u32,
    mode: u8,  // 0=Title, 1=Delivering, 2=KrampusAttack, 3=GameOver
    
    // Scrolling
    scroll_x: f32,
    scroll_speed: f32,
    
    // Player (Santa's sleigh)
    player_y: f32,
    player_vel_y: f32,
    sleigh_tilt: f32,  // Visual tilt when moving
    
    // Chimneys (delivery targets)
    chimneys: Vec<Chimney>,
    next_chimney_spawn: f32,
    
    // Falling gifts
    gifts: Vec<FallingGift>,
    
    // Krampus
    krampus_x: f32,
    krampus_y: f32,
    krampus_active: bool,
    krampus_attack_timer: u32,
    krampus_duration: u32,
    projectiles: Vec<Projectile>,
    krampus_warning: u32,  // Countdown before attack
    
    // Stats
    health: u32,
    score: u32,
    high_score: u32,
    deliveries: u32,
    naughty_meter: u32,
    level: u32,
    
    // Visual effects
    screen_flash: u32,
    flash_color: u32,
    screen_shake: u32,
    invincible_timer: u32,  // Invincibility frames after damage
    
    // Particles
    snowflakes: Vec<Snowflake>,
    
    // Tutorial
    tutorial_timer: u32,   // Counts down during tutorial
    first_play: bool,      // True if this is first game
    tutorial_step: u8,     // Which hint to show
    
    // RNG
    rng_seed: u32,
}

// Game mode constants
const MODE_TITLE: u8 = 0;
const MODE_DELIVERING: u8 = 1;
const MODE_KRAMPUS: u8 = 2;
const MODE_GAMEOVER: u8 = 3;

// Screen dimensions
const SCREEN_W: f32 = 256.0;
const SCREEN_H: f32 = 144.0;

// Player constants
const PLAYER_X: f32 = 40.0;  // Fixed X position
const PLAYER_SPEED: f32 = 2.5;

// Colors
const COLOR_SKY: u32 = 0x1a2744ff;
const COLOR_SNOW: u32 = 0xf0f8ffff;
const COLOR_CHIMNEY: u32 = 0x8b4513ff;
const COLOR_ROOF: u32 = 0xfffffaff;
const COLOR_GIFT: u32 = 0xff0000ff;
const COLOR_GOLD: u32 = 0xffd700ff;

impl GameState {
    pub fn new() -> Self {
        let mut state = Self {
            frame: 0,
            mode: MODE_TITLE,
            
            scroll_x: 0.0,
            scroll_speed: 1.5,
            
            player_y: SCREEN_H / 2.0,
            player_vel_y: 0.0,
            sleigh_tilt: 0.0,
            
            chimneys: vec![],
            next_chimney_spawn: 100.0,
            
            gifts: vec![],
            
            krampus_x: SCREEN_W + 50.0,
            krampus_y: SCREEN_H / 2.0,
            krampus_active: false,
            krampus_attack_timer: 600, // 10 seconds at 60fps
            krampus_duration: 0,
            projectiles: vec![],
            krampus_warning: 0,
            
            health: 3,
            score: 0,
            high_score: 0,
            deliveries: 0,
            naughty_meter: 0,
            level: 1,
            
            screen_flash: 0,
            flash_color: 0xffffffff,
            screen_shake: 0,
            invincible_timer: 0,
            
            snowflakes: vec![],
            
            tutorial_timer: 0,
            first_play: true,
            tutorial_step: 0,
            
            rng_seed: 42,
        };
        state.init_snowflakes();
        state
    }
    
    // ========================================================================
    // RANDOM
    // ========================================================================
    
    fn random(&mut self) -> u32 {
        self.rng_seed = self.rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
        (self.rng_seed >> 16) & 0x7FFF
    }
    
    fn random_range(&mut self, min: f32, max: f32) -> f32 {
        let r = (self.random() % 1000) as f32 / 1000.0;
        min + r * (max - min)
    }
    
    // ========================================================================
    // INITIALIZATION
    // ========================================================================
    
    fn init_snowflakes(&mut self) {
        self.snowflakes.clear();
        for _ in 0..25 {
            let x = self.random_range(0.0, SCREEN_W);
            let y = self.random_range(0.0, SCREEN_H);
            let speed = self.random_range(0.5, 1.5);
            let size = (self.random() % 2 + 1) as u32;
            self.snowflakes.push(Snowflake { x, y, speed, size });
        }
    }
    
    fn start_game(&mut self) {
        self.mode = MODE_DELIVERING;
        self.scroll_x = 0.0;
        self.scroll_speed = 1.5;
        self.player_y = SCREEN_H / 2.0;
        self.player_vel_y = 0.0;
        self.sleigh_tilt = 0.0;
        
        self.chimneys.clear();
        self.next_chimney_spawn = 150.0;
        self.gifts.clear();
        self.projectiles.clear();
        
        self.krampus_active = false;
        self.krampus_attack_timer = 600;
        self.krampus_warning = 0;
        self.krampus_duration = 0;
        
        self.health = 3;
        self.score = 0;
        self.deliveries = 0;
        self.naughty_meter = 0;
        self.level = 1;
        self.invincible_timer = 0;
        
        // Tutorial: 10 seconds on first play
        if self.first_play {
            self.tutorial_timer = 600; // 10 seconds at 60fps
            self.tutorial_step = 0;
        } else {
            self.tutorial_timer = 0;
        }
        
        self.screen_flash = 8;
        self.flash_color = 0xffffffff;
    }
    
    fn reset_game(&mut self) {
        if self.score > self.high_score {
            self.high_score = self.score;
        }
        self.first_play = false; // No more tutorial after first game
        self.start_game();
    }
    
    // ========================================================================
    // PLAYER MOVEMENT
    // ========================================================================
    
    fn move_player(&mut self) {
        let gp = gamepad::get(0);
        
        // Vertical movement only
        if gp.up.pressed() {
            self.player_vel_y = -PLAYER_SPEED;
            self.sleigh_tilt = -8.0; // Tilt up
        } else if gp.down.pressed() {
            self.player_vel_y = PLAYER_SPEED;
            self.sleigh_tilt = 8.0; // Tilt down
        } else {
            self.player_vel_y *= 0.85; // Deceleration
            self.sleigh_tilt *= 0.8;   // Return to level
        }
        
        self.player_y += self.player_vel_y;
        self.player_y = self.player_y.clamp(20.0, SCREEN_H - 30.0);
    }
    
    // ========================================================================
    // CHIMNEY SYSTEM
    // ========================================================================
    
    fn spawn_chimney(&mut self) {
        let y = self.random_range(SCREEN_H * 0.5, SCREEN_H - 20.0);
        self.chimneys.push(Chimney {
            x: SCREEN_W + 20.0,
            y,
            delivered: false,
        });
        
        // Next chimney spawn distance (varies)
        self.next_chimney_spawn = self.random_range(80.0, 150.0);
    }
    
    fn update_chimneys(&mut self) {
        // Spawn new chimneys
        if self.chimneys.is_empty() || 
           self.chimneys.last().map(|c| c.x < SCREEN_W - self.next_chimney_spawn).unwrap_or(true) {
            self.spawn_chimney();
        }
        
        // Track missed chimneys
        let mut missed_count = 0;
        
        // Update positions and remove off-screen
        self.chimneys.retain(|c| {
            if c.x < -40.0 {
                if !c.delivered {
                    missed_count += 1;
                }
                false
            } else {
                true
            }
        });
        
        // Increase naughty meter for missed deliveries
        self.naughty_meter = (self.naughty_meter + missed_count * 20).min(100);
        
        // Move chimneys
        for chimney in &mut self.chimneys {
            chimney.x -= self.scroll_speed;
        }
    }
    
    // ========================================================================
    // GIFT DROPPING
    // ========================================================================
    
    fn drop_gift(&mut self) {
        let gp = gamepad::get(0);
        let kb = keyboard::get();
        
        // Drop with Enter key, Space, or A/B button
        let should_drop = kb.enter().just_pressed() 
            || kb.space().just_pressed()
            || gp.a.just_pressed() 
            || gp.b.just_pressed();
        
        if should_drop {
            // Find the nearest chimney ahead (increased range for easier aiming)
            let mut best_chimney: Option<usize> = None;
            let mut best_dist = f32::MAX;
            
            for (i, chimney) in self.chimneys.iter().enumerate() {
                // Larger detection window: 150 pixels ahead
                if !chimney.delivered && chimney.x > PLAYER_X - 20.0 && chimney.x < PLAYER_X + 150.0 {
                    let dist = (chimney.x - PLAYER_X).abs();
                    if dist < best_dist {
                        best_dist = dist;
                        best_chimney = Some(i);
                    }
                }
            }
            
            // Drop a gift
            self.gifts.push(FallingGift {
                x: PLAYER_X + 8.0,
                y: self.player_y + 12.0,
                vel_y: 1.0,
                target_chimney: best_chimney.unwrap_or(usize::MAX),
                active: true,
            });
            
            // Visual feedback that gift was dropped
            self.screen_flash = 2;
            self.flash_color = 0xffffff44;
        }
    }
    
    fn update_gifts(&mut self) {
        let scroll_speed = self.scroll_speed;
        let mut score_gained = 0u32;
        let mut deliveries_made = 0u32;
        
        // Get chimney positions for collision
        let chimney_data: Vec<(f32, f32, bool)> = self.chimneys
            .iter()
            .map(|c| (c.x, c.y, c.delivered))
            .collect();
        
        for gift in &mut self.gifts {
            if !gift.active { continue; }
            
            // Gift falls with arc (moves with scroll + falls)
            gift.x -= scroll_speed * 0.3;
            gift.y += gift.vel_y;
            gift.vel_y += 0.15; // Gravity
            
            // Check collision with chimneys
            for (i, &(cx, cy, delivered)) in chimney_data.iter().enumerate() {
                if !delivered {
                    let dx = gift.x - cx;
                    let dy = gift.y - cy;
                    
                    // GENEROUS hitbox for easier gameplay (25x30 area)
                    // Gift must be close horizontally and near/past chimney vertically
                    if dx.abs() < 25.0 && dy > -10.0 && dy < 30.0 {
                        gift.active = false;
                        score_gained += 100 + self.level * 10;
                        deliveries_made += 1;
                        
                        // Mark chimney as delivered
                        if i < self.chimneys.len() {
                            self.chimneys[i].delivered = true;
                        }
                        break;
                    }
                }
            }
            
            // Remove if off screen
            if gift.y > SCREEN_H + 20.0 || gift.x < -20.0 {
                gift.active = false;
            }
        }
        
        // Apply score and deliveries
        self.score += score_gained;
        self.deliveries += deliveries_made;
        
        // Flash on delivery
        if deliveries_made > 0 {
            self.screen_flash = 4;
            self.flash_color = 0x00ff00ff;
            // Reduce naughty meter on successful delivery
            self.naughty_meter = self.naughty_meter.saturating_sub(10);
        }
        
        // Clean up inactive gifts
        self.gifts.retain(|g| g.active);
    }
    
    // ========================================================================
    // KRAMPUS SYSTEM
    // ========================================================================
    
    fn check_krampus_trigger(&mut self) {
        if self.krampus_active || self.krampus_warning > 0 { return; }
        
        // Trigger conditions: timer or naughty meter
        self.krampus_attack_timer = self.krampus_attack_timer.saturating_sub(1);
        
        if self.krampus_attack_timer == 0 || self.naughty_meter >= 80 {
            // Start warning countdown
            self.krampus_warning = 120; // 2 seconds warning
            self.screen_shake = 10;
        }
    }
    
    fn update_krampus_warning(&mut self) {
        if self.krampus_warning > 0 {
            self.krampus_warning -= 1;
            
            if self.krampus_warning == 0 {
                // Krampus attack begins!
                self.mode = MODE_KRAMPUS;
                self.krampus_active = true;
                self.krampus_x = SCREEN_W + 30.0;
                self.krampus_y = self.random_range(40.0, SCREEN_H - 40.0);
                self.krampus_duration = 360; // 6 seconds
                self.screen_flash = 15;
                self.flash_color = 0xff0000ff;
                self.screen_shake = 15;
            }
        }
    }
    
    fn update_krampus(&mut self) {
        if !self.krampus_active { return; }
        
        // Krampus flies in from right
        if self.krampus_x > SCREEN_W - 60.0 {
            self.krampus_x -= 3.0; // Faster entry
        }
        
        // Krampus tracks player Y more aggressively
        let dy = self.player_y - self.krampus_y;
        self.krampus_y += dy * 0.035;
        
        // Add bobbing motion for menace
        self.krampus_y += (self.frame as f32 / 10.0).sin() * 0.5;
        
        // Fire projectiles - rate increases with level
        let fire_rate = (50 - (self.level * 5).min(25)).max(20);
        if self.frame % fire_rate == 0 {
            self.fire_projectile_pattern();
        }
        
        // Duration countdown
        self.krampus_duration = self.krampus_duration.saturating_sub(1);
        
        if self.krampus_duration == 0 {
            // Krampus retreats
            self.krampus_active = false;
            self.mode = MODE_DELIVERING;
            self.krampus_attack_timer = (600 - (self.level * 40).min(400)).max(180);
            self.naughty_meter = 0;
            
            // Survival bonus
            self.score += 200 + self.level * 50;
            self.screen_flash = 10;
            self.flash_color = 0x00ff00ff;
            self.screen_shake = 5;
        }
    }
    
    /// Fire projectiles with varying patterns based on level
    fn fire_projectile_pattern(&mut self) {
        let pattern = (self.frame / 60 + self.level as u32) % 4;
        let base_speed = 2.5 + self.level as f32 * 0.3;
        
        let dx = PLAYER_X - self.krampus_x;
        let dy = self.player_y - self.krampus_y;
        let dist = (dx * dx + dy * dy).sqrt().max(1.0);
        let dir_x = dx / dist;
        let dir_y = dy / dist;
        
        match pattern {
            0 => {
                // Single aimed shot
                self.spawn_projectile(dir_x * base_speed, dir_y * base_speed);
            }
            1 => {
                // 3-way spread
                self.spawn_projectile(dir_x * base_speed, dir_y * base_speed);
                self.spawn_projectile(dir_x * base_speed - 0.8, dir_y * base_speed - 0.8);
                self.spawn_projectile(dir_x * base_speed + 0.8, dir_y * base_speed + 0.8);
            }
            2 => {
                // Horizontal wave
                self.spawn_projectile(-base_speed, -1.5);
                self.spawn_projectile(-base_speed, 0.0);
                self.spawn_projectile(-base_speed, 1.5);
            }
            _ => {
                // Diagonal cross
                self.spawn_projectile(-base_speed, -base_speed * 0.5);
                self.spawn_projectile(-base_speed, base_speed * 0.5);
            }
        }
    }
    
    fn spawn_projectile(&mut self, vel_x: f32, vel_y: f32) {
        self.projectiles.push(Projectile {
            x: self.krampus_x - 15.0,
            y: self.krampus_y,
            vel_x,
            vel_y,
            active: true,
        });
    }
    
    fn update_projectiles(&mut self) {
        let player_y = self.player_y;
        let mut hit = false;
        let is_invincible = self.invincible_timer > 0;
        
        for proj in &mut self.projectiles {
            if !proj.active { continue; }
            
            proj.x += proj.vel_x;
            proj.y += proj.vel_y;
            
            // Check collision with player (only if not invincible)
            if !is_invincible {
                let dx = proj.x - PLAYER_X;
                let dy = proj.y - player_y;
                let dist = (dx * dx + dy * dy).sqrt();
                
                if dist < 14.0 {
                    proj.active = false;
                    hit = true;
                }
            }
            
            // Remove if off screen
            if proj.x < -20.0 || proj.x > SCREEN_W + 20.0 ||
               proj.y < -20.0 || proj.y > SCREEN_H + 20.0 {
                proj.active = false;
            }
        }
        
        if hit {
            self.health = self.health.saturating_sub(1);
            self.screen_flash = 15;
            self.flash_color = 0xff0000ff;
            self.screen_shake = 12;
            self.invincible_timer = 90; // 1.5 seconds of invincibility
            
            if self.health == 0 {
                self.mode = MODE_GAMEOVER;
                self.screen_shake = 25;
                self.invincible_timer = 0;
            }
        }
        
        // Decrement invincibility
        if self.invincible_timer > 0 {
            self.invincible_timer -= 1;
        }
        
        self.projectiles.retain(|p| p.active);
    }
    
    // ========================================================================
    // SCROLLING & PARALLAX
    // ========================================================================
    
    fn update_scroll(&mut self) {
        self.scroll_x += self.scroll_speed;
        
        // Progressive difficulty
        if self.deliveries > 0 && self.deliveries % 10 == 0 && self.frame % 60 == 0 {
            self.level += 1;
            self.scroll_speed = (1.5 + self.level as f32 * 0.2).min(4.0);
        }
    }
    
    fn update_snowflakes(&mut self) {
        let frame = self.frame;
        let scroll = self.scroll_speed;
        
        for (i, snow) in self.snowflakes.iter_mut().enumerate() {
            snow.y += snow.speed;
            snow.x -= scroll * 0.5; // Move with background
            snow.x += (frame as f32 / 20.0 + i as f32).sin() * 0.3;
            
            if snow.y > SCREEN_H + 5.0 {
                snow.y = -5.0;
                snow.x = (frame.wrapping_add(i as u32 * 7919) % 256) as f32;
            }
            if snow.x < -5.0 { snow.x = SCREEN_W + 5.0; }
        }
    }
    
    // ========================================================================
    // GET SHAKE OFFSET
    // ========================================================================
    
    fn get_shake(&self) -> (i32, i32) {
        if self.screen_shake > 0 {
            let intensity = (self.screen_shake as f32 / 3.0).min(4.0);
            let sx = ((self.frame as f32 * 1.7).sin() * intensity) as i32;
            let sy = ((self.frame as f32 * 2.3).cos() * intensity) as i32;
            (sx, sy)
        } else {
            (0, 0)
        }
    }
    
    // ========================================================================
    // DRAWING
    // ========================================================================
    
    fn draw_background(&self, shake_x: i32, shake_y: i32) {
        // Sky gradient (simple)
        clear(COLOR_SKY);
        
        // Stars (far layer)
        for i in 0..15 {
            let star_x = ((i * 37 + 10) as f32 - (self.scroll_x * 0.1) % SCREEN_W) as i32;
            let star_y = (i * 7 % 50 + 5) as i32;
            circ!(x = star_x + shake_x, y = star_y + shake_y, d = 2, color = 0xffffff88);
        }
        
        // Mountains (mid layer)
        let mountain_offset = (self.scroll_x * 0.3) as i32 % 120;
        for i in 0..4 {
            let mx = i * 120 - mountain_offset + shake_x;
            let my = 80 + shake_y;
            // Simple triangle mountain
            for row in 0..40 {
                let width = row * 3;
                rect!(x = mx + 60 - width / 2, y = my + row, w = width as u32, h = 1, color = 0x2a3f5fff);
            }
        }
        
        // Ground/snow layer
        rect!(x = shake_x, y = 120 + shake_y, w = 256, h = 30, color = COLOR_SNOW);
        
        // Ground line
        rect!(x = shake_x, y = 119 + shake_y, w = 256, h = 2, color = 0xc0d0e0ff);
    }
    
    fn draw_snowflakes(&self) {
        for snow in &self.snowflakes {
            circ!(x = snow.x as i32, y = snow.y as i32, d = snow.size, color = 0xffffffaa);
        }
    }
    
    fn draw_chimney(&self, chimney: &Chimney, shake_x: i32, shake_y: i32) {
        let cx = chimney.x as i32 + shake_x;
        let cy = chimney.y as i32 + shake_y;
        
        // House base
        rect!(x = cx - 20, y = cy + 10, w = 40, h = 30, color = 0x4a3728ff);
        
        // Roof
        for row in 0..15 {
            let width = 50 - row * 2;
            rect!(x = cx - width / 2, y = cy + 10 - row, w = width as u32, h = 1, color = COLOR_ROOF);
        }
        
        // Snow on roof
        rect!(x = cx - 22, y = cy + 8, w = 44, h = 3, color = 0xf0f8ffff);
        
        // Chimney
        rect!(x = cx - 6, y = cy - 10, w = 12, h = 20, color = COLOR_CHIMNEY);
        
        // Chimney top
        rect!(x = cx - 8, y = cy - 12, w = 16, h = 4, color = 0x5c3a21ff);
        
        // Chimney glow if not delivered
        if !chimney.delivered {
            // Pulsing target indicator
            let pulse = ((self.frame as f32 / 8.0).sin() * 30.0) as u32;
            let glow_color = 0xffff0000 + (pulse << 24);
            circ!(x = cx, y = cy - 8, d = 16 + (pulse / 10) as u32, color = glow_color);
        } else {
            // Checkmark / delivered indicator
            circ!(x = cx, y = cy - 8, d = 12, color = 0x00ff0088);
        }
    }
    
    fn draw_sleigh(&self, shake_x: i32, shake_y: i32) {
        // Blink when invincible (don't draw every other frame)
        if self.invincible_timer > 0 && (self.frame / 4) % 2 == 0 {
            return; // Skip drawing for blink effect
        }
        
        let x = PLAYER_X as i32 + shake_x;
        let y = self.player_y as i32 + shake_y;
        let tilt = self.sleigh_tilt as i32;
        
        // Invincibility glow
        if self.invincible_timer > 0 {
            circ!(x = x + 10, y = y + 4, d = 40, color = 0xffffff22);
        }
        
        // Shadow
        ellipse!(x = x + 8, y = 125 + shake_y, w = 30, h = 6, color = 0x00000044);
        
        // Sleigh body (red with gold trim)
        rect!(x = x - 4, y = y + tilt / 2, w = 28, h = 12, color = 0xcc0000ff);
        rect!(x = x - 6, y = y + 10 + tilt / 2, w = 32, h = 4, color = COLOR_GOLD);
        
        // Runner
        rect!(x = x - 8, y = y + 14 + tilt / 2, w = 36, h = 2, color = 0x333333ff);
        
        // Santa (simplified)
        circ!(x = x + 6, y = y - 2 + tilt / 2, d = 12, color = 0xffdbacff); // Face
        rect!(x = x, y = y - 8 + tilt / 2, w = 12, h = 8, color = 0xff0000ff); // Hat
        circ!(x = x + 6, y = y - 10 + tilt / 2, d = 5, color = 0xffffffff); // Pom
        
        // Reindeer silhouette
        rect!(x = x + 30, y = y + 2 + tilt / 3, w = 16, h = 8, color = 0x8b4513cc);
        circ!(x = x + 48, y = y + tilt / 3, d = 8, color = 0x8b4513cc);
        // Red nose!
        circ!(x = x + 52, y = y + tilt / 3, d = 4, color = 0xff0000ff);
    }
    
    fn draw_falling_gift(&self, gift: &FallingGift, shake_x: i32, shake_y: i32) {
        let x = gift.x as i32 + shake_x;
        let y = gift.y as i32 + shake_y;
        
        // Gift box
        rect!(x = x - 5, y = y - 5, w = 10, h = 10, color = COLOR_GIFT);
        // Ribbon
        rect!(x = x - 1, y = y - 5, w = 2, h = 10, color = COLOR_GOLD);
        rect!(x = x - 5, y = y - 1, w = 10, h = 2, color = COLOR_GOLD);
    }
    
    fn draw_krampus(&self, shake_x: i32, shake_y: i32) {
        if !self.krampus_active { return; }
        
        let x = self.krampus_x as i32 + shake_x;
        let y = self.krampus_y as i32 + shake_y;
        let shake = ((self.frame as f32 / 2.0).sin() * 3.0) as i32;
        let wing_flap = ((self.frame as f32 / 5.0).sin() * 8.0) as i32;
        
        // Ominous red aura
        let aura_pulse = ((self.frame as f32 / 8.0).sin() * 20.0) as u32;
        circ!(x = x, y = y, d = 50 + aura_pulse, color = 0x44000022);
        circ!(x = x, y = y, d = 40 + aura_pulse, color = 0x66000033);
        
        // Wings/cape (flapping)
        rect!(x = x + 10, y = y - 15 + wing_flap / 2, w = 20, h = 25, color = 0x1a0a0aff);
        rect!(x = x + 5, y = y - 10 - wing_flap / 2, w = 25, h = 20, color = 0x1a0a0aff);
        
        // Body (larger, more menacing)
        rect!(x = x - 14 + shake, y = y - 10, w = 28, h = 24, color = 0x2a1a1aff);
        
        // Fur texture lines
        for i in 0..4 {
            rect!(x = x - 10 + i * 6 + shake, y = y - 8 + (i % 2) * 4, w = 2, h = 18, color = 0x3a2a2aff);
        }
        
        // Head
        circ!(x = x + shake, y = y - 14, d = 18, color = 0x3a2020ff);
        
        // Horns (larger, curved look)
        rect!(x = x - 14 + shake, y = y - 30, w = 5, h = 18, color = 0x4a3030ff);
        rect!(x = x - 16 + shake, y = y - 32, w = 5, h = 6, color = 0x5a4040ff);
        rect!(x = x + 9 + shake, y = y - 30, w = 5, h = 18, color = 0x4a3030ff);
        rect!(x = x + 11 + shake, y = y - 32, w = 5, h = 6, color = 0x5a4040ff);
        
        // Glowing eyes (intense)
        let eye_glow = ((self.frame as f32 / 3.0).sin() * 60.0).abs() as u32;
        circ!(x = x - 5 + shake, y = y - 16, d = 6, color = 0xff0000ff);
        circ!(x = x + 5 + shake, y = y - 16, d = 6, color = 0xff0000ff);
        // Eye glow halos
        circ!(x = x - 5 + shake, y = y - 16, d = 10, color = 0xff000044 + (eye_glow << 24));
        circ!(x = x + 5 + shake, y = y - 16, d = 10, color = 0xff000044 + (eye_glow << 24));
        
        // Menacing grin
        rect!(x = x - 6 + shake, y = y - 8, w = 12, h = 3, color = 0x000000ff);
        // Teeth
        for i in 0..4 {
            rect!(x = x - 5 + i * 3 + shake, y = y - 8, w = 2, h = 2, color = 0xffffffcc);
        }
        
        // Chains (swinging)
        let chain_swing = ((self.frame as f32 / 8.0).sin() * 4.0) as i32;
        for i in 0..4 {
            let cx = x - 20 - i * 5 + chain_swing;
            let cy = y + 10 + i * 3;
            circ!(x = cx, y = cy, d = 4, color = 0x555555ff);
        }
        
        // Claws
        rect!(x = x - 18 + shake, y = y + 8, w = 6, h = 8, color = 0x1a0a0aff);
        rect!(x = x + 12 + shake, y = y + 8, w = 6, h = 8, color = 0x1a0a0aff);
    }
    
    fn draw_projectile(&self, proj: &Projectile, shake_x: i32, shake_y: i32) {
        let x = proj.x as i32 + shake_x;
        let y = proj.y as i32 + shake_y;
        
        // Flame trail
        let trail_len = 3;
        for i in 1..=trail_len {
            let trail_x = x + (proj.vel_x * i as f32 * 2.0) as i32;
            let trail_y = y + (proj.vel_y * i as f32 * 2.0) as i32;
            let alpha = (0x88 - i * 0x20) as u32;
            circ!(x = trail_x, y = trail_y, d = 6 - i as u32, color = 0xff330000 + alpha);
        }
        
        // Core fireball with glow
        circ!(x = x, y = y, d = 12, color = 0x44000044); // Outer glow
        circ!(x = x, y = y, d = 8, color = 0x880000ff);  // Dark core
        circ!(x = x, y = y, d = 6, color = 0xff2200ff);  // Fire
        circ!(x = x, y = y, d = 3, color = 0xffcc00ff);  // Hot center
    }
    
    fn draw_ui(&self, shake_x: i32, shake_y: i32) {
        // Health hearts
        for i in 0..3 {
            let hx = 8 + i * 14 + shake_x as u32;
            let color = if i < self.health { 0xff0000ff } else { 0x333333ff };
            // Simple heart shape
            circ!(x = hx as i32, y = 10 + shake_y, d = 8, color = color);
            circ!(x = hx as i32 + 6, y = 10 + shake_y, d = 8, color = color);
            rect!(x = hx as i32 - 4, y = 10 + shake_y, w = 14, h = 6, color = color);
        }
        
        // Score
        text!("SCORE: {}", self.score; x = 180 + shake_x, y = 4 + shake_y, font = "small", color = COLOR_GOLD);
        
        // Deliveries
        text!("Gifts: {}", self.deliveries; x = 180 + shake_x, y = 14 + shake_y, font = "small", color = 0x00ff00ff);
        
        // Level
        text!("Lv.{}", self.level; x = 120 + shake_x, y = 4 + shake_y, font = "small", color = 0xffffffff);
        
        // Naughty meter (if > 0)
        if self.naughty_meter > 0 {
            rect!(x = 60 + shake_x, y = 136 + shake_y, w = 50, h = 6, color = 0x333333ff);
            let bar_w = (self.naughty_meter as u32 * 50 / 100).min(50);
            let bar_color = if self.naughty_meter > 60 { 0xff0000ff } else { 0xffaa00ff };
            rect!(x = 60 + shake_x, y = 136 + shake_y, w = bar_w, h = 6, color = bar_color);
            text!("NAUGHTY", x = 60 + shake_x, y = 128 + shake_y, font = "small", color = bar_color);
        }
        
        // Krampus warning
        if self.krampus_warning > 0 && (self.frame / 8) % 2 == 0 {
            text!("!! KRAMPUS COMING !!", x = 60, y = 60, font = "medium", color = 0xff0000ff);
        }
    }
    
    /// Draw tutorial overlay during first game
    fn draw_tutorial(&self) {
        if self.tutorial_timer == 0 { return; }
        
        // Semi-transparent overlay at bottom
        rect!(x = 0, y = 100, w = 256, h = 44, color = 0x000000aa);
        
        // Calculate which step to show (changes every ~2.5 seconds)
        let time_elapsed = 600 - self.tutorial_timer;
        let step = (time_elapsed / 150) as u8; // 4 steps over 10 seconds
        
        // Blinking effect for emphasis
        let show_text = (self.frame / 15) % 2 == 0;
        
        match step {
            0 => {
                // Step 1: Movement
                text!("TUTORIAL", x = 100, y = 104, font = "medium", color = COLOR_GOLD);
                if show_text {
                    text!("[UP/DOWN] Move sleigh", x = 60, y = 120, font = "small", color = 0xffffffff);
                }
                // Arrow indicators
                text!("^", x = 20, y = 108, font = "medium", color = 0x00ff00ff);
                text!("v", x = 20, y = 128, font = "medium", color = 0x00ff00ff);
            }
            1 => {
                // Step 2: Dropping gifts
                text!("DROP GIFTS", x = 88, y = 104, font = "medium", color = COLOR_GOLD);
                if show_text {
                    text!("[ENTER] or [SPACE] to drop", x = 44, y = 120, font = "small", color = 0xffffffff);
                }
                // Key indicator
                rect!(x = 200, y = 115, w = 48, h = 16, color = 0x00aa00ff);
                text!("ENTER", x = 204, y = 118, font = "small", color = 0xffffffff);
            }
            2 => {
                // Step 3: Hit chimneys
                text!("AIM FOR CHIMNEYS!", x = 68, y = 104, font = "medium", color = COLOR_GOLD);
                if show_text {
                    text!("Drop gifts on glowing targets", x = 32, y = 120, font = "small", color = 0xffffffff);
                }
                // Chimney icon
                rect!(x = 230, y = 110, w = 12, h = 20, color = COLOR_CHIMNEY);
                circ!(x = 236, y = 108, d = 8, color = 0xffff00aa);
            }
            _ => {
                // Step 4: Avoid Krampus
                text!("WATCH OUT!", x = 88, y = 104, font = "medium", color = 0xff0000ff);
                if show_text {
                    text!("Krampus attacks if you miss!", x = 36, y = 120, font = "small", color = 0xffaa00ff);
                }
                // Timer remaining
                let secs = self.tutorial_timer / 60;
                text!("Tutorial: {}s", secs; x = 100, y = 136, font = "small", color = 0x888888ff);
            }
        }
    }
    
    /// Button hints for title screen
    fn draw_controls_hint(&self) {
        // Control box
        rect!(x = 8, y = 115, w = 120, h = 24, color = 0x00000088);
        
        // Arrow keys hint
        rect!(x = 12, y = 120, w = 8, h = 14, color = 0x444444ff);
        text!("^", x = 13, y = 118, font = "small", color = 0x00ff00ff);
        text!("v", x = 13, y = 128, font = "small", color = 0x00ff00ff);
        text!("Move", x = 24, y = 123, font = "small", color = 0xccccccff);
        
        // Enter key hint
        rect!(x = 60, y = 120, w = 40, h = 14, color = 0x00aa00ff);
        text!("ENTER", x = 64, y = 123, font = "small", color = 0xffffffff);
        text!("Drop", x = 104, y = 123, font = "small", color = 0xccccccff);
    }
    
    // ========================================================================
    // MAIN UPDATE
    // ========================================================================
    
    pub fn update(&mut self) {
        self.frame += 1;
        
        // Decrease effects
        if self.screen_flash > 0 { self.screen_flash -= 1; }
        if self.screen_shake > 0 { self.screen_shake -= 1; }
        
        // Update snowflakes always
        self.update_snowflakes();
        
        let (shake_x, shake_y) = self.get_shake();
        
        match self.mode {
            // ================================================================
            // TITLE SCREEN
            // ================================================================
            MODE_TITLE => {
                self.draw_background(0, 0);
                self.draw_snowflakes();
                
                // Title
                text!("SANTA", x = 88, y = 30, font = "large", color = 0xff0000ff);
                text!("DELIVERY", x = 72, y = 50, font = "large", color = 0x00aa00ff);
                
                // Sleigh preview
                let preview_y = 80.0 + (self.frame as f32 / 20.0).sin() * 5.0;
                let old_y = self.player_y;
                self.player_y = preview_y;
                self.draw_sleigh(0, 0);
                self.player_y = old_y;
                
                // Instructions
                if (self.frame / 30) % 2 == 0 {
                    text!("Press START to Fly!", x = 64, y = 100, font = "medium", color = 0xffffffff);
                }
                
                // Button hints
                self.draw_controls_hint();
                
                if self.high_score > 0 {
                    text!("Best: {}", self.high_score; x = 100, y = 138, font = "small", color = COLOR_GOLD);
                }
                
                let gp = gamepad::get(0);
                if gp.start.just_pressed() || gp.a.just_pressed() {
                    self.start_game();
                }
            }
            
            // ================================================================
            // DELIVERING MODE
            // ================================================================
            MODE_DELIVERING => {
                // Update game logic
                self.update_scroll();
                self.move_player();
                self.update_chimneys();
                self.drop_gift();
                self.update_gifts();
                self.check_krampus_trigger();
                self.update_krampus_warning();
                
                // Draw
                self.draw_background(shake_x, shake_y);
                self.draw_snowflakes();
                
                // Draw chimneys
                for chimney in &self.chimneys {
                    self.draw_chimney(chimney, shake_x, shake_y);
                }
                
                // Draw falling gifts
                for gift in &self.gifts {
                    if gift.active {
                        self.draw_falling_gift(gift, shake_x, shake_y);
                    }
                }
                
                // Draw sleigh
                self.draw_sleigh(shake_x, shake_y);
                
                // UI
                self.draw_ui(shake_x, shake_y);
                
                // Tutorial overlay (first game only)
                if self.tutorial_timer > 0 {
                    self.tutorial_timer -= 1;
                    self.draw_tutorial();
                }
            }
            
            // ================================================================
            // KRAMPUS ATTACK MODE
            // ================================================================
            MODE_KRAMPUS => {
                // Update
                self.update_scroll();
                self.move_player();
                self.update_krampus();
                self.update_projectiles();
                
                // Darker background during attack
                clear(0x0a0a14ff);
                
                // Draw stars dimmed
                for i in 0..10 {
                    let star_x = ((i * 37 + 10) as f32 - (self.scroll_x * 0.1) % SCREEN_W) as i32;
                    let star_y = (i * 7 % 50 + 5) as i32;
                    circ!(x = star_x + shake_x, y = star_y + shake_y, d = 2, color = 0xffffff33);
                }
                
                // Ground (darker)
                rect!(x = shake_x, y = 120 + shake_y, w = 256, h = 30, color = 0x808090ff);
                
                self.draw_snowflakes();
                
                // Draw sleigh
                self.draw_sleigh(shake_x, shake_y);
                
                // Draw Krampus
                self.draw_krampus(shake_x, shake_y);
                
                // Draw projectiles
                for proj in &self.projectiles {
                    if proj.active {
                        self.draw_projectile(proj, shake_x, shake_y);
                    }
                }
                
                // UI
                self.draw_ui(shake_x, shake_y);
                
                // Survive timer
                let seconds_left = self.krampus_duration / 60;
                text!("Survive: {}s", seconds_left; x = 100, y = 60, font = "medium", color = 0xff6600ff);
            }
            
            // ================================================================
            // GAME OVER
            // ================================================================
            MODE_GAMEOVER => {
                clear(0x0a0a0aff);
                
                text!("GAME OVER", x = 72, y = 35, font = "large", color = 0xff0000ff);
                
                text!("Score: {}", self.score; x = 92, y = 60, font = "medium", color = 0xffffffff);
                text!("Deliveries: {}", self.deliveries; x = 80, y = 78, font = "small", color = 0x00ff00ff);
                text!("Level: {}", self.level; x = 100, y = 92, font = "small", color = 0xaaaaaaff);
                
                if self.score > self.high_score && self.score > 0 {
                    if (self.frame / 15) % 2 == 0 {
                        text!("NEW HIGH SCORE!", x = 60, y = 108, font = "medium", color = COLOR_GOLD);
                    }
                }
                
                if (self.frame / 25) % 2 == 0 {
                    text!("Press START to Retry", x = 56, y = 125, font = "small", color = 0x888888ff);
                }
                
                let gp = gamepad::get(0);
                if gp.start.just_pressed() || gp.a.just_pressed() {
                    self.reset_game();
                }
            }
            
            _ => {}
        }
        
        // Screen flash overlay
        if self.screen_flash > 0 {
            let alpha = ((self.screen_flash as f32 / 15.0) * 180.0) as u32;
            let flash = (self.flash_color & 0xffffff00) | alpha;
            rect!(x = 0, y = 0, w = 256, h = 144, color = flash);
        }
    }
}
