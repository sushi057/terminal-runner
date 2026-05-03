use std::time::{Duration, Instant};

use crate::player::Player;
use crate::terminal::{Key, Terminal};
use crate::world::World;

const FIXED_DT: f32 = 1.0 / 60.0;
const FRAME_DURATION: Duration = Duration::from_millis(16); // ~60 FPS cap

pub struct Game {
    player: Player,
    world: World,
    camera_x: f32,
    score: u32,
    screen_w: u16,
    screen_h: u16,
    dead: bool,
}

impl Game {
    pub fn new(screen_w: u16, screen_h: u16) -> Self {
        let world = World::new(screen_w, screen_h);
        let player_x = (screen_w as f32 * 0.2).round();
        // Player on top of the starting platform (1 row above it)
        let ground_y = screen_h as f32 * 3.0 / 4.0;
        let player_y = ground_y - 1.0;
        Self {
            player: Player::new(player_x, player_y),
            world,
            camera_x: 0.0,
            score: 0,
            screen_w,
            screen_h,
            dead: false,
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

            // Frame rate cap — sleep remaining time
            let elapsed = last_time.elapsed();
            if elapsed < FRAME_DURATION {
                std::thread::sleep(FRAME_DURATION - elapsed);
            }
        }

        if !self.dead {
            return; // quit via Esc/q, skip death screen
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

        // Ramp difficulty: speed increases with distance
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

        // Ground line at bottom
        for x in 0..self.screen_w {
            term.set_cell(x, self.screen_h - 1, '.');
        }

        // Draw platforms
        for p in &self.world.platforms {
            let sx = (p.x - self.camera_x) as i32;
            if (sx + (p.width as i32) < 0) || (sx >= self.screen_w as i32) {
                continue;
            }
            let x0 = sx.max(0) as u16;
            let x1 = (sx + p.width as i32).min(self.screen_w as i32 - 1) as u16;
            let sy = p.y as u16;
            for x in x0..=x1 {
                term.set_cell(x, sy, '=');
            }
        }

        // Draw player
        let px = (self.player.x - self.camera_x) as u16;
        let py = self.player.y as u16;
        if py < self.screen_h {
            term.set_cell(px, py, '@');
        }

        // HUD
        let hud = format!(" Distance: {}m  Speed: {:.0} ", self.score, self.world.scroll_speed);
        for i in 0..self.screen_w {
            term.set_cell(i, 0, ' ');
        }
        term.set_str(0, 0, &hud);

        term.flush();
    }

    fn render_death(&self, term: &mut Terminal) {
        term.clear();

        let msg = " GAME OVER ";
        let score = format!(" You ran {} meters! ", self.score);
        let prompt = " Press any key to exit ";

        let cx = (self.screen_w as usize - msg.len()) as u16 / 2;
        let cy = self.screen_h / 2 - 2;
        term.set_str(cx, cy, msg);
        term.set_str((self.screen_w as usize - score.len()) as u16 / 2, cy + 1, &score);
        term.set_str((self.screen_w as usize - prompt.len()) as u16 / 2, cy + 3, prompt);

        term.flush();
    }
}
