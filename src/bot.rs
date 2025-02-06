use std::time::Duration;

use gami_mc_protocol::packets::login::server::LoginSuccess;
use gami_mc_protocol::packets::play::client::{Chat, ClientCommand, ClientSettings};
use gami_mc_protocol::packets::play::server::UpdateHealth;
use gami_mc_protocol::packets::{self, play, ServerPacket};
use gami_mc_protocol::packets::{Packet, Packets};
use gami_mc_protocol::registry::tcp::States;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::events::{Dispatchable, EventHandlers, PacketHandler};
use crate::stream::Stream;

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

        let mut bot = Bot {
            username: self.username,
            tcp: Stream::new(stream),
            events: self.events,
        };

        // loop {
        //     tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        //     bot.tcp.read_packets().await?;
        // }

        // FIXME: Bot should always be in Play state, which means we must wait for the LoginSuccess packet
        // We should also wait for the entity to be loaded, inventory to be loadeed, etc.

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
}

impl Bot {
    async fn run(&mut self) -> anyhow::Result<()> {
        let mut interval = tokio::time::interval(Duration::from_millis(50));

        loop {
            interval.tick().await;
            self.tick().await?;
        }
    }

    async fn tick(&mut self) -> anyhow::Result<()> {
        let packets = self.tcp.read_packets().await?;

        for packet in packets {
            Dispatchable::dispatch_packet_event(&packet, self);

            match packet {
                Packets::LoginSuccess(data) => {
                    self.run_on_connect_events(&data).await?;
                }

                Packets::UpdateHealth(data) => {
                    self.run_on_health_update_events(&data).await?;
                }

                _ => {}
            }
        }
        // 8. Tick Client
        // - Update Position
        // Tick Physics

        Ok(())
    }

    pub async fn respawn(&mut self) -> anyhow::Result<()> {
        let packet = ClientCommand::respawn().serialize(self.cmp())?;
        self.tcp.stream.write_all(&packet).await?;
        Ok(())
    }

    pub async fn chat(&mut self, message: &str) -> anyhow::Result<()> {
        let packet = Chat::new(message).serialize(self.cmp())?;
        self.tcp.stream.write_all(&packet).await?;
        Ok(())
    }

    async fn send_settings(&mut self) -> anyhow::Result<()> {
        let packet = ClientSettings::default().serialize(self.cmp())?;
        self.tcp.stream.write_all(&packet).await?;
        Ok(())
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    fn cmp(&self) -> i32 {
        self.tcp.compression_threshold
    }

    // EVENTS HANDLERS
    async fn run_on_connect_events(&mut self, data: &LoginSuccess) -> anyhow::Result<()> {
        // FIXME: Packet is invalid if sent 1ms too early
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        self.send_settings().await?;
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
