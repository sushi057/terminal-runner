use rand::Rng as _;

use crate::player;

#[derive(Clone, Copy, PartialEq)]
pub enum PlatformKind {
    Grass,
    Stone,
    Wood,
    Ice,
}

pub struct Platform {
    pub x: f32,
    pub y: f32, // terminal row of platform surface
    pub width: u16,
    pub kind: PlatformKind,
}

pub struct World {
    pub platforms: Vec<Platform>,
    chunk_cursor: f32,
    pub scroll_speed: f32,
    screen_w: u16,
    screen_h: u16,
    last_plat_x: f32,
    last_plat_y: f32,
    last_plat_w: u16,
    rng: rand::rngs::ThreadRng,
}

impl World {
    pub fn new(screen_w: u16, screen_h: u16) -> Self {
        let ground_y = screen_h as f32 * 3.0 / 4.0;
        let mut world = Self {
            platforms: Vec::new(),
            chunk_cursor: 0.0,
            scroll_speed: 15.0,
            screen_w,
            screen_h,
            last_plat_x: 0.0,
            last_plat_y: ground_y,
            last_plat_w: 60,
            rng: rand::thread_rng(),
        };
        world.generate_initial();
        world
    }

    fn generate_initial(&mut self) {
        let ground_y = self.screen_h as f32 * 3.0 / 4.0;
        self.platforms.push(Platform { x: 0.0, y: ground_y, width: 60, kind: PlatformKind::Grass });
        self.last_plat_x = 0.0;
        self.last_plat_y = ground_y;
        self.last_plat_w = 60;
        self.chunk_cursor = 60.0;
        for _ in 0..6 {
            self.generate_chunk();
        }
    }

    /// Check if the player can reach a target platform from the last platform.
    fn is_reachable(from_y: f32, to_y: f32, gap: f32, speed: f32) -> bool {
        let dy = from_y - to_y; // positive = target is above

        if dy > 0.0 {
            // Need to jump upward
            let peak = player::JUMP_VELOCITY.powi(2) / (2.0 * player::GRAVITY);
            if dy > peak + 1.0 {
                return false;
            }
            let disc = player::JUMP_VELOCITY.powi(2) - 2.0 * player::GRAVITY * dy;
            if disc < 0.0 {
                return false;
            }
            // Descending arc solution — player lands on the platform coming down
            let t = (-player::JUMP_VELOCITY + disc.sqrt()) / player::GRAVITY;
            gap <= speed * t * 1.1 // 10% margin
        } else if dy < 0.0 {
            // Target is below — player falls
            let fall = -dy;
            let t = (2.0 * fall / player::GRAVITY).sqrt();
            gap <= speed * t * 1.1
        } else {
            // Same height — player runs off and immediately starts falling
            // They have one frame (~1 cell of fall) before missing the platform
            let t = (2.0 / player::GRAVITY).sqrt(); // time to fall 1 cell
            gap <= speed * t
        }
    }

    pub fn generate_chunk(&mut self) {
        let min_y = (self.screen_h as f32 * 0.15) as u16;
        let max_y = self.screen_h - 4;
        let speed = self.scroll_speed;

        for _ in 0..20 {
            let gap = self.rng.gen_range(2..9) as f32;

            // Bias toward smaller vertical changes
            let dy: f32 = if self.rng.gen_bool(0.6) {
                self.rng.gen_range(-3..=3) as f32
            } else {
                self.rng.gen_range(-5..=5) as f32
            };

            let new_y = (self.last_plat_y + dy).clamp(min_y as f32, max_y as f32);

            if !Self::is_reachable(self.last_plat_y, new_y, gap, speed) {
                continue;
            }

            let width = self.rng.gen_range(6..18);
            let plat_x = self.chunk_cursor + gap;

            let kind = match self.rng.gen_range(0..6) {
                0 => PlatformKind::Stone,
                1 => PlatformKind::Wood,
                2 => PlatformKind::Ice,
                _ => PlatformKind::Grass,
            };

            self.platforms.push(Platform { x: plat_x, y: new_y, width, kind });
            self.last_plat_x = plat_x;
            self.last_plat_y = new_y;
            self.last_plat_w = width;
            self.chunk_cursor = plat_x + width as f32;
            return;
        }

        // Fallback: easy platform close by
        let width: u16 = self.rng.gen_range(6..18);
        let gap = self.rng.gen_range(1..4) as f32;
        let dy = self.rng.gen_range(-2..=1) as f32;
        let new_y = (self.last_plat_y + dy).clamp(min_y as f32, max_y as f32);
        let plat_x = self.chunk_cursor + gap;

        self.platforms.push(Platform { x: plat_x, y: new_y, width, kind: PlatformKind::Grass });
        self.last_plat_x = plat_x;
        self.last_plat_y = new_y;
        self.last_plat_w = width;
        self.chunk_cursor = plat_x + width as f32;
    }

    pub fn update(&mut self, camera_x: f32) {
        let lookahead = camera_x + self.screen_w as f32 * 2.0;
        while self.chunk_cursor < lookahead {
            self.generate_chunk();
        }
        let remove_before = camera_x - 20.0;
        self.platforms.retain(|p| p.x + p.width as f32 > remove_before);
    }
}
