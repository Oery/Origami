use bot::BotBuilder;

mod bot;
mod stream;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let bot = BotBuilder::new().with_username("Oery");
    bot.connect().await?;

    Ok(())
}
