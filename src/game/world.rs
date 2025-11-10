use crate::config::{SnakeId, WorldConfig};
use crate::game::food::Food;
use crate::game::math;
use crate::game::sector::{BoundBoxPos, SectorSeq};
use crate::game::snake::{Body, Snake};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::collections::HashMap;
use std::f32::consts::PI;

#[derive(Debug)]
pub struct World {
    snakes: HashMap<SnakeId, Snake>,
    dead: Vec<SnakeId>,
    sectors: SectorSeq,
    changes: Vec<SnakeId>,
    last_snake_id: SnakeId,
    ticks: i64,
    frames: u32,
    config: WorldConfig,
    rng: StdRng,
}

impl World {
    pub fn new() -> Self {
        Self {
            snakes: HashMap::new(),
            dead: Vec::new(),
            sectors: SectorSeq::new(),
            changes: Vec::new(),
            last_snake_id: 0,
            ticks: 0,
            frames: 0,
            config: WorldConfig::default(),
            rng: StdRng::from_entropy(),
        }
    }

    pub fn init(&mut self, config: WorldConfig) {
        self.config = config;
        self.init_sectors();
        self.init_food();
    }

    pub fn init_sectors(&mut self) {
        self.sectors.init_sectors();
    }

    pub fn init_food(&mut self) {
        let food_count = 1000;
        for _ in 0..food_count {
            let x = self.rng.gen_range(0..2 * WorldConfig::GAME_RADIUS);
            let y = self.rng.gen_range(0..2 * WorldConfig::GAME_RADIUS);
            let size = self.rng.gen_range(5..15);
            let color = self.rng.gen_range(0..28);

            let food = Food::new(x, y, size, color);
            if let Some(sector) = self.sectors.get_sector_mut(x, y) {
                sector.insert_food(food);
            }
        }
    }

    pub fn create_snake(&mut self) -> SnakeId {
        self.last_snake_id += 1;
        let id = self.last_snake_id;

        let angle = self.rng.gen::<f32>() * 2.0 * PI;
        let dist = 1000.0 + self.rng.gen_range(0..5000) as f32;
        let mut x = WorldConfig::GAME_RADIUS as f32 + dist * angle.cos();
        let mut y = WorldConfig::GAME_RADIUS as f32 + dist * angle.sin();
        let angle = math::normalize_angle(angle + PI);

        let len = 1
            + 2
            + self.config.snake_min_length.max(
                self.rng.gen_range(0..self.config.snake_average_length),
            );

        let mut snake = Snake::new(id, x, y, 0);
        snake.skin = self.rng.gen_range(9..30);

        // Build snake parts
        for _ in 0..len.min(Snake::PARTS_SKIP_COUNT as u16 + Snake::PARTS_START_MOVE_COUNT as u16) {
            snake.parts.push(Body::new(x, y));
            x += angle.cos() * WorldConfig::MOVE_STEP_DISTANCE as f32;
            y += angle.sin() * WorldConfig::MOVE_STEP_DISTANCE as f32;
        }

        for _ in (Snake::PARTS_SKIP_COUNT + Snake::PARTS_START_MOVE_COUNT)..len as usize {
            snake.parts.push(Body::new(x, y));
            x += angle.cos() * Snake::TAIL_STEP_DISTANCE;
            y += angle.sin() * Snake::TAIL_STEP_DISTANCE;
        }

        snake.client_parts_index = snake.parts.len();
        snake.angle = math::normalize_angle(angle + PI);
        snake.wangle = math::normalize_angle(angle + PI);

        snake.update_box_center();
        snake.update_box_radius();
        snake.update_snake_consts();

        self.snakes.insert(id, snake);
        id
    }

    pub fn create_snake_bot(&mut self) -> SnakeId {
        let id = self.create_snake();
        if let Some(snake) = self.snakes.get_mut(&id) {
            snake.bot = true;
        }
        id
    }

    pub fn spawn_num_snakes(&mut self, count: u16) {
        for _ in 0..count {
            self.create_snake_bot();
        }
    }

    pub fn tick(&mut self, dt: i64) {
        self.ticks += dt;
        let vfr = self.ticks / WorldConfig::FRAME_TIME_MS as i64;
        if vfr > 0 {
            let vfr_time = vfr * WorldConfig::FRAME_TIME_MS as i64;
            self.tick_snakes(vfr_time);

            self.ticks -= vfr_time;
            self.frames += vfr as u32;
        }
    }

    fn tick_snakes(&mut self, dt: i64) {
        self.changes.clear();

        // Collect IDs first to avoid borrowing issues
        let snake_ids: Vec<SnakeId> = self.snakes.keys().copied().collect();

        for id in snake_ids {
            if let Some(snake) = self.snakes.get_mut(&id) {
                if snake.tick(dt, &mut self.sectors) {
                    self.changes.push(id);
                }
            }
        }

        // Check bounds for changed snakes
        let changes_copy: Vec<SnakeId> = self.changes.clone();
        for id in changes_copy {
            self.check_snake_bounds(id);
        }
    }

    fn check_snake_bounds(&mut self, id: SnakeId) {
        if let Some(snake) = self.snakes.get(&id) {
            if snake.parts.is_empty() {
                return;
            }

            let head = &snake.parts[0];
            let center_x = WorldConfig::GAME_RADIUS as f32;
            let center_y = WorldConfig::GAME_RADIUS as f32;

            // Check world bounds
            let dist_sq = head.distance_squared(center_x, center_y);
            let death_radius_sq =
                (WorldConfig::DEATH_RADIUS as f32) * (WorldConfig::DEATH_RADIUS as f32);

            if dist_sq >= death_radius_sq {
                if let Some(snake) = self.snakes.get_mut(&id) {
                    snake.update |= 1 << 5; // CHANGE_DYING
                }
                return;
            }

            // TODO: Implement collision detection with other snakes
        }
    }

    pub fn add_snake(&mut self, snake: Snake) {
        self.snakes.insert(snake.id, snake);
    }

    pub fn remove_snake(&mut self, id: SnakeId) {
        self.snakes.remove(&id);
        self.dead.push(id);
    }

    pub fn get_snake(&self, id: SnakeId) -> Option<&Snake> {
        self.snakes.get(&id)
    }

    pub fn get_snake_mut(&mut self, id: SnakeId) -> Option<&mut Snake> {
        self.snakes.get_mut(&id)
    }

    pub fn get_snakes(&self) -> &HashMap<SnakeId, Snake> {
        &self.snakes
    }

    pub fn get_snakes_mut(&mut self) -> &mut HashMap<SnakeId, Snake> {
        &mut self.snakes
    }

    pub fn get_sectors(&self) -> &SectorSeq {
        &self.sectors
    }

    pub fn get_sectors_mut(&mut self) -> &mut SectorSeq {
        &mut self.sectors
    }

    pub fn get_dead(&self) -> &Vec<SnakeId> {
        &self.dead
    }

    pub fn get_changed_snakes(&self) -> &Vec<SnakeId> {
        &self.changes
    }

    pub fn flush_changes(&mut self) {
        self.changes.clear();
        self.dead.clear();
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
