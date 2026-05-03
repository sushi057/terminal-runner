# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Terminal ASCII platformer — infinite runner built in Rust with raw ANSI escape sequences (no ncurses/crossterm). The player runs right across procedurally generated platforms, jumping gaps with increasing scroll speed.

## Build & Run

```sh
cargo build
cargo run
```

## Architecture

```
src/
  main.rs       — Entry point, wires Terminal + Game
  terminal.rs   — Raw mode (libc), diff-based FrameBuffer, input polling
  game.rs       — Game state machine, fixed-timestep loop, rendering, HUD
  player.rs     — Gravity, jump physics, AABB platform collision
  world.rs      — Procedural platform chunk generation, scroll/prune
```

### Key design points

- **Coordinate system**: Terminal-native — y=0 is top row, y increases downward. No Y-flip needed in rendering.
- **Double-buffered rendering**: `FrameBuffer` builds next frame in a `Vec<Vec<char>>` grid, diffs against previous frame, emits only changed cells via ANSI cursor-position escapes. Eliminates flicker.
- **Fixed timestep**: Physics at 60 Hz (`FIXED_DT = 1/60`), rendering capped at ~60 FPS.
- **Platform generation**: Chunks spawn ahead of camera with verically-clamped random positions (`max_dy = 8` between consecutive platforms). Difficulty ramps via `scroll_speed` increase over time.
- **Collision**: Player has 1-cell tall hitbox with feet at `y+1`. Lands when feet cross platform surface from above.
- **Only dependency**: `libc` for `tcgetattr`/`tcsetattr` (raw mode) and `poll` (non-blocking input). `rand` for platform generation.
