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
        if ctx.payload.message.contains("attack") {
            for entity in ctx.bot.world.entities.iter() {
                ctx.bot.attack_entity(entity.id);
            }
        }

        if ctx.payload.message.contains("entity_id") {
            ctx.bot.chat(&format!("Entity ID: {}", ctx.bot.entity_id()));
        }
    });

    bot.on_tick(|ctx: &Context<()>| {
        for entity in ctx.bot.world.entities.iter() {
            ctx.bot.attack_entity(entity.id);
        }
    });

    bot.run().await?;

    Ok(())
}
