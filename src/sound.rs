// AUDIO SYSTEM MODULE

#![allow(dead_code)]
use turbo::audio;
use crate::types::*;

/// Play background music based on current game mode
pub fn play_mode_music(mode: u8) {
    // Stop all music tracks first
    audio::stop("start");
    audio::stop("game");
    audio::stop("krampus");
    audio::stop("game_over");
    
    // Play appropriate track for current mode
    match mode {
        MODE_TITLE => audio::play("start"),
        MODE_DELIVERING => audio::play("game"),
        MODE_KRAMPUS => audio::play("krampus"),
        MODE_GAMEOVER => audio::play("game_over"),
        _ => {}
    }
}

/// Keep music looping (call every frame)
pub fn update_music(mode: u8) {
    match mode {
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
pub fn play_sfx(name: &str) {
    audio::play(name);
}
