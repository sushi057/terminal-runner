use std::time::{Duration, Instant};

use crate::player::Player;
use crate::terminal::{Color, Key, Terminal};
use crate::world::{Platform, PlatformKind, World};

const FIXED_DT: f32 = 1.0 / 60.0;
const FRAME_DURATION: Duration = Duration::from_millis(16);

pub struct Game {
    player: Player,
    world: World,
    camera_x: f32,
    score: u32,
    screen_w: u16,
    screen_h: u16,
    dead: bool,
    stars: Vec<(u16, u16)>, // (x, y) background star positions
    frame_count: u64,
}

impl Game {
    pub fn new(screen_w: u16, screen_h: u16) -> Self {
        let world = World::new(screen_w, screen_h);
        let player_x = (screen_w as f32 * 0.2).round();
        let ground_y = screen_h as f32 * 3.0 / 4.0;
        let player_y = ground_y - 1.0;

        // Generate random background stars
        let mut stars = Vec::new();
        // Use a simple xorshift-like approach seeded by address
        let mut seed: u64 = screen_w as u64 + ((screen_h as u64) << 16) + 0xDEADBEEF;
        for _ in 0..(screen_w as usize * screen_h as usize / 25).max(10).min(80) {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let sx = (seed >> 16) as u16 % screen_w;
            let sy = 2 + ((seed >> 32) as u16 % (screen_h.saturating_sub(4)).max(1));
            stars.push((sx, sy));
        }

        Self {
            player: Player::new(player_x, player_y),
            world,
            camera_x: 0.0,
            score: 0,
            screen_w,
            screen_h,
            dead: false,
            stars,
            frame_count: 0,
        }
    }

    pub fn run(&mut self, term: &mut Terminal) {
        let mut last_time = Instant::now();
        let mut accumulator: f32 = 0.0;

        while !self.dead {
            let now = Instant::now();
            let frame_time = (now - last_time).as_secs_f32().min(0.1);
            last_time = now;
            accumulator += frame_time;

            match term.poll_key() {
                Some(Key::Quit) => break,
                Some(Key::Space) | Some(Key::Up) => self.player.jump(),
                _ => {}
            }

            while accumulator >= FIXED_DT {
                self.tick(FIXED_DT);
                accumulator -= FIXED_DT;
            }

            self.render(term);

            let elapsed = last_time.elapsed();
            if elapsed < FRAME_DURATION {
                std::thread::sleep(FRAME_DURATION - elapsed);
            }
            self.frame_count += 1;
        }

        if !self.dead {
            return;
        }

        self.render_death(term);
        loop {
            if term.poll_key().is_some() {
                break;
            }
        }
    }

    fn tick(&mut self, dt: f32) {
        if self.dead {
            return;
        }

        self.world.scroll_speed = 15.0 + (self.score as f32 * 0.02).min(40.0);
        self.camera_x += self.world.scroll_speed * dt;
        self.score = (self.camera_x / 5.0) as u32;
        self.world.update(self.camera_x);

        let player_world_x = self.camera_x + (self.screen_w as f32 * 0.2);
        self.player.x = player_world_x;

        if self.player.update(dt, &self.world.platforms, self.screen_h) {
            self.dead = true;
        }
    }

    fn render(&self, term: &mut Terminal) {
        term.clear();

        // Background stars
        for &(sx, sy) in &self.stars {
            // Twinkle based on frame count and star position
            let twinkle = (self.frame_count.wrapping_add(sx as u64 * 7 + sy as u64 * 13)) % 60;
            if twinkle < 50 {
                term.set_colored(sx, sy, '.', Color::Gray);
            }
        }

        // Ground line
        for x in 0..self.screen_w {
            term.set_colored(x, self.screen_h - 1, '.', Color::Gray);
        }

        // Draw platforms
        for p in &self.world.platforms {
            let sx = (p.x - self.camera_x) as i32;
            if (sx + (p.width as i32) < 0) || (sx >= self.screen_w as i32) {
                continue;
            }
            let x0 = sx.max(0) as u16;
            let x1 = (sx + p.width as i32).min(self.screen_w as i32 - 1) as u16;
            self.draw_platform(term, x0, x1, p);
        }

        // Player shadow (one row below, dim)
        let px = (self.player.x - self.camera_x) as u16;
        let py = self.player.y as u16;
        if py + 1 < self.screen_h {
            term.set_colored(px, py + 1, '.', Color::Gray);
        }

        // Player sprite (2-char)
        if py < self.screen_h {
            if self.player.on_ground {
                // Running pose — alternates each frame
                if self.frame_count % 8 < 4 {
                    term.set_colored(px, py, 'o', Color::Green);
                    if px + 1 < self.screen_w {
                        term.set_colored(px + 1, py, '/', Color::Green);
                    }
                } else {
                    term.set_colored(px, py, '\\', Color::Green);
                    if px + 1 < self.screen_w {
                        term.set_colored(px + 1, py, 'o', Color::Green);
                    }
                }
            } else {
                // Jumping pose
                term.set_colored(px, py, 'o', Color::Yellow);
                if px + 1 < self.screen_w {
                    term.set_colored(px + 1, py, '^', Color::Yellow);
                }
            }
        }

        // HUD
        let hud = format!(
            " Distance: {}m  Speed: {:.0}  {}",
            self.score,
            self.world.scroll_speed,
            if self.player.on_ground { "[===]" } else { "[ ~ ]" }
        );
        for i in 0..self.screen_w {
            term.set_cell(i, 0, ' ');
        }
        term.set_str(0, 0, &hud);

        term.flush();
    }

    fn draw_platform(&self, term: &mut Terminal, x0: u16, x1: u16, p: &Platform) {
        let sy = p.y as u16;
        let (surface_char, color) = match p.kind {
            PlatformKind::Grass => ('▄', Color::Green),
            PlatformKind::Stone => ('█', Color::Gray),
            PlatformKind::Wood => ('▬', Color::Brown),
            PlatformKind::Ice => ('─', Color::Cyan),
        };

        // Platform surface with edge caps
        for x in x0..=x1 {
            let ch = if x == x0 {
                match p.kind {
                    PlatformKind::Grass => '╭',
                    PlatformKind::Stone => '[',
                    PlatformKind::Wood => '╭',
                    PlatformKind::Ice => '╭',
                }
            } else if x == x1 {
                match p.kind {
                    PlatformKind::Grass => '╮',
                    PlatformKind::Stone => ']',
                    PlatformKind::Wood => '╮',
                    PlatformKind::Ice => '╮',
                }
            } else {
                surface_char
            };
            term.set_colored(x, sy, ch, color);
        }
    }

    fn render_death(&self, term: &mut Terminal) {
        term.clear();

        let msg = " GAME OVER ";
        let score = format!(" You ran {} meters! ", self.score);
        let prompt = " Press any key to exit ";

        let cx = (self.screen_w as usize - msg.len()) as u16 / 2;
        let cy = self.screen_h / 2 - 2;
        for (i, ch) in msg.chars().enumerate() {
            term.set_colored(cx + i as u16, cy, ch, Color::Red);
        }
        term.set_str((self.screen_w as usize - score.len()) as u16 / 2, cy + 1, &score);
        term.set_str((self.screen_w as usize - prompt.len()) as u16 / 2, cy + 3, prompt);

        term.flush();
    }
}
