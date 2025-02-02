use gami_mc_protocol::packets::play;

use bot::BotBuilder;
use events::Context;

mod bot;
mod events;
mod stream;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut bot = BotBuilder::new().with_username("AmeSombre");

    bot.on_chat(|ctx: &Context<play::server::Chat>| {
        println!("Chat from User Event: {}", ctx.data.message);
    });

    bot.run().await?;

    Ok(())
}
