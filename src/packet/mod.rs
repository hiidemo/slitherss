use bytes::{BufMut, BytesMut};
use std::f32::consts::PI;

const PI2: f32 = 2.0 * PI;

// Incoming packet types (from client)
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum InPacketType {
    Angle(u8),          // 0-250: angle value
    Ping = 251,         // Ping packet
    RotLeft = 108,      // 'l' - rotate left
    RotRight = 114,     // 'r' - rotate right
    StartAcc = 253,     // Start acceleration
    StopAcc = 254,      // Stop acceleration
    UsernameSkin = 115, // 's' - set username and skin
    VictoryMessage = 255,
}

// Outgoing packet types (to client)
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
#[allow(dead_code)]
pub enum OutPacketType {
    Init = b'a',               // Initial setup
    RotCCWWangSp = b'E',       // Rotation variants
    RotCCWAngWang = b'3',
    RotCCWAng = b'e',
    RotCWAngWangSp = b'4',
    RotCWAngWang = b'5',
    SetFullness = b'h',
    RemPart = b'r',
    Move = b'g',
    MoveRel = b'G',
    Inc = b'n',
    IncRel = b'N',
    Leaderboard = b'l',
    End = b'v',
    AddSector = b'W',
    RemSector = b'w',
    Highscore = b'm',
    Pong = b'p',
    Minimap = b'u',
    Snake = b's',
    SetFood = b'F',
    SpawnFood = b'b',
    AddFood = b'f',
    EatFood = b'c',
}

/// Helper functions for encoding values
pub fn write_u8(buf: &mut BytesMut, val: u8) {
    buf.put_u8(val);
}

pub fn write_u16(buf: &mut BytesMut, val: u16) {
    buf.put_u8((val >> 8) as u8);  // MSB first (big-endian)
    buf.put_u8(val as u8);
}

pub fn write_u24(buf: &mut BytesMut, val: u32) {
    buf.put_u8((val >> 16) as u8);
    buf.put_u8((val >> 8) as u8);
    buf.put_u8(val as u8);
}

pub fn write_fp8(buf: &mut BytesMut, val: f32) {
    buf.put_u8((val * 10.0) as u8);
}

pub fn write_fp16_2(buf: &mut BytesMut, val: f32) {
    write_u16(buf, (val * 100.0) as u16);
}

pub fn write_fp16_3(buf: &mut BytesMut, val: f32) {
    write_u16(buf, (val * 1000.0) as u16);
}

pub fn write_ang8(buf: &mut BytesMut, angle: f32) {
    buf.put_u8((256.0 * angle / PI2) as u8);
}

pub fn write_ang24(buf: &mut BytesMut, angle: f32) {
    write_u24(buf, (0xFFFFFF as f32 * angle / PI2) as u32);
}

pub fn write_string(buf: &mut BytesMut, s: &str) {
    buf.put_u8(s.len() as u8);
    buf.put_slice(s.as_bytes());
}

/// Base packet structure: [client_time: u16][packet_type: u8]
pub struct PacketBase {
    pub client_time: u16,
    pub packet_type: OutPacketType,
}

impl PacketBase {
    pub fn new(packet_type: OutPacketType) -> Self {
        Self {
            client_time: 0,
            packet_type,
        }
    }

    pub fn encode(&self, buf: &mut BytesMut) {
        write_u16(buf, self.client_time);
        write_u8(buf, self.packet_type as u8);
    }
}

/// Init packet ('a')
pub struct PacketInit {
    pub base: PacketBase,
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
        use crate::config::WorldConfig;

        Self {
            base: PacketBase::new(OutPacketType::Init),
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

        self.base.encode(&mut buf);
        write_u24(&mut buf, self.game_radius);
        write_u16(&mut buf, self.max_snake_parts);
        write_u16(&mut buf, self.sector_size);
        write_u16(&mut buf, self.sector_count_along_edge);
        write_fp8(&mut buf, self.spangdv);
        write_fp16_2(&mut buf, self.nsp1);
        write_fp16_2(&mut buf, self.nsp2);
        write_fp16_2(&mut buf, self.nsp3);
        write_fp16_3(&mut buf, self.snake_ang_speed);
        write_fp16_3(&mut buf, self.prey_ang_speed);
        write_fp16_3(&mut buf, self.snake_tail_k);
        write_u8(&mut buf, self.protocol_version);

        buf.to_vec()
    }
}

/// Pong packet ('p')
pub struct PacketPong {
    pub base: PacketBase,
}

impl PacketPong {
    pub fn new() -> Self {
        Self {
            base: PacketBase::new(OutPacketType::Pong),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(3);
        self.base.encode(&mut buf);
        buf.to_vec()
    }
}

/// Move packet ('g')
pub struct PacketMove {
    pub base: PacketBase,
    pub snake_id: u16,
    pub x: u16,
    pub y: u16,
}

impl PacketMove {
    pub fn new(snake_id: u16, x: f32, y: f32) -> Self {
        Self {
            base: PacketBase::new(OutPacketType::Move),
            snake_id,
            x: x as u16,
            y: y as u16,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(9);
        self.base.encode(&mut buf);
        write_u16(&mut buf, self.snake_id);
        write_u16(&mut buf, self.x);
        write_u16(&mut buf, self.y);
        buf.to_vec()
    }
}

/// Add snake packet ('s')
pub struct PacketAddSnake {
    pub base: PacketBase,
    pub snake_id: u16,
    pub angle: f32,
    pub speed: f32,
    pub fullness: f32,
    pub skin: u8,
    pub head_x: f32,
    pub head_y: f32,
    pub name: String,
    pub parts: Vec<(f32, f32)>, // Body parts (x, y)
}

impl PacketAddSnake {
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(256);

        self.base.encode(&mut buf);
        write_u16(&mut buf, self.snake_id);
        write_ang24(&mut buf, self.angle); // ehang
        write_u8(&mut buf, 0); // unused byte
        write_ang24(&mut buf, self.angle); // eangle
        write_fp16_3(&mut buf, self.speed / 32.0);
        write_u24(&mut buf, (self.fullness / 100.0 * 0xFFFFFF as f32) as u32);
        write_u8(&mut buf, self.skin);
        write_u24(&mut buf, (self.head_x * 5.0) as u32);
        write_u24(&mut buf, (self.head_y * 5.0) as u32);
        write_string(&mut buf, &self.name);

        // Encode body parts (from tail to head, excluding head)
        if !self.parts.is_empty() {
            let tail = self.parts.last().unwrap();
            write_u24(&mut buf, (tail.0 * 5.0) as u32);
            write_u24(&mut buf, (tail.1 * 5.0) as u32);

            let mut hx = tail.0;
            let mut hy = tail.1;

            for i in (0..self.parts.len() - 1).rev() {
                let bpx = self.parts[i].0 - hx;
                let bpy = self.parts[i].1 - hy;

                write_u8(&mut buf, (bpx * 2.0 + 127.0) as u8);
                write_u8(&mut buf, (bpy * 2.0 + 127.0) as u8);

                hx += bpx;
                hy += bpy;
            }
        }

        buf.to_vec()
    }
}

/// Remove snake packet ('s' with status)
pub struct PacketRemoveSnake {
    pub base: PacketBase,
    pub snake_id: u16,
    pub status: u8, // 0 = left range, 1 = died
}

impl PacketRemoveSnake {
    pub fn new(snake_id: u16, died: bool) -> Self {
        Self {
            base: PacketBase::new(OutPacketType::Snake),
            snake_id,
            status: if died { 1 } else { 0 },
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(6);
        self.base.encode(&mut buf);
        write_u16(&mut buf, self.snake_id);
        write_u8(&mut buf, self.status);
        buf.to_vec()
    }
}

/// Rotation packet (multiple variants: 'E', '3', 'e', '4', '5')
pub struct PacketRotation {
    pub base: PacketBase,
    pub snake_id: u16,
    pub ang: Option<f32>,
    pub wang: Option<f32>,
    pub speed: Option<f32>,
}

impl PacketRotation {
    pub fn new(snake_id: u16) -> Self {
        Self {
            base: PacketBase::new(OutPacketType::RotCCWAng), // Will be determined later
            snake_id,
            ang: None,
            wang: None,
            speed: None,
        }
    }

    fn is_clockwise(&self) -> bool {
        if let (Some(a), Some(w)) = (self.ang, self.wang) {
            let mut d_angle = w - a;
            while d_angle < 0.0 { d_angle += PI2; }
            while d_angle >= PI2 { d_angle -= PI2; }

            if d_angle > PI {
                d_angle -= PI2;
            }
            d_angle > 0.0
        } else {
            false
        }
    }

    fn determine_packet_type(&self) -> OutPacketType {
        match (self.wang, self.ang, self.speed) {
            (None, Some(_), None) => OutPacketType::RotCCWAng,
            (None, None, Some(_)) => OutPacketType::RotCCWAngWang,
            (None, Some(_), Some(_)) => OutPacketType::RotCCWAng,
            (Some(_), None, None) => OutPacketType::RotCCWWangSp,
            (Some(_), None, Some(_)) => OutPacketType::RotCCWWangSp,
            (Some(_), Some(_), None) => {
                if self.is_clockwise() {
                    OutPacketType::RotCWAngWang
                } else {
                    OutPacketType::RotCCWAngWang
                }
            }
            (Some(_), Some(_), Some(_)) => {
                if self.is_clockwise() {
                    OutPacketType::RotCWAngWangSp
                } else {
                    OutPacketType::RotCCWAng
                }
            }
            _ => OutPacketType::RotCCWAng,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let packet_type = self.determine_packet_type();
        let mut buf = BytesMut::with_capacity(8);

        let mut base = PacketBase::new(packet_type);
        base.client_time = self.base.client_time;
        base.encode(&mut buf);

        write_u16(&mut buf, self.snake_id);

        if let Some(a) = self.ang {
            write_ang8(&mut buf, a);
        }

        if let Some(w) = self.wang {
            write_ang8(&mut buf, w);
        }

        if let Some(s) = self.speed {
            write_u8(&mut buf, (s * 18.0) as u8);
        }

        buf.to_vec()
    }
}

/// End/death packet ('v')
#[allow(dead_code)]
pub struct PacketEnd {
    pub base: PacketBase,
    pub status: u8, // 0 = normal, 1 = death
}

#[allow(dead_code)]
impl PacketEnd {
    pub fn death() -> Self {
        Self {
            base: PacketBase::new(OutPacketType::End),
            status: 1,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(4);
        self.base.encode(&mut buf);
        write_u8(&mut buf, self.status);
        buf.to_vec()
    }
}

/// Parse incoming packet type from byte
pub fn parse_in_packet_type(byte: u8) -> InPacketType {
    match byte {
        108 => InPacketType::RotLeft,
        114 => InPacketType::RotRight,
        115 => InPacketType::UsernameSkin,
        251 => InPacketType::Ping,
        253 => InPacketType::StartAcc,
        254 => InPacketType::StopAcc,
        255 => InPacketType::VictoryMessage,
        0..=250 => InPacketType::Angle(byte),
        _ => InPacketType::Angle(0), // Default
    }
}

/// Convert angle byte to radians
pub fn angle_byte_to_radians(byte: u8) -> f32 {
    PI * byte as f32 / 125.0
}
