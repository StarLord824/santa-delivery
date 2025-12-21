use turbo::*;
use borsh::{BorshSerialize, BorshDeserialize};
use serde::{Serialize, Deserialize};

// ============================================================================
// GAME MODES
// ============================================================================

#[derive(Clone, Copy, PartialEq, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
enum GameMode {
    Title,
    Jingle,
    Hurry,
    Krampus,
    GameOver,
}

// ============================================================================
// GIFT STRUCTURE
// ============================================================================

#[derive(Clone, Copy, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
struct Gift {
    x: f32,
    y: f32,
    collected: bool,
}

// ============================================================================
// GAME STATE
// ============================================================================

#[turbo::game]
struct GameState {
    frame: u32,
    mode: GameMode,
    
    // Timers (in seconds)
    jingle_timer: i32,
    hurry_timer: i32,
    krampus_timer: i32,
    
    // Player
    player_x: f32,
    player_y: f32,
    speed: f32,
    
    // Krampus
    krampus_x: f32,
    krampus_y: f32,
    krampus_speed: f32,
    
    // Traps (dynamic array simulation with fixed size)
    traps: [(f32, f32); 5],
    trap_count: usize,
    
    // Gifts
    gifts: [Gift; 6],
    gift_count: usize,
    
    // Scoring & Progression
    score: u32,
    high_score: u32,
    round: u32,
    gifts_collected_this_round: u32,
    
    // Visual effects
    screen_flash: u32,
    flash_color: u32,
    transition_frame: u32,
    
    // Random seed (simple LCG)
    rng_seed: u32,
}

impl GameState {
    pub fn new() -> Self {
        let mut state = Self {
            frame: 0,
            mode: GameMode::Title,
            
            jingle_timer: 15,
            hurry_timer: 6,
            krampus_timer: 10,
            
            player_x: 128.0,
            player_y: 72.0,
            speed: 2.0,
            
            krampus_x: 10.0,
            krampus_y: 10.0,
            krampus_speed: 1.2,
            
            traps: [(0.0, 0.0); 5],
            trap_count: 3,
            
            gifts: [Gift { x: 0.0, y: 0.0, collected: false }; 6],
            gift_count: 4,
            
            score: 0,
            high_score: 0,
            round: 1,
            gifts_collected_this_round: 0,
            
            screen_flash: 0,
            flash_color: 0xffffffff,
            transition_frame: 0,
            
            rng_seed: 12345,
        };
        state.spawn_round_elements();
        state
    }
    
    // Simple pseudo-random number generator
    fn random(&mut self) -> u32 {
        self.rng_seed = self.rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
        (self.rng_seed >> 16) & 0x7FFF
    }
    
    fn random_range(&mut self, min: f32, max: f32) -> f32 {
        let r = (self.random() % 1000) as f32 / 1000.0;
        min + r * (max - min)
    }
    
    // Spawn traps and gifts for current round
    fn spawn_round_elements(&mut self) {
        // Calculate counts based on round (scales up)
        self.trap_count = (2 + self.round as usize).min(5);
        self.gift_count = (3 + (self.round / 2) as usize).min(6);
        
        // Spawn traps at random positions (avoiding center spawn area)
        for i in 0..self.trap_count {
            loop {
                let x = self.random_range(20.0, 236.0);
                let y = self.random_range(20.0, 124.0);
                
                // Avoid spawning near player start position
                let dist_to_center = ((x - 128.0).powi(2) + (y - 72.0).powi(2)).sqrt();
                if dist_to_center > 40.0 {
                    self.traps[i] = (x, y);
                    break;
                }
            }
        }
        
        // Spawn gifts at random positions
        for i in 0..self.gift_count {
            loop {
                let x = self.random_range(24.0, 232.0);
                let y = self.random_range(24.0, 120.0);
                
                // Check distance from traps
                let mut too_close = false;
                for j in 0..self.trap_count {
                    let (tx, ty) = self.traps[j];
                    let dist = ((x - tx).powi(2) + (y - ty).powi(2)).sqrt();
                    if dist < 24.0 {
                        too_close = true;
                        break;
                    }
                }
                
                if !too_close {
                    self.gifts[i] = Gift { x, y, collected: false };
                    break;
                }
            }
        }
    }
    
    fn start_round(&mut self) {
        self.mode = GameMode::Jingle;
        self.jingle_timer = (15 - (self.round as i32 - 1).min(5)).max(8); // Gets shorter each round
        self.hurry_timer = (6 - (self.round as i32 / 3)).max(3);
        self.krampus_timer = (10 - (self.round as i32 / 2)).max(5);
        
        // Increase Krampus speed each round
        self.krampus_speed = 1.2 + (self.round as f32 - 1.0) * 0.15;
        
        // Reset player to center
        self.player_x = 128.0;
        self.player_y = 72.0;
        
        self.gifts_collected_this_round = 0;
        self.transition_frame = 10;
        
        // Respawn elements
        self.spawn_round_elements();
        
        // Flash effect
        self.screen_flash = 8;
        self.flash_color = 0xffffffff;
    }
    
    fn reset_game(&mut self) {
        if self.score > self.high_score {
            self.high_score = self.score;
        }
        self.score = 0;
        self.round = 1;
        self.krampus_speed = 1.2;
        self.start_round();
    }

    fn tick_timer(&mut self) {
        if self.frame % 60 == 0 {
            match self.mode {
                GameMode::Jingle => self.jingle_timer -= 1,
                GameMode::Hurry => self.hurry_timer -= 1,
                GameMode::Krampus => self.krampus_timer -= 1,
                _ => {}
            }
        }
    }

    fn move_player(&mut self) {
        let mut dx = 0.0;
        let mut dy = 0.0;

        // Arrow keys
        if keyboard().is_down(Key::Left) || keyboard().is_down(Key::A) { dx -= self.speed; }
        if keyboard().is_down(Key::Right) || keyboard().is_down(Key::D) { dx += self.speed; }
        if keyboard().is_down(Key::Up) || keyboard().is_down(Key::W) { dy -= self.speed; }
        if keyboard().is_down(Key::Down) || keyboard().is_down(Key::S) { dy += self.speed; }
        
        // Normalize diagonal movement
        if dx != 0.0 && dy != 0.0 {
            let factor = 0.707; // 1/sqrt(2)
            dx *= factor;
            dy *= factor;
        }

        self.player_x = (self.player_x + dx).clamp(12.0, 244.0);
        self.player_y = (self.player_y + dy).clamp(16.0, 132.0);
    }
    
    fn collect_gifts(&mut self) {
        for i in 0..self.gift_count {
            if !self.gifts[i].collected {
                let gx = self.gifts[i].x;
                let gy = self.gifts[i].y;
                
                let dist = ((self.player_x - gx).powi(2) + (self.player_y - gy).powi(2)).sqrt();
                if dist < 12.0 {
                    self.gifts[i].collected = true;
                    self.score += 25;
                    self.gifts_collected_this_round += 1;
                    
                    // Small flash effect
                    self.screen_flash = 3;
                    self.flash_color = 0x00ff00ff; // Green flash
                }
            }
        }
    }
    
    fn all_gifts_collected(&self) -> bool {
        for i in 0..self.gift_count {
            if !self.gifts[i].collected {
                return false;
            }
        }
        true
    }

    fn stepped_on_trap(&self) -> bool {
        for i in 0..self.trap_count {
            let (tx, ty) = self.traps[i];
            let dist = ((self.player_x - tx).powi(2) + (self.player_y - ty).powi(2)).sqrt();
            if dist < 10.0 {
                return true;
            }
        }
        false
    }

    fn update_krampus(&mut self) {
        let dx = self.player_x - self.krampus_x;
        let dy = self.player_y - self.krampus_y;
        let dist = (dx * dx + dy * dy).sqrt().max(0.01);

        self.krampus_x += dx / dist * self.krampus_speed;
        self.krampus_y += dy / dist * self.krampus_speed;

        // Collision check
        if dist < 12.0 {
            self.mode = GameMode::GameOver;
            self.screen_flash = 15;
            self.flash_color = 0xff0000ff;
        }
    }
    
    fn draw_timer_bar(&self) {
        let (current, max, color) = match self.mode {
            GameMode::Jingle => (self.jingle_timer, 15, 0x4ade80ff), // Green
            GameMode::Hurry => (self.hurry_timer, 6, 0xfbbf24ff),   // Yellow
            GameMode::Krampus => (self.krampus_timer, 10, 0xef4444ff), // Red
            _ => (0, 1, 0x666666ff),
        };
        
        let bar_width = ((current as f32 / max as f32) * 200.0) as u32;
        
        // Background
        rect!(x = 28, y = 2, w = 200, h = 6, color = 0x333333ff);
        // Timer bar
        rect!(x = 28, y = 2, w = bar_width, h = 6, color = color);
    }
    
    fn draw_mode_indicator(&self) {
        let (mode_text, color) = match self.mode {
            GameMode::Title => ("", 0x000000ff),
            GameMode::Jingle => ("JINGLE", 0x4ade80ff),
            GameMode::Hurry => ("HURRY!", 0xfbbf24ff),
            GameMode::Krampus => ("KRAMPUS", 0xef4444ff),
            GameMode::GameOver => ("", 0x000000ff),
        };
        
        if !mode_text.is_empty() {
            text!(mode_text, x = 232, y = 4, font = "pixel", color = color);
        }
    }

    pub fn update(&mut self) {
        self.frame += 1;
        
        // Decrease screen flash
        if self.screen_flash > 0 {
            self.screen_flash -= 1;
        }
        
        // Decrease transition frame
        if self.transition_frame > 0 {
            self.transition_frame -= 1;
        }
        
        self.tick_timer();

        match self.mode {
            // ================================================================
            // TITLE SCREEN
            // ================================================================
            GameMode::Title => {
                // Animated background
                let bg_pulse = ((self.frame as f32 / 30.0).sin() * 20.0) as u32;
                clear(0x1a3a2aff + (bg_pulse << 8));
                
                // Title
                text!("KRAMPUS", x = 80, y = 40, font = "pixel", color = 0xff4444ff);
                text!("NIGHT", x = 96, y = 55, font = "pixel", color = 0x44ff44ff);
                
                // Blinking prompt
                if (self.frame / 30) % 2 == 0 {
                    text!("Press SPACE to Start", x = 56, y = 90, font = "pixel", color = 0xffffffff);
                }
                
                // Controls hint
                text!("WASD/Arrows to Move", x = 64, y = 115, font = "pixel", color = 0x888888ff);
                
                // High score
                if self.high_score > 0 {
                    text!("High Score: {}", self.high_score; x = 72, y = 130, font = "pixel", color = 0xffd700ff);
                }
                
                if keyboard().is_pressed(Key::Space) {
                    self.start_round();
                }
            }
            
            // ================================================================
            // JINGLE MODE (Good Vibes)
            // ================================================================
            GameMode::Jingle => {
                // Cheerful blue-green background
                clear(0x1a5f6aff);
                
                self.move_player();
                self.collect_gifts();
                
                // Check for round completion (all gifts collected OR timer ran out successfully)
                if self.jingle_timer <= 0 {
                    if self.all_gifts_collected() {
                        // BONUS! Completed perfectly
                        self.score += 100 + (self.round * 25);
                        self.round += 1;
                        self.screen_flash = 12;
                        self.flash_color = 0xffd700ff; // Gold flash
                        self.start_round();
                    } else {
                        // Didn't collect all gifts - enter Hurry mode
                        self.mode = GameMode::Hurry;
                        self.hurry_timer = (6 - (self.round as i32 / 3)).max(3);
                        self.screen_flash = 8;
                        self.flash_color = 0xffa500ff; // Orange flash
                    }
                }
                
                // Draw traps (hidden/subtle in Jingle mode)
                for i in 0..self.trap_count {
                    let (tx, ty) = self.traps[i];
                    // Subtle trap indicator
                    circ!(x = tx as i32, y = ty as i32, d = 8, color = 0x44444466);
                }
                
                // Draw gifts
                for i in 0..self.gift_count {
                    if !self.gifts[i].collected {
                        let gx = self.gifts[i].x;
                        let gy = self.gifts[i].y;
                        // Pulsing gift effect
                        let pulse = ((self.frame as f32 / 10.0 + i as f32).sin() * 2.0) as i32;
                        sprite!("gift", x = gx - 6.0, y = gy - 6.0 + pulse as f32);
                    }
                }
            }

            // ================================================================
            // HURRY MODE (Tension)
            // ================================================================
            GameMode::Hurry => {
                // Flickering orange background
                let flicker = if (self.frame / 4) % 2 == 0 { 0x10 } else { 0x00 };
                clear(0xcc5522ff + (flicker << 16));
                
                self.move_player();
                self.collect_gifts();

                // Check trap collision
                if self.stepped_on_trap() {
                    self.mode = GameMode::Krampus;
                    self.krampus_timer = (10 - (self.round as i32 / 2)).max(5);
                    self.krampus_x = if self.player_x > 128.0 { 20.0 } else { 236.0 };
                    self.krampus_y = if self.player_y > 72.0 { 20.0 } else { 124.0 };
                    self.screen_flash = 12;
                    self.flash_color = 0xff0000ff;
                } else if self.hurry_timer <= 0 {
                    // Survived hurry mode without hitting trap!
                    self.score += 50;
                    self.round += 1;
                    self.start_round();
                }
                
                // Draw traps (visible and pulsing in Hurry mode!)
                for i in 0..self.trap_count {
                    let (tx, ty) = self.traps[i];
                    let pulse = ((self.frame as f32 / 5.0).sin() * 3.0).abs() as u32;
                    sprite!("trap", x = tx - 6.0 - pulse as f32 / 2.0, y = ty - 6.0 - pulse as f32 / 2.0);
                }
                
                // Draw remaining gifts
                for i in 0..self.gift_count {
                    if !self.gifts[i].collected {
                        let gx = self.gifts[i].x;
                        let gy = self.gifts[i].y;
                        sprite!("gift", x = gx - 6.0, y = gy - 6.0);
                    }
                }
            }

            // ================================================================
            // KRAMPUS MODE (Horror)
            // ================================================================
            GameMode::Krampus => {
                // Dark, pulsing horror background
                let pulse = ((self.frame as f32 / 20.0).sin() * 10.0) as u32;
                clear(0x0d0d1aff + (pulse << 8));
                
                self.move_player();
                self.update_krampus();

                if self.krampus_timer <= 0 {
                    // SURVIVED!
                    self.score += 150 + (self.round * 50);
                    self.round += 1;
                    self.screen_flash = 15;
                    self.flash_color = 0x00ff00ff; // Green survival flash
                    self.start_round();
                }
                
                // Draw Krampus with menacing effect
                let shake_x = ((self.frame as f32 / 2.0).sin() * 2.0) as f32;
                let shake_y = ((self.frame as f32 / 3.0).cos() * 2.0) as f32;
                sprite!("krampus", x = self.krampus_x - 12.0 + shake_x, y = self.krampus_y - 12.0 + shake_y);
                
                // Warning text
                if (self.frame / 15) % 2 == 0 {
                    text!("RUN!", x = 112, y = 130, font = "pixel", color = 0xff0000ff);
                }
            }

            // ================================================================
            // GAME OVER
            // ================================================================
            GameMode::GameOver => {
                clear(0x000000ff);
                
                // Dramatic pause before showing text
                if self.transition_frame == 0 || self.frame % 60 > 30 {
                    text!("GAME OVER", x = 80, y = 45, font = "pixel", color = 0xff0000ff);
                }
                
                text!("Score: {}", self.score; x = 90, y = 65, font = "pixel", color = 0xffffffff);
                text!("Round: {}", self.round; x = 98, y = 80, font = "pixel", color = 0xaaaaaaff);
                
                // New high score?
                if self.score > self.high_score && self.score > 0 {
                    if (self.frame / 20) % 2 == 0 {
                        text!("NEW HIGH SCORE!", x = 64, y = 100, font = "pixel", color = 0xffd700ff);
                    }
                }
                
                // Restart prompt
                if (self.frame / 30) % 2 == 0 {
                    text!("Press R to Restart", x = 64, y = 120, font = "pixel", color = 0x888888ff);
                }

                if keyboard().is_pressed(Key::R) {
                    self.reset_game();
                }
            }
        }

        // ====================================================================
        // DRAW PLAYER (in all gameplay modes)
        // ====================================================================
        if self.mode == GameMode::Jingle || self.mode == GameMode::Hurry || self.mode == GameMode::Krampus {
            // Player wobble when moving
            let wobble = if keyboard().is_down(Key::Left) || keyboard().is_down(Key::Right) ||
                           keyboard().is_down(Key::Up) || keyboard().is_down(Key::Down) ||
                           keyboard().is_down(Key::A) || keyboard().is_down(Key::D) ||
                           keyboard().is_down(Key::W) || keyboard().is_down(Key::S) {
                ((self.frame as f32 / 4.0).sin() * 1.5) as f32
            } else {
                0.0
            };
            
            sprite!("player", x = self.player_x - 8.0, y = self.player_y - 8.0 + wobble);
        }

        // ====================================================================
        // UI OVERLAY
        // ====================================================================
        if self.mode != GameMode::Title && self.mode != GameMode::GameOver {
            // Score
            text!("Score: {}", self.score; x = 4, y = 136, font = "pixel", color = 0xffffffff);
            
            // Round
            text!("R{}", self.round; x = 4, y = 4, font = "pixel", color = 0xffd700ff);
            
            // Timer bar
            self.draw_timer_bar();
            
            // Mode indicator
            self.draw_mode_indicator();
            
            // Gifts remaining (in Jingle/Hurry)
            if self.mode == GameMode::Jingle || self.mode == GameMode::Hurry {
                let remaining = self.gift_count - self.gifts_collected_this_round as usize;
                text!("Gifts: {}", remaining; x = 180, y = 136, font = "pixel", color = 0x44ff44ff);
            }
        }
        
        // ====================================================================
        // SCREEN FLASH EFFECT
        // ====================================================================
        if self.screen_flash > 0 {
            let alpha = ((self.screen_flash as f32 / 15.0) * 180.0) as u32;
            let flash = (self.flash_color & 0xffffff00) | alpha;
            rect!(x = 0, y = 0, w = 256, h = 144, color = flash);
        }
    }
}
