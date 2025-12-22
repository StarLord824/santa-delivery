use turbo::*;

// MODULES

mod types;
mod sound;

use types::*;


#[turbo::game]
struct GameState {
    frame: u32,
    mode: u8,  // 0=Title, 1=Delivering, 2=KrampusAttack, 3=GameOver, 4=Paused
    previous_mode: u8,  // For pause/resume
    
    // Scrolling
    scroll_x: f32,
    scroll_speed: f32,
    base_scroll_speed: f32,  // For progressive difficulty
    
    // Player (Santa's sleigh)
    player_y: f32,
    player_vel_y: f32,
    sleigh_tilt: f32,
    
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
    krampus_warning: u32,
    
    // Stats
    health: u32,
    score: u32,
    high_score: u32,
    deliveries: u32,
    naughty_meter: u32,
    level: u32,
    
    // Combo system
    combo_count: u32,
    combo_timer: u32,  // Frames until combo resets
    max_combo: u32,
    
    // Visual effects
    screen_flash: u32,
    flash_color: u32,
    screen_shake: u32,
    invincible_timer: u32,
    
    // Screen transitions
    fade_alpha: u32,      // 0-255 for fade effect
    fade_direction: i32,  // 1 = fading in, -1 = fading out, 0 = none
    
    // Particles
    snowflakes: Vec<Snowflake>,
    particles: Vec<Particle>,
    
    // Power-ups
    powerups: Vec<PowerUp>,
    powerup_spawn_timer: u32,
    star_power_timer: u32,  // Invincibility from star power-up
    
    // Tutorial
    tutorial_timer: u32,
    first_play: bool,
    tutorial_step: u8,
    
    // Audio settings
    music_volume: f32,  // 0.0 to 1.0
    sfx_volume: f32,    // 0.0 to 1.0
    
    // RNG
    rng_seed: u32,
}

// Game mode constants


// Screen dimensions (larger for better resolution)
// Constants imported from types.rs

impl GameState {
    pub fn new() -> Self {
        let mut state = Self {
            frame: 0,
            mode: MODE_TITLE,
            previous_mode: MODE_TITLE,
            
            scroll_x: 0.0,
            scroll_speed: 1.5,
            base_scroll_speed: 1.5,
            
            player_y: SCREEN_H / 2.0,
            player_vel_y: 0.0,
            sleigh_tilt: 0.0,
            
            chimneys: vec![],
            next_chimney_spawn: 100.0,
            
            gifts: vec![],
            
            krampus_x: SCREEN_W + 50.0,
            krampus_y: SCREEN_H / 2.0,
            krampus_active: false,
            krampus_attack_timer: 1200,
            krampus_duration: 0,
            projectiles: vec![],
            krampus_warning: 0,
            
            health: 3,
            score: 0,
            high_score: 0,
            deliveries: 0,
            naughty_meter: 0,
            level: 1,
            
            // Combo system
            combo_count: 0,
            combo_timer: 0,
            max_combo: 0,
            
            // Visual effects
            screen_flash: 0,
            flash_color: 0xffffffff,
            screen_shake: 0,
            invincible_timer: 0,
            
            // Screen transitions
            fade_alpha: 255,
            fade_direction: 1,  // Fade in on start
            
            // Particles
            snowflakes: vec![],
            particles: vec![],
            
            // Power-ups
            powerups: vec![],
            powerup_spawn_timer: 600,  // First power-up after 10 seconds
            star_power_timer: 0,
            
            // Tutorial
            tutorial_timer: 0,
            first_play: true,
            tutorial_step: 0,
            
            // Audio settings (default full volume)
            music_volume: 1.0,
            sfx_volume: 1.0,
            
            rng_seed: 42,
        };
        state.init_snowflakes();
        state.load_high_score();
        state
    }
    
    // RANDOM
    
    fn random(&mut self) -> u32 {
        self.rng_seed = self.rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
        (self.rng_seed >> 16) & 0x7FFF
    }
    
    fn random_range(&mut self, min: f32, max: f32) -> f32 {
        let r = (self.random() % 1000) as f32 / 1000.0;
        min + r * (max - min)
    }
    
    // AUDIO SYSTEM
    
    /// Play background music based on current game mode
    fn play_mode_music(&self) {
        // Stop all music tracks first
        audio::stop("start");
        audio::stop("game");
        audio::stop("krampus");
        audio::stop("game_over");
        
        // Play appropriate track for current mode
        match self.mode {
            MODE_TITLE => audio::play("start"),
            MODE_DELIVERING => audio::play("game"),
            MODE_KRAMPUS => audio::play("krampus"),
            MODE_GAMEOVER => audio::play("game_over"),
            _ => {}
        }
    }
    
    /// Keep music looping (call every frame)
    fn update_music(&self) {
        match self.mode {
            MODE_TITLE => {
                if !audio::is_playing("start") {
                    audio::play("start");
                }
            }
            MODE_DELIVERING => {
                if !audio::is_playing("game") {
                    audio::play("game");
                }
            }
            MODE_KRAMPUS => {
                if !audio::is_playing("krampus") {
                    audio::play("krampus");
                }
            }
            MODE_GAMEOVER => {
                if !audio::is_playing("game_over") {
                    audio::play("game_over");
                }
            }
            _ => {}
        }
    }
    
    /// Play a one-shot sound effect
    fn play_sfx(name: &str) {
        audio::play(name);
    }
    
    // INITIALIZATION
    
    fn init_snowflakes(&mut self) {
        self.snowflakes.clear();
        // More snowflakes for larger screen
        for _ in 0..50 {
            let x = self.random_range(0.0, SCREEN_W);
            let y = self.random_range(0.0, SCREEN_H);
            let speed = self.random_range(0.5, 1.8);
            let size = (self.random() % 3 + 1) as u32;
            self.snowflakes.push(Snowflake { x, y, speed, size });
        }
    }
    
    fn start_game(&mut self) {
        self.mode = MODE_DELIVERING;
        self.scroll_x = 0.0;
        self.scroll_speed = 1.5;
        self.base_scroll_speed = 1.5;
        self.player_y = SCREEN_H / 2.0;
        self.player_vel_y = 0.0;
        self.sleigh_tilt = 0.0;
        
        self.chimneys.clear();
        self.next_chimney_spawn = 150.0;
        self.gifts.clear();
        self.projectiles.clear();
        self.particles.clear();
        self.powerups.clear();
        
        self.krampus_active = false;
        self.krampus_attack_timer = 1200;
        self.krampus_warning = 0;
        self.krampus_duration = 0;
        
        self.health = 3;
        self.score = 0;
        self.deliveries = 0;
        self.naughty_meter = 0;
        self.level = 1;
        self.invincible_timer = 0;
        self.star_power_timer = 0;
        
        // Combo system
        self.combo_count = 0;
        self.combo_timer = 0;
        self.max_combo = 0;
        
        // Power-ups
        self.powerup_spawn_timer = 600;
        
        // Screen transition (fade in)
        self.start_fade_in();
        
        // Tutorial
        if self.first_play {
            self.tutorial_timer = 600;
            self.tutorial_step = 0;
        } else {
            self.tutorial_timer = 0;
        }
        
        self.screen_flash = 8;
        self.flash_color = 0xffffffff;
        
        // Start game music
        self.play_mode_music();
    }
    
    fn reset_game(&mut self) {
        if self.score > self.high_score {
            self.high_score = self.score;
            self.save_high_score();
        }
        self.first_play = false;
        self.start_game();
    }
    
    // ========================================================================
    // HIGH SCORE PERSISTENCE
    // ========================================================================
    
    fn load_high_score(&mut self) {
        // Try to load high score from local storage
        if let Ok(data) = local::load() {
            if data.len() >= 4 {
                self.high_score = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
            }
        }
    }
    
    fn save_high_score(&self) {
        let bytes = self.high_score.to_le_bytes();
        let _ = local::save(&bytes);
    }
    
    // ========================================================================
    // PARTICLE SYSTEM
    // ========================================================================
    
    fn spawn_particles(&mut self, x: f32, y: f32, count: u32, colors: &[u32]) {
        // Pre-generate random values to avoid borrow conflicts
        let mut particle_data: Vec<(f32, f32, u32, u32)> = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let angle = self.random_range(0.0, 6.28);
            let speed = self.random_range(1.0, 4.0);
            let life = self.random() % 30 + 20;
            let size = self.random() % 3 + 2;
            particle_data.push((angle, speed, life, size));
        }
        
        for (i, (angle, speed, life, size)) in particle_data.into_iter().enumerate() {
            let color_idx = i % colors.len();
            self.particles.push(Particle {
                x,
                y,
                vel_x: angle.cos() * speed,
                vel_y: angle.sin() * speed - 2.0,
                life,
                color: colors[color_idx],
                size,
            });
        }
    }
    
    fn spawn_delivery_particles(&mut self, x: f32, y: f32) {
        // Confetti colors for successful delivery
        let colors = [COLOR_GOLD, 0xff4444ff, 0x44ff44ff, 0x4444ffff, 0xff44ffff];
        self.spawn_particles(x, y, 15, &colors);
    }
    
    fn spawn_star_particles(&mut self, x: f32, y: f32) {
        // Golden star particles
        let colors = [COLOR_GOLD, COLOR_STAR, 0xffffaaff];
        self.spawn_particles(x, y, 10, &colors);
    }
    
    fn update_particles(&mut self) {
        for particle in self.particles.iter_mut() {
            particle.x += particle.vel_x;
            particle.y += particle.vel_y;
            particle.vel_y += 0.15;  // Gravity
            if particle.life > 0 {
                particle.life -= 1;
            }
        }
        self.particles.retain(|p| p.life > 0);
    }
    
    // ========================================================================
    // POWER-UP SYSTEM
    // ========================================================================
    
    fn spawn_powerup(&mut self) {
        let kind = if self.random() % 3 == 0 { POWERUP_INVINCIBLE } else { POWERUP_HEALTH };
        let y = self.random_range(40.0, SCREEN_H * 0.6);
        let bob_offset = self.random_range(0.0, 6.28);
        self.powerups.push(PowerUp {
            x: SCREEN_W + 20.0,
            y,
            kind,
            active: true,
            bob_offset,
        });
    }
    
    fn update_powerups(&mut self) {
        // Spawn timer
        if self.powerup_spawn_timer > 0 {
            self.powerup_spawn_timer -= 1;
        } else {
            self.spawn_powerup();
            self.powerup_spawn_timer = 900 + self.random() % 600;
        }
        
        // Collect collision data first to avoid borrow conflicts
        let mut collected: Vec<(f32, f32, u8)> = Vec::new();
        let player_y = self.player_y;
        let scroll_speed = self.scroll_speed;
        
        for powerup in self.powerups.iter_mut() {
            powerup.x -= scroll_speed;
            powerup.bob_offset += 0.1;
            
            let bob_y = powerup.y + (powerup.bob_offset.sin() * 5.0);
            let dx = (PLAYER_X - powerup.x).abs();
            let dy = (player_y - bob_y).abs();
            
            if dx < 25.0 && dy < 20.0 && powerup.active {
                powerup.active = false;
                collected.push((powerup.x, bob_y, powerup.kind));
            }
        }
        
        // Now process collected power-ups
        for (x, y, kind) in collected {
            self.spawn_star_particles(x, y);
            match kind {
                POWERUP_HEALTH => {
                    self.health = (self.health + 1).min(5);
                    Self::play_sfx("delivery");
                }
                POWERUP_INVINCIBLE => {
                    self.star_power_timer = 300;
                    Self::play_sfx("survive");
                }
                _ => {}
            }
        }
        
        // Remove off-screen or collected power-ups
        self.powerups.retain(|p| p.x > -30.0 && p.active);
        
        // Update star power timer
        if self.star_power_timer > 0 {
            self.star_power_timer -= 1;
        }
    }
    
    // ========================================================================
    // COMBO SYSTEM
    // ========================================================================
    
    fn add_combo(&mut self) {
        self.combo_count += 1;
        self.combo_timer = 180;  // 3 seconds to maintain combo
        if self.combo_count > self.max_combo {
            self.max_combo = self.combo_count;
        }
        
        // Bonus points for combos
        let bonus = match self.combo_count {
            2 => 50,
            3 => 100,
            4 => 200,
            5..=9 => 300,
            _ => 500,
        };
        self.score += bonus;
    }
    
    fn update_combo(&mut self) {
        if self.combo_timer > 0 {
            self.combo_timer -= 1;
        } else if self.combo_count > 0 {
            self.combo_count = 0;
        }
    }
    
    fn break_combo(&mut self) {
        self.combo_count = 0;
        self.combo_timer = 0;
    }
    
    // ========================================================================
    // PROGRESSIVE DIFFICULTY
    // ========================================================================
    
    fn update_difficulty(&mut self) {
        // Increase scroll speed based on level
        self.scroll_speed = self.base_scroll_speed + (self.level as f32 - 1.0) * 0.2;
        
        // Cap at reasonable speed
        self.scroll_speed = self.scroll_speed.min(3.5);
    }
    
    fn level_up(&mut self) {
        self.level += 1;
        self.update_difficulty();
        
        // Level up effects
        self.screen_flash = 15;
        self.flash_color = COLOR_GOLD;
        Self::play_sfx("survive");  // Jingle for level up
        
        // Spawn celebration particles
        self.spawn_particles(SCREEN_W / 2.0, SCREEN_H / 2.0, 25, &[COLOR_GOLD, 0xffffffff, 0xff4444ff]);
    }
    
    // ========================================================================
    // SCREEN TRANSITIONS
    // ========================================================================
    
    fn update_fade(&mut self) {
        match self.fade_direction {
            1 => {  // Fading in (alpha decreasing)
                if self.fade_alpha > 8 {
                    self.fade_alpha -= 8;
                } else {
                    self.fade_alpha = 0;
                    self.fade_direction = 0;
                }
            }
            -1 => {  // Fading out (alpha increasing)
                if self.fade_alpha < 247 {
                    self.fade_alpha += 8;
                } else {
                    self.fade_alpha = 255;
                    self.fade_direction = 0;
                }
            }
            _ => {}
        }
    }
    
    fn start_fade_out(&mut self) {
        self.fade_direction = -1;
    }
    
    fn start_fade_in(&mut self) {
        self.fade_alpha = 255;
        self.fade_direction = 1;
    }
    
    // ========================================================================
    // PAUSE SYSTEM
    // ========================================================================
    
    fn toggle_pause(&mut self) {
        if self.mode == MODE_PAUSED {
            self.mode = self.previous_mode;
            self.play_mode_music();
        } else if self.mode == MODE_DELIVERING || self.mode == MODE_KRAMPUS {
            self.previous_mode = self.mode;
            self.mode = MODE_PAUSED;
            audio::stop("game");
            audio::stop("krampus");
        }
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
    
    // CHIMNEY SYSTEM
        
    fn spawn_chimney(&mut self) {
        // Spawn chimneys on the ground (78% of screen height)
        let ground_y = SCREEN_H * 0.78;
        let y = self.random_range(ground_y - 30.0, ground_y - 10.0);
        // Random house style (0-2)
        let style = (self.random_range(0.0, 3.0) as u8).min(2);
        self.chimneys.push(Chimney {
            x: SCREEN_W + 40.0,
            y,
            delivered: false,
            style,
        });
        
        // Next chimney spawn distance (varies, more space for larger screen)
        self.next_chimney_spawn = self.random_range(120.0, 200.0);
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
    
    
    // GIFT DROPPING
    
    
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
            
            // Drop sound
            Self::play_sfx("drop");
        }
    }
    
    fn update_gifts(&mut self) {
        let scroll_speed = self.scroll_speed;
        let mut score_gained = 0u32;
        let mut deliveries_made = 0u32;
        let mut delivery_positions: Vec<(f32, f32)> = Vec::new();  // For particles
        
        // Get chimney positions for collision
        let chimney_data: Vec<(f32, f32, bool)> = self.chimneys
            .iter()
            .map(|c| (c.x, c.y, c.delivered))
            .collect();
        
        for gift in &mut self.gifts {
            if !gift.active { continue; }
            
            // Gift falls with arc
            gift.x -= scroll_speed * 0.3;
            gift.y += gift.vel_y;
            gift.vel_y += 0.15;
            
            // Check collision with chimneys
            for (i, &(cx, cy, delivered)) in chimney_data.iter().enumerate() {
                if !delivered {
                    let dx = gift.x - cx;
                    let dy = gift.y - cy;
                    
                    // GENEROUS hitbox
                    if dx.abs() < 25.0 && dy > -10.0 && dy < 30.0 {
                        gift.active = false;
                        score_gained += 100 + self.level * 10;
                        deliveries_made += 1;
                        delivery_positions.push((cx, cy));
                        
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
        
        // Check for any gifts that fell off screen (missed)
        let had_missed = self.gifts.iter().any(|g| !g.active && g.y > SCREEN_H);
        
        // Apply score and deliveries
        self.score += score_gained;
        self.deliveries += deliveries_made;
        
        // Break combo if missed
        if had_missed {
            self.break_combo();
        }
        
        // Effects on delivery
        if deliveries_made > 0 {
            self.screen_flash = 4;
            self.flash_color = 0x00ff00ff;
            self.naughty_meter = self.naughty_meter.saturating_sub(10);
            Self::play_sfx("delivery");
            
            // Add combo for each delivery
            for _ in 0..deliveries_made {
                self.add_combo();
            }
            
            // Spawn particles at delivery locations
            for (x, y) in delivery_positions {
                self.spawn_delivery_particles(x, y);
            }
            
            // Level up every 5 deliveries
            if self.deliveries % 5 == 0 && self.deliveries > 0 {
                self.level_up();
            }
        }
        
        // Clean up inactive gifts
        self.gifts.retain(|g| g.active);
    }
    
    
    // KRAMPUS SYSTEM
    
    
    fn check_krampus_trigger(&mut self) {
        if self.krampus_active || self.krampus_warning > 0 { return; }
        
        // Trigger conditions: timer or naughty meter
        self.krampus_attack_timer = self.krampus_attack_timer.saturating_sub(1);
        
        if self.krampus_attack_timer == 0 || self.naughty_meter >= 80 {
            // Start warning countdown
            self.krampus_warning = 120; // 2 seconds warning
            self.screen_shake = 10;
            
            // Warning sound
            Self::play_sfx("warning");
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
                
                // Krampus attack music and sound
                self.play_mode_music();
                Self::play_sfx("krampus");
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
            
            // Back to normal music + victory sound
            self.play_mode_music();
            Self::play_sfx("survive");
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
            
            // Hit sound
            Self::play_sfx("hit");
            
            if self.health == 0 {
                self.mode = MODE_GAMEOVER;
                self.screen_shake = 25;
                self.invincible_timer = 0;
                
                // Game over music and sound
                self.play_mode_music();
                Self::play_sfx("game-over");
            }
        }
        
        // Decrement invincibility
        if self.invincible_timer > 0 {
            self.invincible_timer -= 1;
        }
        
        self.projectiles.retain(|p| p.active);
    }
    
    
    // SCROLLING & PARALLAX
    
    
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
    
    
    // GET SHAKE OFFSET
    
    
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
    
    
    // DRAWING
    
    
    fn draw_background(&self, shake_x: i32, shake_y: i32) {
        // Level-based sky color
        let sky_idx = ((self.level - 1) as usize).min(4);
        let sky_color = SKY_COLORS[sky_idx];
        clear(sky_color);
        
        // Aurora effect at higher levels (level 3+)
        if self.level >= 3 {
            let aurora_offset = (self.frame as f32 / 30.0).sin() * 20.0;
            for i in 0..5u32 {
                let ay = 30 + i as i32 * 8 + aurora_offset as i32;
                let alpha = 0x22u32 - i * 0x04;
                let color = if self.level >= 4 { 0x8800ff00 + alpha } else { 0x00ff8800 + alpha };
                rect!(x = 0, y = ay, w = SCREEN_W as u32, h = 6, color = color);
            }
        }
        
        // Stars (far layer) - enhanced with twinkle effect
        for i in 0..40u32 {
            let star_x = ((i * 47 + 10) as f32 - (self.scroll_x * 0.1) % SCREEN_W) as i32;
            let star_y = (i * 7 % 90 + 5) as i32;
            let twinkle = if (self.frame + i * 17) % 60 < 30 { 0xffffffff } else { 0xffffff88 };
            let size = if i % 5 == 0 { 3 } else { 2 };
            circ!(x = star_x + shake_x, y = star_y + shake_y, d = size, color = twinkle);
            // Add cross sparkle for brighter stars
            if i % 7 == 0 {
                rect!(x = star_x + shake_x - 2, y = star_y + shake_y, w = 5, h = 1, color = 0xffffff66);
                rect!(x = star_x + shake_x, y = star_y + shake_y - 2, w = 1, h = 5, color = 0xffffff66);
            }
        }
        
        // Moon (positioned in upper right, properly sized)
        let moon_x = 340 + shake_x;
        let moon_y = 25 + shake_y;
        // Moon glow (draw first, behind)
        circ!(x = moon_x, y = moon_y, d = 45, color = 0xffff8811);
        circ!(x = moon_x, y = moon_y, d = 38, color = 0xffff8822);
        // Main moon
        let moon_color = if self.level >= 3 { 0xffddaaff } else { 0xfff8e0ff };
        circ!(x = moon_x, y = moon_y, d = 28, color = moon_color);
        circ!(x = moon_x + 4, y = moon_y - 2, d = 22, color = sky_color); // Crescent shadow
        
        // Mountains (mid layer) - larger for bigger screen
        let mountain_offset = (self.scroll_x * 0.2) as i32 % 180;
        for i in 0..4 {
            let mx = i * 180 - mountain_offset + shake_x;
            let my = 110 + shake_y;
            // Larger mountains
            let mountain_color = if self.level >= 4 { 0x1a2040ff } else { 0x2a3f5fff };
            for row in 0..60 {
                let width = row * 4;
                rect!(x = mx + 90 - width / 2, y = my + row, w = width as u32, h = 1, color = mountain_color);
            }
            // Snow cap
            for row in 0..15 {
                let width = row * 2;
                rect!(x = mx + 90 - width / 2, y = my + row, w = width as u32, h = 1, color = 0xddddddff);
            }
        }
        
        // Pine trees with VARIATION (3 different styles)
        // Fixed: Use spacing that matches tree count for seamless scrolling
        let tree_spacing = 60;
        let tree_offset = (self.scroll_x * 0.5) as i32 % tree_spacing;
        for i in 0..10 {  // More trees to cover screen + buffer
            let tx = i * tree_spacing - tree_offset + shake_x;
            let tree_style = i % 3; // 3 different tree styles
            let size_mult = match i % 4 { 0 => 1.2, 1 => 0.8, 2 => 1.0, _ => 0.9 };
            let ty = 155 + shake_y + if i % 2 == 0 { 0 } else { 5 }; // Slight Y variation
            
            let tree_green = if self.level >= 3 { 0x1a4a2aff } else { 0x2a6a3aff };
            let tree_dark = if self.level >= 3 { 0x0f2a1aff } else { 0x1a4a2aff };
            
            match tree_style {
                0 => {
                    // Classic pine - tall and narrow
                    let h = (50.0 * size_mult) as i32;
                    rect!(x = tx - 3, y = ty, w = 6, h = 15, color = 0x5a3020ff);
                    for layer in 0..6 {
                        let lw = ((22 - layer * 3) as f32 * size_mult) as i32;
                        let ly = ty - 5 - (layer as f32 * 8.0 * size_mult) as i32;
                        rect!(x = tx - lw / 2, y = ly, w = lw as u32, h = 10, color = tree_green);
                    }
                    circ!(x = tx, y = ty - h, d = 8, color = 0xf8f8ffee);
                }
                1 => {
                    // Bushy pine - wider and shorter
                    rect!(x = tx - 4, y = ty, w = 8, h = 12, color = 0x5a3020ff);
                    for layer in 0..4 {
                        let lw = ((32 - layer * 6) as f32 * size_mult) as i32;
                        let ly = ty - 4 - (layer as f32 * 10.0 * size_mult) as i32;
                        rect!(x = tx - lw / 2, y = ly, w = lw as u32, h = 12, color = tree_green);
                        rect!(x = tx - lw / 2 + 2, y = ly + 2, w = (lw - 4) as u32, h = 8, color = tree_dark);
                    }
                    // Snow patches
                    ellipse!(x = tx - 8, y = ty - 10, w = 16, h = 4, color = 0xf8f8ffcc);
                    ellipse!(x = tx + 4, y = ty - 25, w = 10, h = 3, color = 0xf8f8ffcc);
                }
                _ => {
                    // Decorated pine - Christmas tree style
                    rect!(x = tx - 3, y = ty, w = 6, h = 14, color = 0x5a3020ff);
                    for layer in 0..5 {
                        let lw = ((26 - layer * 4) as f32 * size_mult) as i32;
                        let ly = ty - 6 - (layer as f32 * 9.0 * size_mult) as i32;
                        rect!(x = tx - lw / 2, y = ly, w = lw as u32, h = 11, color = tree_green);
                    }
                    // Star on top
                    circ!(x = tx, y = ty - 48, d = 6, color = COLOR_GOLD);
                    // Snow
                    circ!(x = tx, y = ty - 42, d = 8, color = 0xf8f8ffdd);
                    rect!(x = tx - 10, y = ty - 8, w = 20, h = 3, color = 0xf8f8ffbb);
                }
            }
        }
        
        // Ground/snow layer - adjusted for larger screen
        let ground_y = (SCREEN_H * 0.78) as i32;
        rect!(x = shake_x, y = ground_y + shake_y, w = SCREEN_W as u32, h = 50, color = COLOR_SNOW);
        
        // Ground line with subtle shadow
        rect!(x = shake_x, y = ground_y - 1 + shake_y, w = SCREEN_W as u32, h = 2, color = 0xc0d0e0ff);
        
        // Snow mounds (decorative)
        for i in 0..8 {
            let mound_x = (i * 60 + 20) as i32 - ((self.scroll_x * 0.4) as i32 % 60) + shake_x;
            ellipse!(x = mound_x, y = ground_y + 10 + shake_y, w = 30 + (i % 3) as u32 * 10, h = 10, color = 0xf8f8ffff);
        }
    }
    
    fn draw_snowflakes(&self) {
        for snow in &self.snowflakes {
            // Enhanced snowflake with multiple layers
            let x = snow.x as i32;
            let y = snow.y as i32;
            let s = snow.size;
            circ!(x = x, y = y, d = s + 1, color = 0xffffffcc);
            if s > 1 {
                // Add sparkle cross pattern for larger flakes
                rect!(x = x - s as i32 / 2, y = y, w = s, h = 1, color = 0xffffffaa);
                rect!(x = x, y = y - s as i32 / 2, w = 1, h = s, color = 0xffffffaa);
            }
        }
    }
    
    fn draw_chimney(&self, chimney: &Chimney, shake_x: i32, shake_y: i32) {
        let cx = chimney.x as i32 + shake_x;
        let cy = chimney.y as i32 + shake_y;
        
        match chimney.style {
            0 => {
                // STYLE 0: Cozy cottage (brown, warm)
                // House base
                rect!(x = cx - 25, y = cy + 10, w = 50, h = 35, color = 0x5a4030ff);
                rect!(x = cx - 22, y = cy + 12, w = 44, h = 31, color = 0x6a5040ff);
                
                // Windows (lit, warm glow)
                rect!(x = cx - 15, y = cy + 18, w = 10, h = 10, color = 0xffdd66ff);
                rect!(x = cx + 5, y = cy + 18, w = 10, h = 10, color = 0xffdd66ff);
                // Window frames
                rect!(x = cx - 15, y = cy + 22, w = 10, h = 1, color = 0x3a2a20ff);
                rect!(x = cx - 11, y = cy + 18, w = 1, h = 10, color = 0x3a2a20ff);
                rect!(x = cx + 5, y = cy + 22, w = 10, h = 1, color = 0x3a2a20ff);
                rect!(x = cx + 9, y = cy + 18, w = 1, h = 10, color = 0x3a2a20ff);
                // Door
                rect!(x = cx - 4, y = cy + 30, w = 8, h = 12, color = 0x4a3020ff);
                circ!(x = cx + 2, y = cy + 36, d = 2, color = COLOR_GOLD);
                
                // Roof (red triangular)
                for row in 0..20 {
                    let width = 60 - row * 3;
                    rect!(x = cx - width / 2, y = cy + 10 - row, w = width as u32, h = 1, color = 0x8b1a1aff);
                }
                // Snow on roof
                for row in 0..4 {
                    let width = 58 - row * 4;
                    rect!(x = cx - width / 2, y = cy + 8 - row, w = width as u32, h = 1, color = 0xf8f8ffff);
                }
                
                // Chimney
                rect!(x = cx + 8, y = cy - 15, w = 14, h = 25, color = 0x8b4513ff);
                rect!(x = cx + 6, y = cy - 17, w = 18, h = 4, color = 0x6b3a10ff);
            }
            1 => {
                // STYLE 1: Tall cabin (green/blue, nordic)
                // House base (taller)
                rect!(x = cx - 22, y = cy + 5, w = 44, h = 40, color = 0x2a4a3aff);
                rect!(x = cx - 19, y = cy + 7, w = 38, h = 36, color = 0x3a5a4aff);
                
                // Round window (attic)
                circ!(x = cx, y = cy + 12, d = 10, color = 0xffee88ff);
                circ!(x = cx, y = cy + 12, d = 6, color = 0xffdd66ff);
                
                // Windows (two small)
                rect!(x = cx - 14, y = cy + 24, w = 8, h = 8, color = 0xffdd66ff);
                rect!(x = cx + 6, y = cy + 24, w = 8, h = 8, color = 0xffdd66ff);
                // Window crosses
                rect!(x = cx - 11, y = cy + 24, w = 1, h = 8, color = 0x2a3a2aff);
                rect!(x = cx + 9, y = cy + 24, w = 1, h = 8, color = 0x2a3a2aff);
                
                // Steep roof (blue/gray)
                for row in 0..28 {
                    let width = 54 - row * 2;
                    rect!(x = cx - width / 2, y = cy + 5 - row, w = width as u32, h = 1, color = 0x4a5a6aff);
                }
                // Snow on roof
                for row in 0..5 {
                    let width = 52 - row * 3;
                    rect!(x = cx - width / 2, y = cy + 3 - row, w = width as u32, h = 1, color = 0xf8f8ffff);
                }
                
                // Chimney (stone)
                rect!(x = cx + 10, y = cy - 28, w = 12, h = 30, color = 0x555555ff);
                rect!(x = cx + 8, y = cy - 30, w = 16, h = 4, color = 0x444444ff);
            }
            _ => {
                // STYLE 2: Wide mansion (gray stone, elegant)
                // House base (wider)
                rect!(x = cx - 35, y = cy + 8, w = 70, h = 38, color = 0x555566ff);
                rect!(x = cx - 32, y = cy + 10, w = 64, h = 34, color = 0x666677ff);
                
                // Windows (three)
                rect!(x = cx - 26, y = cy + 18, w = 12, h = 12, color = 0xffee88ff);
                rect!(x = cx - 6, y = cy + 18, w = 12, h = 12, color = 0xffee88ff);
                rect!(x = cx + 14, y = cy + 18, w = 12, h = 12, color = 0xffee88ff);
                // Window shutters
                rect!(x = cx - 28, y = cy + 18, w = 2, h = 12, color = 0x3a3a4aff);
                rect!(x = cx - 14, y = cy + 18, w = 2, h = 12, color = 0x3a3a4aff);
                rect!(x = cx - 8, y = cy + 18, w = 2, h = 12, color = 0x3a3a4aff);
                rect!(x = cx + 6, y = cy + 18, w = 2, h = 12, color = 0x3a3a4aff);
                rect!(x = cx + 12, y = cy + 18, w = 2, h = 12, color = 0x3a3a4aff);
                rect!(x = cx + 26, y = cy + 18, w = 2, h = 12, color = 0x3a3a4aff);
                
                // Grand door
                rect!(x = cx - 5, y = cy + 32, w = 10, h = 14, color = 0x4a3a2aff);
                circ!(x = cx, y = cy + 35, d = 8, color = 0x5a4a3aff);
                
                // Flat roof with rim
                rect!(x = cx - 38, y = cy + 5, w = 76, h = 5, color = 0x444455ff);
                rect!(x = cx - 36, y = cy + 8, w = 72, h = 2, color = 0x555566ff);
                // Snow on roof
                rect!(x = cx - 36, y = cy + 4, w = 72, h = 3, color = 0xf8f8ffff);
                
                // Two chimneys
                rect!(x = cx - 28, y = cy - 12, w = 10, h = 18, color = 0x555555ff);
                rect!(x = cx - 30, y = cy - 14, w = 14, h = 4, color = 0x444444ff);
                rect!(x = cx + 18, y = cy - 12, w = 10, h = 18, color = 0x555555ff);
                rect!(x = cx + 16, y = cy - 14, w = 14, h = 4, color = 0x444444ff);
            }
        }
        
        // Chimney glow if not delivered (draw above house)
        let chimney_x = match chimney.style {
            0 => cx + 15,
            1 => cx + 16,
            _ => cx - 23, // Left chimney for mansion
        };
        let chimney_y = match chimney.style {
            0 => cy - 12,
            1 => cy - 25,
            _ => cy - 10,
        };
        
        if !chimney.delivered {
            let pulse = ((self.frame as f32 / 8.0).sin() * 30.0) as u32;
            let glow_color = 0xffff0000 + (pulse << 24);
            circ!(x = chimney_x, y = chimney_y, d = 22 + (pulse / 8) as u32, color = glow_color);
        } else {
            circ!(x = chimney_x, y = chimney_y, d = 18, color = 0x00ff0088);
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
            circ!(x = x + 24, y = y + 4, d = 55, color = 0xffffff33);
        }
        
        // Shadow (on ground)
        let ground_y = (SCREEN_H * 0.78) as i32;
        ellipse!(x = x + 24, y = ground_y + 8 + shake_y, w = 55, h = 12, color = 0x00000044);
        
        // Sleigh body (detailed with trim)
        rect!(x = x - 4, y = y + tilt / 2, w = 34, h = 14, color = 0xcc0000ff);
        rect!(x = x - 2, y = y + 2 + tilt / 2, w = 30, h = 10, color = 0xee2222ff);
        rect!(x = x - 6, y = y + 12 + tilt / 2, w = 40, h = 5, color = COLOR_GOLD);
        // Sleigh back rest
        rect!(x = x - 6, y = y - 4 + tilt / 2, w = 4, h = 16, color = 0xcc0000ff);
        
        // Runner (curved with detail)
        rect!(x = x - 8, y = y + 17 + tilt / 2, w = 44, h = 3, color = 0x555555ff);
        rect!(x = x - 10, y = y + 15 + tilt / 2, w = 4, h = 5, color = 0x555555ff);
        // Runner curve detail
        circ!(x = x - 8, y = y + 17 + tilt / 2, d = 4, color = 0x666666ff);
        
        // === SANTA (highly detailed) ===
        // Body (red coat)
        rect!(x = x + 2, y = y + tilt / 2, w = 16, h = 12, color = 0xdd0000ff);
        rect!(x = x + 4, y = y + 2 + tilt / 2, w = 12, h = 8, color = 0xcc0000ff);
        // White fur trim on coat
        rect!(x = x + 2, y = y + 10 + tilt / 2, w = 16, h = 2, color = 0xffffffff);
        
        // Face (detailed)
        circ!(x = x + 10, y = y - 4 + tilt / 2, d = 14, color = 0xffdbacff); // Face
        // Eyes
        circ!(x = x + 7, y = y - 6 + tilt / 2, d = 3, color = 0x000000ff);  // Left eye
        circ!(x = x + 13, y = y - 6 + tilt / 2, d = 3, color = 0x000000ff); // Right eye
        // Eye highlights
        circ!(x = x + 6, y = y - 7 + tilt / 2, d = 1, color = 0xffffffff);
        circ!(x = x + 12, y = y - 7 + tilt / 2, d = 1, color = 0xffffffff);
        // Rosy cheeks
        circ!(x = x + 4, y = y - 3 + tilt / 2, d = 4, color = 0xffaaaa88);
        circ!(x = x + 16, y = y - 3 + tilt / 2, d = 4, color = 0xffaaaa88);
        // Nose
        circ!(x = x + 10, y = y - 3 + tilt / 2, d = 4, color = 0xffccaaff);
        // Smile
        rect!(x = x + 7, y = y - 1 + tilt / 2, w = 6, h = 1, color = 0xcc8888ff);
        
        // White beard
        circ!(x = x + 10, y = y + 4 + tilt / 2, d = 12, color = 0xffffffff);
        circ!(x = x + 6, y = y + 2 + tilt / 2, d = 8, color = 0xffffffff);
        circ!(x = x + 14, y = y + 2 + tilt / 2, d = 8, color = 0xffffffff);
        // Beard point
        rect!(x = x + 8, y = y + 8 + tilt / 2, w = 4, h = 4, color = 0xffffffff);
        
        // Hat (detailed)
        rect!(x = x + 4, y = y - 12 + tilt / 2, w = 14, h = 10, color = 0xff0000ff);
        rect!(x = x + 12, y = y - 16 + tilt / 2, w = 8, h = 6, color = 0xff0000ff); // Hat tip bent
        circ!(x = x + 18, y = y - 14 + tilt / 2, d = 6, color = 0xffffffff); // Pom
        // Hat band (white fur)
        rect!(x = x + 4, y = y - 4 + tilt / 2, w = 14, h = 3, color = 0xffffffff);
        
        // Arms (holding reins)
        rect!(x = x + 18, y = y + 2 + tilt / 2, w = 12, h = 4, color = 0xdd0000ff); // Arm
        circ!(x = x + 28, y = y + 4 + tilt / 2, d = 5, color = 0xffdbacff); // Hand
        // Reins
        rect!(x = x + 28, y = y + 4 + tilt / 3, w = 12, h = 1, color = 0x8b4513ff);
        
        // === REINDEER (detailed) ===
        // Body
        rect!(x = x + 40, y = y + 2 + tilt / 3, w = 22, h = 12, color = 0x8b4513ff);
        rect!(x = x + 42, y = y + 4 + tilt / 3, w = 18, h = 8, color = 0x9b5523ff);
        // Head
        circ!(x = x + 64, y = y + 2 + tilt / 3, d = 12, color = 0x8b4513ff);
        // Snout
        ellipse!(x = x + 68, y = y + 4 + tilt / 3, w = 8, h = 6, color = 0x9b5523ff);
        // Ears
        circ!(x = x + 58, y = y - 4 + tilt / 3, d = 5, color = 0x8b4513ff);
        circ!(x = x + 66, y = y - 4 + tilt / 3, d = 5, color = 0x8b4513ff);
        // Antlers (branched)
        rect!(x = x + 56, y = y - 12 + tilt / 3, w = 2, h = 10, color = 0x5a3010ff);
        rect!(x = x + 54, y = y - 14 + tilt / 3, w = 6, h = 2, color = 0x5a3010ff);
        rect!(x = x + 64, y = y - 12 + tilt / 3, w = 2, h = 10, color = 0x5a3010ff);
        rect!(x = x + 62, y = y - 14 + tilt / 3, w = 6, h = 2, color = 0x5a3010ff);
        // Eye
        circ!(x = x + 62, y = y + 1 + tilt / 3, d = 3, color = 0x000000ff);
        // RED NOSE (glowing!)
        circ!(x = x + 72, y = y + 4 + tilt / 3, d = 6, color = 0xff0000ff);
        circ!(x = x + 72, y = y + 4 + tilt / 3, d = 10, color = 0xff000033); // Glow
        // Legs
        rect!(x = x + 44, y = y + 12 + tilt / 3, w = 3, h = 7, color = 0x6a3010ff);
        rect!(x = x + 52, y = y + 12 + tilt / 3, w = 3, h = 7, color = 0x6a3010ff);
        // Tail
        circ!(x = x + 38, y = y + 4 + tilt / 3, d = 4, color = 0x8b4513ff);
    }
    
    fn draw_falling_gift(&self, gift: &FallingGift, shake_x: i32, shake_y: i32) {
        let x = gift.x as i32 + shake_x;
        let y = gift.y as i32 + shake_y;
        
        // Gift box with ribbon (detailed)
        rect!(x = x - 7, y = y - 7, w = 14, h = 14, color = 0xff0000ff);
        rect!(x = x - 6, y = y - 6, w = 12, h = 12, color = 0xcc0000ff);
        // Ribbon
        rect!(x = x - 1, y = y - 7, w = 3, h = 14, color = COLOR_GOLD);
        rect!(x = x - 7, y = y - 1, w = 14, h = 3, color = COLOR_GOLD);
        // Bow
        circ!(x = x - 2, y = y - 5, d = 4, color = COLOR_GOLD);
        circ!(x = x + 2, y = y - 5, d = 4, color = COLOR_GOLD);
    }
    
    fn draw_krampus(&self, shake_x: i32, shake_y: i32) {
        if !self.krampus_active { return; }
        
        let x = self.krampus_x as i32 + shake_x;
        let y = self.krampus_y as i32 + shake_y;
        let shake = ((self.frame as f32 / 2.0).sin() * 3.0) as i32;
        let wing_flap = ((self.frame as f32 / 5.0).sin() * 8.0) as i32;
        
        // Ominous red aura (pulsing)
        let aura_pulse = ((self.frame as f32 / 8.0).sin() * 20.0) as u32;
        circ!(x = x, y = y, d = 70 + aura_pulse, color = 0x44000022);
        circ!(x = x, y = y, d = 55 + aura_pulse, color = 0x66000033);
        
        // Wings/cape (flapping)
        rect!(x = x + 12, y = y - 18 + wing_flap / 2, w = 25, h = 30, color = 0x1a0a0aff);
        rect!(x = x + 6, y = y - 12 - wing_flap / 2, w = 28, h = 24, color = 0x1a0a0aff);
        
        // Body (larger, more menacing)
        rect!(x = x - 16 + shake, y = y - 12, w = 32, h = 28, color = 0x2a1a1aff);
        rect!(x = x - 14 + shake, y = y - 10, w = 28, h = 24, color = 0x3a2020ff);
        
        // Fur texture lines
        for i in 0..5 {
            rect!(x = x - 12 + i * 6 + shake, y = y - 8 + (i % 2) * 4, w = 2, h = 20, color = 0x4a2a2aff);
        }
        
        // Head
        circ!(x = x + shake, y = y - 16, d = 22, color = 0x3a2020ff);
        
        // Horns (larger, curved look)
        rect!(x = x - 16 + shake, y = y - 36, w = 6, h = 22, color = 0x4a3030ff);
        rect!(x = x - 18 + shake, y = y - 38, w = 6, h = 8, color = 0x5a4040ff);
        rect!(x = x + 10 + shake, y = y - 36, w = 6, h = 22, color = 0x4a3030ff);
        rect!(x = x + 12 + shake, y = y - 38, w = 6, h = 8, color = 0x5a4040ff);
        
        // Glowing eyes (intense)
        let eye_glow = ((self.frame as f32 / 3.0).sin() * 60.0).abs() as u32;
        circ!(x = x - 6 + shake, y = y - 18, d = 8, color = 0xff0000ff);
        circ!(x = x + 6 + shake, y = y - 18, d = 8, color = 0xff0000ff);
        circ!(x = x - 6 + shake, y = y - 18, d = 12, color = 0xff000044 + (eye_glow << 24));
        circ!(x = x + 6 + shake, y = y - 18, d = 12, color = 0xff000044 + (eye_glow << 24));
        
        // Menacing grin with fangs
        rect!(x = x - 8 + shake, y = y - 8, w = 16, h = 4, color = 0x000000ff);
        for i in 0..5 {
            rect!(x = x - 7 + i * 3 + shake, y = y - 9, w = 2, h = 3, color = 0xffffffee);
        }
        
        // Chains (swinging)
        let chain_swing = ((self.frame as f32 / 8.0).sin() * 4.0) as i32;
        for i in 0..5 {
            let cx = x - 25 - i * 5 + chain_swing;
            let cy = y + 12 + i * 4;
            circ!(x = cx, y = cy, d = 5, color = 0x666666ff);
        }
        
        // Claws
        rect!(x = x - 22 + shake, y = y + 10, w = 8, h = 10, color = 0x1a0a0aff);
        rect!(x = x + 14 + shake, y = y + 10, w = 8, h = 10, color = 0x1a0a0aff);
    }
    
    fn draw_projectile(&self, proj: &Projectile, shake_x: i32, shake_y: i32) {
        let x = proj.x as i32 + shake_x;
        let y = proj.y as i32 + shake_y;
        
        // Flame trail (glow circles)
        for i in 1..=4i32 {
            let trail_x = x + (proj.vel_x * i as f32 * 2.5) as i32;
            let trail_y = y + (proj.vel_y * i as f32 * 2.5) as i32;
            let alpha = 0x88 - i as u32 * 0x18;
            circ!(x = trail_x, y = trail_y, d = (12 - i * 2) as u32, color = 0xff440000 + alpha);
        }
        
        // Core fireball with glow layers
        circ!(x = x, y = y, d = 16, color = 0x44000044); // Outer glow
        circ!(x = x, y = y, d = 12, color = 0x880000ff); // Dark core
        circ!(x = x, y = y, d = 9, color = 0xff2200ff);  // Fire
        circ!(x = x, y = y, d = 5, color = 0xffcc00ff);  // Hot center
    }
    
    fn draw_ui(&self, shake_x: i32, shake_y: i32) {
        // Health hearts (detailed)
        for i in 0..3 {
            let hx = 12 + i * 22 + shake_x as u32;
            let hy = 12 + shake_y;
            let color = if i < self.health { 0xff0000ff } else { 0x444444ff };
            let highlight = if i < self.health { 0xff6666ff } else { 0x555555ff };
            // Heart shape
            circ!(x = hx as i32 - 3, y = hy, d = 10, color = color);
            circ!(x = hx as i32 + 3, y = hy, d = 10, color = color);
            rect!(x = hx as i32 - 8, y = hy, w = 16, h = 8, color = color);
            // Point of heart
            for row in 0..6 {
                let w = 16 - row * 3;
                if w > 0 {
                    rect!(x = hx as i32 - w as i32 / 2, y = hy + 6 + row, w = w as u32, h = 1, color = color);
                }
            }
            // Highlight
            circ!(x = hx as i32 - 4, y = hy - 2, d = 4, color = highlight);
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
        rect!(x = 100, y = 185, w = 184, h = 26, color = 0x222222ff);
        
        // Arrow keys hint
        rect!(x = 110, y = 190, w = 10, h = 16, color = 0x444444ff);
        text!("^", x = 112, y = 188, font = "small", color = 0x00ff00ff);
        text!("v", x = 112, y = 198, font = "small", color = 0x00ff00ff);
        text!("Move", x = 125, y = 194, font = "small", color = 0xffffffff);
        
        // Enter key hint
        rect!(x = 180, y = 190, w = 45, h = 16, color = 0x00aa00ff);
        text!("ENTER", x = 185, y = 194, font = "small", color = 0xffffffff);
        text!("Drop", x = 230, y = 194, font = "small", color = 0xffffffff);
    }
    
    // ========================================================================
    // PAUSE SCREEN
    // ========================================================================
    
    fn draw_pause_screen(&self) {
        // Semi-transparent overlay
        rect!(x = 0, y = 0, w = SCREEN_W as u32, h = SCREEN_H as u32, color = 0x000000aa);
        
        // Pause panel
        rect!(x = 120, y = 70, w = 144, h = 80, color = 0x222233ee);
        rect!(x = 122, y = 72, w = 140, h = 76, color = 0x111122ff);
        
        text!("PAUSED", x = 155, y = 85, font = "large", color = 0xffffffff);
        
        text!("ESC to Resume", x = 145, y = 115, font = "small", color = 0xaaaaaaff);
        
        // Show current stats
        text!("Score: {}", self.score; x = 155, y = 130, font = "small", color = COLOR_GOLD);
    }
    
    // ========================================================================
    // PARTICLE DRAWING
    // ========================================================================
    
    fn draw_particles(&self, shake_x: i32, shake_y: i32) {
        for particle in &self.particles {
            let alpha = ((particle.life as f32 / 50.0) * 255.0).min(255.0) as u32;
            let color = (particle.color & 0xffffff00) | alpha;
            let px = particle.x as i32 + shake_x;
            let py = particle.y as i32 + shake_y;
            circ!(x = px, y = py, d = particle.size, color = color);
        }
    }
    
    // ========================================================================
    // POWER-UP DRAWING
    // ========================================================================
    
    fn draw_powerups(&self, shake_x: i32, shake_y: i32) {
        for powerup in &self.powerups {
            if !powerup.active { continue; }
            
            let bob_y = powerup.y + (powerup.bob_offset.sin() * 5.0);
            let px = powerup.x as i32 + shake_x;
            let py = bob_y as i32 + shake_y;
            
            // Glow effect
            let glow_size = 20 + ((self.frame as f32 / 10.0).sin() * 3.0) as u32;
            let glow_color = if powerup.kind == POWERUP_HEALTH { 0x44ff4422 } else { 0xffff0022 };
            circ!(x = px, y = py, d = glow_size, color = glow_color);
            
            match powerup.kind {
                POWERUP_HEALTH => {
                    // Candy cane
                    rect!(x = px - 2, y = py - 8, w = 4, h = 16, color = 0xffffffff);
                    rect!(x = px - 1, y = py - 7, w = 2, h = 3, color = 0xff0000ff);
                    rect!(x = px - 1, y = py - 1, w = 2, h = 3, color = 0xff0000ff);
                    rect!(x = px - 1, y = py + 5, w = 2, h = 3, color = 0xff0000ff);
                    // Hook
                    rect!(x = px - 6, y = py - 8, w = 6, h = 4, color = 0xffffffff);
                    rect!(x = px - 7, y = py - 5, w = 4, h = 3, color = 0xff0000ff);
                }
                POWERUP_INVINCIBLE => {
                    // Star
                    let star_color = if (self.frame / 5) % 2 == 0 { COLOR_GOLD } else { COLOR_STAR };
                    // Simple star shape
                    circ!(x = px, y = py, d = 10, color = star_color);
                    rect!(x = px - 1, y = py - 8, w = 3, h = 16, color = star_color);
                    rect!(x = px - 8, y = py - 1, w = 16, h = 3, color = star_color);
                }
                _ => {}
            }
        }
    }
    
    // ========================================================================
    // COMBO DISPLAY
    // ========================================================================
    
    fn draw_combo(&self) {
        if self.combo_count >= 2 {
            let combo_color = match self.combo_count {
                2..=3 => 0x00ff00ff,
                4..=5 => 0xffff00ff,
                _ => 0xff00ffff,
            };
            
            // Pulsing effect
            let pulse = ((self.frame as f32 / 8.0).sin() * 2.0) as i32;
            text!("COMBO x{}", self.combo_count; x = 280 + pulse, y = 50, font = "medium", color = combo_color);
        }
    }
    
    // ========================================================================
    // FADE OVERLAY
    // ========================================================================
    
    fn draw_fade(&self) {
        if self.fade_alpha > 0 {
            let color = 0x000000ff & 0xffffff00 | self.fade_alpha;
            rect!(x = 0, y = 0, w = SCREEN_W as u32, h = SCREEN_H as u32, color = color);
        }
    }
    
    // ========================================================================
    // MAIN UPDATE
    // ========================================================================
    
    pub fn update(&mut self) {
        self.frame += 1;
        
        // Handle pause toggle
        let kb = keyboard::get();
        if kb.escape().just_pressed() && (self.mode == MODE_DELIVERING || self.mode == MODE_KRAMPUS || self.mode == MODE_PAUSED) {
            self.toggle_pause();
        }
        
        // If paused, just draw pause screen and return
        if self.mode == MODE_PAUSED {
            self.draw_pause_screen();
            return;
        }
        
        // Decrease effects
        if self.screen_flash > 0 { self.screen_flash -= 1; }
        if self.screen_shake > 0 { self.screen_shake -= 1; }
        if self.invincible_timer > 0 { self.invincible_timer -= 1; }
        
        // Update particles
        self.update_particles();
        
        // Update fade transitions
        self.update_fade();
        
        // Update combo timer
        self.update_combo();
        
        // Update snowflakes always
        self.update_snowflakes();
        
        // Keep music playing (auto-loop)
        self.update_music();
        
        let (shake_x, shake_y) = self.get_shake();
        
        match self.mode {
            // ================================================================
            // TITLE SCREEN
            // ================================================================
            MODE_TITLE => {
                self.draw_background(0, 0);
                self.draw_snowflakes();
                
                // Animated title position (subtle bounce)
                let title_bounce = ((self.frame as f32 / 25.0).sin() * 3.0) as i32;
                let title_y = 50 + title_bounce;
                
                // Title glow effect (pulsing)
                let glow_alpha = ((self.frame as f32 / 15.0).sin() * 40.0 + 60.0) as u32;
                let glow_color = 0xffff0000 | glow_alpha;
                rect!(x = 98, y = 38 + title_bounce, w = 188, h = 44, color = glow_color);
                
                // Title background panel
                rect!(x = 100, y = 40 + title_bounce, w = 184, h = 40, color = 0x00000099);
                
                // Title text with animation
                // Shadow layer
                text!("SANTA", x = 112, y = title_y + 2, font = "large", color = 0x00000088);
                text!("DELIVERY", x = 177, y = title_y + 2, font = "large", color = 0x00000088);
                // Main text with color pulse
                let santa_red = if (self.frame / 20) % 2 == 0 { 0xff0000ff } else { 0xff2222ff };
                text!("SANTA", x = 110, y = title_y, font = "large", color = santa_red);
                text!("DELIVERY", x = 175, y = title_y, font = "large", color = COLOR_GOLD);
                
                // Sleigh preview (centered)
                let preview_y = 120.0 + (self.frame as f32 / 20.0).sin() * 8.0;
                let old_y = self.player_y;
                self.player_y = preview_y;
                self.draw_sleigh(100, 0); // Offset to center
                self.player_y = old_y;
                
                // Instructions
                if (self.frame / 30) % 2 == 0 {
                    text!("Press ENTER or START to Fly!", x = 100, y = 155, font = "medium", color = 0xffffffff);
                }
                
                // Button hints
                self.draw_controls_hint();
                
                // High score
                if self.high_score > 0 {
                    text!("Best: {}", self.high_score; x = 160, y = 180, font = "small", color = COLOR_GOLD);
                }
                
                // Exit hint
                text!("ESC to Exit", x = 320, y = 200, font = "small", color = 0x666666ff);
                
                // Input handling
                let gp = gamepad::get(0);
                let kb = keyboard::get();
                
                if gp.start.just_pressed() || gp.a.just_pressed() || kb.enter().just_pressed() {
                    self.start_game();
                }
                
                // Exit game with ESC (note: in browser this may just unfocus)
                // Exit game with ESC
                if kb.escape().just_pressed() {
                    std::process::exit(0);
                }
                
                if gp.start.just_pressed() || kb.enter().just_pressed() {
                    Self::play_sfx("start"); // Play sound immediately
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
                self.update_powerups();  // NEW: Power-ups
                
                // Draw
                self.draw_background(shake_x, shake_y);
                self.draw_snowflakes();
                
                // Draw power-ups (behind other elements)
                self.draw_powerups(shake_x, shake_y);
                
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
                
                // Draw particles (above gifts, below sleigh)
                self.draw_particles(shake_x, shake_y);
                
                // Draw sleigh (with star power glow if active)
                if self.star_power_timer > 0 {
                    // Draw aura around sleigh
                    let glow_alpha = ((self.frame as f32 / 5.0).sin() * 50.0 + 150.0) as u32;
                    circ!(x = PLAYER_X as i32 + shake_x, y = self.player_y as i32 + shake_y, d = 50, color = 0xffff0000 | glow_alpha);
                }
                self.draw_sleigh(shake_x, shake_y);
                
                // UI
                self.draw_ui(shake_x, shake_y);
                
                // Combo display
                self.draw_combo();
                
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
                // Ground (darker for Krampus mode, full width)
                let ground_y = (SCREEN_H * 0.78) as i32;
                rect!(x = shake_x, y = ground_y + shake_y, w = SCREEN_W as u32, h = 50, color = 0x404050ff);
                
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
                
                text!("Score: {}", self.score; x = 92, y = 55, font = "medium", color = 0xffffffff);
                text!("Deliveries: {}", self.deliveries; x = 80, y = 73, font = "small", color = 0x00ff00ff);
                text!("Max Combo: {}", self.max_combo; x = 80, y = 87, font = "small", color = 0xff00ffff);
                text!("Level: {}", self.level; x = 100, y = 101, font = "small", color = 0xaaaaaaff);
                
                if self.score > self.high_score && self.score > 0 {
                    if (self.frame / 15) % 2 == 0 {
                        text!("NEW HIGH SCORE!", x = 60, y = 118, font = "medium", color = COLOR_GOLD);
                    }
                }
                
                if (self.frame / 25) % 2 == 0 {
                    text!("Press START to Retry", x = 56, y = 135, font = "small", color = 0x888888ff);
                }
                
                let gp = gamepad::get(0);
                let kb = keyboard::get();
                if gp.start.just_pressed() || gp.a.just_pressed() || kb.enter().just_pressed() {
                    self.reset_game();
                }
            }
            
            _ => {}
        }
        
        // Screen flash overlay
        if self.screen_flash > 0 {
            let alpha = ((self.screen_flash as f32 / 15.0) * 180.0) as u32;
            let flash = (self.flash_color & 0xffffff00) | alpha;
            rect!(x = 0, y = 0, w = SCREEN_W as u32, h = SCREEN_H as u32, color = flash);
        }
        
        // Fade transition overlay
        self.draw_fade();
    }
}
