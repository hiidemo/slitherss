#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct Food {
    pub x: u16,
    pub y: u16,
    pub size: u8,
    pub color: u8,
}

impl Food {
    pub fn new(x: u16, y: u16, size: u8, color: u8) -> Self {
        Self { x, y, size, color }
    }
}

pub type FoodSeq = Vec<Food>;
