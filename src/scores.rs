use std::collections::{HashMap, HashSet};

use gami_mc_protocol::{
    packets::play::server::{CreateTeam, FriendlyFire, ScoreboardKind},
    registry::TextColor,
};

#[derive(Default, Debug)]
pub struct Scores {
    pub objectives: HashMap<String, Objective>,
    pub teams: HashMap<String, Team>,
    pub below_name: Option<String>,
    pub player_list: Option<String>,
    pub sidebar: Option<String>,
    pub team_sidebar: Option<String>,
}

#[derive(Default, Debug)]
pub struct Objective {
    pub display_name: String,
    pub kind: ScoreboardKind,
    pub scores: HashMap<String, i32>,
}

impl Objective {
    pub fn new(kind: ScoreboardKind) -> Self {
        Self {
            kind,
            ..Default::default()
        }
    }
}

#[derive(Default, Debug)]
pub struct Team {
    pub players: HashSet<String>,
    pub display_name: String,
    pub prefix: String,
    pub suffix: String,
    pub friendly_fire: FriendlyFire,
    pub nametag_visibility: String,
    pub color: TextColor,
}

impl From<&CreateTeam> for Team {
    fn from(data: &CreateTeam) -> Self {
        Self {
            players: HashSet::from_iter(data.players.iter().cloned()),
            display_name: data.display_name.clone(),
            prefix: data.prefix.clone(),
            suffix: data.suffix.clone(),
            friendly_fire: data.friendly_fire,
            nametag_visibility: data.nametag_visibility.clone(),
            color: data.color,
        }
    }
}
