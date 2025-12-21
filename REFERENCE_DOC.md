# 🎄🩸 Project Reference Document

## Retro Christmas Arcade Game (Turbo + Rust + WASM)

---

## 1. Project Overview

**Working Title:** _Jingle / Krampus_ (final name TBD)

# 🎄🩸 Project Reference Document

Retro Christmas Arcade Game (Turbo + Rust + WASM)

---

## 1. Project Overview

**Working Title:** _Jingle / Krampus_ (final name TBD)

**Genre:** 2D Arcade (Retro, Horror + Good Vibes)

**Theme:** Christmas (90s arcade nostalgia with psychological horror contrast)

**Engine:** Turbo

**Language:** Rust

**Platform:** Browser (WebAssembly)

**Core Idea:**
The game alternates between Good Vibes (Jingle Mode) and Horror (Krampus Mode) based on player actions and timers. One wrong step can summon Krampus. Survive the punishment phase to return to joy. Getting caught is an instant Game Over.

---

## 2. Core Gameplay Loop

1. Player starts in **Jingle Mode**
2. A timer runs
3. If the timer completes successfully → rewards (Santa, gifts, score)
4. If the timer expires → **Hurry Mode**
5. In Hurry Mode, stepping on traps summons **Krampus Mode**
6. In Krampus Mode:
   - Player cannot fight
   - Must survive until timer ends
7. If Krampus catches the player → Game Over
8. If the player survives → return to Jingle Mode (difficulty increases)
9. Loop repeats

This is an arcade loop: short runs, fast restarts, escalating tension.

---

## 3. Game Modes (State Machine)

### Enum

```rust
enum GameMode {
        Jingle,
        Hurry,
        Krampus,
        GameOver,
}
```

### Jingle Mode (Good Vibes)

- Bright visuals
- Cheerful chiptune
- Subtle traps
- Main scoring phase
- Timer-based survival

Success:

- Gifts spawn
- Santa appears
- Score bonus
- Difficulty increases

Failure:

- Timer expires → Hurry Mode

### Hurry Mode (Tension)

- Flickering lights
- Faster music
- Visual desaturation
- Short timer

Rules:

- Traps are now lethal triggers
- Stepping on a trap → Krampus Mode

### Krampus Mode (Horror)

- Dark palette
- Fog / shadows
- Distorted audio
- Krampus actively chases player

Rules:

- No combat; only evasion
- One hit = Game Over
- Survive until timer ends to escape

### Game Over

- Immediate hard cut
- Score shown
- Restart with one key press (arcade-style)

---

## 4. Technical Constraints

- Must use Turbo
- Must compile to WebAssembly
- Target smooth browser performance (60 FPS)
- Avoid overengineering; focus on polish and feel

---

## 5. Project Structure (suggested)

```
project-root/
├── src/
│   └── lib.rs
├── sprites/
│   ├── player.webp
│   ├── krampus.webp
│   ├── gifts.webp
│   └── tiles/
├── audio/
│   ├── jingle_loop.wav
│   ├── hurry_loop.wav
│   ├── krampus_loop.wav
│   └── sfx/
├── fonts/
├── turbo.toml
├── Cargo.toml
└── REFERENCE_DOC.md
```

---

## 6. Turbo Core Concepts (For Coding Agents)

### Entry Point

```rust
#[turbo::game]
struct GameState { /* ... */ }
```

### Main Loop

```rust
impl GameState {
        pub fn update(&mut self) {
                // runs ~60 times per second
                // update() handles logic + rendering
        }
}
```

Use frame counting for timers (60 frames ≈ 1 second).

---

## 7. Common Turbo APIs

- Drawing

  - `clear(color)`
  - `sprite!("name", x=..., y=...)`
  - `text!("text", x=..., y=..., font="large")`
  - `rect!()`, `circ!()`

- Input

  - `keyboard().is_down(Key::Left)`
  - `keyboard().is_pressed(Key::R)`
  - `gamepad::get(0).left.pressed()`

- Timing

  - `update()` runs at ~60 FPS
  - Use `frame % 60 == 0` for second-based checks

- Audio
  - Audio files auto-loaded from `audio/`
  - Use Turbo audio helpers to play loops and SFX

---

## 8. Turbo Documentation & Learning References

- Getting Started: https://docs.turbo.computer/learn/getting-started/
- Cheatsheet: https://docs.turbo.computer/learn/cheatsheet
- Tutorials:
  - Hello World: https://docs.turbo.computer/learn/tutorials/hello-world
  - Pancake Cat (Sprites, Input, Collisions): https://docs.turbo.computer/learn/tutorials/pancake-cat
  - Space Shooter Part 1 (Game Structure, Audio, Scaling): https://docs.turbo.computer/learn/tutorials/space-shooter-part-1
  - Character Sheet (UI, Stats, State): https://docs.turbo.computer/learn/tutorials/character-sheet

---

## 9. Design Priorities (IMPORTANT)

- Feel > Features
- Instant restart
- Clear feedback on mistakes
- Strong audio cues
- Obvious mode transitions
- Minimal UI clutter

---

## 10. Non-Goals (DO NOT IMPLEMENT)

- Complex ECS frameworks
- Save systems
- Long cutscenes
- Story text dumps
- Menus beyond restart

---

## 11. Visual & Audio Direction

Visual:

- Low resolution (e.g. 256x144)
- Pixel art
- CRT / VHS effects
- Hard cuts instead of smooth fades

Audio:

- Chiptune Christmas melodies
- Distorted versions for horror
- Loud footsteps for Krampus
- Strategic silence

---

## 12. Restart Philosophy

- Game Over → one key press → restart immediately
- No confirmation dialogs

---

## 13. Success Criteria

- Player understands rules in < 10 seconds
- First Krampus encounter is memorable
- Game feels responsive
- Mode transitions are emotionally clear

---

## 14. Notes for Coding Agents

- Keep Rust code simple and explicit
- Prefer clarity over abstraction
- Use enums and `match` statements
- Avoid unnecessary lifetime complexity
- Optimize later, polish early
