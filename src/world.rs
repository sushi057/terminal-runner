use rand::Rng as _;

pub struct Platform {
    pub x: f32,
    pub y: f32,  // terminal row of platform surface
    pub width: u16,
}

pub struct World {
    pub platforms: Vec<Platform>,
    chunk_cursor: f32,
    pub scroll_speed: f32,
    screen_w: u16,
    screen_h: u16,
    last_plat_y: f32,
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
            last_plat_y: ground_y,
            rng: rand::thread_rng(),
        };
        world.generate_initial();
        world
    }

    fn generate_initial(&mut self) {
        let ground_y = self.screen_h as f32 * 3.0 / 4.0;
        self.platforms.push(Platform { x: 0.0, y: ground_y, width: 60 });
        self.last_plat_y = ground_y;
        self.chunk_cursor = 60.0;
        for _ in 0..6 {
            self.generate_chunk();
        }
    }

    pub fn generate_chunk(&mut self) {
        let gap = self.rng.gen_range(3..12) as f32;
        let plat_x = self.chunk_cursor + gap;

        // Platform y range: top quarter to just above bottom
        let min_y = (self.screen_h as f32 * 0.2) as u16;
        let max_y = self.screen_h - 4;
        let max_dy: f32 = 8.0;

        let target_y = self.rng.gen_range(min_y..max_y) as f32;
        let new_y = target_y.clamp(self.last_plat_y - max_dy, self.last_plat_y + max_dy);
        let new_y = new_y.clamp(min_y as f32, max_y as f32);

        let width = self.rng.gen_range(5..18);

        self.platforms.push(Platform { x: plat_x, y: new_y, width });
        self.last_plat_y = new_y;
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
