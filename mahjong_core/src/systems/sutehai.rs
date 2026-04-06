use crate::components::{DiscardPile, Hand, RiichiStatus};
use crate::mahjong_generated::open_mahjong::PaiT;
use crate::systems::types::SystemError;

pub struct SutehaiView<'w> {
    pub hand: &'w mut Hand,
    pub discard_pile: &'w mut DiscardPile,
    pub riichi_status: &'w mut RiichiStatus,
}

pub struct SutehaiInput {
    pub kyoku_id: u64,
    pub teban: usize,
    pub seq: u32,
    pub index: usize,
    pub is_riichi: bool,
}

pub struct SutehaiEvent {
    pub kawahai: PaiT,
    pub kyoku_id: u64,
    pub teban: usize,
    pub seq: u32,
}

pub fn run_sutehai(
    view: SutehaiView<'_>,
    input: &SutehaiInput,
) -> Result<SutehaiEvent, SystemError> {
    let tehai_len = view.hand.tiles.len();
    let is_tsumogiri = input.index >= tehai_len;

    if view.riichi_status.is_riichi {
        if !is_tsumogiri {
            return Err(SystemError::InvalidOperation(
                "リーチ後はツモ切りのみです".to_string(),
            ));
        }
    }

    if input.is_riichi {
        if view.riichi_status.is_riichi {
            return Err(SystemError::InvalidOperation(
                "すでにリーチしています".to_string(),
            ));
        }
        view.riichi_status.is_riichi = true;
        view.riichi_status.is_ippatsu = true;
    } else {
        view.riichi_status.is_ippatsu = false;
    }

    let mut kawahai = if is_tsumogiri {
        if !view.hand.is_tsumo {
            return Err(SystemError::InvalidOperation(
                "ツモしていません (ツモ切り不可)".to_string(),
            ));
        }
        view.hand.tsumohai.clone().unwrap()
    } else {
        view.hand.tiles[input.index].clone()
    };

    kawahai.is_tsumogiri = is_tsumogiri;
    kawahai.is_riichi = input.is_riichi;

    if !is_tsumogiri {
        view.hand.tiles.remove(input.index);
        if view.hand.is_tsumo {
            if let Some(tsumo) = view.hand.tsumohai.clone() {
                view.hand.tiles.push(tsumo);
                view.hand
                    .tiles
                    .sort_unstable_by(|a, b| a.pai_num.cmp(&b.pai_num).then(a.id.cmp(&b.id)));
            }
        }
    }

    view.hand.is_tsumo = false;
    view.hand.tsumohai = None;

    view.discard_pile.tiles.push(kawahai.clone());

    Ok(SutehaiEvent {
        kawahai,
        kyoku_id: input.kyoku_id,
        teban: input.teban,
        seq: input.seq,
    })
}
