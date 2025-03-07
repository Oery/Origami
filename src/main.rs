use events::Context;
use gami_mc_protocol::registry::EntityKind;
use packets::play::server::Chat;

mod bot;
mod events;
mod inventory;
mod scores;
mod stream;
mod world;

pub use bot::{Bot, BotBuilder};
pub use gami_mc_protocol::packets;
pub use inventory::Inventory;
pub use world::World;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut bot = BotBuilder::new();

    bot.on_chat(|ctx: &Context<Chat>| {
        if ctx.payload.message.contains("attack") {
            for entity in ctx.bot.world().entities.values() {
                ctx.bot.attack_entity(entity.as_entity().id());
            }
        }

        if ctx.payload.message.contains("entity_id") {
            ctx.bot.chat(&format!("Entity ID: {}", ctx.bot.entity_id()));
        }

        if ctx.payload.message.contains("sheep") {
            for entity in ctx.bot.world().entities.values() {
                if let EntityKind::Sheep(sheep) = entity {
                    ctx.bot.chat(&format!("Sheep Color: {}", sheep.color));
                }
            }
        }

        if ctx.payload.message.contains("pig") {
            let pigs = ctx.bot.world().entities.values().filter_map(|e| {
                if let EntityKind::Pig(pig) = e {
                    Some(pig)
                } else {
                    None
                }
            });

            for pig in pigs {
                ctx.bot
                    .chat(&format!("Pig: {}, Has_Saddle: {}", pig.id, pig.has_saddle));
            }
        }
    });

    bot.on_tick(|ctx: &Context<()>| {
        for entity in ctx.bot.world().entities.values() {
            if let EntityKind::Pig(pig) = entity {
                if pig.has_saddle {
                    ctx.bot.attack_entity(entity.id());
                }
            }
        }
    });

    bot.on_connect(|ctx: &Context<()>| {
        ctx.bot.chat("Connected!");
        ctx.bot.chat(&format!("UUID: {}", ctx.bot.uuid()));
        ctx.bot.chat(&format!("Entity ID: {}", ctx.bot.entity_id()));
        ctx.bot.chat(&format!("Game Mode: {}", ctx.bot.game_mode()));
        ctx.bot.chat(&format!("Username: {}", ctx.bot.username()));
    });

    bot.run().await?;

    Ok(())
}
