# Origami

A programmable Minecraft client written in pure Rust. It can be used to build agents/bots with low memory and CPU footprint. It features a simple API that allows developers to easily create custom behaviors for their bots while still having access to a more powerful packet interface.

## Example

```rust
#[tokio::main]
async fn main() {
    // Create the bot configuration
    let mut bot = BotBuilder::new();

    // Run when bot connects to a server
    bot.on_connect(|_ctx: &Context<()>| {
        println!("Bot Connected!");
        ctx.bot.chat(&format!("Hey!, I'm {}", ctx.bot.username));
    });

    // Run if the bot get disconnected
    bot.on_disconnect(|ctx: &Context<KickDisconnect>| {
        println!("Bot Disconnected: {:?}", ctx.payload.reason);
    });

    // Run when the bot receives a chat message
    bot.on_chat(|ctx: &Context<Chat>| {
        if ctx.payload.message.contains("wolf") {
            // Iterate over all the entities in the bot's memory
            for entity in ctx.bot.world.entities.values() {
                if let EntityKind::Wolf(wolf) = entity {
                    println!("Name: {} - Color: {}", wolf.name_tag, wolf.collar_color);
                }
            }
        }
    });

    // Run on every tick -> every 50ms
    bot.on_tick(|ctx: &Context<()>| {
        println!("Clock ticked");
    });

    // Run when the packet provided in Context<T> is received
    bot.on_packet(|ctx: &Context<SpawnPlayer>| {
        dbg!(&ctx.payload.player_uuid);
    });

    // Run the bot
    bot.run().await?;

    Ok(())
}
```

#### Target Features

- [x] Login
- [x] Chat
- [x] Event System
- [x] Inventory
- [ ] World
- [ ] Physics Engine

#### Extra Features

- [ ] Support for more versions of Minecraft
- [ ] Pathfinding
- [ ] Shared World State (Huge memory savings when running multiple bots in the same world)
- [ ] World Caching
