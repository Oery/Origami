use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use gami_mc_protocol::packets::play::server::UpdateTeam;
use gami_mc_protocol::{
    packets::play::server::{CreateTeam, FriendlyFire, ScoreboardKind},
    registry::TextColor,
};

#[derive(Default, Debug)]
pub struct Scores {
    pub objectives: HashMap<Rc<str>, Objective>,
    pub teams: HashMap<Rc<str>, Team>,
    pub below_name: Option<Rc<str>>,
    pub player_list: Option<Rc<str>>,
    pub sidebar: Option<Rc<str>>,
    pub team_sidebar: Option<Rc<str>>,
}

impl Scores {
    pub fn get_objective(&mut self, name: impl Into<Rc<str>>) -> Option<&mut Objective> {
        self.objectives.get_mut::<Rc<str>>(&name.into())
    }

    pub fn create_team(&mut self, name: impl Into<Rc<str>>, team: &CreateTeam) {
        self.teams.insert(name.into(), Team::from(team));
    }

    pub fn remove_team(&mut self, name: impl Into<Rc<str>>) {
        self.teams.remove(&name.into());
    }

    pub fn get_team(&mut self, name: impl Into<Rc<str>>) -> Option<&mut Team> {
        self.teams.get_mut(&name.into())
    }

    pub fn update_team(&mut self, name: impl Into<Rc<str>>, team: &UpdateTeam) {
        if let Some(prev_team) = self.teams.get_mut(&name.into()) {
            prev_team.display_name = team.display_name.clone();
            prev_team.prefix = team.prefix.clone();
            prev_team.suffix = team.suffix.clone();
            prev_team.friendly_fire = team.friendly_fire;
            prev_team.nametag_visibility = team.nametag_visibility.clone();
            prev_team.color = team.color;
        }
    }
}

#[derive(Default, Debug)]
pub struct Objective {
    pub display_name: String,
    pub kind: ScoreboardKind,
    pub scores: HashMap<Rc<str>, i32>,
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

impl Team {
    pub fn add_players(&mut self, players: &[String]) {
        self.players.extend(players.iter().cloned());
    }

    pub fn remove_players(&mut self, players: &[String]) {
        self.players.retain(|player| !players.contains(player));
    }
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
