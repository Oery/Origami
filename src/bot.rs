use std::time::Duration;

use gami_mc_protocol::packets::login::server::LoginSuccess;
use gami_mc_protocol::packets::play::client::{Chat, ClientCommand, ClientSettings};
use gami_mc_protocol::packets::play::server::UpdateHealth;
use gami_mc_protocol::packets::{self, play, ServerPacket};
use gami_mc_protocol::packets::{Packet, Packets};
use gami_mc_protocol::registry::tcp::States;
use tokio::sync::mpsc;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::entity::Coordinates;
use crate::events::{Context, Dispatchable, EventHandlers, PacketHandler};
use crate::stream::Stream;
use crate::World;

pub struct BotBuilder {
    username: String,
    host: String,
    port: u16,
    events: EventHandlers,
}

impl BotBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_username(mut self, username: impl ToString) -> Self {
        self.username = username.to_string();
        self
    }

    pub fn with_host(mut self, host: impl ToString) -> Self {
        self.host = host.to_string();
        self
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let addr = format!("{}:{}", self.host, self.port);
        let mut stream = TcpStream::connect(addr).await?;

        self.set_protocol(&mut stream).await?;
        self.login(&mut stream).await?;

        let (reader, writer) = stream.into_split();
        let (tx, rx) = mpsc::channel::<Vec<u8>>(100);

        let mut stream = Stream::new(reader, tx);
        let mut packets = vec![];
        let mut uuid = String::new();
        let mut entity_id = None;
        let mut game_mode = None;

        while entity_id.is_none() {
            let mut new_packets = stream.read_packets().await?;

            for packet in &new_packets {
                if let Packets::LoginSuccess(data) = packet {
                    uuid = data.uuid.clone();
                } else if let Packets::JoinGame(data) = packet {
                    entity_id = Some(data.entity_id);
                    game_mode = Some(data.game_mode);
                }
            }

            packets.append(&mut new_packets);
        }

        let mut bot = Bot {
            username: self.username,
            tcp: stream,
            events: self.events,
            world: World::default(),
            uuid,
            entity_id: entity_id.unwrap(),
            game_mode: game_mode.unwrap(),
        };

        bot.tcp.listen(writer, rx);
        bot.handle_packets(packets).await?;

        if let Err(e) = bot.run().await {
            eprintln!("Bot Error: {:?}", e);
        }

        Ok(())
    }

    async fn set_protocol(&self, stream: &mut TcpStream) -> anyhow::Result<()> {
        let packet = packets::handshake::client::SetProtocol {
            protocol_version: 47,
            server_host: self.host.clone(),
            server_port: self.port,
            next_state: States::Login,
        };

        stream.write_all(&packet.serialize(-1)?).await?;

        Ok(())
    }

    async fn login(&self, stream: &mut TcpStream) -> anyhow::Result<()> {
        let packet = packets::login::client::LoginStart {
            username: self.username.to_string(),
        };

        stream.write_all(&packet.serialize(-1)?).await?;

        Ok(())
    }

    pub fn on_tick<T: Fn(&Context<'_, '_, ()>) + 'static>(&mut self, f: T) {
        self.events.tick_handlers.push(Box::new(f))
    }

    pub fn on_packet<T: ServerPacket>(&mut self, f: impl PacketHandler<T>) {
        f.register(&mut self.events);
    }

    pub fn on_chat(&mut self, f: impl PacketHandler<play::server::Chat>) {
        f.register(&mut self.events);
    }
}

impl Default for BotBuilder {
    fn default() -> Self {
        Self {
            username: "minecraft_bot_1".to_string(),
            host: "127.0.0.1".to_string(),
            port: 25565,
            events: EventHandlers::default(),
        }
    }
}

pub struct Bot {
    username: String,
    tcp: Stream,
    pub events: EventHandlers,
    pub world: World,
    uuid: String,
    entity_id: i32,
    game_mode: u8,
}

impl Bot {
    async fn run(&mut self) -> anyhow::Result<()> {
        let mut interval = tokio::time::interval(Duration::from_millis(50));

        loop {
            interval.tick().await;
            self.tick().await?;
        }
    }

    pub(crate) async fn tick(&mut self) -> anyhow::Result<()> {
        let packets = self.tcp.read_packets().await?;
        self.handle_packets(packets).await?;

        self.run_on_tick_events().await?;

        // TODO: Tick Physics / Update Position

        Ok(())
    }

    pub(crate) async fn handle_packets(&mut self, packets: Vec<Packets>) -> anyhow::Result<()> {
        for packet in packets {
            Dispatchable::dispatch_packet_event(&packet, self);

            match packet {
                Packets::LoginSuccess(data) => {
                    self.run_on_connect_events(&data).await?;
                }

                Packets::UpdateHealth(data) => {
                    self.run_on_health_update_events(&data).await?;
                }

                Packets::Entity(data) => {
                    self.world.spawn_entity(data.entity_id);
                }
                Packets::EntityVelocity(data) => {
                    self.world.spawn_entity(data.entity_id);
                }
                Packets::EntityMoveLook(data) => {
                    self.world.spawn_entity(data.entity_id);
                }

                Packets::EntityRelativeMove(data) => {
                    let entity = self.world.get_entity(data.entity_id)?;
                    if let Some(ref mut coords) = entity.coords {
                        coords.x += data.d_x as i32;
                        coords.y += data.d_y as i32;
                        coords.z += data.d_z as i32;
                    }
                }

                Packets::EntityHeadRotation(data) => {
                    let entity = self.world.get_entity(data.entity_id)?;
                    if let Some(ref mut coords) = entity.coords {
                        coords.yaw = data.head_yaw;
                    }
                }

                Packets::EntityTeleport(data) => {
                    let entity = self.world.get_entity(data.entity_id)?;
                    entity.coords = Some(Coordinates {
                        x: data.x,
                        y: data.y,
                        z: data.z,
                        yaw: data.yaw,
                        pitch: data.pitch,
                    });
                }

                _ => {}
            }
        }

        Ok(())
    }

    pub async fn respawn(&mut self) -> anyhow::Result<()> {
        let packet = ClientCommand::respawn().serialize(self.cmp())?;
        self.tcp.tx.send(packet).await?;
        Ok(())
    }

    pub fn chat(&self, message: &str) {
        let Ok(packet) = Chat::new(message).serialize(self.cmp()) else {
            eprintln!("[ERROR] bot.chat() : Failed to serialize chat packet");
            return;
        };

        // TODO: Improve this
        let tx = self.tcp.tx.clone();
        tokio::spawn(async move {
            tx.send(packet).await.unwrap();
        });
    }

    async fn send_settings(&mut self) -> anyhow::Result<()> {
        let packet = ClientSettings::default().serialize(self.cmp())?;
        self.tcp.tx.send(packet).await?;
        Ok(())
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn entity_id(&self) -> i32 {
        self.entity_id
    }

    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    pub fn game_mode(&self) -> u8 {
        self.game_mode
    }

    fn cmp(&self) -> i32 {
        self.tcp.compression_threshold
    }

    // EVENTS HANDLERS
    async fn run_on_connect_events(&mut self, data: &LoginSuccess) -> anyhow::Result<()> {
        self.send_settings().await?;
        // TODO: Add User Event

        Ok(())
    }

    async fn run_on_tick_events(&mut self) -> anyhow::Result<()> {
        self.events.tick_handlers.iter().for_each(|e| {
            e(&Context {
                bot: self,
                payload: &(),
            })
        });

        Ok(())
    }

    async fn run_on_death_events(&mut self) -> anyhow::Result<()> {
        // TODO: Add AutoRespawn flag
        self.respawn().await?;

        Ok(())
    }

    async fn run_on_health_update_events(&mut self, data: &UpdateHealth) -> anyhow::Result<()> {
        if data.health <= 0.0 {
            self.run_on_death_events().await?;
        }

        Ok(())
    }
}
