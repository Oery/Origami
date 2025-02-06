use crate::packets::{play, Packets, ServerPacket};
use crate::Bot;

pub struct Context<'bot, 'payload, T> {
    pub bot: &'bot Bot,
    pub payload: &'payload T,
}

pub type EventHandler<T> = Box<dyn Fn(&Context<T>)>;

#[derive(Default)]
pub struct EventHandlers {
    pub chat_handlers: Vec<EventHandler<play::server::Chat>>,
    pub login_handlers: Vec<EventHandler<play::server::Login>>,
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

impl<F> PacketHandler<play::server::Chat> for F
where
    F: Fn(&Context<play::server::Chat>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.chat_handlers.push(Box::new(self));
    }
}

impl<F> PacketHandler<play::server::Login> for F
where
    F: Fn(&Context<play::server::Login>) + 'static,
{
    fn register(self, events: &mut EventHandlers) {
        events.login_handlers.push(Box::new(self));
    }
}

pub trait Dispatchable {
    fn dispatch_packet_event(&self, bot: &Bot);
}

impl Dispatchable for Packets {
    fn dispatch_packet_event(&self, bot: &Bot) {
        match self {
            Self::ServerChat(payload) => bot.events.dispatch(payload, bot),
            Self::Login(payload) => bot.events.dispatch(payload, bot),
            _ => {}
        };
    }
}

impl Dispatchable for Context<'_, '_, play::server::Chat> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.chat_handlers {
            event(self);
        }
    }
}

impl Dispatchable for Context<'_, '_, play::server::Login> {
    fn dispatch_packet_event(&self, bot: &Bot) {
        for event in &bot.events.login_handlers {
            event(self);
        }
    }
}
