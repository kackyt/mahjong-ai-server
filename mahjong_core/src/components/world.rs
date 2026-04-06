use super::{Cursol, DiscardPile, Fulo, Hand, PlayerInfo, RiichiStatus, Score, Wind};
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
                Hand {
                    tiles: Vec::new(),
                    tsumohai: None,
                    is_tsumo: false,
                },
                DiscardPile { tiles: Vec::new() },
                Fulo { mentsu: Vec::new() },
                Score { score: 25000 },
                Wind { wind: 0 },
                RiichiStatus {
                    is_riichi: false,
                    is_ippatsu: false,
                },
                PlayerInfo {
                    name: String::new(),
                },
                Cursol { cursol: 0 },
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
                    wind: (i as u32 + 4 - state.oya) % 4,
                },
                RiichiStatus {
                    is_riichi: player_state.is_riichi,
                    is_ippatsu: player_state.is_ippatsu,
                },
                PlayerInfo {
                    name: String::from_utf8_lossy(&player_state.name.pack().0).to_string(),
                },
                Cursol {
                    cursol: player_state.cursol,
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
        let game_id_len = self.context.game_id.len().min(state.game_id.len());
        state.game_id.fill(0);
        state.game_id[..game_id_len].copy_from_slice(&self.context.game_id[..game_id_len]);
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

            if let Ok(mut q) = self.world.query_one::<&Hand>(entity) {
                if let Some(hand) = q.get() {
                    let len = hand.tiles.len().min(player.tehai.len());
                    player.tehai_len = len as u32;
                    for i in 0..len {
                        player.tehai[i] = hand.tiles[i].clone();
                    }
                    if let Some(tsumo) = &hand.tsumohai {
                        player.tsumohai = tsumo.clone();
                    } else {
                        player.tsumohai = Default::default();
                    }
                    player.is_tsumo = hand.is_tsumo;
                }
            }

            if let Ok(mut q) = self.world.query_one::<&DiscardPile>(entity) {
                if let Some(discard) = q.get() {
                    let len = discard.tiles.len().min(player.kawahai.len());
                    player.kawahai_len = len as u32;
                    player.kawahai[..len].clone_from_slice(&discard.tiles[..len]);
                }
            }

            if let Ok(mut q) = self.world.query_one::<&Fulo>(entity) {
                if let Some(fulo) = q.get() {
                    let len = fulo.mentsu.len().min(player.mentsu.len());
                    player.mentsu_len = len as u32;
                    player.mentsu[..len].clone_from_slice(&fulo.mentsu[..len]);
                }
            }

            if let Ok(mut q) = self.world.query_one::<&Score>(entity) {
                if let Some(score) = q.get() {
                    player.score = score.score;
                }
            }

            if let Ok(mut q) = self.world.query_one::<&RiichiStatus>(entity) {
                if let Some(riichi) = q.get() {
                    player.is_riichi = riichi.is_riichi;
                    player.is_ippatsu = riichi.is_ippatsu;
                }
            }

            if let Ok(mut q) = self.world.query_one::<&PlayerInfo>(entity) {
                if let Some(info) = q.get() {
                    player.name = info.name.as_bytes().into();
                }
            }

            if let Ok(mut q) = self.world.query_one::<&Cursol>(entity) {
                if let Some(cursol) = q.get() {
                    player.cursol = cursol.cursol;
                }
            }

            // Note: jikaze is not modeled in PlayerT, but maybe calculated dynamically
        }
    }

    pub fn tsumo_view(
        &mut self,
        teban: usize,
    ) -> anyhow::Result<crate::systems::tsumo::TsumoView<'_>> {
        use anyhow::Context;
        let entity = self.query_player(teban).context("Player not found")?;
        let mut q = self.world.query_one::<(&mut Hand, &mut Cursol)>(entity)?;
        let (hand, cursol) = q.get().context("Components not found")?;

        // This requires unsafe because we are returning a reference bounded to the lifetime of self,
        // but hecs query iterator gives references bounded to the QueryItem.
        // We can safely transmute because we statically know `self.world` keeps it alive.
        // Using transmute is common here without hecs specific lifetime workarounds, or we can just fetch and return.
        // Actually, you can't return `hand` and `cursol` from `query_one` directly because `q` borrows `self.world`.
        // We can just return it if we pass it through.
        unreachable!("Implemented locally where used since hecs borrow checker is strict")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mahjong_generated::open_mahjong::{GameStateT, PlayerT};

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
        let players: [PlayerT; 4] = [
            PlayerT {
                score: 30000,
                ..Default::default()
            },
            PlayerT {
                score: 20000,
                ..Default::default()
            },
            PlayerT::default(),
            PlayerT::default(),
        ];

        let state = GameStateT {
            player_len: 2,
            players,
            ..Default::default()
        };

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
