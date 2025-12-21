use turbo::*;

// ============================================================================
// GIFT STRUCTURE
// ============================================================================

#[turbo::serialize]
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
    mode: u8,  // 0=Title, 1=Jingle, 2=Hurry, 3=Krampus, 4=GameOver
    
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
    
    // Traps (fixed size array)
    trap_x: [f32; 5],
    trap_y: [f32; 5],
    trap_count: u32,
    
    // Gifts
    gifts: Vec<Gift>,
    
    // Scoring & Progression
    score: u32,
    high_score: u32,
    round: u32,
    gifts_collected_this_round: u32,
    
    // Visual effects
    screen_flash: u32,
    flash_color: u32,
    
    // Random seed (simple LCG)
    rng_seed: u32,
}

// Game mode constants
const MODE_TITLE: u8 = 0;
const MODE_JINGLE: u8 = 1;
const MODE_HURRY: u8 = 2;
const MODE_KRAMPUS: u8 = 3;
const MODE_GAMEOVER: u8 = 4;

impl GameState {
    pub fn new() -> Self {
        let mut state = Self {
            frame: 0,
            mode: MODE_TITLE,
            
            jingle_timer: 15,
            hurry_timer: 6,
            krampus_timer: 10,
            
            player_x: 128.0,
            player_y: 72.0,
            speed: 2.0,
            
            krampus_x: 10.0,
            krampus_y: 10.0,
            krampus_speed: 1.2,
            
            trap_x: [0.0; 5],
            trap_y: [0.0; 5],
            trap_count: 3,
            
            gifts: vec![],
            
            score: 0,
            high_score: 0,
            round: 1,
            gifts_collected_this_round: 0,
            
            screen_flash: 0,
            flash_color: 0xffffffff,
            
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
        self.trap_count = (2 + self.round).min(5);
        let gift_count = (3 + (self.round / 2)).min(6);
        
        // Spawn traps at random positions (avoiding center spawn area)
        for i in 0..self.trap_count as usize {
            loop {
                let x = self.random_range(20.0, 236.0);
                let y = self.random_range(20.0, 124.0);
                
                // Avoid spawning near player start position
                let dist_to_center = ((x - 128.0).powi(2) + (y - 72.0).powi(2)).sqrt();
                if dist_to_center > 40.0 {
                    self.trap_x[i] = x;
                    self.trap_y[i] = y;
                    break;
                }
            }
        }
        
        // Clear and spawn gifts
        self.gifts.clear();
        for _ in 0..gift_count {
            loop {
                let x = self.random_range(24.0, 232.0);
                let y = self.random_range(24.0, 120.0);
                
                // Check distance from traps
                let mut too_close = false;
                for j in 0..self.trap_count as usize {
                    let dist = ((x - self.trap_x[j]).powi(2) + (y - self.trap_y[j]).powi(2)).sqrt();
                    if dist < 24.0 {
                        too_close = true;
                        break;
                    }
                }
                
                if !too_close {
                    self.gifts.push(Gift { x, y, collected: false });
                    break;
                }
            }
        }
    }
    
    fn start_round(&mut self) {
        self.mode = MODE_JINGLE;
        self.jingle_timer = (15 - (self.round as i32 - 1).min(5)).max(8);
        self.hurry_timer = (6 - (self.round as i32 / 3)).max(3);
        self.krampus_timer = (10 - (self.round as i32 / 2)).max(5);
        
        // Increase Krampus speed each round
        self.krampus_speed = 1.2 + (self.round as f32 - 1.0) * 0.15;
        
        // Reset player to center
        self.player_x = 128.0;
        self.player_y = 72.0;
        
        self.gifts_collected_this_round = 0;
        
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
                MODE_JINGLE => self.jingle_timer -= 1,
                MODE_HURRY => self.hurry_timer -= 1,
                MODE_KRAMPUS => self.krampus_timer -= 1,
                _ => {}
            }
        }
    }

    fn move_player(&mut self) {
        let mut dx = 0.0;
        let mut dy = 0.0;

        // Gamepad controls (works with keyboard arrows too)
        let gp = gamepad::get(0);
        if gp.left.pressed() { dx -= self.speed; }
        if gp.right.pressed() { dx += self.speed; }
        if gp.up.pressed() { dy -= self.speed; }
        if gp.down.pressed() { dy += self.speed; }
        
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
        for gift in &mut self.gifts {
            if !gift.collected {
                let dist = ((self.player_x - gift.x).powi(2) + (self.player_y - gift.y).powi(2)).sqrt();
                if dist < 12.0 {
                    gift.collected = true;
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
        self.gifts.iter().all(|g| g.collected)
    }

    fn stepped_on_trap(&self) -> bool {
        for i in 0..self.trap_count as usize {
            let dist = ((self.player_x - self.trap_x[i]).powi(2) + (self.player_y - self.trap_y[i]).powi(2)).sqrt();
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
            self.mode = MODE_GAMEOVER;
            self.screen_flash = 15;
            self.flash_color = 0xff0000ff;
        }
    }
    
    fn draw_timer_bar(&self) {
        let (current, max, color) = match self.mode {
            MODE_JINGLE => (self.jingle_timer, 15, 0x4ade80ff),   // Green
            MODE_HURRY => (self.hurry_timer, 6, 0xfbbf24ff),      // Yellow
            MODE_KRAMPUS => (self.krampus_timer, 10, 0xef4444ff), // Red
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
            MODE_JINGLE => ("JINGLE", 0x4ade80ff),
            MODE_HURRY => ("HURRY!", 0xfbbf24ff),
            MODE_KRAMPUS => ("KRAMPUS", 0xef4444ff),
            _ => ("", 0x000000ff),
        };
        
        if !mode_text.is_empty() {
            text!(mode_text, x = 200, y = 4, font = "medium", color = color);
        }
    }

    pub fn update(&mut self) {
        self.frame += 1;
        
        // Decrease screen flash
        if self.screen_flash > 0 {
            self.screen_flash -= 1;
        }
        
        self.tick_timer();

        match self.mode {
            // ================================================================
            // TITLE SCREEN
            // ================================================================
            MODE_TITLE => {
                // Animated background
                let bg_pulse = ((self.frame as f32 / 30.0).sin() * 20.0) as u32;
                clear(0x1a3a2aff + (bg_pulse << 8));
                
                // Title
                text!("KRAMPUS", x = 80, y = 35, font = "large", color = 0xff4444ff);
                text!("NIGHT", x = 96, y = 55, font = "large", color = 0x44ff44ff);
                
                // Blinking prompt
                if (self.frame / 30) % 2 == 0 {
                    text!("Press START to Play", x = 60, y = 90, font = "medium", color = 0xffffffff);
                }
                
                // Controls hint
                text!("D-Pad to Move", x = 80, y = 110, font = "small", color = 0x888888ff);
                
                // High score
                if self.high_score > 0 {
                    text!("Best: {}", self.high_score; x = 100, y = 128, font = "small", color = 0xffd700ff);
                }
                
                let gp = gamepad::get(0);
                if gp.start.just_pressed() || gp.a.just_pressed() {
                    self.start_round();
                }
            }
            
            // ================================================================
            // JINGLE MODE (Good Vibes)
            // ================================================================
            MODE_JINGLE => {
                // Cheerful blue-green background
                clear(0x1a5f6aff);
                
                self.move_player();
                self.collect_gifts();
                
                // Check for round completion
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
                        self.mode = MODE_HURRY;
                        self.hurry_timer = (6 - (self.round as i32 / 3)).max(3);
                        self.screen_flash = 8;
                        self.flash_color = 0xffa500ff; // Orange flash
                    }
                }
                
                // Draw traps (hidden/subtle in Jingle mode)
                for i in 0..self.trap_count as usize {
                    circ!(x = self.trap_x[i] as i32, y = self.trap_y[i] as i32, d = 8, color = 0x44444466);
                }
                
                // Draw gifts
                for (idx, gift) in self.gifts.iter().enumerate() {
                    if !gift.collected {
                        // Pulsing gift effect
                        let pulse = ((self.frame as f32 / 10.0 + idx as f32).sin() * 2.0) as f32;
                        sprite!("gift", x = gift.x - 6.0, y = gift.y - 6.0 + pulse);
                    }
                }
            }

            // ================================================================
            // HURRY MODE (Tension)
            // ================================================================
            MODE_HURRY => {
                // Flickering orange background
                let flicker = if (self.frame / 4) % 2 == 0 { 0x10 } else { 0x00 };
                clear(0xcc5522ff + (flicker << 16));
                
                self.move_player();
                self.collect_gifts();

                // Check trap collision
                if self.stepped_on_trap() {
                    self.mode = MODE_KRAMPUS;
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
                for i in 0..self.trap_count as usize {
                    let pulse = ((self.frame as f32 / 5.0).sin() * 3.0).abs() as f32;
                    sprite!("trap", x = self.trap_x[i] - 6.0 - pulse / 2.0, y = self.trap_y[i] - 6.0 - pulse / 2.0);
                }
                
                // Draw remaining gifts
                for gift in &self.gifts {
                    if !gift.collected {
                        sprite!("gift", x = gift.x - 6.0, y = gift.y - 6.0);
                    }
                }
            }

            // ================================================================
            // KRAMPUS MODE (Horror)
            // ================================================================
            MODE_KRAMPUS => {
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
                    text!("RUN!", x = 112, y = 130, font = "medium", color = 0xff0000ff);
                }
            }

            // ================================================================
            // GAME OVER
            // ================================================================
            MODE_GAMEOVER => {
                clear(0x000000ff);
                
                text!("GAME OVER", x = 72, y = 40, font = "large", color = 0xff0000ff);
                
                text!("Score: {}", self.score; x = 90, y = 65, font = "medium", color = 0xffffffff);
                text!("Round: {}", self.round; x = 98, y = 80, font = "small", color = 0xaaaaaaff);
                
                // New high score?
                if self.score > self.high_score && self.score > 0 {
                    if (self.frame / 20) % 2 == 0 {
                        text!("NEW HIGH SCORE!", x = 56, y = 100, font = "medium", color = 0xffd700ff);
                    }
                }
                
                // Restart prompt
                if (self.frame / 30) % 2 == 0 {
                    text!("Press START to Retry", x = 56, y = 120, font = "small", color = 0x888888ff);
                }

                let gp = gamepad::get(0);
                if gp.start.just_pressed() || gp.a.just_pressed() {
                    self.reset_game();
                }
            }
            
            _ => {}
        }

        // ====================================================================
        // DRAW PLAYER (in all gameplay modes)
        // ====================================================================
        if self.mode == MODE_JINGLE || self.mode == MODE_HURRY || self.mode == MODE_KRAMPUS {
            // Player wobble when moving
            let gp = gamepad::get(0);
            let wobble = if gp.left.pressed() || gp.right.pressed() || gp.up.pressed() || gp.down.pressed() {
                ((self.frame as f32 / 4.0).sin() * 1.5) as f32
            } else {
                0.0
            };
            
            sprite!("player", x = self.player_x - 8.0, y = self.player_y - 8.0 + wobble);
        }

        // ====================================================================
        // UI OVERLAY
        // ====================================================================
        if self.mode != MODE_TITLE && self.mode != MODE_GAMEOVER {
            // Score
            text!("Score: {}", self.score; x = 4, y = 136, font = "small", color = 0xffffffff);
            
            // Round
            text!("R{}", self.round; x = 4, y = 4, font = "small", color = 0xffd700ff);
            
            // Timer bar
            self.draw_timer_bar();
            
            // Mode indicator
            self.draw_mode_indicator();
            
            // Gifts remaining (in Jingle/Hurry)
            if self.mode == MODE_JINGLE || self.mode == MODE_HURRY {
                let remaining = self.gifts.iter().filter(|g| !g.collected).count();
                text!("Gifts: {}", remaining; x = 180, y = 136, font = "small", color = 0x44ff44ff);
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
