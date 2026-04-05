pub mod agari;
pub mod fulo;
pub mod scoring;
pub mod sutehai;
pub mod tsumo;
pub mod types;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::world::MahjongWorld;
    use crate::fbs_utils::TakuControl;
    use crate::play_log::PlayLog;

    #[test]
    fn test_tsumo_to_sutehai() {
        let mut world = MahjongWorld::new(4);
        
        let teban = world.context.teban as usize;
        let seq = world.context.seq;
        let kyoku_id = world.context.kyoku_id;
        let index = 0;
        let is_riichi = false;
        
        let entity = world.query_player(teban).unwrap();
        let taku_cursol = world.context.taku_cursol as usize;
        let tsumohai = world.context.taku.get(taku_cursol).unwrap();

        let mut q_tsumo = world.world.query_one::<(&mut crate::components::Hand, &mut crate::components::Cursol)>(entity).unwrap();
        let (hand, cursol) = q_tsumo.get().unwrap();
        
        let tsumo_view = tsumo::TsumoView { hand, cursol };
        let tsumo_input = tsumo::TsumoInput {
            teban,
            seq,
            kyoku_id,
            is_non_duplicate: world.context.is_non_duplicate,
            taku_cursol,
            tsumohai: tsumohai.clone(),
        };
        
        let _tsumo_event = tsumo::run_tsumo(tsumo_view, &tsumo_input).unwrap();
        world.context.taku_cursol += 1; // normally done globally

        let mut q_sutehai = world.world.query_one::<(&mut crate::components::Hand, &mut crate::components::DiscardPile, &mut crate::components::RiichiStatus)>(entity).unwrap();
        let (hand, discard_pile, riichi_status) = q_sutehai.get().unwrap();

        let sutehai_view = sutehai::SutehaiView {
            hand,
            discard_pile,
            riichi_status,
        };
        let sutehai_input = sutehai::SutehaiInput {
            kyoku_id,
            teban,
            seq: seq + 1,
            index,
            is_riichi,
        };

        let _sutehai_event = sutehai::run_sutehai(sutehai_view, &sutehai_input).unwrap();
    }
}
