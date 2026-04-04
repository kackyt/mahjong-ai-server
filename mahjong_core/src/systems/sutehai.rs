use crate::components::world::MahjongWorld;
use crate::mahjong_generated::open_mahjong::PaiT;
use crate::play_log::PlayLog;
use crate::components::{Hand, DiscardPile};
use anyhow::Result;
pub fn run_sutehai(
    world: &mut MahjongWorld,
    play_log: &mut PlayLog,
    index: usize,
    is_riichi: bool,
) -> Result<PaiT> {
    let teban = world.context.teban as usize;
    let kyoku_id = world.context.kyoku_id;
    let seq = world.context.seq;
    
    let entity = world.query_player(teban).unwrap();
    let mut binding = world.world.query_one::<(&mut Hand, &mut DiscardPile)>(entity).unwrap();
    let (hand, discard_pile) = binding.get().unwrap();
    
    // Some logic checks, like ensure!(hand.is_tsumo, "ツモしていません"); 
    // are currently deferred back to game_process.rs as they intertwine with non-ECS states (like riichi flags which we didn't migrate to Hand yet).
    let tehai_len = hand.tiles.len();

    let is_tsumogiri = index >= tehai_len;

    let mut kawahai = if is_tsumogiri {
        hand.tsumohai.clone().unwrap()
    } else {
        hand.tiles[index].clone()
    };

    kawahai.is_tsumogiri = is_tsumogiri;
    kawahai.is_riichi = is_riichi;

    if !is_tsumogiri {
        hand.tiles.remove(index);
        if hand.is_tsumo {
            if let Some(tsumo) = hand.tsumohai.clone() {
                hand.tiles.push(tsumo);
                hand.tiles.sort_unstable();
            }
        }
    }

    hand.is_tsumo = false;
    hand.tsumohai = None;

    discard_pile.tiles.push(kawahai.clone());

    play_log.append_actions_log(
        kyoku_id,
        teban as i32,
        seq as i32,
        String::from("sutehai"),
        kawahai.get_pai_id(),
    );

    world.context.seq += 1;
    // Turn cycling logic will still run correctly from `to_game_state` mapping but we can map it here
    // However, it's safer to let game_process handle `teban` changes until full rewrite since it relies on player_len which isn't always 4
    
    Ok(kawahai)
}
