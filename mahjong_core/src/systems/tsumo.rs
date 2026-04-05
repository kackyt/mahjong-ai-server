use crate::components::world::MahjongWorld;
#[allow(unused_imports)]
use crate::mahjong_generated::open_mahjong::GameStateT;
use crate::play_log::PlayLog;
use anyhow::Result;

use crate::components::{Hand, Cursol};
use crate::fbs_utils::TakuControl;

pub fn run_tsumo(world: &mut MahjongWorld, play_log: &mut PlayLog) -> Result<()> {
    let teban = world.context.teban as usize;
    let seq = world.context.seq;
    let kyoku_id = world.context.kyoku_id;
    let is_non_duplicate = world.context.is_non_duplicate;

    let entity = world.query_player(teban).unwrap();
    let mut q = world.world.query_one::<(&mut Hand, &mut Cursol)>(entity).unwrap();
    let (hand, cursol_comp) = q.get().unwrap();
    
    hand.is_tsumo = true;

    let cursol_val = if is_non_duplicate {
        world.context.taku_cursol as usize
    } else {
        cursol_comp.cursol as usize
    };
    
    let tsumohai = world.context.taku.get(cursol_val)?;
    hand.tsumohai = Some(tsumohai.clone());

    play_log.append_actions_log(
        kyoku_id,
        teban as i32,
        seq as i32,
        String::from("tsumo"),
        tsumohai.get_pai_id(),
    );

    world.context.seq += 1;

    // Equivalent to self.next_cursol() logic
    if is_non_duplicate {
        world.context.taku_cursol += 1;
    } else {
        cursol_comp.cursol += 1;
    }

    Ok(())
}
