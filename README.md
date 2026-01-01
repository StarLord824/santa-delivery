# 🎅 Santa Delivery Arcade : https://courageous-speculoos-fd8fa9.netlify.app/

A retro-style pixel art arcade game where you play as Santa delivering gifts while dodging Krampus! Built with Rust and Turbo.

![Game Screenshot](https://raw.githubusercontent.com/getting-started/screenshot.png)

## 🎮 How to Play

- **Arrow Keys** or **D-Pad**: Move Santa Up/Down
- **ENTER**, **SPACE**, or **A Button**: Drop Gift
- **ESC**: Exit game

### Objective
- Deliver gifts to chimneys with glowing targets.
- Hitting a chimney gives points and counts towards deliveries.
- Missing too many chimneys increases the **Naughty Meter**.
- Every night (level), Krampus attacks! Dodge his fireballs and survive the timer.

## 🌟 Features

- **Dynamic Gameplay**: Gravity-based physics for gift dropping.
- **Krampus Boss Mode**: Intense survival sections with bullet-hell elements.
- **Pixel Art Visuals**: Custom sprites for Santa, Krampus, houses, and effects.
- **Dynamic Audio**: Adaptive music system that changes with game modes.
- **High Score System**: Track your best performance.

## 🛠️ Installation & Running

1. Install [Turbo](https://turbo.computer)
2. Clone this repository
3. Run the game:
   ```bash
   turbo run -w
   ```

## 📁 Project Structure

- `src/lib.rs` - Main game logic and state management
- `src/types.rs` - Data structures and constants
- `sprites/` - Pixel art assets
- `audio/` - Music and sound effects
- `turbo.toml` - Game configuration

## 🎨 Credits

- **Code & Design**: StarLord824
- **Engine**: Turbo (Rust)
