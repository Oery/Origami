use crate::packets::{play, Packets, ServerPacket};
use crate::Bot;

pub struct Context<'bot, 'payload, T> {
    pub bot: &'bot Bot,
    pub payload: &'payload T,
}

pub type EventHandler<T> = Box<dyn Fn(&Context<T>)>;

#[derive(Default)]
pub struct EventHandlers {
    pub tick_handlers: Vec<EventHandler<()>>,
    pub on_connect_handlers: Vec<EventHandler<()>>,
    keep_alive_handlers: Vec<EventHandler<play::server::KeepAlive>>,
    join_game_handlers: Vec<EventHandler<play::server::JoinGame>>,
    chat_handlers: Vec<EventHandler<play::server::Chat>>,
    update_time_handlers: Vec<EventHandler<play::server::UpdateTime>>,
    spawn_position_handlers: Vec<EventHandler<play::server::SpawnPosition>>,
    update_health_handlers: Vec<EventHandler<play::server::UpdateHealth>>,
    respawn_handlers: Vec<EventHandler<play::server::Respawn>>,
    position_handlers: Vec<EventHandler<play::server::Position>>,
    held_item_slot_handlers: Vec<EventHandler<play::server::HeldItemSlot>>,
    bed_handlers: Vec<EventHandler<play::server::Bed>>,
    animation_handlers: Vec<EventHandler<play::server::Animation>>,
    collect_handlers: Vec<EventHandler<play::server::Collect>>,
    spawn_entity_painting_handlers: Vec<EventHandler<play::server::SpawnEntityPainting>>,
    spawn_entity_experience_orb_handlers: Vec<EventHandler<play::server::SpawnEntityExperienceOrb>>,
    entity_velocity_handlers: Vec<EventHandler<play::server::EntityVelocity>>,
    entity_destroy_handlers: Vec<EventHandler<play::server::EntityDestroy>>,
    entity_handlers: Vec<EventHandler<play::server::Entity>>,
    entity_relative_move_handlers: Vec<EventHandler<play::server::EntityRelativeMove>>,
    entity_look_handlers: Vec<EventHandler<play::server::EntityLook>>,
    entity_move_look_handlers: Vec<EventHandler<play::server::EntityMoveLook>>,
    entity_teleport_handlers: Vec<EventHandler<play::server::EntityTeleport>>,
    entity_head_rotation_handlers: Vec<EventHandler<play::server::EntityHeadRotation>>,
    entity_status_handlers: Vec<EventHandler<play::server::EntityStatus>>,
    attach_entity_handlers: Vec<EventHandler<play::server::AttachEntity>>,
    entity_metadata_handlers: Vec<EventHandler<play::server::EntityMetadata>>,
    entity_effect_handlers: Vec<EventHandler<play::server::EntityEffect>>,
    remove_entity_effect_handlers: Vec<EventHandler<play::server::RemoveEntityEffect>>,
    block_change_handlers: Vec<EventHandler<play::server::BlockChange>>,
    kick_disconnect_handlers: Vec<EventHandler<play::server::KickDisconnect>>,
    server_difficulty_handlers: Vec<EventHandler<play::server::ServerDifficulty>>,
}

impl EventHandlers {
    pub fn dispatch<'bot, 'packet, T>(&self, payload: &'packet T, bot: &'bot Bot)
    where
        Context<'bot, 'packet, T>: 'packet + Dispatchable,
    {
        let ctx = Context { bot, payload };
        ctx.dispatch_packet_event(bot);
    }
}

pub trait PacketHandler<T: ServerPacket> {
    fn register(self, events: &mut EventHandlers);
}

impl<F> PacketHandler<play::server::KeepAlive> for F
where
    F: Fn(&Context<play::server::KeepAlive>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.keep_alive_handlers.push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::JoinGame> for F
where
    F: Fn(&Context<play::server::JoinGame>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.join_game_handlers.push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::Chat> for F
where
    F: Fn(&Context<play::server::Chat>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.chat_handlers.push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::UpdateTime> for F
where
    F: Fn(&Context<play::server::UpdateTime>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.update_time_handlers.push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::SpawnPosition> for F
where
    F: Fn(&Context<play::server::SpawnPosition>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.spawn_position_handlers.push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::UpdateHealth> for F
where
    F: Fn(&Context<play::server::UpdateHealth>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.update_health_handlers.push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::Respawn> for F
where
    F: Fn(&Context<play::server::Respawn>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.respawn_handlers.push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::Position> for F
where
    F: Fn(&Context<play::server::Position>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.position_handlers.push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::HeldItemSlot> for F
where
    F: Fn(&Context<play::server::HeldItemSlot>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.held_item_slot_handlers.push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::Bed> for F
where
    F: Fn(&Context<play::server::Bed>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.bed_handlers.push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::Animation> for F
where
    F: Fn(&Context<play::server::Animation>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.animation_handlers.push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::Collect> for F
where
    F: Fn(&Context<play::server::Collect>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.collect_handlers.push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::SpawnEntityPainting> for F
where
    F: Fn(&Context<play::server::SpawnEntityPainting>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.spawn_entity_painting_handlers.push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::SpawnEntityExperienceOrb> for F
where
    F: Fn(&Context<play::server::SpawnEntityExperienceOrb>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events
            .spawn_entity_experience_orb_handlers
            .push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::EntityVelocity> for F
where
    F: Fn(&Context<play::server::EntityVelocity>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.entity_velocity_handlers.push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::EntityDestroy> for F
where
    F: Fn(&Context<play::server::EntityDestroy>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.entity_destroy_handlers.push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::Entity> for F
where
    F: Fn(&Context<play::server::Entity>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.entity_handlers.push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::EntityRelativeMove> for F
where
    F: Fn(&Context<play::server::EntityRelativeMove>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.entity_relative_move_handlers.push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::EntityLook> for F
where
    F: Fn(&Context<play::server::EntityLook>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.entity_look_handlers.push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::EntityMoveLook> for F
where
    F: Fn(&Context<play::server::EntityMoveLook>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.entity_move_look_handlers.push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::EntityTeleport> for F
where
    F: Fn(&Context<play::server::EntityTeleport>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.entity_teleport_handlers.push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::EntityHeadRotation> for F
where
    F: Fn(&Context<play::server::EntityHeadRotation>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.entity_head_rotation_handlers.push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::EntityStatus> for F
where
    F: Fn(&Context<play::server::EntityStatus>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.entity_status_handlers.push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::AttachEntity> for F
where
    F: Fn(&Context<play::server::AttachEntity>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.attach_entity_handlers.push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::EntityMetadata> for F
where
    F: Fn(&Context<play::server::EntityMetadata>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.entity_metadata_handlers.push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::EntityEffect> for F
where
    F: Fn(&Context<play::server::EntityEffect>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.entity_effect_handlers.push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::RemoveEntityEffect> for F
where
    F: Fn(&Context<play::server::RemoveEntityEffect>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.remove_entity_effect_handlers.push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::BlockChange> for F
where
    F: Fn(&Context<play::server::BlockChange>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.block_change_handlers.push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::KickDisconnect> for F
where
    F: Fn(&Context<play::server::KickDisconnect>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.kick_disconnect_handlers.push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::ServerDifficulty> for F
where
    F: Fn(&Context<play::server::ServerDifficulty>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.server_difficulty_handlers.push(Box::new(self));
    }
}

pub trait Dispatchable {
    fn dispatch_packet_event(&self, bot: &Bot);
}

impl Dispatchable for Packets {
    fn dispatch_packet_event(&self, bot: &Bot) {
        match self {
            Self::ServerKeepAlive(payload) => bot.events.dispatch(payload, bot),
            Self::ServerChat(payload) => bot.events.dispatch(payload, bot),
            Self::JoinGame(payload) => bot.events.dispatch(payload, bot),
            Self::UpdateTime(payload) => bot.events.dispatch(payload, bot),
            Self::SpawnPosition(payload) => bot.events.dispatch(payload, bot),
            Self::UpdateHealth(payload) => bot.events.dispatch(payload, bot),
            Self::Respawn(payload) => bot.events.dispatch(payload, bot),
            Self::ServerPosition(payload) => bot.events.dispatch(payload, bot),
            Self::ServerHeldItemSlot(payload) => bot.events.dispatch(payload, bot),
            Self::Bed(payload) => bot.events.dispatch(payload, bot),
            Self::Animation(payload) => bot.events.dispatch(payload, bot),
            Self::Collect(payload) => bot.events.dispatch(payload, bot),
            Self::SpawnEntityPainting(payload) => bot.events.dispatch(payload, bot),
            Self::SpawnEntityExperienceOrb(payload) => bot.events.dispatch(payload, bot),
            Self::EntityVelocity(payload) => bot.events.dispatch(payload, bot),
            Self::EntityDestroy(payload) => bot.events.dispatch(payload, bot),
            Self::Entity(payload) => bot.events.dispatch(payload, bot),
            Self::EntityRelativeMove(payload) => bot.events.dispatch(payload, bot),
            Self::EntityLook(payload) => bot.events.dispatch(payload, bot),
            Self::EntityMoveLook(payload) => bot.events.dispatch(payload, bot),
            Self::EntityTeleport(payload) => bot.events.dispatch(payload, bot),
            Self::EntityHeadRotation(payload) => bot.events.dispatch(payload, bot),
            Self::EntityStatus(payload) => bot.events.dispatch(payload, bot),
            Self::AttachEntity(payload) => bot.events.dispatch(payload, bot),
            Self::EntityMetadata(payload) => bot.events.dispatch(payload, bot),
            Self::EntityEffect(payload) => bot.events.dispatch(payload, bot),
            Self::RemoveEntityEffect(payload) => bot.events.dispatch(payload, bot),
            Self::BlockChange(payload) => bot.events.dispatch(payload, bot),
            Self::KickDisconnect(payload) => bot.events.dispatch(payload, bot),
            Self::ServerDifficulty(payload) => bot.events.dispatch(payload, bot),
            _ => {}
        };
    }
}

impl Dispatchable for Context<'_, '_, play::server::KeepAlive> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.keep_alive_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::Chat> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.chat_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::JoinGame> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.join_game_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::UpdateTime> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.update_time_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::SpawnPosition> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.spawn_position_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::UpdateHealth> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.update_health_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::Respawn> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.respawn_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::Position> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.position_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::HeldItemSlot> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.held_item_slot_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::Bed> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.bed_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::Animation> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.animation_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::Collect> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.collect_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::SpawnEntityPainting> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.spawn_entity_painting_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::SpawnEntityExperienceOrb> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.spawn_entity_experience_orb_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::EntityVelocity> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.entity_velocity_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::EntityDestroy> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.entity_destroy_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::Entity> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.entity_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::EntityRelativeMove> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.entity_relative_move_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::EntityLook> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.entity_look_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::EntityMoveLook> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.entity_move_look_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::EntityTeleport> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.entity_teleport_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::EntityHeadRotation> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.entity_head_rotation_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::EntityStatus> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.entity_status_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::AttachEntity> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.attach_entity_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::EntityMetadata> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.entity_metadata_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::EntityEffect> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.entity_effect_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::RemoveEntityEffect> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.remove_entity_effect_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::BlockChange> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.block_change_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::KickDisconnect> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.kick_disconnect_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::ServerDifficulty> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.server_difficulty_handlers {
            event(self);
        }
    }
}
