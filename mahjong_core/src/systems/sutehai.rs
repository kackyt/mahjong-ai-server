use crate::components::{DiscardPile, Hand, KyokuId, RiichiStatus, SeqCount};
use crate::mahjong_generated::open_mahjong::PaiT;
use crate::systems::types::SystemError;

/// 打牌システムのビュー。手牌、河、および立直状態の変更権限を持ちます。
pub struct SutehaiView<'w> {
    pub hand: &'w mut Hand,
    pub discard_pile: &'w mut DiscardPile,
    pub riichi_status: &'w mut RiichiStatus,
}

/// 打牌システムの入力データ
pub struct SutehaiInput {
    /// 局ID
    pub kyoku_id: KyokuId,
    /// 手番
    pub teban: usize,
    /// シーケンス番号
    pub seq: SeqCount,
    /// 手出し/ツモ切りのインデックス（手牌の中のインデックス、またはツモ牌を示す特定の値）
    pub index: usize,
    /// 立直を宣言したか
    pub is_riichi: bool,
}

/// 打牌システムから発行されるイベントデータ
pub struct SutehaiEvent {
    /// 捨てられた牌
    pub kawahai: PaiT,
    /// 局ID
    pub kyoku_id: KyokuId,
    /// 手番
    pub teban: usize,
    /// シーケンス番号
    pub seq: SeqCount,
}

/// 打牌システムを実行し、状態を更新します。
pub fn run_sutehai(
    view: SutehaiView<'_>,
    input: &SutehaiInput,
) -> Result<SutehaiEvent, SystemError> {
    let tehai_len = view.hand.tiles.len();
    let is_tsumogiri = input.index >= tehai_len;

    // リーチ済みの場合は、ツモ切り以外不許可
    if view.riichi_status.is_riichi && !is_tsumogiri {
        return Err(SystemError::InvalidOperation(
            "リーチ後はツモ切りのみです".to_string(),
        ));
    }

    // リーチ宣言の処理
    if input.is_riichi {
        if view.riichi_status.is_riichi {
            return Err(SystemError::InvalidOperation(
                "すでにリーチしています".to_string(),
            ));
        }
        view.riichi_status.is_riichi = true;
        view.riichi_status.is_ippatsu = true;
    } else {
        // 誰かが鳴くか、自分が捨てるまで一発は継続するが、
        // ここでは自分の打牌によって一発が消える（もしくはリーチによる一発付与）
        view.riichi_status.is_ippatsu = false;
    }

    // 捨て牌の特定
    let mut kawahai = if is_tsumogiri {
        if !view.hand.is_tsumo {
            return Err(SystemError::InvalidOperation(
                "ツモしていません (ツモ切り不可)".to_string(),
            ));
        }
        view.hand
            .tsumohai
            .as_ref()
            .ok_or_else(|| SystemError::InvalidOperation("ツモ牌が存在しません".to_string()))?
            .clone()
    } else {
        view.hand.tiles[input.index].clone()
    };

    kawahai.is_tsumogiri = is_tsumogiri;
    kawahai.is_riichi = input.is_riichi;

    // 手出しの場合、手牌から削除し、ツモ牌を手牌に加える（理牌）
    if !is_tsumogiri {
        view.hand.tiles.remove(input.index);
        if view.hand.is_tsumo {
            if let Some(tsumo) = view.hand.tsumohai.take() {
                view.hand.tiles.push(tsumo);
                view.hand
                    .tiles
                    .sort_unstable_by(|a, b| a.pai_num.cmp(&b.pai_num).then(a.id.cmp(&b.id)));
            }
        }
    }

    // ツモ状態の解除
    view.hand.is_tsumo = false;
    view.hand.tsumohai = None;

    // 河へ追加
    view.discard_pile.tiles.push(kawahai.clone());

    Ok(SutehaiEvent {
        kawahai,
        kyoku_id: input.kyoku_id,
        teban: input.teban,
        seq: input.seq,
    })
}
