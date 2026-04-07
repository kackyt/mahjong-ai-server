use super::{
    BakazeIndex, Cursol, CursolPos, DiscardPile, DoraLen, Fulo, Hand, KyokuId, OyaIndex,
    PlayerInfo, RiichiStatus, RiichibouCount, Score, ScorePoint, SeqCount, TakuCursolPos,
    TsumobouCount, Wind, WindIndex,
};
use crate::mahjong_generated::open_mahjong::{GameStateT, RuleT, TakuT};
use hecs::{Entity, World};
use thiserror::Error;

/// ドメイン層のWorldに関連するエラー定義
#[derive(Error, Debug)]
pub enum WorldError {
    /// 指定された手番のプレイヤーが見つからない
    #[error("Player not found for teban: {0}")]
    PlayerNotFound(usize),
    /// コンポーネントが見つからない
    #[error("Components not found for player")]
    ComponentsNotFound,
    /// hecsのクエリエラー
    #[error("HECS query error: {0}")]
    HecsError(#[from] hecs::ComponentError),
    /// 特定のエンティティ操作におけるエラー
    #[error("Entity error: {0}")]
    EntityError(#[from] hecs::NoSuchEntity),
}

/// ゲーム全体のコンテキスト情報を保持する構造体
pub struct GameContext {
    /// ゲームのタイトル
    pub title: String,
    /// ゲームのユニークID
    pub game_id: Vec<u8>,
    /// 局をまたいで一意なID
    pub kyoku_id: KyokuId,
    /// 場風 (東, 南, 西, 北)
    pub bakaze: BakazeIndex,
    /// 親のプレイヤーインデックス
    pub oya: OyaIndex,
    /// 積み棒の数
    pub tsumobou: TsumobouCount,
    /// 立直棒の数
    pub riichibou: RiichibouCount,
    /// 現在の手番プレイヤーインデックス
    pub teban: u32,
    /// ログやイベントのシーケンス番号
    pub seq: SeqCount,
    /// ドラの数
    pub dora_len: DoraLen,
    /// 裏ドラの数
    pub uradora_len: DoraLen,
    /// 牌の重複を許さないかの設定
    pub is_non_duplicate: bool,
    /// ゲームルール設定
    pub rule: RuleT,
    /// 卓上の牌（山）の状態
    pub taku: TakuT,
    /// 卓上のカーソル位置
    pub taku_cursol: TakuCursolPos,
}

/// 麻雀のゲーム状態を管理するメインの世界（ECSのWorldを保持）
pub struct MahjongWorld {
    /// hecsによるEntity情報管理
    pub world: World,
    /// プレイヤーエンティティのリスト
    pub players: Vec<Entity>,
    /// ゲーム全体で共有されるコンテキスト
    pub context: GameContext,
}

impl MahjongWorld {
    /// 指定されたプレイヤー人数で新しい世界を作成します
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
                Score {
                    score: ScorePoint(25000),
                },
                Wind { wind: WindIndex(0) },
                RiichiStatus {
                    is_riichi: false,
                    is_ippatsu: false,
                },
                PlayerInfo {
                    name: String::new(),
                },
                Cursol {
                    cursol: CursolPos(0),
                },
            ));
            players.push(entity);
        }

        Self {
            world,
            players,
            context: GameContext {
                title: String::new(),
                game_id: Vec::new(),
                kyoku_id: KyokuId(0),
                bakaze: BakazeIndex(0),
                oya: OyaIndex(0),
                tsumobou: TsumobouCount(0),
                riichibou: RiichibouCount(0),
                teban: 0,
                seq: SeqCount(0),
                dora_len: DoraLen(0),
                uradora_len: DoraLen(0),
                is_non_duplicate: false,
                rule: RuleT::default(),
                taku: TakuT::default(),
                taku_cursol: TakuCursolPos(0),
            },
        }
    }

    /// プレイヤーのインデックスからエンティティを取得します
    pub fn query_player(&self, idx: usize) -> Option<Entity> {
        self.players.get(idx).copied()
    }

    /// FlatBuffersのGameStateTからMahjongWorldを再構築します
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

            let tehai_len = (player_state.tehai_len as usize).min(player_state.tehai.len());
            let kawahai_len = (player_state.kawahai_len as usize).min(player_state.kawahai.len());
            let mentsu_len = (player_state.mentsu_len as usize).min(player_state.mentsu.len());

            let entity = world.spawn((
                Hand {
                    tiles: player_state.tehai[..tehai_len].to_vec(),
                    tsumohai,
                    is_tsumo: player_state.is_tsumo,
                },
                DiscardPile {
                    tiles: player_state.kawahai[..kawahai_len].to_vec(),
                },
                Fulo {
                    mentsu: player_state.mentsu[..mentsu_len].to_vec(),
                },
                Score {
                    score: ScorePoint(player_state.score),
                },
                Wind {
                    wind: WindIndex((i as u32 + 4 - state.oya) % 4),
                },
                RiichiStatus {
                    is_riichi: player_state.is_riichi,
                    is_ippatsu: player_state.is_ippatsu,
                },
                PlayerInfo {
                    name: String::from_utf8_lossy(&player_state.name.pack().0).to_string(),
                },
                Cursol {
                    cursol: CursolPos(player_state.cursol),
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
                kyoku_id: KyokuId(state.kyoku_id),
                bakaze: BakazeIndex(state.bakaze),
                oya: OyaIndex(state.oya),
                tsumobou: TsumobouCount(state.tsumobou),
                riichibou: RiichibouCount(state.riichibou),
                teban: state.teban,
                seq: SeqCount(state.seq),
                dora_len: DoraLen(state.dora_len),
                uradora_len: DoraLen(state.uradora_len),
                is_non_duplicate: state.is_non_duplicate,
                rule: state.rule.clone(),
                taku: state.taku.clone(),
                taku_cursol: TakuCursolPos(state.taku_cursol),
            },
        }
    }

    /// MahjongWorldの状態をFlatBuffersのGameStateTに書き戻します
    pub fn to_game_state(&self, state: &mut GameStateT) {
        state.title = self.context.title.as_bytes().into();
        let game_id_len = self.context.game_id.len().min(state.game_id.len());
        state.game_id.fill(0);
        state.game_id[..game_id_len].copy_from_slice(&self.context.game_id[..game_id_len]);
        state.kyoku_id = self.context.kyoku_id.0;
        state.bakaze = self.context.bakaze.0;
        state.oya = self.context.oya.0;
        state.tsumobou = self.context.tsumobou.0;
        state.riichibou = self.context.riichibou.0;
        state.teban = self.context.teban;
        state.seq = self.context.seq.0;
        state.dora_len = self.context.dora_len.0;
        state.uradora_len = self.context.uradora_len.0;
        state.is_non_duplicate = self.context.is_non_duplicate;
        state.rule = self.context.rule.clone();
        state.taku = self.context.taku.clone();
        state.taku_cursol = self.context.taku_cursol.0;
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
                    player.score = score.score.0;
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
                    player.cursol = cursol.cursol.0;
                }
            }

            // Note: jikaze is not modeled in PlayerT, but maybe calculated dynamically
        }
    }

    /// ツモ時の手牌とカーソルのビューを使用してクロージャを実行します
    pub fn with_tsumo_view<F, R>(&mut self, teban: usize, f: F) -> Result<R, WorldError>
    where
        F: FnOnce(crate::systems::tsumo::TsumoView<'_>) -> R,
    {
        let entity = self
            .query_player(teban)
            .ok_or(WorldError::PlayerNotFound(teban))?;
        let mut q = self
            .world
            .query_one::<(&mut Hand, &mut Cursol)>(entity)
            .map_err(WorldError::EntityError)?;
        let (hand, cursol) = q.get().ok_or(WorldError::ComponentsNotFound)?;

        Ok(f(crate::systems::tsumo::TsumoView { hand, cursol }))
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
        assert_eq!(score.score.0, 25000);
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
        assert_eq!(score0.score.0, 30000);

        let entity1 = world.query_player(1).unwrap();
        let mut q1 = world.world.query_one::<&Score>(entity1).unwrap();
        let score1 = q1.get().unwrap();
        assert_eq!(score1.score.0, 20000);
    }
}
