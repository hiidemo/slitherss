use crate::config::{SnakeId, WorldConfig};
use crate::game::food::{Food, FoodSeq};
use crate::game::math;
use crate::game::sector::{BoundBoxPos, SectorSeq, SnakeBoundBox, ViewPort};
use std::f32::consts::PI;

const CHANGE_POS: u8 = 1;
const CHANGE_ANGLE: u8 = 1 << 1;
const CHANGE_WANGLE: u8 = 1 << 2;
const CHANGE_SPEED: u8 = 1 << 3;
const CHANGE_FULLNESS: u8 = 1 << 4;
const CHANGE_DYING: u8 = 1 << 5;
const CHANGE_DEAD: u8 = 1 << 6;

#[derive(Debug, Clone, Copy)]
pub struct Body {
    pub x: f32,
    pub y: f32,
}

impl Body {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance_squared(&self, x: f32, y: f32) -> f32 {
        let dx = self.x - x;
        let dy = self.y - y;
        dx * dx + dy * dy
    }
}

#[derive(Debug, Clone)]
pub struct Snake {
    pub id: SnakeId,
    pub skin: u8,
    pub update: u8,
    pub acceleration: bool,
    pub bot: bool,
    pub name: String,
    pub speed: u16,
    pub angle: f32,
    pub wangle: f32,
    pub fullness: u16,
    pub sbb: SnakeBoundBox,
    pub vp: ViewPort,
    pub parts: Vec<Body>,
    pub eaten: FoodSeq,
    pub spawn: FoodSeq,
    pub client_parts_index: usize,

    // Private fields
    mov_ticks: i64,
    rot_ticks: i64,
    ai_ticks: i64,
    gsc: f32,
    sc: f32,
    scang: f32,
    ssp: f32,
    fsp: f32,
    sbpr: f32,
}

impl Snake {
    // Constants
    pub const SPANGDV: f32 = 4.8;
    pub const NSP1: f32 = 5.39;
    pub const NSP2: f32 = 0.4;
    pub const NSP3: f32 = 14.0;
    pub const BASE_MOVE_SPEED: u16 = 185;
    pub const BOOST_SPEED: u16 = 448;
    pub const SPEED_ACCELERATION: u16 = 1000;
    pub const PREY_ANGULAR_SPEED: f32 = 3.5;
    pub const SNAKE_ANGULAR_SPEED: f32 = 4.125;
    pub const SNAKE_TAIL_K: f32 = 0.43;
    pub const PARTS_SKIP_COUNT: usize = 3;
    pub const PARTS_START_MOVE_COUNT: usize = 4;
    pub const TAIL_STEP_DISTANCE: f32 = 24.0;
    pub const ROT_STEP_ANGLE: f32 =
        1.0 * WorldConfig::MOVE_STEP_DISTANCE as f32 / Self::BOOST_SPEED as f32 * Self::SNAKE_ANGULAR_SPEED;
    pub const ROT_STEP_INTERVAL: i64 = (1000.0 * Self::ROT_STEP_ANGLE / Self::SNAKE_ANGULAR_SPEED) as i64;
    pub const AI_STEP_INTERVAL: i64 = 1000;

    pub fn new(id: SnakeId, x: f32, y: f32, parts_count: usize) -> Self {
        let pos = BoundBoxPos::new(x, y, 100.0);
        let mut parts = Vec::new();

        // Initialize parts in a line
        for i in 0..parts_count {
            parts.push(Body::new(x - i as f32 * 10.0, y));
        }

        Self {
            id,
            skin: 0,
            update: 0,
            acceleration: false,
            bot: false,
            name: format!("Snake {}", id),
            speed: Self::BASE_MOVE_SPEED,
            angle: 0.0,
            wangle: 0.0,
            fullness: 100,
            sbb: SnakeBoundBox::new(pos, id),
            vp: ViewPort::new(pos, id),
            parts,
            eaten: Vec::new(),
            spawn: Vec::new(),
            client_parts_index: 0,
            mov_ticks: 0,
            rot_ticks: 0,
            ai_ticks: 0,
            gsc: 0.0,
            sc: 0.0,
            scang: 0.0,
            ssp: 0.0,
            fsp: 0.0,
            sbpr: 0.0,
        }
    }

    pub fn tick(&mut self, dt: i64, sectors: &mut SectorSeq) -> bool {
        let mut changes: u8 = 0;

        if self.update & (CHANGE_DYING | CHANGE_DEAD) != 0 {
            return false;
        }

        // AI tick for bots
        if self.bot {
            self.ai_ticks += dt;
            if self.ai_ticks > Self::AI_STEP_INTERVAL {
                let frames = self.ai_ticks / Self::AI_STEP_INTERVAL;
                self.tick_ai(frames);
                self.ai_ticks -= frames * Self::AI_STEP_INTERVAL;
            }
        }

        // Rotation
        if (self.angle - self.wangle).abs() > 0.001 {
            self.rot_ticks += dt;
            if self.rot_ticks >= Self::ROT_STEP_INTERVAL {
                let frames = self.rot_ticks / Self::ROT_STEP_INTERVAL;
                let frames_ticks = frames * Self::ROT_STEP_INTERVAL;
                let rotation = Self::SNAKE_ANGULAR_SPEED * frames_ticks as f32 / 1000.0;
                let mut d_angle = math::normalize_angle(self.wangle - self.angle);

                if d_angle > PI {
                    d_angle -= 2.0 * PI;
                }

                if d_angle.abs() < rotation {
                    self.angle = self.wangle;
                } else {
                    self.angle += rotation * if d_angle > 0.0 { 1.0 } else { -1.0 };
                }

                self.angle = math::normalize_angle(self.angle);
                changes |= CHANGE_ANGLE;
                self.rot_ticks -= frames_ticks;
            }
        }

        // Movement
        self.mov_ticks += dt;
        let mov_frame_interval = 1000 * WorldConfig::MOVE_STEP_DISTANCE as i64 / self.speed as i64;
        if self.mov_ticks >= mov_frame_interval {
            let frames = self.mov_ticks / mov_frame_interval;
            let frames_ticks = frames * mov_frame_interval;
            let move_dist = self.speed as f32 * frames_ticks as f32 / 1000.0;
            let len = self.parts.len();

            if len > 0 {
                // Move head
                let prev_head = self.parts[0];
                self.parts[0].x += self.angle.cos() * move_dist;
                self.parts[0].y += self.angle.sin() * move_dist;

                self.sbb.update_sectors(
                    sectors,
                    WorldConfig::SECTOR_SIZE as f32 / 2.0,
                    self.parts[0].x,
                    self.parts[0].y,
                    prev_head.x,
                    prev_head.y,
                );

                if !self.bot {
                    self.vp.update_sectors(
                        sectors,
                        self.parts[0].x,
                        self.parts[0].y,
                        prev_head.x,
                        prev_head.y,
                    );
                }

                // Move body
                let mut prev = prev_head;
                for i in 1..len.min(Self::PARTS_SKIP_COUNT) {
                    let old = self.parts[i];
                    self.parts[i] = prev;
                    prev = old;
                }

                // Move intermediate parts
                let mut j = 0;
                for i in Self::PARTS_SKIP_COUNT..(len.min(Self::PARTS_SKIP_COUNT + Self::PARTS_START_MOVE_COUNT)) {
                    let last = self.parts[i - 1];
                    let old = self.parts[i];

                    self.parts[i] = prev;
                    j += 1;
                    let move_coeff = Self::SNAKE_TAIL_K * j as f32 / Self::PARTS_START_MOVE_COUNT as f32;
                    self.parts[i].x += move_coeff * (last.x - self.parts[i].x);
                    self.parts[i].y += move_coeff * (last.y - self.parts[i].y);

                    prev = old;
                }

                // Move tail
                for i in (Self::PARTS_SKIP_COUNT + Self::PARTS_START_MOVE_COUNT)..len {
                    let last = self.parts[i - 1];
                    let old = self.parts[i];

                    self.parts[i] = prev;
                    self.parts[i].x += Self::SNAKE_TAIL_K * (last.x - self.parts[i].x);
                    self.parts[i].y += Self::SNAKE_TAIL_K * (last.y - self.parts[i].y);

                    prev = old;
                }

                changes |= CHANGE_POS;

                // Update bounding box
                self.update_box_center();
                self.update_box_radius();
            }

            // Update speed
            if self.acceleration {
                if self.parts.len() <= 3 {
                    self.acceleration = false;
                } else {
                    self.decrease_snake(33);
                }
            }

            let wanted_speed = if self.acceleration {
                Self::BOOST_SPEED
            } else {
                Self::BASE_MOVE_SPEED
            };

            if self.speed != wanted_speed {
                let sgn = if wanted_speed > self.speed { 1 } else { -1 };
                let acc = (Self::SPEED_ACCELERATION as i64 * frames_ticks / 1000) as u16;
                if (wanted_speed as i32 - self.speed as i32).abs() <= acc as i32 {
                    self.speed = wanted_speed;
                } else {
                    self.speed = (self.speed as i32 + sgn * acc as i32) as u16;
                }
                changes |= CHANGE_SPEED;
            }

            self.mov_ticks -= frames_ticks;
        }

        if changes > 0 && changes != self.update {
            self.update |= changes;
            return true;
        }

        false
    }

    pub fn tick_ai(&mut self, _frames: i64) {
        // Simple AI: random turning
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let turn = rng.gen_range(-0.5..0.5);
        self.wangle = math::normalize_angle(self.wangle + turn);
    }

    pub fn update_box_center(&mut self) {
        if self.parts.is_empty() {
            return;
        }

        let mut x = 0.0;
        let mut y = 0.0;

        for part in &self.parts {
            x += part.x;
            y += part.y;
        }

        x /= self.parts.len() as f32;
        y /= self.parts.len() as f32;

        self.sbb.bound_box.pos.x = x;
        self.sbb.bound_box.pos.y = y;

        self.vp.bound_box.pos.x = self.parts[0].x;
        self.vp.bound_box.pos.y = self.parts[0].y;
    }

    pub fn update_box_radius(&mut self) {
        let mut d = 42.0 + 42.0 + 42.0 + 37.7 + 37.7 + 33.0 + 28.5;

        if self.parts.len() > 8 {
            d += Self::TAIL_STEP_DISTANCE * (self.parts.len() - 8) as f32;
        }

        self.sbb.bound_box.pos.r = (d + WorldConfig::MOVE_STEP_DISTANCE as f32) / 2.0;
        self.vp.bound_box.pos.r = WorldConfig::SECTOR_DIAG_SIZE as f32 * 3.0;
    }

    pub fn increase_snake(&mut self, volume: u16) {
        let parts_to_add = (volume / 100).max(1);
        for _ in 0..parts_to_add {
            if let Some(last) = self.parts.last() {
                self.parts.push(*last);
            }
        }
        self.update_snake_consts();
    }

    pub fn decrease_snake(&mut self, volume: u16) {
        let parts_to_remove = (volume / 100).max(1).min(self.parts.len() as u16 - 3);
        for _ in 0..parts_to_remove {
            if self.parts.len() > 3 {
                if let Some(part) = self.parts.pop() {
                    self.spawn.push(Food::new(
                        part.x as u16,
                        part.y as u16,
                        5,
                        10,
                    ));
                }
            }
        }
        self.update_snake_consts();
    }

    pub fn update_snake_consts(&mut self) {
        let len = self.parts.len();
        self.gsc = 0.5 + 0.4 / (1.0 + (len as f32 - 1.0 + 16.0) / 36.0).max(1.0);
        self.sc = (6.0_f32).min(1.0 + (len as f32 - 2.0) / 106.0);
        self.scang = 0.13 + 0.87 * ((7.0 - self.sc) / 6.0).powi(2);
        self.ssp = Self::NSP1 + Self::NSP2 * self.sc;
        self.fsp = self.ssp + 0.1;
        self.sbpr = 14.5;
    }

    pub fn get_snake_scale(&self) -> f32 {
        self.gsc
    }

    pub fn get_snake_body_part_radius(&self) -> f32 {
        self.sbpr
    }

    pub fn get_snake_score(&self) -> u16 {
        (15.0 * (self.parts.len() as f32 / 10.0).floor()) as u16
    }

    pub fn intersect(&self, other: &BoundBoxPos) -> bool {
        self.sbb.bound_box.pos.intersect(other)
    }
}
