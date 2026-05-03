use crate::world::Platform;

pub const GRAVITY: f32 = 80.0;
pub const JUMP_VELOCITY: f32 = -35.0;
const PLAYER_WIDTH: u16 = 2;

pub struct Player {
    pub x: f32,
    pub y: f32, // terminal row where '@' is drawn
    pub vy: f32,
    pub on_ground: bool,
}

impl Player {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y, vy: 0.0, on_ground: false }
    }

    pub fn update(&mut self, dt: f32, platforms: &[Platform], screen_h: u16) -> bool {
        self.vy += GRAVITY * dt;
        let new_y = self.y + self.vy * dt;

        let mut landed = false;
        let player_left = self.x;
        let player_right = self.x + PLAYER_WIDTH as f32;
        let player_feet = self.y + 1.0;
        let new_feet = new_y + 1.0;

        if self.vy > 0.0 {
            for p in platforms {
                if player_right > p.x && player_left < p.x + p.width as f32 {
                    // Player feet cross the platform surface from above
                    if player_feet <= p.y && new_feet >= p.y {
                        landed = true;
                        self.y = p.y - 1.0;
                        self.vy = 0.0;
                        break;
                    }
                }
            }
        }

        if !landed {
            self.y = new_y;
            self.on_ground = false;
        } else {
            self.on_ground = true;
        }

        self.y >= (screen_h + 3) as f32
    }

    pub fn jump(&mut self) {
        if self.on_ground {
            self.vy = JUMP_VELOCITY;
            self.on_ground = false;
        }
    }
}
