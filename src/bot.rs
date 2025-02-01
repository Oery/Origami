use std::time::Duration;

use kagami::minecraft::packets::login::server::LoginSuccess;
use kagami::minecraft::packets::play;
use kagami::minecraft::packets::play::client::{Chat, ClientCommand, ClientSettings};
use kagami::minecraft::packets::play::server::UpdateHealth;
use kagami::minecraft::{packets, Packet, Packets};
use kagami::tcp::State;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::events::{Context, Event, Events};
use crate::stream::Stream;

pub struct BotBuilder {
    username: String,
    host: String,
    port: u16,
    events: Events,
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

    pub fn on_chat(&mut self, callback: Event<play::server::Chat>) {
        self.events.chat.push(callback);
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

        stream.write_all(&packet.serialize_raw(0)?).await?;

        Ok(())
    }

    async fn login(&self, stream: &mut TcpStream) -> anyhow::Result<()> {
        let packet = packets::login::client::LoginStart {
            username: self.username.to_string(),
        };

        stream.write_all(&packet.serialize_raw(0)?).await?;

        Ok(())
    }
}

impl Default for BotBuilder {
    fn default() -> Self {
        Self {
            username: "minecraft_bot_1".to_string(),
            host: "127.0.0.1".to_string(),
            port: 25565,
            events: Events::default(),
        }
    }
}

pub struct Bot {
    username: String,
    tcp: Stream,
    events: Events,
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
            match packet {
                Packets::LoginSuccess(ctx) => {
                    self.run_on_connect_events(ctx).await?;
                }

                Packets::UpdateHealth(ctx) => {
                    self.run_on_health_update_events(ctx).await?;
                }

                Packets::ServerChat(ctx) => {
                    self.run_on_chat_events(ctx).await?;
                }

                _ => {}
            }
        }
        // 7. User Events
        // 8. Tick Client
        // - Update Position

        Ok(())
    }

    pub async fn respawn(&mut self) -> anyhow::Result<()> {
        let packet = ClientCommand::respawn().serialize_raw(self.cmp())?;
        self.tcp.stream.write_all(&packet).await?;
        Ok(())
    }

    pub async fn chat(&mut self, message: &str) -> anyhow::Result<()> {
        let packet = Chat::new(message).serialize_raw(self.cmp())?;
        self.tcp.stream.write_all(&packet).await?;
        Ok(())
    }

    async fn send_settings(&mut self) -> anyhow::Result<()> {
        let packet = ClientSettings::default().serialize_raw(self.cmp())?;
        self.tcp.stream.write_all(&packet).await?;
        Ok(())
    }

    fn cmp(&self) -> i32 {
        self.tcp.compression_threshold
    }

    // EVENTS HANDLERS
    async fn run_on_connect_events(&mut self, ctx: LoginSuccess) -> anyhow::Result<()> {
        // FIXME: Packet is invalid if sent 1ms too early
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        self.send_settings().await?;
        Ok(())
    }

    async fn run_on_death_events(&mut self) -> anyhow::Result<()> {
        self.respawn().await?;
        Ok(())
    }

    async fn run_on_health_update_events(&mut self, ctx: UpdateHealth) -> anyhow::Result<()> {
        println!("Health: {}", ctx.health);

        if ctx.health <= 0.0 {
            println!("Health is 0, bot has died");
            self.run_on_death_events().await?;
        }

        Ok(())
    }

    async fn run_on_chat_events(&mut self, ctx: play::server::Chat) -> anyhow::Result<()> {
        let context = Context {
            bot: self,
            data: &ctx,
        };

        self.events.chat.iter().for_each(|event| event(&context));

        Ok(())
    }
}
