use bot::BotBuilder;

mod bot;
mod stream;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut bot = BotBuilder::new().with_username("AmeSombre");
    bot.run().await?;

    Ok(())
}
