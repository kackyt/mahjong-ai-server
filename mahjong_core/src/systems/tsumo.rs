use crate::components::world::MahjongWorld;
#[allow(unused_imports)]
use crate::mahjong_generated::open_mahjong::GameStateT;
use crate::play_log::PlayLog;
use anyhow::Result;

use crate::components::Hand;
use crate::fbs_utils::TakuControl;

pub fn run_tsumo(world: &mut MahjongWorld, play_log: &mut PlayLog) -> Result<()> {
    let teban = world.context.teban as usize;
    let seq = world.context.seq;
    let kyoku_id = world.context.kyoku_id;
    let is_non_duplicate = world.context.is_non_duplicate;

    let entity = world.query_player(teban).unwrap();
    let mut q = world.world.query_one::<&mut Hand>(entity).unwrap();
    let hand = q.get().unwrap();

    hand.is_tsumo = true;

    let cursol = if is_non_duplicate {
        world.context.taku_cursol as usize
    } else {
        // Technically player.cursol is not in ECS yet, so default to taku_cursol
        world.context.taku_cursol as usize
    };
    let tsumohai = world.context.taku.get(cursol)?;
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
    world.context.taku_cursol += 1;

    Ok(())
}
