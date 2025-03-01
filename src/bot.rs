use std::time::Duration;

use gami_mc_protocol::packets::{self, play::*, ServerPacket};
use gami_mc_protocol::packets::{Packet, Packets};
use gami_mc_protocol::registry::tcp::States;
use gami_mc_protocol::registry::EntityKind;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::events::{Context, Dispatchable, EventHandlers, PacketHandler};
use crate::stream::Stream;
use crate::World;

pub struct BotBuilder {
    username: String,
    host: String,
    port: u16,
    events: EventHandlers,
    autoreconnect: bool,
    reconnect_delay: Duration,
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

    pub fn with_autoreconnect(mut self, autoreconnect: bool) -> Self {
        self.autoreconnect = autoreconnect;
        self
    }

    pub fn with_reconnect_delay(mut self, delay: Duration) -> Self {
        self.reconnect_delay = delay;
        self
    }

    pub async fn run(self) -> anyhow::Result<()> {
        loop {
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
                username: &self.username,
                tcp: stream,
                events: &self.events,
                world: World::default(),
                uuid,
                entity_id: entity_id.unwrap(),
                game_mode: game_mode.unwrap(),
            };

            bot.tcp.listen(writer, rx);
            bot.handle_packets(packets).await?;

            if let Err(e) = bot.run().await {
                eprintln!("Bot Error: {:?}", e);

                if self.autoreconnect {
                    sleep(self.reconnect_delay).await;
                    continue;
                }
            }

            return Ok(());
        }
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

    pub fn on_connect<T: Fn(&Context<'_, '_, ()>) + 'static>(&mut self, f: T) {
        self.events.on_connect_handlers.push(Box::new(f))
    }

    pub fn on_disconnect(&mut self, f: impl PacketHandler<server::KickDisconnect>) {
        f.register(&mut self.events);
    }

    pub fn on_chat(&mut self, f: impl PacketHandler<server::Chat>) {
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
            autoreconnect: true,
            reconnect_delay: Duration::from_secs(5),
        }
    }
}

pub struct Bot<'a> {
    username: &'a String,
    tcp: Stream,
    pub events: &'a EventHandlers,
    world: World,
    uuid: String,
    entity_id: i32,
    game_mode: u8,
}

impl Bot<'_> {
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
                Packets::LoginSuccess(_) => {
                    self.run_on_connect_events().await?;
                }

                Packets::UpdateHealth(data) => {
                    self.run_on_health_update_events(&data).await?;
                }

                // Packets::Entity(data) => {
                //     self.world.spawn_entity(data.entity_id);
                // }
                // Packets::EntityVelocity(data) => {
                //     self.world.spawn_entity(data.entity_id);
                // }
                // Packets::EntityMoveLook(data) => {
                //     self.world.spawn_entity(data.entity_id);
                // }

                // Packets::EntityRelativeMove(data) => {
                //     let entity = self.world.get_entity(data.entity_id)?;
                //     if let Some(ref mut coords) = entity.coords {
                //         coords.x += data.d_x as i32;
                //         coords.y += data.d_y as i32;
                //         coords.z += data.d_z as i32;
                //     }
                // }

                // Packets::EntityHeadRotation(data) => {
                //     let entity = self.world.get_entity(data.entity_id)?;
                //     if let Some(ref mut coords) = entity.coords {
                //         coords.yaw = data.head_yaw;
                //     }
                // }
                // Packets::EntityTeleport(data) => {
                //     if let Some(entity) = self.world.get_entity(data.entity_id) {

                //     }
                //     entity.coords = Some(Coordinates {
                //         x: data.x,
                //         y: data.y,
                //         z: data.z,
                //         yaw: data.yaw,
                //         pitch: data.pitch,
                //     });
                // }
                Packets::SpawnObject(object) => {
                    let entity = EntityKind::from(&object);
                    self.world.entities.insert(entity.id(), entity);
                }

                Packets::SpawnMob(mob) => {
                    let entity = EntityKind::from(&mob);
                    self.world.entities.insert(entity.id(), entity);
                }

                Packets::EntityMetadata(data) => {
                    if let Some(entity) = self.world.entities.get_mut(&data.entity_id) {
                        entity.update(&data.metadatas);
                    }
                }

                Packets::KickDisconnect(data) => {
                    return Err(anyhow::anyhow!(
                        "Disconnected from server: {:?}",
                        data.reason
                    ));
                }

                _ => {}
            }
        }

        Ok(())
    }

    pub async fn respawn(&mut self) -> anyhow::Result<()> {
        let packet = packets::play::client::ClientCommand::respawn().serialize(self.cmp())?;
        self.tcp.tx.send(packet).await?;
        Ok(())
    }

    pub fn chat(&self, message: &str) {
        let Ok(packet) = client::Chat::new(message).serialize(self.cmp()) else {
            eprintln!("[ERROR] bot.chat() : Failed to serialize chat packet");
            return;
        };

        // TODO: Improve this
        let tx = self.tcp.tx.clone();
        tokio::spawn(async move {
            tx.send(packet).await.unwrap();
        });
    }

    pub fn attack_entity(&self, id: i32) -> anyhow::Result<()> {
        if id == self.entity_id {
            return Err(anyhow::anyhow!("Cannot attack self"));
        }

        let Some(entity) = self.world.entities.get(&id) else {
            return Err(anyhow::anyhow!("Entity not found"));
        };

        match entity {
            EntityKind::ItemStack(_) => return Err(anyhow::anyhow!("Cannot attack entity")),
            EntityKind::ExperienceOrb(_) => return Err(anyhow::anyhow!("Cannot attack entity")),
            _ => {}
        }

        let Ok(packet) = client::UseEntity::attack(entity.id()).serialize(self.cmp()) else {
            return Err(anyhow::anyhow!("Failed to serialize packet"));
        };

        // TODO: Improve this
        let tx = self.tcp.tx.clone();
        tokio::spawn(async move {
            tx.send(packet).await.unwrap();
        });

        Ok(())
    }

    async fn send_settings(&mut self) -> anyhow::Result<()> {
        let packet = client::ClientSettings::default().serialize(self.cmp())?;
        self.tcp.tx.send(packet).await?;
        Ok(())
    }

    pub fn username(&self) -> &str {
        self.username
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

    pub fn world(&self) -> &World {
        &self.world
    }

    fn cmp(&self) -> i32 {
        self.tcp.compression_threshold
    }

    // EVENTS HANDLERS
    async fn run_on_connect_events(&mut self) -> anyhow::Result<()> {
        self.send_settings().await?;

        self.events.on_connect_handlers.iter().for_each(|e| {
            e(&Context {
                bot: self,
                payload: &(),
            })
        });

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

    async fn run_on_health_update_events(
        &mut self,
        data: &server::UpdateHealth,
    ) -> anyhow::Result<()> {
        if data.health <= 0.0 {
            self.run_on_death_events().await?;
        }

        Ok(())
    }
}
