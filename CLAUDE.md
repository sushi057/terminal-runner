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
  terminal.rs   — Raw mode (libc), Cell-based diff buffer with ANSI color SGR, input polling
  game.rs       — Game state, fixed-timestep loop, colored rendering, HUD, background
  player.rs     — Gravity, jump physics, AABB platform collision
  world.rs      — Physics-aware procedural platform generation, platform types
```

### Key design points

- **Coordinate system**: Terminal-native — y=0 is top row, y increases downward. No Y-flip needed.
- **Cell-based double buffering**: Each cell stores a char + optional Color. `flush()` diffs against the previous frame and emits only changed cells with ANSI cursor-positioning. Color SGR codes are emitted inline when the color changes between cells within a row, then reset at row end.
- **Color enum**: Green, Brown, Cyan, Gray, White, Yellow, Red, Blue — mapped to ANSI 3/4-bit SGR codes in `emit_sgr()`.
- **Fixed timestep**: Physics at 60 Hz (`FIXED_DT = 1/60`), rendering capped at ~60 FPS with frame timing.
- **Physics-aware platform generation**: `is_reachable()` checks whether the player can reach a candidate platform given the vertical delta, horizontal gap, and current scroll speed. Upward jumps use the full quadratic trajectory (descending arc landing). Downward steps use free-fall time. Falls back to an easy platform if no reachable candidate is found within 20 attempts.
- **Platform types**: Grass (`╭▄╮` green), Stone (`[█]` gray), Wood (`╭▬╮` brown), Ice (`╭─╮` cyan) — each with distinct characters and edge caps.
- **Player sprite**: 2-char running animation (`o/` / `\o`) in green on ground, jumping pose (`o^`) in yellow when airborne. Shadow dot below.
- **Background**: Randomly placed twinkling stars (gray `.`) that fade in/out based on frame count. Ground line of dots.
- **Collision**: Player hitbox has feet at `y+1`. Lands when feet cross a platform surface from above on a downward trajectory.
- **Only dependencies**: `libc` for raw terminal I/O, `rand` for generation.
