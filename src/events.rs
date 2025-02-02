use gami_mc_protocol::packets::play;

use crate::bot::Bot;

pub struct Context<'a, 'b, T> {
    pub bot: &'a Bot,
    pub data: &'b T,
}

#[derive(Default)]
pub struct Events {
    pub chat: Vec<Event<play::server::Chat>>,
}

pub type Event<T> = fn(&Context<T>);
