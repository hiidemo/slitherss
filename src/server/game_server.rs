use crate::config::{Config, SnakeId};
use crate::game::World;
use crate::packet::*;
use futures_util::{SinkExt, StreamExt};
use log::{info, warn};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, RwLock};
use tokio::time::{interval, Duration};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

type Tx = futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<TcpStream>,
    Message,
>;

#[derive(Clone)]
pub struct Session {
    pub snake_id: SnakeId,
    pub name: String,
    pub protocol_version: u8,
    pub skin: u8,
    pub last_packet_time: Instant,
}

pub struct GameServer {
    world: Arc<RwLock<World>>,
    sessions: Arc<Mutex<HashMap<SocketAddr, Session>>>,
    senders: Arc<Mutex<HashMap<SocketAddr, Tx>>>,
    config: Config,
}

impl GameServer {
    pub fn new(config: Config) -> Self {
        Self {
            world: Arc::new(RwLock::new(World::new())),
            sessions: Arc::new(Mutex::new(HashMap::new())),
            senders: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize world
        {
            let mut world = self.world.write().await;
            world.init(self.config.world_config());

            // Spawn bots
            if self.config.bots > 0 {
                info!("Spawning {} bots", self.config.bots);
                world.spawn_num_snakes(self.config.bots);
            }
        }

        // Start game loop
        let world_clone = Arc::clone(&self.world);
        let sessions_clone = Arc::clone(&self.sessions);
        let senders_clone = Arc::clone(&self.senders);

        tokio::spawn(async move {
            Self::game_loop(world_clone, sessions_clone, senders_clone).await;
        });

        // Start WebSocket server
        let addr = format!("127.0.0.1:{}", self.config.port);
        let listener = TcpListener::bind(&addr).await?;
        info!("Server listening on {}", addr);

        while let Ok((stream, peer_addr)) = listener.accept().await {
            let world = Arc::clone(&self.world);
            let sessions = Arc::clone(&self.sessions);
            let senders = Arc::clone(&self.senders);

            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(stream, peer_addr, world, sessions, senders).await {
                    warn!("Error handling connection from {}: {}", peer_addr, e);
                }
            });
        }

        Ok(())
    }

    async fn game_loop(
        world: Arc<RwLock<World>>,
        sessions: Arc<Mutex<HashMap<SocketAddr, Session>>>,
        senders: Arc<Mutex<HashMap<SocketAddr, Tx>>>,
    ) {
        let mut tick_timer = interval(Duration::from_millis(10));

        loop {
            tick_timer.tick().await;

            // Tick the world
            {
                let mut world = world.write().await;
                world.tick(10); // 10ms tick
            }

            // Broadcast updates
            Self::broadcast_updates(&world, &sessions, &senders).await;
        }
    }

    async fn broadcast_updates(
        world: &Arc<RwLock<World>>,
        sessions: &Arc<Mutex<HashMap<SocketAddr, Session>>>,
        senders: &Arc<Mutex<HashMap<SocketAddr, Tx>>>,
    ) {
        let world = world.read().await;
        let changed_snakes = world.get_changed_snakes();

        if changed_snakes.is_empty() {
            return;
        }

        let mut senders = senders.lock().await;

        for &snake_id in changed_snakes {
            if let Some(snake) = world.get_snake(snake_id) {
                // Broadcast move packet
                if !snake.parts.is_empty() {
                    let move_packet = PacketMove::new(
                        snake.id,
                        snake.parts[0].x,
                        snake.parts[0].y,
                    );
                    let data = move_packet.encode();

                    // Send to all clients
                    for (_, tx) in senders.iter_mut() {
                        let _ = tx.send(Message::Binary(data.clone())).await;
                    }
                }

                // Send rotation packet if angle/speed changed
                if snake.update & (1 << 1 | 1 << 3) != 0 {
                    let mut rot_packet = PacketRotation::new(snake.id);
                    rot_packet.ang = Some(snake.angle);
                    rot_packet.wang = Some(snake.wangle);
                    rot_packet.speed = Some(snake.speed as f32);

                    let data = rot_packet.encode();
                    for (_, tx) in senders.iter_mut() {
                        let _ = tx.send(Message::Binary(data.clone())).await;
                    }
                }
            }
        }
    }

    async fn handle_connection(
        stream: TcpStream,
        peer_addr: SocketAddr,
        world: Arc<RwLock<World>>,
        sessions: Arc<Mutex<HashMap<SocketAddr, Session>>>,
        senders: Arc<Mutex<HashMap<SocketAddr, Tx>>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("New connection from {}", peer_addr);

        let ws_stream = accept_async(stream).await?;
        let (mut write, mut read) = ws_stream.split();

        // Send init packet
        let init_packet = PacketInit::default();
        let init_data = init_packet.encode();
        write.send(Message::Binary(init_data)).await?;

        // Create snake for this session
        let snake_id = {
            let mut world = world.write().await;
            world.create_snake()
        };

        // Create session
        {
            let mut sessions = sessions.lock().await;
            sessions.insert(
                peer_addr,
                Session {
                    snake_id,
                    name: String::from("Player"),
                    protocol_version: 8,
                    skin: 0,
                    last_packet_time: Instant::now(),
                },
            );
        }

        // Send add snake packet to all clients
        {
            let world = world.read().await;
            if let Some(snake) = world.get_snake(snake_id) {
                let mut parts = Vec::new();
                for part in &snake.parts {
                    parts.push((part.x, part.y));
                }

                let add_snake_packet = PacketAddSnake {
                    base: PacketBase::new(OutPacketType::Snake),
                    snake_id: snake.id,
                    angle: snake.angle,
                    speed: snake.speed as f32,
                    fullness: snake.fullness as f32,
                    skin: snake.skin,
                    head_x: if !snake.parts.is_empty() { snake.parts[0].x } else { 0.0 },
                    head_y: if !snake.parts.is_empty() { snake.parts[0].y } else { 0.0 },
                    name: snake.name.clone(),
                    parts,
                };

                let data = add_snake_packet.encode();

                // Broadcast to all clients
                let mut senders_lock = senders.lock().await;
                for (_, tx) in senders_lock.iter_mut() {
                    let _ = tx.send(Message::Binary(data.clone())).await;
                }
            }

            // Send existing snakes to new client
            for (id, snake) in world.get_snakes() {
                if *id != snake_id {
                    let mut parts = Vec::new();
                    for part in &snake.parts {
                        parts.push((part.x, part.y));
                    }

                    let add_snake_packet = PacketAddSnake {
                        base: PacketBase::new(OutPacketType::Snake),
                        snake_id: snake.id,
                        angle: snake.angle,
                        speed: snake.speed as f32,
                        fullness: snake.fullness as f32,
                        skin: snake.skin,
                        head_x: if !snake.parts.is_empty() { snake.parts[0].x } else { 0.0 },
                        head_y: if !snake.parts.is_empty() { snake.parts[0].y } else { 0.0 },
                        name: snake.name.clone(),
                        parts,
                    };

                    let data = add_snake_packet.encode();
                    write.send(Message::Binary(data)).await?;
                }
            }
        }

        // Store sender
        {
            let mut senders_lock = senders.lock().await;
            senders_lock.insert(peer_addr, write);
        }

        info!("Created snake {} for {}", snake_id, peer_addr);

        // Handle messages
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Binary(data)) => {
                    Self::handle_packet(&data, snake_id, peer_addr, &world, &sessions).await;
                }
                Ok(Message::Close(_)) => {
                    info!("Client {} disconnected", peer_addr);
                    break;
                }
                Err(e) => {
                    warn!("WebSocket error from {}: {}", peer_addr, e);
                    break;
                }
                _ => {}
            }
        }

        // Cleanup
        {
            let mut sessions_lock = sessions.lock().await;
            sessions_lock.remove(&peer_addr);
        }

        {
            let mut senders_lock = senders.lock().await;
            senders_lock.remove(&peer_addr);
        }

        {
            let mut world = world.write().await;
            world.remove_snake(snake_id);
        }

        // Broadcast snake removal
        {
            let remove_packet = PacketRemoveSnake::new(snake_id, true);
            let data = remove_packet.encode();

            let mut senders_lock = senders.lock().await;
            for (_, tx) in senders_lock.iter_mut() {
                let _ = tx.send(Message::Binary(data.clone())).await;
            }
        }

        Ok(())
    }

    async fn handle_packet(
        data: &[u8],
        snake_id: SnakeId,
        peer_addr: SocketAddr,
        world: &Arc<RwLock<World>>,
        sessions: &Arc<Mutex<HashMap<SocketAddr, Session>>>,
    ) {
        if data.is_empty() {
            return;
        }

        let packet_type = parse_in_packet_type(data[0]);

        match packet_type {
            InPacketType::Angle(angle_byte) => {
                // Movement packet - set angle
                let angle = angle_byte_to_radians(angle_byte);

                let mut world = world.write().await;
                if let Some(snake) = world.get_snake_mut(snake_id) {
                    snake.wangle = angle;
                }
            }
            InPacketType::Ping => {
                // Respond with pong
                info!("Ping from {}", peer_addr);
                // Note: We would need to get sender here to reply
            }
            InPacketType::StartAcc => {
                let mut world = world.write().await;
                if let Some(snake) = world.get_snake_mut(snake_id) {
                    snake.acceleration = true;
                }
            }
            InPacketType::StopAcc => {
                let mut world = world.write().await;
                if let Some(snake) = world.get_snake_mut(snake_id) {
                    snake.acceleration = false;
                }
            }
            InPacketType::UsernameSkin => {
                // Parse username and skin
                if data.len() >= 3 {
                    let protocol_version = data[1];
                    let skin = data[2];

                    let name = if data.len() > 3 {
                        String::from_utf8_lossy(&data[3..]).to_string()
                    } else {
                        String::from("Player")
                    };

                    let mut sessions_lock = sessions.lock().await;
                    if let Some(session) = sessions_lock.get_mut(&peer_addr) {
                        session.protocol_version = protocol_version;
                        session.skin = skin;
                        session.name = name.clone();
                    }

                    let mut world = world.write().await;
                    if let Some(snake) = world.get_snake_mut(snake_id) {
                        snake.name = name;
                        snake.skin = skin;
                    }
                }
            }
            _ => {
                // Unknown or unhandled packet
            }
        }
    }
}
