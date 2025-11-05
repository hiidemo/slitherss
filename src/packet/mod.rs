use crate::config::WorldConfig;
use bytes::{BufMut, BytesMut};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PacketType {
    Init = b'6',
    Snake = b's',
    Food = b'F',
    Move = b'e',
    Rotation = b'h',
    Pong = b'p',
}

pub struct PacketInit {
    pub game_radius: u32,
    pub max_snake_parts: u16,
    pub sector_size: u16,
    pub sector_count_along_edge: u16,
    pub spangdv: f32,
    pub nsp1: f32,
    pub nsp2: f32,
    pub nsp3: f32,
    pub snake_ang_speed: f32,
    pub prey_ang_speed: f32,
    pub snake_tail_k: f32,
    pub protocol_version: u8,
}

impl Default for PacketInit {
    fn default() -> Self {
        Self {
            game_radius: WorldConfig::GAME_RADIUS as u32,
            max_snake_parts: WorldConfig::MAX_SNAKE_PARTS,
            sector_size: WorldConfig::SECTOR_SIZE,
            sector_count_along_edge: WorldConfig::SECTOR_COUNT_ALONG_EDGE,
            spangdv: 4.8,
            nsp1: 5.39,
            nsp2: 0.4,
            nsp3: 14.0,
            snake_ang_speed: 0.033,
            prey_ang_speed: 0.028,
            snake_tail_k: 0.43,
            protocol_version: WorldConfig::PROTOCOL_VERSION,
        }
    }
}

impl PacketInit {
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(26);

        // Packet type
        buf.put_u8(PacketType::Init as u8);
        // Client time (placeholder)
        buf.put_u16(0);

        // Game radius (24-bit)
        buf.put_u8((self.game_radius >> 16) as u8);
        buf.put_u8((self.game_radius >> 8) as u8);
        buf.put_u8(self.game_radius as u8);

        // Other fields
        buf.put_u16(self.max_snake_parts);
        buf.put_u16(self.sector_size);
        buf.put_u16(self.sector_count_along_edge);
        buf.put_u8((self.spangdv * 10.0) as u8);
        buf.put_u16((self.nsp1 * 100.0) as u16);
        buf.put_u16((self.nsp2 * 100.0) as u16);
        buf.put_u16((self.nsp3 * 100.0) as u16);
        buf.put_u16((self.snake_ang_speed * 1000.0) as u16);
        buf.put_u16((self.prey_ang_speed * 1000.0) as u16);
        buf.put_u16((self.snake_tail_k * 1000.0) as u16);
        buf.put_u8(self.protocol_version);

        buf.to_vec()
    }
}
