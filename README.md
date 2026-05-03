# terminal-runner

A terminal ASCII platformer — infinite runner built in Rust with raw ANSI escape sequences.

## Controls

- **Space / Up Arrow** — Jump
- **Esc / Q** — Quit

## Run

```sh
cargo run
```

## How it works

You're `@`, running right forever across scrolling `===` platforms. Jump gaps, don't fall. Speed increases with distance. No external dependencies beyond `libc` and `rand` — rendering is done with raw ANSI escape sequences.
