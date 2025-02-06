use events::Context;
use origami::packets::play;
use packets::play::server::Chat;

mod bot;
mod events;
mod stream;

pub use bot::{Bot, BotBuilder};
pub use gami_mc_protocol::packets;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut bot = BotBuilder::new().with_username("AmeSombre");

    bot.on_chat(|ctx: &Context<Chat>| {
        println!("Chat from User Event: {}", ctx.payload.message);
    });

    bot.on_packet(|ctx: &Context<play::server::KickDisconnect>| {
        dbg!(ctx.payload);
    });

    bot.run().await?;

    Ok(())
}
