use crate::components::{Cursol, Hand, KyokuId, SeqCount, TakuCursolPos};
use crate::systems::types::{InvalidOperationReason, SystemError};

/// ツモシステムのビュー。手牌とカーソルの変更権限を持ちます。
pub struct TsumoView<'w> {
    pub hand: &'w mut Hand,
    pub cursol: &'w mut Cursol,
}

/// ツモシステムの入力データ
pub struct TsumoInput {
    /// ツモを行うプレイヤーの手番
    pub teban: usize,
    /// 現在のシーケンス番号
    pub seq: SeqCount,
    /// 現在の局ID
    pub kyoku_id: KyokuId,
    /// 牌の重複を許さないかの設定
    pub is_non_duplicate: bool,
    /// 現在の卓上のカーソル位置
    pub taku_cursol: TakuCursolPos,
    /// ツモった牌
    pub tsumohai: crate::mahjong_generated::open_mahjong::PaiT,
}

/// ツモシステムから発行されるイベントデータ
pub struct TsumoEvent {
    /// ツモった牌
    pub tsumohai: crate::mahjong_generated::open_mahjong::PaiT,
    /// 局ID
    pub kyoku_id: KyokuId,
    /// 手番
    pub teban: usize,
    /// シーケンス番号
    pub seq: SeqCount,
}

/// ツモシステムを実行し、状態を更新します。
pub fn run_tsumo(view: TsumoView<'_>, input: &TsumoInput) -> Result<TsumoEvent, SystemError> {
    if view.hand.is_tsumo {
        return Err(SystemError::InvalidOperation(InvalidOperationReason(
            "すでにツモしています".to_string(),
        )));
    }
    view.hand.is_tsumo = true;

    // input.tsumohai.clone() は flatbuffers の struct であるため必要
    view.hand.tsumohai = Some(input.tsumohai.clone());

    if input.is_non_duplicate {
        view.cursol.cursol.0 += 1;
    }

    Ok(TsumoEvent {
        tsumohai: input.tsumohai.clone(),
        kyoku_id: input.kyoku_id,
        teban: input.teban,
        seq: input.seq,
    })
}
