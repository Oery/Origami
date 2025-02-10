use events::Context;
use packets::play::server::Chat;

mod bot;
mod entity;
mod events;
mod stream;
mod world;

pub use bot::{Bot, BotBuilder};
pub use gami_mc_protocol::packets;
pub use world::World;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut bot = BotBuilder::new();

    bot.on_chat(|ctx: &Context<Chat>| {
        if ctx.payload.message.contains("ping") {
            ctx.bot.chat("pong!");
        }
    });

    bot.run().await?;

    Ok(())
}
