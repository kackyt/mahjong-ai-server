pub mod world;

use crate::mahjong_generated::open_mahjong::{MentsuT, PaiT};

/// 得点を表す値オブジェクト
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct ScorePoint(pub i32);

/// 風を表す値オブジェクト (0:東, 1:南, 2:西, 3:北)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WindIndex(pub u32);

/// 場風を表す値オブジェクト (0:東, 1:南, 2:西, 3:北)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BakazeIndex(pub u32);

/// 親のインデックスを表す値オブジェクト
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OyaIndex(pub u32);

/// 本場を表す値オブジェクト
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TsumobouCount(pub u32);

/// 供託（立直棒）を表す値オブジェクト
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RiichibouCount(pub u32);

/// 手番の順番（巡目等）を表す値オブジェクト
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PartialOrd, Ord)]
pub struct SeqCount(pub u32);

/// ドラの枚数等を表す値オブジェクト
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DoraLen(pub u32);

/// 卓上（山）のカーソル位置を表す値オブジェクト
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TakuCursolPos(pub u32);

/// 局を識別するID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KyokuId(pub u64);

/// 牌を選択するためのカーソル位置を表す値オブジェクト
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CursolPos(pub u32);

/// 手牌コンポーネント
pub struct Hand {
    /// 手牌のリスト
    pub tiles: Vec<PaiT>,
    /// ツモ牌 (存在する場合)
    pub tsumohai: Option<PaiT>,
    /// ツモ状態フラグ
    pub is_tsumo: bool,
}

/// 捨て牌（河）コンポーネント
pub struct DiscardPile {
    /// 捨てられた牌のリスト
    pub tiles: Vec<PaiT>,
}

/// 副露面子コンポーネント
pub struct Fulo {
    /// 鳴いた面子のリスト
    pub mentsu: Vec<MentsuT>,
}

/// スコアコンポーネント
pub struct Score {
    /// プレイヤーの持ち点
    pub score: ScorePoint,
}

/// 自風コンポーネント
pub struct Wind {
    /// プレイヤーの自風
    pub wind: WindIndex,
}

/// 立直状態コンポーネント
pub struct RiichiStatus {
    /// 立直しているか
    pub is_riichi: bool,
    /// 一発状態か
    pub is_ippatsu: bool,
}

/// プレイヤー情報コンポーネント
pub struct PlayerInfo {
    /// プレイヤー名
    pub name: String,
}

/// カーソルコンポーネント
pub struct Cursol {
    /// 選択中のインデックス
    pub cursol: CursolPos,
}
