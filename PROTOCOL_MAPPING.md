# Protocol Mapping: C++ to Rust

This document details the exact correspondence between the C++ and Rust implementations of the Slither.io protocol to ensure 100% compatibility.

## Packet Structure

### Base Packet Format

Both C++ and Rust follow the same base structure:

```
[client_time: 2 bytes (uint16, big-endian)]
[packet_type: 1 byte (uint8)]
[payload data...]
```

**C++ Implementation:**
```cpp
struct PacketBase {
  uint16_t client_time;  // Time since last message
  out_packet_t packet_type;
};

std::ostream& operator<<(std::ostream& out, const PacketBase& p) {
  return out << write_uint16(p.client_time) << write_uint8(p.packet_type);
}
```

**Rust Implementation:**
```rust
pub struct PacketBase {
    pub client_time: u16,
    pub packet_type: OutPacketType,
}

impl PacketBase {
    pub fn encode(&self, buf: &mut BytesMut) {
        write_u16(buf, self.client_time);  // Big-endian
        write_u8(buf, self.packet_type as u8);
    }
}
```

## Encoding Functions

### Integer Encoding (Big-Endian)

| Type | C++ | Rust | Format |
|------|-----|------|--------|
| uint8 | `write_uint8(v)` | `write_u8(buf, v)` | 1 byte |
| uint16 | `write_uint16(v)` | `write_u16(buf, v)` | MSB, LSB |
| uint24 | `write_uint24(v)` | `write_u24(buf, v)` | MSB, mid, LSB |

**C++ uint16:**
```cpp
std::ostream& operator<<(std::ostream& __os, ostream_write_value<uint16_t> __f) {
  return __os
      .put(static_cast<char>(__f.v >> 8))   // MSB
      .put(static_cast<char>(__f.v));       // LSB
}
```

**Rust uint16:**
```rust
pub fn write_u16(buf: &mut BytesMut, val: u16) {
    buf.put_u8((val >> 8) as u8);  // MSB first
    buf.put_u8(val as u8);          // LSB
}
```

### Fixed-Point Encoding

| Type | C++ | Rust | Multiplier |
|------|-----|------|------------|
| fp8 | `write_fp8(v)` | `write_fp8(buf, v)` | × 10 |
| fp16<2> | `write_fp16<2>(v)` | `write_fp16_2(buf, v)` | × 100 |
| fp16<3> | `write_fp16<3>(v)` | `write_fp16_3(buf, v)` | × 1000 |
| fp24 | `write_fp24(v)` | Not used in init | × 0xFFFFFF |

**C++ fp16<3>:**
```cpp
template <>
ostream_write_value<uint16_t> write_fp16<3>(fixed_point_t v) {
  return {(uint16_t)(1000.0f * v)};
}
```

**Rust fp16_3:**
```rust
pub fn write_fp16_3(buf: &mut BytesMut, val: f32) {
    write_u16(buf, (val * 1000.0) as u16);
}
```

### Angle Encoding

Angles are encoded in radians to discrete values:

| Type | C++ | Rust | Formula |
|------|-----|------|---------|
| ang8 | `write_ang8(v)` | `write_ang8(buf, v)` | `angle * 256 / (2*PI)` |
| ang24 | `write_ang24(v)` | `write_ang24(buf, v)` | `angle * 0xFFFFFF / (2*PI)` |

**C++ ang8:**
```cpp
inline ostream_write_value<uint8_t> write_ang8(fixed_point_t v) {
  return {(uint8_t)(256 * v / M_2PI)};
}
```

**Rust ang8:**
```rust
pub fn write_ang8(buf: &mut BytesMut, angle: f32) {
    buf.put_u8((256.0 * angle / PI2) as u8);
}
```

### String Encoding

Format: `[length: 1 byte][string bytes...]`

**C++:**
```cpp
std::ostream& operator<<(std::ostream& __os, ostream_write_value<const std::string&> __f) {
  __os.put(static_cast<char>(__f.v.length()));
  for (const char c : __f.v) {
    __os.put(c);
  }
  return __os;
}
```

**Rust:**
```rust
pub fn write_string(buf: &mut BytesMut, s: &str) {
    buf.put_u8(s.len() as u8);
    buf.put_slice(s.as_bytes());
}
```

## Packet Types

### Init Packet ('a' = 0x61)

**Format (26 bytes total):**
```
[0-1]   uint16  client_time
[2]     uint8   packet_type ('a')
[3-5]   uint24  game_radius
[6-7]   uint16  max_snake_parts
[8-9]   uint16  sector_size
[10-11] uint16  sector_count_along_edge
[12]    fp8     spangdv
[13-14] fp16<2> nsp1
[15-16] fp16<2> nsp2
[17-18] fp16<2> nsp3
[19-20] fp16<3> snake_ang_speed
[21-22] fp16<3> prey_ang_speed
[23-24] fp16<3> snake_tail_k
[25]    uint8   protocol_version
```

**C++ Implementation:**
```cpp
std::ostream& operator<<(std::ostream& out, const PacketInit& p) {
  return out << static_cast<PacketBase>(p)
             << write_uint24(p.game_radius)
             << write_uint16(p.max_snake_parts)
             << write_uint16(p.sector_size)
             << write_uint16(p.sector_count_along_edge)
             << write_fp8(p.spangdv)
             << write_fp16<2>(p.nsp1)
             << write_fp16<2>(p.nsp2)
             << write_fp16<2>(p.nsp3)
             << write_fp16<3>(p.snake_ang_speed)
             << write_fp16<3>(p.prey_ang_speed)
             << write_fp16<3>(p.snake_tail_k)
             << write_uint8(p.protocol_version);
}
```

**Rust Implementation:**
```rust
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
```

### Move Packet ('g' = 0x67)

**Format (9 bytes):**
```
[0-1]   uint16  client_time
[2]     uint8   packet_type ('g')
[3-4]   uint16  snake_id
[5-6]   uint16  x
[7-8]   uint16  y
```

**C++:**
```cpp
std::ostream& operator<<(std::ostream& out, const packet_move& p) {
  out << static_cast<PacketBase>(p);
  out << write_uint16(p.snakeId);
  out << write_uint16(p.x);
  out << write_uint16(p.y);
  return out;
}
```

**Rust:**
```rust
pub fn encode(&self) -> Vec<u8> {
    let mut buf = BytesMut::with_capacity(9);
    self.base.encode(&mut buf);
    write_u16(&mut buf, self.snake_id);
    write_u16(&mut buf, self.x);
    write_u16(&mut buf, self.y);
    buf.to_vec()
}
```

### Add Snake Packet ('s' = 0x73)

**Format (variable length):**
```
[0-1]    uint16  client_time
[2]      uint8   packet_type ('s')
[3-4]    uint16  snake_id
[5-7]    ang24   angle (ehang)
[8]      uint8   unused (0)
[9-11]   ang24   angle (eangle)
[12-13]  fp16<3> speed / 32
[14-16]  uint24  fullness (fp24)
[17]     uint8   skin
[18-20]  uint24  head_x * 5
[21-23]  uint24  head_y * 5
[24]     uint8   name_length
[25+len] string  name
[...]    uint24  tail_x * 5
[...]    uint24  tail_y * 5
[...]    uint8[] body parts (dx*2+127, dy*2+127)
```

**C++:**
```cpp
out << write_uint16(s->id)
    << write_ang24(s->angle)
    << write_uint8(0)
    << write_ang24(s->angle)
    << write_fp16<3>(s->speed / 32.0f)
    << write_fp24(s->fullness / 100.0f)
    << write_uint8(s->skin)
    << write_uint24(s->get_head_x() * 5.0f)
    << write_uint24(s->get_head_y() * 5.0f)
    << write_string(s->name);
```

**Rust:**
```rust
write_u16(&mut buf, self.snake_id);
write_ang24(&mut buf, self.angle);
write_u8(&mut buf, 0);
write_ang24(&mut buf, self.angle);
write_fp16_3(&mut buf, self.speed / 32.0);
write_u24(&mut buf, (self.fullness / 100.0 * 0xFFFFFF as f32) as u32);
write_u8(&mut buf, self.skin);
write_u24(&mut buf, (self.head_x * 5.0) as u32);
write_u24(&mut buf, (self.head_y * 5.0) as u32);
write_string(&mut buf, &self.name);
```

### Rotation Packet (Multiple Types)

The rotation packet has multiple subtypes based on which fields are present:

| Packet Type | C++ Char | Rust Enum | Fields |
|-------------|----------|-----------|--------|
| RotCCWAng | 'e' | RotCCWAng | ang |
| RotCCWAngWang | '3' | RotCCWAngWang | ang, wang |
| RotCCWWangSp | 'E' | RotCCWWangSp | wang, speed |
| RotCWAngWang | '5' | RotCWAngWang | ang, wang (CW) |
| RotCWAngWangSp | '4' | RotCWAngWangSp | ang, wang, speed (CW) |

**Speed encoding:** `speed * 18` (not `speed / 32` like in add_snake)

## Incoming Packets (Client → Server)

### Angle Packet (0-250)

Single byte representing angle.

**Decoding:**
```cpp
const float angle = Math::f_pi * packet_type / 125.0f;
```

```rust
pub fn angle_byte_to_radians(byte: u8) -> f32 {
    PI * byte as f32 / 125.0
}
```

### Control Packets

| Packet Type | Value | C++ | Rust |
|-------------|-------|-----|------|
| Ping | 251 | `in_packet_t_ping` | `InPacketType::Ping` |
| Start Acceleration | 253 | `in_packet_t_start_acc` | `InPacketType::StartAcc` |
| Stop Acceleration | 254 | `in_packet_t_stop_acc` | `InPacketType::StopAcc` |
| Username/Skin | 115 ('s') | `in_packet_t_username_skin` | `InPacketType::UsernameSkin` |

### Username/Skin Packet Format

```
[0]    uint8  packet_type (115 = 's')
[1]    uint8  protocol_version
[2]    uint8  skin
[3+]   string username
```

**C++:**
```cpp
case in_packet_t_username_skin:
  buf >> ss.protocol_version;
  buf >> ss.skin;
  buf.str(ss.name);
  break;
```

**Rust:**
```rust
InPacketType::UsernameSkin => {
    let protocol_version = data[1];
    let skin = data[2];
    let name = String::from_utf8_lossy(&data[3..]).to_string();
}
```

## Constants Match

All game constants are identical:

| Constant | C++ | Rust |
|----------|-----|------|
| GAME_RADIUS | 21600 | 21600 |
| MAX_SNAKE_PARTS | 411 | 411 |
| SECTOR_SIZE | 300 | 300 |
| SECTOR_COUNT_ALONG_EDGE | 144 | 144 |
| FRAME_TIME_MS | 8 | 8 |
| PROTOCOL_VERSION | 8 | 8 |
| SNAKE_ANGULAR_SPEED | 4.125 rad/s | 4.125 rad/s |
| BASE_MOVE_SPEED | 185 px/s | 185 px/s |
| BOOST_SPEED | 448 px/s | 448 px/s |

## Testing Checklist

- [x] Init packet has correct byte order (client_time, then packet_type)
- [x] All integers use big-endian encoding
- [x] Fixed-point values use correct multipliers
- [x] Angle encoding uses correct formulas
- [x] String encoding includes length prefix
- [x] Incoming angle packets decode correctly (PI * byte / 125)
- [x] Acceleration packets (253/254) handled correctly
- [x] Username/skin packet parsed correctly
- [x] All packet types use correct ASCII character codes

## Binary Format Verification

To verify packet correctness, compare hex dumps:

**Init packet should start with:**
```
00 00 61 00 54 60 01 9B 01 2C 00 90 30 02 1B ...
      ^^-- 'a' packet type
^^^^^----- client_time (00 00)
```

**Move packet example:**
```
00 00 67 00 01 1A 2C 3E 4F
      ^^-- 'g' packet type
          ^^^^^-- snake_id
                ^^^^^^^^^^-- x, y coordinates
```

This ensures that clients can connect to the Rust server without any protocol incompatibilities.
