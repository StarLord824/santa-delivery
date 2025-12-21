use turbo::*;

// ============================================================================
// SNOWFLAKE PARTICLE
// ============================================================================

#[turbo::serialize]
struct Snowflake {
    x: f32,
    y: f32,
    speed: f32,
    size: u32,
    drift: f32,
}

// ============================================================================
// SCORE POPUP (for juice!)
// ============================================================================

#[turbo::serialize]
struct ScorePopup {
    x: f32,
    y: f32,
    value: u32,
    life: u32,
}

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
    screen_shake: u32,
    
    // Particles
    snowflakes: Vec<Snowflake>,
    score_popups: Vec<ScorePopup>,
    
    // Random seed (simple LCG)
    rng_seed: u32,
}

// Game mode constants
const MODE_TITLE: u8 = 0;
const MODE_JINGLE: u8 = 1;
const MODE_HURRY: u8 = 2;
const MODE_KRAMPUS: u8 = 3;
const MODE_GAMEOVER: u8 = 4;

// Color palettes for each mode
const COLOR_JINGLE_BG: u32 = 0x1a4a5aff;      // Deep teal
const COLOR_HURRY_BG: u32 = 0x8b4513ff;       // Saddle brown
const COLOR_KRAMPUS_BG: u32 = 0x0a0a12ff;     // Near black
const COLOR_SNOW: u32 = 0xffffffcc;           // Semi-transparent white
const COLOR_GIFT_BOX: u32 = 0xdc143cff;       // Crimson red
const COLOR_GIFT_BOW: u32 = 0xffd700ff;       // Gold
const COLOR_PLAYER_BODY: u32 = 0x228b22ff;    // Forest green
const COLOR_PLAYER_SKIN: u32 = 0xffdbacff;    // Skin tone
const COLOR_PLAYER_HAT: u32 = 0xff0000ff;     // Red

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
            screen_shake: 0,
            
            snowflakes: vec![],
            score_popups: vec![],
            
            rng_seed: 12345,
        };
        state.init_snowflakes();
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
    
    // Initialize snowflakes
    fn init_snowflakes(&mut self) {
        self.snowflakes.clear();
        for _ in 0..30 {
            // Pre-compute random values to avoid borrow issues
            let x = self.random_range(0.0, 256.0);
            let y = self.random_range(0.0, 144.0);
            let speed = self.random_range(0.3, 1.2);
            let size = (self.random() % 3 + 1) as u32;
            let drift = self.random_range(-0.3, 0.3);
            
            self.snowflakes.push(Snowflake { x, y, speed, size, drift });
        }
    }
    
    // Update snowflakes
    fn update_snowflakes(&mut self) {
        let frame = self.frame;
        let seed = self.rng_seed;
        
        for (i, snow) in self.snowflakes.iter_mut().enumerate() {
            snow.y += snow.speed;
            snow.x += snow.drift + (frame as f32 / 20.0).sin() * 0.2;
            
            // Wrap around
            if snow.y > 150.0 {
                snow.y = -5.0;
                // Use deterministic pseudo-random based on frame and index
                snow.x = ((seed.wrapping_add(frame).wrapping_add(i as u32 * 7919)) % 256) as f32;
            }
            if snow.x < -5.0 { snow.x = 260.0; }
            if snow.x > 260.0 { snow.x = -5.0; }
        }
    }
    
    // Draw snowflakes
    fn draw_snowflakes(&self) {
        for snow in &self.snowflakes {
            circ!(x = snow.x as i32, y = snow.y as i32, d = snow.size, color = COLOR_SNOW);
        }
    }
    
    // Add score popup
    fn add_score_popup(&mut self, x: f32, y: f32, value: u32) {
        self.score_popups.push(ScorePopup {
            x,
            y,
            value,
            life: 45, // 0.75 seconds at 60fps
        });
    }
    
    // Update and draw score popups
    fn update_score_popups(&mut self) {
        self.score_popups.retain_mut(|popup| {
            popup.y -= 0.8; // Float upward
            popup.life -= 1;
            popup.life > 0
        });
    }
    
    fn draw_score_popups(&self) {
        for popup in &self.score_popups {
            let alpha = ((popup.life as f32 / 45.0) * 255.0) as u32;
            let color = 0xffd70000 | alpha;
            text!("+{}", popup.value; x = popup.x as i32, y = popup.y as i32, font = "small", color = color);
        }
    }
    
    // Get screen shake offset
    fn get_shake_offset(&self) -> (i32, i32) {
        if self.screen_shake > 0 {
            let intensity = (self.screen_shake as f32 / 2.0).min(4.0);
            let shake_x = ((self.frame as f32 * 1.7).sin() * intensity) as i32;
            let shake_y = ((self.frame as f32 * 2.3).cos() * intensity) as i32;
            (shake_x, shake_y)
        } else {
            (0, 0)
        }
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
        self.screen_shake = 6;
    }
    
    fn reset_game(&mut self) {
        if self.score > self.high_score {
            self.high_score = self.score;
        }
        self.score = 0;
        self.round = 1;
        self.krampus_speed = 1.2;
        self.score_popups.clear();
        self.start_round();
    }
    
    // ========================================================================
    // AUDIO SYSTEM
    // ========================================================================
    
    /// Play background music based on current mode
    fn play_mode_music(&self) {
        // Stop all music tracks first
        audio::stop("jingle_bgm");
        audio::stop("hurry_bgm");
        audio::stop("krampus_bgm");
        audio::stop("title_bgm");
        
        // Play the appropriate track
        match self.mode {
            MODE_TITLE => {
                if !audio::is_playing("title_bgm") {
                    audio::play("title_bgm");
                }
            }
            MODE_JINGLE => {
                if !audio::is_playing("jingle_bgm") {
                    audio::play("jingle_bgm");
                }
            }
            MODE_HURRY => {
                if !audio::is_playing("hurry_bgm") {
                    audio::play("hurry_bgm");
                }
            }
            MODE_KRAMPUS => {
                if !audio::is_playing("krampus_bgm") {
                    audio::play("krampus_bgm");
                }
            }
            MODE_GAMEOVER => {
                // Silence or play game over music
                audio::stop("jingle_bgm");
                audio::stop("hurry_bgm");
                audio::stop("krampus_bgm");
            }
            _ => {}
        }
    }
    
    /// Keep background music looping
    fn update_music(&self) {
        match self.mode {
            MODE_TITLE => {
                if !audio::is_playing("title_bgm") {
                    audio::play("title_bgm");
                }
            }
            MODE_JINGLE => {
                if !audio::is_playing("jingle_bgm") {
                    audio::play("jingle_bgm");
                }
            }
            MODE_HURRY => {
                if !audio::is_playing("hurry_bgm") {
                    audio::play("hurry_bgm");
                }
            }
            MODE_KRAMPUS => {
                if !audio::is_playing("krampus_bgm") {
                    audio::play("krampus_bgm");
                }
            }
            _ => {}
        }
    }
    
    /// Play sound effects
    fn play_sfx(name: &str) {
        audio::play(name);
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
        // First pass: find collected gifts and gather popup data
        let mut popups_to_add: Vec<(f32, f32, u32)> = vec![];
        let player_x = self.player_x;
        let player_y = self.player_y;
        let round = self.round;
        
        for gift in &mut self.gifts {
            if !gift.collected {
                let dist = ((player_x - gift.x).powi(2) + (player_y - gift.y).powi(2)).sqrt();
                if dist < 14.0 {
                    gift.collected = true;
                    let points = 25 + (round * 5);
                    self.score += points;
                    self.gifts_collected_this_round += 1;
                    
                    // Queue popup for later
                    popups_to_add.push((gift.x, gift.y - 10.0, points));
                    
                    // Play pickup sound
                    Self::play_sfx("pickup");
                    
                    // Small flash effect
                    self.screen_flash = 4;
                    self.flash_color = 0x00ff00ff; // Green flash
                }
            }
        }
        
        // Second pass: add popups
        for (x, y, points) in popups_to_add {
            self.add_score_popup(x, y, points);
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
        if dist < 14.0 {
            self.mode = MODE_GAMEOVER;
            self.screen_flash = 20;
            self.flash_color = 0xff0000ff;
            self.screen_shake = 15;
            Self::play_sfx("gameover");
            self.play_mode_music();
        }
    }
    
    fn draw_timer_bar(&self, shake_x: i32, shake_y: i32) {
        let (current, max, color) = match self.mode {
            MODE_JINGLE => (self.jingle_timer, 15, 0x4ade80ff),   // Green
            MODE_HURRY => (self.hurry_timer, 6, 0xfbbf24ff),      // Yellow
            MODE_KRAMPUS => (self.krampus_timer, 10, 0xef4444ff), // Red
            _ => (0, 1, 0x666666ff),
        };
        
        let bar_width = ((current as f32 / max as f32) * 200.0) as u32;
        
        // Background
        rect!(x = 28 + shake_x, y = 2 + shake_y, w = 200, h = 6, color = 0x333333ff);
        // Timer bar
        rect!(x = 28 + shake_x, y = 2 + shake_y, w = bar_width, h = 6, color = color);
    }
    
    fn draw_mode_indicator(&self, shake_x: i32, shake_y: i32) {
        let (mode_text, color) = match self.mode {
            MODE_JINGLE => ("JINGLE", 0x4ade80ff),
            MODE_HURRY => ("HURRY!", 0xfbbf24ff),
            MODE_KRAMPUS => ("KRAMPUS", 0xef4444ff),
            _ => ("", 0x000000ff),
        };
        
        if !mode_text.is_empty() {
            text!(mode_text, x = 200 + shake_x, y = 4 + shake_y, font = "medium", color = color);
        }
    }
    
    // Draw Christmas tree decorations in background
    fn draw_christmas_decorations(&self, shake_x: i32, shake_y: i32) {
        // Draw some decorative elements based on mode
        if self.mode == MODE_JINGLE || self.mode == MODE_HURRY {
            // Corner decorations (holly leaves)
            let pulse = ((self.frame as f32 / 30.0).sin() * 2.0) as i32;
            
            // Top-left holly
            circ!(x = 10 + shake_x, y = 20 + shake_y + pulse, d = 8, color = 0x228b22ff);
            circ!(x = 16 + shake_x, y = 16 + shake_y + pulse, d = 8, color = 0x228b22ff);
            circ!(x = 13 + shake_x, y = 18 + shake_y + pulse, d = 4, color = 0xff0000ff);
            
            // Bottom-right holly  
            circ!(x = 246 + shake_x, y = 124 + shake_y - pulse, d = 8, color = 0x228b22ff);
            circ!(x = 240 + shake_x, y = 128 + shake_y - pulse, d = 8, color = 0x228b22ff);
            circ!(x = 243 + shake_x, y = 126 + shake_y - pulse, d = 4, color = 0xff0000ff);
        }
    }
    
    // Draw player with enhanced visuals
    fn draw_player(&self, shake_x: i32, shake_y: i32) {
        let gp = gamepad::get(0);
        let wobble = if gp.left.pressed() || gp.right.pressed() || gp.up.pressed() || gp.down.pressed() {
            ((self.frame as f32 / 4.0).sin() * 1.5) as f32
        } else {
            0.0
        };
        
        let px = (self.player_x - 8.0) as i32 + shake_x;
        let py = (self.player_y - 8.0 + wobble) as i32 + shake_y;
        
        // Shadow
        ellipse!(x = px + 8, y = py + 18, w = 14, h = 6, color = 0x00000044);
        
        // Body (green tunic)
        rect!(x = px + 2, y = py + 6, w = 12, h = 10, color = COLOR_PLAYER_BODY);
        
        // Belt
        rect!(x = px + 2, y = py + 10, w = 12, h = 2, color = 0x8b4513ff);
        circ!(x = px + 8, y = py + 11, d = 3, color = COLOR_GIFT_BOW);
        
        // Face
        circ!(x = px + 8, y = py + 5, d = 10, color = COLOR_PLAYER_SKIN);
        
        // Eyes
        circ!(x = px + 6, y = py + 4, d = 2, color = 0x000000ff);
        circ!(x = px + 10, y = py + 4, d = 2, color = 0x000000ff);
        
        // Red pointy hat
        rect!(x = px + 3, y = py - 2, w = 10, h = 6, color = COLOR_PLAYER_HAT);
        rect!(x = px + 5, y = py - 5, w = 6, h = 4, color = COLOR_PLAYER_HAT);
        rect!(x = px + 7, y = py - 7, w = 2, h = 3, color = COLOR_PLAYER_HAT);
        
        // White pom-pom on hat
        circ!(x = px + 8, y = py - 7, d = 4, color = 0xffffffff);
    }
    
    // Draw gift with enhanced visuals
    fn draw_gift(&self, x: f32, y: f32, pulse: f32, shake_x: i32, shake_y: i32) {
        let gx = (x - 6.0) as i32 + shake_x;
        let gy = (y - 6.0 + pulse) as i32 + shake_y;
        
        // Shadow
        ellipse!(x = gx + 6, y = gy + 14, w = 12, h = 4, color = 0x00000044);
        
        // Gift box body
        rect!(x = gx, y = gy, w = 12, h = 12, color = COLOR_GIFT_BOX);
        
        // Gift box highlight
        rect!(x = gx, y = gy, w = 3, h = 12, color = 0xff3333ff);
        
        // Ribbon vertical
        rect!(x = gx + 5, y = gy, w = 2, h = 12, color = COLOR_GIFT_BOW);
        
        // Ribbon horizontal
        rect!(x = gx, y = gy + 5, w = 12, h = 2, color = COLOR_GIFT_BOW);
        
        // Bow
        circ!(x = gx + 6, y = gy - 1, d = 5, color = COLOR_GIFT_BOW);
        circ!(x = gx + 2, y = gy - 2, d = 4, color = COLOR_GIFT_BOW);
        circ!(x = gx + 10, y = gy - 2, d = 4, color = COLOR_GIFT_BOW);
    }
    
    // Draw trap with enhanced visuals
    fn draw_trap(&self, x: f32, y: f32, visible: bool, shake_x: i32, shake_y: i32) {
        let tx = x as i32 + shake_x;
        let ty = y as i32 + shake_y;
        
        if visible {
            // Visible trap - looks like dark ice/coal
            let pulse = ((self.frame as f32 / 5.0).sin() * 3.0).abs() as u32;
            
            // Outer glow
            circ!(x = tx, y = ty, d = 14 + pulse, color = 0x44000088);
            
            // Dark center
            circ!(x = tx, y = ty, d = 12, color = 0x220000ff);
            circ!(x = tx, y = ty, d = 8, color = 0x440000ff);
            
            // Warning sparkle
            if (self.frame / 8) % 2 == 0 {
                circ!(x = tx - 3, y = ty - 3, d = 2, color = 0xff6600ff);
            }
        } else {
            // Hidden trap - subtle hint
            circ!(x = tx, y = ty, d = 6, color = 0x33333344);
        }
    }
    
    // Draw Krampus with enhanced visuals
    fn draw_krampus(&self, shake_x: i32, shake_y: i32) {
        let shake_x_extra = ((self.frame as f32 / 2.0).sin() * 2.0) as i32;
        let shake_y_extra = ((self.frame as f32 / 3.0).cos() * 2.0) as i32;
        
        let kx = (self.krampus_x - 12.0) as i32 + shake_x + shake_x_extra;
        let ky = (self.krampus_y - 14.0) as i32 + shake_y + shake_y_extra;
        
        // Shadow
        ellipse!(x = kx + 12, y = ky + 28, w = 20, h = 6, color = 0x00000066);
        
        // Body (dark fur)
        rect!(x = kx + 4, y = ky + 10, w = 16, h = 16, color = 0x2a1a1aff);
        
        // Head
        circ!(x = kx + 12, y = ky + 10, d = 14, color = 0x3a2020ff);
        
        // Horns
        rect!(x = kx + 2, y = ky - 2, w = 4, h = 12, color = 0x4a3030ff);
        rect!(x = kx + 18, y = ky - 2, w = 4, h = 12, color = 0x4a3030ff);
        rect!(x = kx, y = ky - 4, w = 4, h = 4, color = 0x4a3030ff);
        rect!(x = kx + 20, y = ky - 4, w = 4, h = 4, color = 0x4a3030ff);
        
        // Glowing red eyes
        let eye_pulse = ((self.frame as f32 / 4.0).sin() * 30.0) as u32;
        let eye_color = 0xff0000ff - (eye_pulse << 8);
        circ!(x = kx + 8, y = ky + 8, d = 4, color = eye_color);
        circ!(x = kx + 16, y = ky + 8, d = 4, color = eye_color);
        
        // Eye glow
        circ!(x = kx + 8, y = ky + 8, d = 6, color = 0xff000044);
        circ!(x = kx + 16, y = ky + 8, d = 6, color = 0xff000044);
        
        // Mouth (menacing grin)
        rect!(x = kx + 7, y = ky + 14, w = 10, h = 2, color = 0x000000ff);
        
        // Chains (dragging behind)
        let chain_offset = (self.frame % 10) as i32;
        for i in 0..3 {
            let cx = kx - 5 - i * 4 - chain_offset / 2;
            let cy = ky + 20 + (i as f32 * 1.5) as i32;
            circ!(x = cx, y = cy, d = 3, color = 0x666666ff);
        }
    }

    pub fn update(&mut self) {
        self.frame += 1;
        
        // Decrease screen flash
        if self.screen_flash > 0 {
            self.screen_flash -= 1;
        }
        
        // Decrease screen shake
        if self.screen_shake > 0 {
            self.screen_shake -= 1;
        }
        
        // Update particles
        self.update_snowflakes();
        self.update_score_popups();
        
        // Update background music (keeps it looping)
        self.update_music();
        
        self.tick_timer();
        
        // Get shake offset for all rendering
        let (shake_x, shake_y) = self.get_shake_offset();

        match self.mode {
            // ================================================================
            // TITLE SCREEN
            // ================================================================
            MODE_TITLE => {
                // Animated dark green background
                let bg_pulse = ((self.frame as f32 / 30.0).sin() * 10.0) as u32;
                clear(0x0f2f1fff + (bg_pulse << 8));
                
                // Draw snow on title screen too
                self.draw_snowflakes();
                
                // Christmas decorations
                self.draw_christmas_decorations(0, 0);
                
                // Title with shadow
                text!("KRAMPUS", x = 82, y = 37, font = "large", color = 0x440000ff); // Shadow
                text!("KRAMPUS", x = 80, y = 35, font = "large", color = 0xff4444ff);
                
                text!("NIGHT", x = 98, y = 57, font = "large", color = 0x004400ff); // Shadow
                text!("NIGHT", x = 96, y = 55, font = "large", color = 0x44ff44ff);
                
                // Blinking prompt
                if (self.frame / 30) % 2 == 0 {
                    text!("Press START to Play", x = 60, y = 90, font = "medium", color = 0xffffffff);
                }
                
                // Controls hint
                text!("D-Pad to Move", x = 80, y = 110, font = "small", color = 0x888888ff);
                
                // Flash "Collect Gifts, Avoid Traps!" text
                if (self.frame / 60) % 2 == 0 {
                    text!("Collect Gifts - Avoid Krampus!", x = 32, y = 125, font = "small", color = 0xffd700ff);
                }
                
                // High score
                if self.high_score > 0 {
                    text!("Best: {}", self.high_score; x = 100, y = 138, font = "small", color = 0xffd700ff);
                }
                
                let gp = gamepad::get(0);
                if gp.start.just_pressed() || gp.a.just_pressed() {
                    Self::play_sfx("start");
                    self.start_round();
                    self.play_mode_music();
                }
            }
            
            // ================================================================
            // JINGLE MODE (Good Vibes)
            // ================================================================
            MODE_JINGLE => {
                // Cheerful background with gradient effect
                clear(COLOR_JINGLE_BG);
                
                // Draw decorations first (background layer)
                self.draw_christmas_decorations(shake_x, shake_y);
                
                // Draw snow
                self.draw_snowflakes();
                
                self.move_player();
                self.collect_gifts();
                
                // Check for round completion
                if self.jingle_timer <= 0 {
                    if self.all_gifts_collected() {
                        // BONUS! Completed perfectly
                        let bonus = 100 + (self.round * 25);
                        self.score += bonus;
                        self.add_score_popup(128.0, 60.0, bonus);
                        self.round += 1;
                        self.screen_flash = 12;
                        self.flash_color = 0xffd700ff; // Gold flash
                        self.screen_shake = 8;
                        Self::play_sfx("bonus");
                        self.start_round();
                    } else {
                        // Didn't collect all gifts - enter Hurry mode
                        self.mode = MODE_HURRY;
                        self.hurry_timer = (6 - (self.round as i32 / 3)).max(3);
                        self.screen_flash = 10;
                        self.flash_color = 0xffa500ff; // Orange flash
                        self.screen_shake = 6;
                        Self::play_sfx("hurry");
                        self.play_mode_music();
                    }
                }
                
                // Draw traps (hidden/subtle in Jingle mode)
                for i in 0..self.trap_count as usize {
                    self.draw_trap(self.trap_x[i], self.trap_y[i], false, shake_x, shake_y);
                }
                
                // Draw gifts
                for (idx, gift) in self.gifts.iter().enumerate() {
                    if !gift.collected {
                        let pulse = ((self.frame as f32 / 10.0 + idx as f32).sin() * 2.0) as f32;
                        self.draw_gift(gift.x, gift.y, pulse, shake_x, shake_y);
                    }
                }
                
                // Draw player
                self.draw_player(shake_x, shake_y);
            }

            // ================================================================
            // HURRY MODE (Tension)
            // ================================================================
            MODE_HURRY => {
                // Flickering orange/brown background
                let flicker = if (self.frame / 3) % 2 == 0 { 0x10 } else { 0x00 };
                clear(COLOR_HURRY_BG + (flicker << 16));
                
                // Still draw snow but faster/more chaotic
                self.draw_snowflakes();
                
                self.move_player();
                self.collect_gifts();

                // Check trap collision
                if self.stepped_on_trap() {
                    self.mode = MODE_KRAMPUS;
                    self.krampus_timer = (10 - (self.round as i32 / 2)).max(5);
                    self.krampus_x = if self.player_x > 128.0 { 20.0 } else { 236.0 };
                    self.krampus_y = if self.player_y > 72.0 { 20.0 } else { 124.0 };
                    self.screen_flash = 15;
                    self.flash_color = 0xff0000ff;
                    self.screen_shake = 12;
                    Self::play_sfx("trap");
                    self.play_mode_music();
                } else if self.hurry_timer <= 0 {
                    // Survived hurry mode without hitting trap!
                    let bonus = 50 + (self.round * 10);
                    self.score += bonus;
                    self.add_score_popup(128.0, 60.0, bonus);
                    self.round += 1;
                    Self::play_sfx("bonus");
                    self.start_round();
                    self.play_mode_music();
                }
                
                // Draw traps (visible and pulsing in Hurry mode!)
                for i in 0..self.trap_count as usize {
                    self.draw_trap(self.trap_x[i], self.trap_y[i], true, shake_x, shake_y);
                }
                
                // Draw remaining gifts
                for gift in &self.gifts {
                    if !gift.collected {
                        self.draw_gift(gift.x, gift.y, 0.0, shake_x, shake_y);
                    }
                }
                
                // Draw player
                self.draw_player(shake_x, shake_y);
                
                // Urgent warning text
                if (self.frame / 10) % 2 == 0 {
                    text!("WATCH OUT!", x = 88, y = 130, font = "small", color = 0xff6600ff);
                }
            }

            // ================================================================
            // KRAMPUS MODE (Horror)
            // ================================================================
            MODE_KRAMPUS => {
                // Dark, pulsing horror background
                let pulse = ((self.frame as f32 / 15.0).sin() * 8.0) as u32;
                clear(COLOR_KRAMPUS_BG + (pulse << 8));
                
                // Fog effect (dark circles at edges)
                for i in 0..8 {
                    let offset = (self.frame as f32 / 20.0 + i as f32).sin() * 10.0;
                    circ!(x = (i * 35 - 10 + offset as i32) + shake_x, y = -5 + shake_y, d = 30, color = 0x00000088);
                    circ!(x = (i * 35 - 10 - offset as i32) + shake_x, y = 149 + shake_y, d = 30, color = 0x00000088);
                }
                
                self.move_player();
                self.update_krampus();

                if self.krampus_timer <= 0 {
                    // SURVIVED!
                    let bonus = 150 + (self.round * 50);
                    self.score += bonus;
                    self.add_score_popup(128.0, 60.0, bonus);
                    self.round += 1;
                    self.screen_flash = 15;
                    self.flash_color = 0x00ff00ff; // Green survival flash
                    self.screen_shake = 10;
                    Self::play_sfx("survive");
                    self.start_round();
                    self.play_mode_music();
                }
                
                // Draw player first (Krampus renders on top for fear effect)
                self.draw_player(shake_x, shake_y);
                
                // Draw Krampus
                self.draw_krampus(shake_x, shake_y);
                
                // Warning text (more urgent)
                if (self.frame / 8) % 2 == 0 {
                    text!("RUN!", x = 112, y = 130, font = "large", color = 0xff0000ff);
                }
                
                // Timer urgency
                if self.krampus_timer <= 3 && (self.frame / 15) % 2 == 0 {
                    text!("ALMOST THERE!", x = 70, y = 60, font = "medium", color = 0x00ff00ff);
                }
            }

            // ================================================================
            // GAME OVER
            // ================================================================
            MODE_GAMEOVER => {
                clear(0x0a0a0aff);
                
                // Dramatic red vignette
                for i in 0..5 {
                    let size = 300 - i * 30;
                    let alpha = 0x11 * i as u32;
                    rect!(x = 128 - size as i32 / 2, y = 72 - size as i32 / 2, w = size as u32, h = size as u32, color = (0x22000000 + alpha));
                }
                
                // Title with glitch effect
                let glitch = if (self.frame / 30) % 5 == 0 { 2 } else { 0 };
                text!("GAME OVER", x = 72 + glitch, y = 40, font = "large", color = 0xff0000ff);
                
                text!("Score: {}", self.score; x = 90, y = 65, font = "medium", color = 0xffffffff);
                text!("Round: {}", self.round; x = 98, y = 82, font = "small", color = 0xaaaaaaff);
                
                // New high score?
                if self.score > self.high_score && self.score > 0 {
                    if (self.frame / 15) % 2 == 0 {
                        text!("NEW HIGH SCORE!", x = 56, y = 100, font = "medium", color = 0xffd700ff);
                    }
                }
                
                // Restart prompt
                if (self.frame / 25) % 2 == 0 {
                    text!("Press START to Retry", x = 56, y = 122, font = "small", color = 0x888888ff);
                }

                let gp = gamepad::get(0);
                if gp.start.just_pressed() || gp.a.just_pressed() {
                    Self::play_sfx("start");
                    self.reset_game();
                    self.play_mode_music();
                }
            }
            
            _ => {}
        }

        // ====================================================================
        // UI OVERLAY (on top of everything except flash)
        // ====================================================================
        if self.mode != MODE_TITLE && self.mode != MODE_GAMEOVER {
            // Score
            text!("Score: {}", self.score; x = 4 + shake_x, y = 136 + shake_y, font = "small", color = 0xffffffff);
            
            // Round indicator with flair
            rect!(x = 2 + shake_x, y = 2 + shake_y, w = 22, h = 10, color = 0x00000088);
            text!("R{}", self.round; x = 4 + shake_x, y = 4 + shake_y, font = "small", color = 0xffd700ff);
            
            // Timer bar
            self.draw_timer_bar(shake_x, shake_y);
            
            // Mode indicator
            self.draw_mode_indicator(shake_x, shake_y);
            
            // Gifts remaining (in Jingle/Hurry)
            if self.mode == MODE_JINGLE || self.mode == MODE_HURRY {
                let remaining = self.gifts.iter().filter(|g| !g.collected).count();
                text!("Gifts: {}", remaining; x = 180 + shake_x, y = 136 + shake_y, font = "small", color = 0x44ff44ff);
            }
        }
        
        // Draw score popups (always on top)
        self.draw_score_popups();
        
        // ====================================================================
        // SCREEN FLASH EFFECT (very top layer)
        // ====================================================================
        if self.screen_flash > 0 {
            let alpha = ((self.screen_flash as f32 / 20.0) * 200.0) as u32;
            let flash = (self.flash_color & 0xffffff00) | alpha;
            rect!(x = 0, y = 0, w = 256, h = 144, color = flash);
        }
    }
}
