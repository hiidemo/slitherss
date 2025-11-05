use crate::config::{SnakeId, WorldConfig};
use crate::game::food::{Food, FoodSeq};
use crate::game::math;

#[derive(Debug, Clone, Copy)]
pub struct BoundBoxPos {
    pub x: f32,
    pub y: f32,
    pub r: f32,
}

impl BoundBoxPos {
    pub fn new(x: f32, y: f32, r: f32) -> Self {
        Self { x, y, r }
    }

    pub fn intersect(&self, other: &BoundBoxPos) -> bool {
        math::intersect_circle(self.x, self.y, other.x, other.y, self.r + other.r)
    }
}

#[derive(Debug, Clone)]
pub struct BoundBox {
    pub pos: BoundBoxPos,
    pub id: SnakeId,
    pub sectors: Vec<usize>,
}

impl BoundBox {
    pub fn new(pos: BoundBoxPos, id: SnakeId) -> Self {
        Self {
            pos,
            id,
            sectors: Vec::new(),
        }
    }

    pub fn insert_sector(&mut self, sector_idx: usize) {
        if !self.sectors.contains(&sector_idx) {
            self.sectors.push(sector_idx);
        }
    }

    pub fn is_present(&self, sector_idx: usize) -> bool {
        self.sectors.contains(&sector_idx)
    }

    pub fn sort_sectors(&mut self) {
        self.sectors.sort_unstable();
    }
}

#[derive(Debug, Clone)]
pub struct Sector {
    pub x: u8,
    pub y: u8,
    pub box_pos: BoundBoxPos,
    pub snakes: Vec<SnakeId>,
    pub food: FoodSeq,
}

impl Sector {
    pub fn new(x: u8, y: u8) -> Self {
        let half = WorldConfig::SECTOR_SIZE / 2;
        let r = WorldConfig::SECTOR_DIAG_SIZE as f32 / 2.0;

        let box_x = (WorldConfig::SECTOR_SIZE * x as u16 + half) as f32;
        let box_y = (WorldConfig::SECTOR_SIZE * y as u16 + half) as f32;

        Self {
            x,
            y,
            box_pos: BoundBoxPos::new(box_x, box_y, r),
            snakes: Vec::new(),
            food: Vec::new(),
        }
    }

    pub fn intersect(&self, box_pos: &BoundBoxPos) -> bool {
        self.box_pos.intersect(box_pos)
    }

    pub fn insert_food(&mut self, food: Food) {
        self.food.push(food);
    }

    pub fn remove_snake(&mut self, id: SnakeId) {
        self.snakes.retain(|&snake_id| snake_id != id);
    }

    pub fn sort_food(&mut self) {
        self.food.sort_by_key(|f| f.x);
    }
}

#[derive(Debug, Clone)]
pub struct SectorSeq {
    sectors: Vec<Sector>,
}

impl SectorSeq {
    pub fn new() -> Self {
        Self {
            sectors: Vec::new(),
        }
    }

    pub fn init_sectors(&mut self) {
        let count = WorldConfig::SECTOR_COUNT_ALONG_EDGE as usize;
        self.sectors.clear();
        self.sectors.reserve(count * count);

        for y in 0..count {
            for x in 0..count {
                self.sectors.push(Sector::new(x as u8, y as u8));
            }
        }
    }

    pub fn get_index(&self, x: u16, y: u16) -> usize {
        let sx = (x / WorldConfig::SECTOR_SIZE) as usize;
        let sy = (y / WorldConfig::SECTOR_SIZE) as usize;
        let edge = WorldConfig::SECTOR_COUNT_ALONG_EDGE as usize;
        sy * edge + sx
    }

    pub fn get_sector(&self, x: u16, y: u16) -> Option<&Sector> {
        let idx = self.get_index(x, y);
        self.sectors.get(idx)
    }

    pub fn get_sector_mut(&mut self, x: u16, y: u16) -> Option<&mut Sector> {
        let idx = self.get_index(x, y);
        self.sectors.get_mut(idx)
    }

    pub fn get_by_index(&self, idx: usize) -> Option<&Sector> {
        self.sectors.get(idx)
    }

    pub fn get_by_index_mut(&mut self, idx: usize) -> Option<&mut Sector> {
        self.sectors.get_mut(idx)
    }

    pub fn len(&self) -> usize {
        self.sectors.len()
    }
}

impl Default for SectorSeq {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct SnakeBoundBox {
    pub bound_box: BoundBox,
}

impl SnakeBoundBox {
    pub fn new(pos: BoundBoxPos, id: SnakeId) -> Self {
        Self {
            bound_box: BoundBox::new(pos, id),
        }
    }

    pub fn update_sectors(
        &mut self,
        sectors: &mut SectorSeq,
        bb_r: f32,
        new_x: f32,
        new_y: f32,
        old_x: f32,
        old_y: f32,
    ) {
        let new_box = BoundBoxPos::new(new_x, new_y, bb_r);
        let old_box = BoundBoxPos::new(old_x, old_y, bb_r);

        for i in 0..sectors.len() {
            if let Some(sector) = sectors.get_by_index(i) {
                let intersects_new = sector.intersect(&new_box);
                let intersects_old = sector.intersect(&old_box);
                let is_present = self.bound_box.is_present(i);

                if intersects_new && !is_present {
                    self.bound_box.insert_sector(i);
                } else if !intersects_new && is_present {
                    // Remove sector
                    self.bound_box.sectors.retain(|&idx| idx != i);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ViewPort {
    pub bound_box: BoundBox,
    pub new_sectors: Vec<usize>,
    pub old_sectors: Vec<usize>,
}

impl ViewPort {
    pub fn new(pos: BoundBoxPos, id: SnakeId) -> Self {
        Self {
            bound_box: BoundBox::new(pos, id),
            new_sectors: Vec::new(),
            old_sectors: Vec::new(),
        }
    }

    pub fn update_sectors(
        &mut self,
        sectors: &SectorSeq,
        new_x: f32,
        new_y: f32,
        old_x: f32,
        old_y: f32,
    ) {
        // Viewport radius is typically larger than snake bounding box
        let vp_radius = 1500.0; // Adjust based on game requirements

        let new_box = BoundBoxPos::new(new_x, new_y, vp_radius);
        let old_box = BoundBoxPos::new(old_x, old_y, vp_radius);

        self.new_sectors.clear();
        self.old_sectors.clear();

        for i in 0..sectors.len() {
            if let Some(sector) = sectors.get_by_index(i) {
                let intersects_new = sector.intersect(&new_box);
                let intersects_old = sector.intersect(&old_box);
                let is_present = self.bound_box.is_present(i);

                if intersects_new && !is_present {
                    self.new_sectors.push(i);
                    self.bound_box.insert_sector(i);
                } else if !intersects_new && is_present {
                    self.old_sectors.push(i);
                    self.bound_box.sectors.retain(|&idx| idx != i);
                }
            }
        }
    }
}
