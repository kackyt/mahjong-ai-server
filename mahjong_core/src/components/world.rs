use super::{DiscardPile, Hand, Score, Wind, Fulo};
use crate::mahjong_generated::open_mahjong::{GameStateT, RuleT, TakuT};
use hecs::{Entity, World};

pub struct GameContext {
    pub title: String,
    pub game_id: Vec<u8>,
    pub kyoku_id: u64,
    pub bakaze: u32,
    pub oya: u32,
    pub tsumobou: u32,
    pub riichibou: u32,
    pub teban: u32,
    pub seq: u32,
    pub dora_len: u32,
    pub uradora_len: u32,
    pub is_non_duplicate: bool,
    pub rule: RuleT,
    pub taku: TakuT,
    pub taku_cursol: u32,
}

pub struct MahjongWorld {
    pub world: World,
    pub players: Vec<Entity>,
    pub context: GameContext,
}

impl MahjongWorld {
    pub fn new(player_len: usize) -> Self {
        let mut world = World::new();
        let mut players = Vec::new();

        for _ in 0..player_len {
            let entity = world.spawn((
                Hand { tiles: Vec::new(), tsumohai: None, is_tsumo: false },
                DiscardPile { tiles: Vec::new() },
                Fulo { mentsu: Vec::new() },
                Score { score: 25000 },
                Wind { wind: 0 },
            ));
            players.push(entity);
        }

        Self {
            world,
            players,
            context: GameContext {
                title: String::new(),
                game_id: Vec::new(),
                kyoku_id: 0,
                bakaze: 0,
                oya: 0,
                tsumobou: 0,
                riichibou: 0,
                teban: 0,
                seq: 0,
                dora_len: 0,
                uradora_len: 0,
                is_non_duplicate: false,
                rule: RuleT::default(),
                taku: TakuT::default(),
                taku_cursol: 0,
            },
        }
    }

    pub fn query_player(&self, idx: usize) -> Option<Entity> {
        self.players.get(idx).copied()
    }

    pub fn from_game_state(state: &GameStateT) -> Self {
        let mut world = World::new();
        let mut players = Vec::new();

        for (i, player_state) in state
            .players
            .iter()
            .take(state.player_len as usize)
            .enumerate()
        {
            let tsumohai = if player_state.is_tsumo {
                Some(player_state.tsumohai.clone())
            } else {
                None
            };
            
            let entity = world.spawn((
                Hand {
                    tiles: player_state.tehai[..player_state.tehai_len as usize].to_vec(),
                    tsumohai,
                    is_tsumo: player_state.is_tsumo,
                },
                DiscardPile {
                    tiles: player_state.kawahai[..player_state.kawahai_len as usize].to_vec(),
                },
                Fulo {
                    mentsu: player_state.mentsu[..player_state.mentsu_len as usize].to_vec(),
                },
                Score {
                    score: player_state.score,
                },
                Wind {
                    wind: (state.bakaze + i as u32) % 4, // Needs true wind calc
                },
            ));
            players.push(entity);
        }

        Self {
            world,
            players,
            context: GameContext {
                title: String::from_utf8_lossy(&state.title.pack().0).to_string(),
                game_id: state.game_id.to_vec(),
                kyoku_id: state.kyoku_id,
                bakaze: state.bakaze,
                oya: state.oya,
                tsumobou: state.tsumobou,
                riichibou: state.riichibou,
                teban: state.teban,
                seq: state.seq,
                dora_len: state.dora_len,
                uradora_len: state.uradora_len,
                is_non_duplicate: state.is_non_duplicate,
                rule: state.rule.clone(),
                taku: state.taku.clone(),
                taku_cursol: state.taku_cursol,
            },
        }
    }
    
    pub fn to_game_state(&self, state: &mut GameStateT) {
        state.title = self.context.title.as_bytes().into();
        state.game_id.copy_from_slice(&self.context.game_id);
        state.kyoku_id = self.context.kyoku_id;
        state.bakaze = self.context.bakaze;
        state.oya = self.context.oya;
        state.tsumobou = self.context.tsumobou;
        state.riichibou = self.context.riichibou;
        state.teban = self.context.teban;
        state.seq = self.context.seq;
        state.dora_len = self.context.dora_len;
        state.uradora_len = self.context.uradora_len;
        state.is_non_duplicate = self.context.is_non_duplicate;
        state.rule = self.context.rule.clone();
        state.taku = self.context.taku.clone();
        state.taku_cursol = self.context.taku_cursol;
        state.player_len = self.players.len() as u32;

        for (i, &entity) in self.players.iter().enumerate() {
            let player = &mut state.players[i];
            
            if let Some(hand) = self.world.query_one::<&Hand>(entity).unwrap().get() {
                player.tehai_len = hand.tiles.len() as u32;
                player.tehai[..hand.tiles.len()].clone_from_slice(&hand.tiles);
                player.is_tsumo = hand.is_tsumo;
                if let Some(tsumo) = &hand.tsumohai {
                    player.tsumohai = tsumo.clone();
                }
            }
            
            if let Some(discard) = self.world.query_one::<&DiscardPile>(entity).unwrap().get() {
                player.kawahai_len = discard.tiles.len() as u32;
                player.kawahai[..discard.tiles.len()].clone_from_slice(&discard.tiles);
            }
            
            if let Some(fulo) = self.world.query_one::<&Fulo>(entity).unwrap().get() {
                player.mentsu_len = fulo.mentsu.len() as u32;
                player.mentsu[..fulo.mentsu.len()].clone_from_slice(&fulo.mentsu);
            }
            
            if let Some(score) = self.world.query_one::<&Score>(entity).unwrap().get() {
                player.score = score.score;
            }
            
            // Note: jikaze is not modeled in PlayerT, but maybe calculated dynamically
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mahjong_world_new() {
        let world = MahjongWorld::new(4);
        assert_eq!(world.players.len(), 4);

        let entity = world.query_player(0).unwrap();
        let mut q = world.world.query_one::<&Score>(entity).unwrap();
        let score = q.get().unwrap();
        assert_eq!(score.score, 25000);
    }

    #[test]
    fn test_mahjong_world_from_game_state() {
        let mut state = GameStateT::default();
        state.player_len = 2;
        state.players[0].score = 30000;
        state.players[1].score = 20000;

        let world = MahjongWorld::from_game_state(&state);
        assert_eq!(world.players.len(), 2);

        let entity0 = world.query_player(0).unwrap();
        let mut q0 = world.world.query_one::<&Score>(entity0).unwrap();
        let score0 = q0.get().unwrap();
        assert_eq!(score0.score, 30000);

        let entity1 = world.query_player(1).unwrap();
        let mut q1 = world.world.query_one::<&Score>(entity1).unwrap();
        let score1 = q1.get().unwrap();
        assert_eq!(score1.score, 20000);
    }
}
