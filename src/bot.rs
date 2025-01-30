use std::time::Duration;

use kagami::minecraft::packets::handshake::client::SetProtocol;
use kagami::minecraft::packets::login::client::LoginStart;
use kagami::minecraft::packets::play::client::Chat;
use kagami::minecraft::Packet;
use kagami::tcp::State;

use tokio::{io::AsyncWriteExt, net::TcpStream};

pub struct OfflineBot {
    pub username: String,
    pub host: String,
    pub port: u16,
}

impl OfflineBot {
    pub async fn connect(&self) -> anyhow::Result<()> {
        let addr = format!("{}:{}", self.host, self.port);
        let mut stream = TcpStream::connect(addr).await?;

        self.set_protocol(&mut stream).await?;
        self.login(&mut stream).await?;

        let mut bot = Bot {
            username: self.username.clone(),
            stream,
            state: State::Login,
            host: self.host.clone(),
            port: self.port,
        };

        if let Err(e) = bot.run().await {
            eprintln!("Bot Error: {:?}", e);
        }

        Ok(())
    }

    async fn set_protocol(&self, stream: &mut TcpStream) -> anyhow::Result<()> {
        let packet = SetProtocol {
            protocol_version: 47,
            server_host: self.host.clone(),
            server_port: self.port,
            next_state: State::Login,
        };

        stream
            .write_all(&to_raw(packet.serialize_packet()?))
            .await?;

        Ok(())
    }

    async fn login(&self, stream: &mut TcpStream) -> anyhow::Result<()> {
        let packet = LoginStart {
            username: self.username.clone(),
        };

        stream
            .write_all(&to_raw(packet.serialize_packet()?))
            .await?;

        Ok(())
    }
}

pub struct Bot {
    pub username: String,
    pub stream: TcpStream,
    pub state: State,
    pub host: String,
    pub port: u16,
}

impl Bot {
    pub async fn run(&mut self) -> anyhow::Result<()> {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        self.respawn().await?;

        let mut interval = tokio::time::interval(Duration::from_millis(50));

        loop {
            interval.tick().await;
            self.tick().await?;
        }
    }

    pub async fn respawn(&mut self) -> anyhow::Result<()> {
        self.stream.write_all(&[3, 0, 22, 0]).await?;
        Ok(())
    }

    pub async fn chat(&mut self, message: String) -> anyhow::Result<()> {
        let packet = Chat { message }.serialize_packet()?;
        self.stream.write_all(&to_raw(packet)).await?;
        Ok(())
    }

    pub async fn tick(&mut self) -> anyhow::Result<()> {
        println!("Ticking Bot...");

        // 4. Get Full Packets
        // 5. Deserialize Packets
        // 6. System Events
        // 7. User Events
        // 8. Tick Client
        // - Update Position

        Ok(())
    }
}

fn to_raw(packet: (Vec<u8>, Vec<u8>)) -> Vec<u8> {
    packet.0.into_iter().chain(packet.1).collect()
}
