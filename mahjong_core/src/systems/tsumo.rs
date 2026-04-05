use crate::components::{Cursol, Hand};
use crate::systems::types::SystemError;

pub struct TsumoView<'w> {
    pub hand: &'w mut Hand,
    pub cursol: &'w mut Cursol,
}

pub struct TsumoInput {
    pub teban: usize,
    pub seq: u32,
    pub kyoku_id: u64,
    pub is_non_duplicate: bool,
    pub taku_cursol: usize,
    pub tsumohai: crate::mahjong_generated::open_mahjong::PaiT,
}

pub struct TsumoEvent {
    pub tsumohai: crate::mahjong_generated::open_mahjong::PaiT,
    pub kyoku_id: u64,
    pub teban: usize,
    pub seq: u32,
}

pub fn run_tsumo(view: TsumoView<'_>, input: &TsumoInput) -> Result<TsumoEvent, SystemError> {
    if view.hand.is_tsumo {
        return Err(SystemError::InvalidOperation("すでにツモしています".to_string()));
    }
    view.hand.is_tsumo = true;

    view.hand.tsumohai = Some(input.tsumohai.clone());

    if !input.is_non_duplicate {
        view.cursol.cursol += 1;
    }

    Ok(TsumoEvent {
        tsumohai: input.tsumohai.clone(),
        kyoku_id: input.kyoku_id,
        teban: input.teban,
        seq: input.seq,
    })
}
