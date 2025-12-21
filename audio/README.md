# 🎵 Audio Files Required

Place these audio files in the `audio/` folder. Supported formats: `.wav`, `.ogg`, `.mp3`, `.flac`

## Background Music (Looping)

| Filename | Description | Style |
|----------|-------------|-------|
| `title_bgm.wav` | Title screen music | Cheerful Christmas jingle |
| `jingle_bgm.wav` | Main gameplay (Jingle mode) | Upbeat chiptune Christmas |
| `hurry_bgm.wav` | Hurry mode music | Faster, tense version |
| `krampus_bgm.wav` | Krampus chase music | Dark, scary, intense |

## Sound Effects (One-shot)

| Filename | Description | Style |
|----------|-------------|-------|
| `start.wav` | Game start / restart | Short jingle or chime |
| `pickup.wav` | Gift collected | Happy "ding" or coin sound |
| `bonus.wav` | Round complete bonus | Triumphant fanfare |
| `hurry.wav` | Entering Hurry mode | Warning alarm |
| `trap.wav` | Stepped on trap | Scary stinger |
| `survive.wav` | Survived Krampus | Relief sound, victory |
| `gameover.wav` | Caught by Krampus | Death/failure sound |

---

## 🆓 Free Chiptune Resources

### OpenGameArt.org (Recommended)
- https://opengameart.org/art-search-advanced?keys=christmas&field_art_type_tid%5B%5D=13
- https://opengameart.org/content/8-bit-chiptune-jingles

### Freesound.org
- https://freesound.org/search/?q=8bit+christmas
- https://freesound.org/search/?q=chiptune+sfx

### itch.io (Chiptune Packs)
- https://tallbeard.itch.io/music-loop-bundle
- Search "chiptune Christmas" on itch.io

### AI Generation
- Use Suno.ai or similar to generate custom chiptune Christmas music
- Request "8-bit retro Christmas arcade game music"

---

## 🔧 Quick Test Without Audio

The game will run fine without audio files - it will just be silent.
Audio functions gracefully handle missing files.

---

## ⚡ Quick Setup (Using Freesound)

1. Go to https://freesound.org
2. Search for each sound type
3. Download WAV format
4. Rename to match the filenames above
5. Place in `audio/` folder
6. Restart `turbo run -w`
