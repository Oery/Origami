use std::time::Duration;

use kagami::minecraft::{packets, Packet};
use kagami::tcp::State;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::stream::Stream;

pub struct BotBuilder {
    username: String,
    host: String,
    port: u16,
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

    pub async fn connect(self) -> anyhow::Result<()> {
        let addr = format!("{}:{}", self.host, self.port);
        let mut stream = TcpStream::connect(addr).await?;

        self.set_protocol(&mut stream).await?;
        self.login(&mut stream).await?;

        let mut bot = Bot {
            username: self.username,
            tcp: Stream::new(stream),
        };

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
            next_state: State::Login,
        };

        stream
            .write_all(&to_raw(packet.serialize_packet(0)?))
            .await?;

        Ok(())
    }

    async fn login(&self, stream: &mut TcpStream) -> anyhow::Result<()> {
        let packet = packets::login::client::LoginStart {
            username: self.username.clone(),
        };

        stream
            .write_all(&to_raw(packet.serialize_packet(0)?))
            .await?;

        Ok(())
    }
}

impl Default for BotBuilder {
    fn default() -> Self {
        Self {
            username: "minecraft_bot_1".to_string(),
            host: "127.0.0.1".to_string(),
            port: 25565,
        }
    }
}

pub struct Bot {
    username: String,
    tcp: Stream,
}

impl Bot {
    async fn run(&mut self) -> anyhow::Result<()> {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        self.respawn().await?;

        let mut interval = tokio::time::interval(Duration::from_millis(50));

        loop {
            interval.tick().await;
            self.tick().await?;
        }
    }

    async fn tick(&mut self) -> anyhow::Result<()> {
        let packets = self.tcp.read_packets().await?;
        // 7. User Events
        // 8. Tick Client
        // - Update Position

        Ok(())
    }

    pub async fn respawn(&mut self) -> anyhow::Result<()> {
        self.tcp.stream.write_all(&[3, 0, 22, 0]).await?;
        Ok(())
    }

    pub async fn chat(&mut self, message: String) -> anyhow::Result<()> {
        let packet = packets::play::client::Chat { message }
            .serialize_packet(self.tcp.compression_threshold)?;
        self.tcp.stream.write_all(&to_raw(packet)).await?;
        Ok(())
    }
}

fn to_raw(packet: (Vec<u8>, Vec<u8>)) -> Vec<u8> {
    packet.0.into_iter().chain(packet.1).collect()
}
