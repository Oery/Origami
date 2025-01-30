use bot::OfflineBot;

const DEV_SERVER: &str = "127.0.0.1:25565";
const USERNAME: &str = "Oery";

mod bot;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (host, port) = DEV_SERVER.split_once(":").unwrap();

    let bot_config = OfflineBot {
        username: USERNAME.to_string(),
        host: host.to_string(),
        port: port.parse().unwrap(),
    };

    bot_config.connect().await?;

    Ok(())
}
