use crate::components::world::MahjongWorld;
use crate::mahjong_generated::open_mahjong::PaiT;
use crate::play_log::PlayLog;
use crate::components::{Hand, DiscardPile, RiichiStatus};
use anyhow::{Result, ensure};

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
    let mut q = world.world.query_one::<(&mut Hand, &mut DiscardPile, &mut RiichiStatus)>(entity).unwrap();
    let (hand, discard_pile, riichi_status) = q.get().unwrap();
    
    let tehai_len = hand.tiles.len();
    let is_tsumogiri = index >= tehai_len;

    if riichi_status.is_riichi {
        ensure!(is_tsumogiri, "リーチ後はツモ切りのみです");
    }

    if is_riichi {
        ensure!(!riichi_status.is_riichi, "すでにリーチしています");
        riichi_status.is_riichi = true;
        riichi_status.is_ippatsu = true;
    } else {
        riichi_status.is_ippatsu = false;
    }

    let mut kawahai = if is_tsumogiri {
        ensure!(hand.is_tsumo, "ツモしていません (ツモ切り不可)");
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
    
    // Turn cycling
    world.context.teban = (world.context.teban + 1) % world.players.len() as u32;
    
    Ok(kawahai)
}
