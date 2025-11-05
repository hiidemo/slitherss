use clap::Parser;

pub type SnakeId = u16;

#[derive(Debug, Clone)]
pub struct WorldConfig {
    pub bots: u16,
    pub snake_average_length: u16,
    pub snake_min_length: u16,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            bots: 0,
            snake_average_length: 2,
            snake_min_length: 2,
        }
    }
}

impl WorldConfig {
    pub const GAME_RADIUS: u16 = 21600;
    pub const MAX_SNAKE_PARTS: u16 = 411;
    pub const SECTOR_SIZE: u16 = 300;
    pub const SECTOR_COUNT_ALONG_EDGE: u16 = 2 * Self::GAME_RADIUS / Self::SECTOR_SIZE;
    pub const DEATH_RADIUS: u16 = Self::GAME_RADIUS - Self::SECTOR_SIZE;
    pub const SECTOR_DIAG_SIZE: u16 = 425;
    pub const MOVE_STEP_DISTANCE: u16 = 42;
    pub const FRAME_TIME_MS: u64 = 8;
    pub const PROTOCOL_VERSION: u8 = 8;
}

#[derive(Debug, Parser)]
#[command(name = "slitherss")]
#[command(about = "Slither.io server in Rust", long_about = None)]
pub struct Config {
    #[arg(short, long, default_value = "8080")]
    pub port: u16,

    #[arg(long, default_value = "0")]
    pub bots: u16,

    #[arg(long, default_value = "2")]
    pub avg_len: u16,

    #[arg(long, default_value = "2")]
    pub min_len: u16,

    #[arg(short, long)]
    pub verbose: bool,

    #[arg(short, long)]
    pub debug: bool,
}

impl Config {
    pub fn world_config(&self) -> WorldConfig {
        WorldConfig {
            bots: self.bots,
            snake_average_length: self.avg_len,
            snake_min_length: self.min_len,
        }
    }
}
