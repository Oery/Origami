use std::time::Duration;

use gami_mc_protocol::packets::play::server::{
    ScoreboardObjectiveAction, ScoreboardPosition, TeamsAction,
};
use gami_mc_protocol::packets::{self, play::*, ServerPacket};
use gami_mc_protocol::packets::{Packet, Packets};
use gami_mc_protocol::registry::tcp::States;
use gami_mc_protocol::registry::EntityKind;
use tokio::sync::mpsc;
use tokio::time;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::events::{Context, Dispatchable, EventHandlers, PacketHandler};
use crate::scores::{Objective, Scores, Team};
use crate::stream::Stream;
use crate::{Inventory, World};

pub struct BotBuilder {
    username: String,
    host: String,
    port: u16,
    events: EventHandlers,
    autoreconnect: Option<Duration>,
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

    pub fn with_autoreconnect(mut self, delay: Option<Duration>) -> Self {
        self.autoreconnect = delay;
        self
    }

    pub async fn run(self) -> anyhow::Result<()> {
        'run: loop {
            let addr = format!("{}:{}", self.host, self.port);

            let mut stream = match TcpStream::connect(addr).await {
                Ok(stream) => stream,
                Err(err) => {
                    if let Some(delay) = self.autoreconnect {
                        time::sleep(delay).await;
                        continue;
                    }
                    return Err(err.into());
                }
            };

            self.set_protocol(&mut stream).await?;
            self.login(&mut stream).await?;

            let (reader, writer) = stream.into_split();
            let (tx, rx) = mpsc::channel::<Vec<u8>>(100);

            let mut stream = Stream::new(reader, tx);
            let mut packets = vec![];
            let mut uuid = String::new();
            let mut entity_id = None;
            let mut game_mode = None;

            while entity_id.is_none() {
                let mut new_packets = stream.read_packets().await?;

                for packet in &new_packets {
                    match packet {
                        Packets::LoginSuccess(data) => {
                            uuid = data.uuid.clone();
                        }

                        Packets::JoinGame(data) => {
                            entity_id = Some(data.entity_id);
                            game_mode = Some(data.game_mode);
                        }

                        Packets::Disconnect(data) => {
                            if let Some(delay) = self.autoreconnect {
                                time::sleep(delay).await;
                                continue 'run;
                            }

                            return Err(anyhow::anyhow!(
                                "Disconnected from server: {:?}",
                                data.reason
                            ));
                        }
                        _ => {}
                    }
                }

                packets.append(&mut new_packets);
            }

            let mut bot = Bot {
                username: &self.username,
                tcp: stream,
                events: &self.events,
                world: World::default(),
                uuid,
                entity_id: entity_id.unwrap(),
                game_mode: game_mode.unwrap(),
                scores: Scores::default(),
                inventory: Inventory::default(),
            };

            bot.tcp.listen(writer, rx);
            bot.handle_packets(packets).await?;

            if let Err(e) = bot.run().await {
                eprintln!("Bot Error: {:?}", e);

                if let Some(delay) = self.autoreconnect {
                    time::sleep(delay).await;
                    continue;
                }
            }

            return Ok(());
        }
    }

    async fn set_protocol(&self, stream: &mut TcpStream) -> anyhow::Result<()> {
        let packet = packets::handshake::client::SetProtocol {
            protocol_version: 47,
            server_host: self.host.clone(),
            server_port: self.port,
            next_state: States::Login,
        };

        stream.write_all(&packet.serialize(-1)?).await?;

        Ok(())
    }

    async fn login(&self, stream: &mut TcpStream) -> anyhow::Result<()> {
        let packet = packets::login::client::LoginStart {
            username: self.username.to_string(),
        };

        stream.write_all(&packet.serialize(-1)?).await?;

        Ok(())
    }

    pub fn on_tick<T: Fn(&Context<'_, '_, ()>) + 'static>(&mut self, f: T) {
        self.events.tick_handlers.push(Box::new(f))
    }

    pub fn on_packet<T: ServerPacket>(&mut self, f: impl PacketHandler<T>) {
        f.register(&mut self.events);
    }

    pub fn on_connect<T: Fn(&Context<'_, '_, ()>) + 'static>(&mut self, f: T) {
        self.events.on_connect_handlers.push(Box::new(f))
    }

    pub fn on_disconnect(&mut self, f: impl PacketHandler<server::KickDisconnect>) {
        f.register(&mut self.events);
    }

    pub fn on_chat(&mut self, f: impl PacketHandler<server::Chat>) {
        f.register(&mut self.events);
    }

    pub fn on_scoreboard_action(&mut self, f: impl PacketHandler<server::ScoreboardObjective>) {
        f.register(&mut self.events);
    }

    pub fn on_scoreboard_display(&mut self, f: impl PacketHandler<server::ScoreboardDisplay>) {
        f.register(&mut self.events);
    }

    pub fn on_teams_action(&mut self, f: impl PacketHandler<server::Teams>) {
        f.register(&mut self.events);
    }
}

impl Default for BotBuilder {
    fn default() -> Self {
        Self {
            username: "minecraft_bot_1".to_string(),
            host: "127.0.0.1".to_string(),
            port: 25565,
            events: EventHandlers::default(),
            autoreconnect: Some(Duration::from_secs(5)),
        }
    }
}

pub struct Bot<'a> {
    pub username: &'a String,
    tcp: Stream,
    pub events: &'a EventHandlers,
    pub inventory: Inventory,
    pub world: World,
    pub uuid: String,
    pub entity_id: i32,
    pub game_mode: u8,
    pub scores: Scores,
}

impl Bot<'_> {
    async fn run(&mut self) -> anyhow::Result<()> {
        let mut interval = tokio::time::interval(Duration::from_millis(50));

        loop {
            interval.tick().await;
            self.tick().await?;
        }
    }

    pub(crate) async fn tick(&mut self) -> anyhow::Result<()> {
        let packets = self.tcp.read_packets().await?;
        self.handle_packets(packets).await?;

        self.run_on_tick_events().await?;

        // TODO: Tick Physics / Update Position

        Ok(())
    }

    pub(crate) async fn handle_packets(&mut self, packets: Vec<Packets>) -> anyhow::Result<()> {
        for packet in packets {
            Dispatchable::dispatch_packet_event(&packet, self);

            match packet {
                Packets::LoginSuccess(_) => {
                    self.run_on_connect_events().await?;
                }

                Packets::UpdateHealth(data) => {
                    self.run_on_health_update_events(&data).await?;
                }

                // Packets::Entity(data) => {
                //     self.world.spawn_entity(data.entity_id);
                // }
                // Packets::EntityVelocity(data) => {
                //     self.world.spawn_entity(data.entity_id);
                // }
                // Packets::EntityMoveLook(data) => {
                //     self.world.spawn_entity(data.entity_id);
                // }

                // Packets::EntityRelativeMove(data) => {
                //     let entity = self.world.get_entity(data.entity_id)?;
                //     if let Some(ref mut coords) = entity.coords {
                //         coords.x += data.d_x as i32;
                //         coords.y += data.d_y as i32;
                //         coords.z += data.d_z as i32;
                //     }
                // }

                // Packets::EntityHeadRotation(data) => {
                //     let entity = self.world.get_entity(data.entity_id)?;
                //     if let Some(ref mut coords) = entity.coords {
                //         coords.yaw = data.head_yaw;
                //     }
                // }
                // Packets::EntityTeleport(data) => {
                //     if let Some(entity) = self.world.get_entity(data.entity_id) {

                //     }
                //     entity.coords = Some(Coordinates {
                //         x: data.x,
                //         y: data.y,
                //         z: data.z,
                //         yaw: data.yaw,
                //         pitch: data.pitch,
                //     });
                // }
                Packets::SpawnPlayer(player) => {
                    let entity = EntityKind::from(&player);
                    self.world.entities.insert(entity.id(), entity);
                }

                Packets::SpawnObject(object) => {
                    let entity = EntityKind::from(&object);
                    self.world.entities.insert(entity.id(), entity);
                }

                Packets::SpawnMob(mob) => {
                    let entity = EntityKind::from(&mob);
                    self.world.entities.insert(entity.id(), entity);
                }

                Packets::EntityMetadata(data) => {
                    if let Some(entity) = self.world.entities.get_mut(&data.entity_id) {
                        entity.update(&data.metadatas);
                    }
                }

                Packets::ScoreboardObjective(data) => match data.action {
                    ScoreboardObjectiveAction::Add(action_data) => {
                        self.scores
                            .objectives
                            .insert(data.objective, Objective::new(action_data.kind));
                    }
                    ScoreboardObjectiveAction::Remove => {
                        self.scores.objectives.remove(&data.objective);
                    }
                    ScoreboardObjectiveAction::UpdateDisplayText(action_data) => {
                        if let Some(objective) = self.scores.objectives.get_mut(&data.objective) {
                            objective.display_name = action_data.display_name;
                            objective.kind = action_data.kind;
                        }
                    }
                },

                Packets::ScoreboardUpdate(data) => {
                    if data.objective.is_empty() {
                        self.scores.objectives.values_mut().for_each(|obj| {
                            obj.scores.remove(&data.player);
                        });
                    } else if let Some(objective) = self.scores.objectives.get_mut(&data.objective)
                    {
                        match data.value {
                            Some(value) => objective.scores.insert(data.player, value),
                            None => objective.scores.remove(&data.player),
                        };
                    }
                }

                Packets::ScoreboardDisplay(data) => {
                    let value = match data.name.is_empty() {
                        true => None,
                        false => Some(data.name),
                    };

                    match data.position {
                        ScoreboardPosition::List => self.scores.player_list = value,
                        ScoreboardPosition::Sidebar => self.scores.sidebar = value,
                        ScoreboardPosition::BelowName => self.scores.below_name = value,
                        _ => self.scores.team_sidebar = value,
                    };
                }

                Packets::Teams(data) => match data.action {
                    TeamsAction::CreateTeam(team) => {
                        self.scores.teams.insert(data.name, Team::from(&team));
                    }
                    TeamsAction::RemoveTeam => {
                        self.scores.teams.remove(&data.name);
                    }
                    TeamsAction::UpdateTeam(team) => {
                        if let Some(prev_team) = self.scores.teams.get_mut(&data.name) {
                            prev_team.display_name = team.display_name.clone();
                            prev_team.prefix = team.prefix.clone();
                            prev_team.suffix = team.suffix.clone();
                            prev_team.friendly_fire = team.friendly_fire;
                            prev_team.nametag_visibility = team.nametag_visibility.clone();
                            prev_team.color = team.color;
                        }
                    }
                    TeamsAction::AddPlayers(players) => {
                        if let Some(team) = self.scores.teams.get_mut(&data.name) {
                            team.players.extend(players.players.iter().cloned());
                        }
                    }
                    TeamsAction::RemovePlayers(players) => {
                        if let Some(team) = self.scores.teams.get_mut(&data.name) {
                            team.players
                                .retain(|player| !players.players.contains(player));
                        }
                    }
                },

                Packets::KickDisconnect(data) => {
                    return Err(anyhow::anyhow!(
                        "Disconnected from server: {:?}",
                        data.reason
                    ));
                }

                Packets::SetSlot(data) => {
                    if data.window_id == 0 {
                        match data.slot {
                            -1 => self.inventory.carried = data.item,
                            i => self.inventory.slots[i as usize] = data.item,
                        };
                    }
                }

                Packets::EntityEquipment(data) => {
                    if self.entity_id == data.entity_id {
                        self.inventory.armor_slots_mut()[data.slot as usize] = Some(data.item);
                    }
                }

                _ => {}
            }
        }

        Ok(())
    }

    pub async fn respawn(&mut self) -> anyhow::Result<()> {
        let packet = packets::play::client::ClientCommand::respawn().serialize(self.cmp())?;
        self.tcp.tx.send(packet).await?;
        Ok(())
    }

    pub fn chat(&self, message: &str) {
        let Ok(packet) = client::Chat::new(message).serialize(self.cmp()) else {
            eprintln!("[ERROR] bot.chat() : Failed to serialize chat packet");
            return;
        };

        // TODO: Improve this
        let tx = self.tcp.tx.clone();
        tokio::spawn(async move {
            tx.send(packet).await.unwrap();
        });
    }

    pub fn attack_entity(&self, id: i32) {
        if id == self.entity_id {
            eprintln!("[WARN] Cannot attack self, canceling...");
            return;
        }

        let Some(entity) = self.world.entities.get(&id) else {
            eprintln!("[ERROR] Entity not found");
            return;
        };

        let is_attackable = !matches!(
            entity,
            EntityKind::ItemStack(_) | EntityKind::ExperienceOrb(_)
        );

        if !is_attackable {
            eprintln!("[WARN] Cannot attack entity, canceling...");
            return;
        }

        let Ok(packet) = client::UseEntity::attack(entity.id()).serialize(self.cmp()) else {
            eprintln!("[ERROR] Failed to serialize packet");
            return;
        };

        // TODO: Improve this
        let tx = self.tcp.tx.clone();
        tokio::spawn(async move {
            tx.send(packet).await.unwrap();
        });
    }

    async fn send_settings(&mut self) -> anyhow::Result<()> {
        let packet = client::ClientSettings::default().serialize(self.cmp())?;
        self.tcp.tx.send(packet).await?;
        Ok(())
    }

    fn cmp(&self) -> i32 {
        self.tcp.compression_threshold
    }

    // EVENTS HANDLERS
    async fn run_on_connect_events(&mut self) -> anyhow::Result<()> {
        self.send_settings().await?;

        self.events.on_connect_handlers.iter().for_each(|e| {
            e(&Context {
                bot: self,
                payload: &(),
            })
        });

        Ok(())
    }

    async fn run_on_tick_events(&mut self) -> anyhow::Result<()> {
        self.events.tick_handlers.iter().for_each(|e| {
            e(&Context {
                bot: self,
                payload: &(),
            })
        });

        Ok(())
    }

    async fn run_on_death_events(&mut self) -> anyhow::Result<()> {
        // TODO: Add AutoRespawn flag
        self.respawn().await?;

        Ok(())
    }

    async fn run_on_health_update_events(
        &mut self,
        data: &server::UpdateHealth,
    ) -> anyhow::Result<()> {
        if data.health <= 0.0 {
            self.run_on_death_events().await?;
        }

        Ok(())
    }
}
