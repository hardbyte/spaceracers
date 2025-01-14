use crate::components::ship::Ship;
use crate::components::Player;
use crate::map::{load_map, Map};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum GameStatus {
    Queued,
    Running,
    Finished,
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub game_id: Uuid,
    pub players: Vec<Player>,
    pub finish_times: HashMap<Uuid, f32>,

    pub ships: Vec<Ship>,
    pub map: Map,
    pub state: GameStatus,
}

impl TryFrom<PendingGame> for GameState {
    type Error = ();

    fn try_from(pending_game: PendingGame) -> Result<Self, Self::Error> {

        let map = load_map(pending_game.map_name.as_str()).ok_or(())?;

        Ok(GameState {
            game_id: pending_game.game_id,
            players: pending_game.players,
            ships: vec![],
            map,
            state: GameStatus::Queued,
            finish_times: HashMap::new(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PendingGame {
    pub game_id: Uuid,
    pub players: Vec<Player>,
    pub map_name: String,
}

impl PendingGame {
    pub(crate) fn new(map_name: String) -> PendingGame {
        PendingGame {
            game_id: Uuid::new_v4(),
            players: vec![],
            map_name,
        }
    }
}
