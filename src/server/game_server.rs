use crate::config::{Config, SnakeId};
use crate::game::World;
use crate::packet::PacketInit;
use futures_util::{SinkExt, StreamExt};
use log::{info, warn};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
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
}

pub struct GameServer {
    world: Arc<RwLock<World>>,
    sessions: Arc<Mutex<HashMap<SocketAddr, Session>>>,
    config: Config,
}

impl GameServer {
    pub fn new(config: Config) -> Self {
        Self {
            world: Arc::new(RwLock::new(World::new())),
            sessions: Arc::new(Mutex::new(HashMap::new())),
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
        tokio::spawn(async move {
            Self::game_loop(world_clone).await;
        });

        // Start WebSocket server
        let addr = format!("127.0.0.1:{}", self.config.port);
        let listener = TcpListener::bind(&addr).await?;
        info!("Server listening on {}", addr);

        while let Ok((stream, peer_addr)) = listener.accept().await {
            let world = Arc::clone(&self.world);
            let sessions = Arc::clone(&self.sessions);

            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(stream, peer_addr, world, sessions).await {
                    warn!("Error handling connection: {}", e);
                }
            });
        }

        Ok(())
    }

    async fn game_loop(world: Arc<RwLock<World>>) {
        let mut tick_timer = interval(Duration::from_millis(10));

        loop {
            tick_timer.tick().await;

            let mut world = world.write().await;
            world.tick(10); // 10ms tick
        }
    }

    async fn handle_connection(
        stream: TcpStream,
        peer_addr: SocketAddr,
        world: Arc<RwLock<World>>,
        sessions: Arc<Mutex<HashMap<SocketAddr, Session>>>,
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
                },
            );
        }

        info!("Created snake {} for {}", snake_id, peer_addr);

        // Handle messages
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Binary(data)) => {
                    Self::handle_packet(&data, snake_id, &world).await;
                }
                Ok(Message::Close(_)) => {
                    info!("Client {} disconnected", peer_addr);
                    break;
                }
                Err(e) => {
                    warn!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        // Cleanup
        {
            let mut sessions = sessions.lock().await;
            sessions.remove(&peer_addr);
        }

        {
            let mut world = world.write().await;
            world.remove_snake(snake_id);
        }

        Ok(())
    }

    async fn handle_packet(data: &[u8], snake_id: SnakeId, world: &Arc<RwLock<World>>) {
        if data.is_empty() {
            return;
        }

        match data[0] {
            b'e' => {
                // Movement packet - set angle
                if data.len() >= 3 {
                    let angle_raw = ((data[1] as u16) << 8) | (data[2] as u16);
                    let angle = angle_raw as f32 * 2.0 * std::f32::consts::PI / 65535.0;

                    let mut world = world.write().await;
                    if let Some(snake) = world.get_snake_mut(snake_id) {
                        snake.wangle = angle;
                    }
                }
            }
            b's' => {
                // Speed boost
                let mut world = world.write().await;
                if let Some(snake) = world.get_snake_mut(snake_id) {
                    snake.acceleration = data.len() > 1 && data[1] == 1;
                }
            }
            _ => {}
        }
    }
}
