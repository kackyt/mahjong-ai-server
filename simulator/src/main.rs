use anyhow::Result;
use clap::Parser;
use mahjong_ai::evaluator::eval_sutehai;
use mahjong_core::{
    load_pailist::load_pailist,
    mahjong_generated::open_mahjong::{ActionType, GameStateT, PaiT},
    play_log::PlayLog,
    shanten::PaiState,
};
use serde::Serialize;
use std::fs::File;
use std::io::Write;

#[derive(Parser, Debug)]
#[command(author, about, version)]
struct Command {
    #[arg(short, long)]
    pai_list_file: Option<String>,
    #[arg(short, long, default_value_t = 0)]
    index: usize,
    #[arg(long, default_value_t = 1, value_parser = validate_players)]
    players: usize,
    #[arg(short, long, default_value = "simulator_log.jsonl")]
    log_file: String,
}

fn validate_players(s: &str) -> Result<usize, String> {
    let s = s.parse::<usize>().map_err(|e| e.to_string())?;
    if s == 1 || s == 4 {
        Ok(s)
    } else {
        Err(format!("Support value is 1 or 4, but got {}", s))
    }
}

#[derive(Serialize)]
struct ActionLog {
    turn: usize,
    player_idx: usize,
    tehai: Vec<u8>,
    tsumohai: u8,
    shanten: i32,
    discard: u8,
    action_type: String,
}

fn main() -> Result<()> {
    let args = Command::parse();
    let mut game_state: GameStateT = Default::default();
    let mut play_log = PlayLog::new();
    let title = "simulator".as_bytes();

    println!("initialize");

    let num_players = args.players;
    game_state.create(title, num_players as u32, &mut play_log);

    if let Some(pai_list) = args.pai_list_file {
        let hai_ids = load_pailist(pai_list, args.index)?;
        game_state.load(&hai_ids);
    } else {
        game_state.shuffle();
    }

    game_state.start(&mut play_log);

    let mut log_file = File::create(&args.log_file)?;
    run_simulation(&mut game_state, &mut play_log, &mut log_file)
}

fn run_simulation<W: Write>(
    game_state: &mut GameStateT,
    play_log: &mut PlayLog,
    log_file: &mut W,
) -> Result<()> {
    let mut turn = 0;

    loop {
        // 1. ツモ前の状態評価（流局・和了チェック）
        // 和了は基本的にはツモ直後に発生するが、天和などの特殊ケースや
        // ループの整合性のためにここでチェックする。
        match game_state.evaluate_post_draw_status() {
            mahjong_core::game_process::PostDrawAction::Ryuukyoku => {
                let current_player = game_state.teban as usize;
                let player = &game_state.players[current_player];
                let tehai_nums: Vec<u8> = player.tehai[..player.tehai_len as usize]
                    .iter()
                    .map(|p| p.pai_num)
                    .collect();
                let mut state = PaiState::from(&player.tehai[..player.tehai_len as usize]);
                let shanten = state.get_shanten(player.mentsu_len as usize);

                let log_json = serde_json::to_string(&ActionLog {
                    turn,
                    player_idx: current_player,
                    tehai: tehai_nums,
                    tsumohai: 255,
                    shanten,
                    discard: 255,
                    action_type: "RYUUKYOKU".to_string(),
                })?;
                writeln!(log_file, "{}", log_json)?;
                break;
            }
            mahjong_core::game_process::PostDrawAction::TsumoAgari => {
                let current_player = game_state.teban as usize;
                let player = &game_state.players[current_player];
                let tehai_nums: Vec<u8> = player.tehai[..player.tehai_len as usize]
                    .iter()
                    .map(|p| p.pai_num)
                    .collect();

                let log_json = serde_json::to_string(&ActionLog {
                    turn,
                    player_idx: current_player,
                    tehai: tehai_nums,
                    tsumohai: player.tsumohai.pai_num,
                    shanten: -1,
                    discard: player.tsumohai.pai_num,
                    action_type: "TSUMO_AGARI".to_string(),
                })?;
                writeln!(log_file, "{}", log_json)?;
                break;
            }
            _ => {}
        }

        // 2. ツモ
        game_state.tsumo(play_log)?;

        // 3. ツモ直後の和了チェック
        if game_state.evaluate_post_draw_status()
            == mahjong_core::game_process::PostDrawAction::TsumoAgari
        {
            let current_player = game_state.teban as usize;
            let player = &game_state.players[current_player];
            let tehai_nums: Vec<u8> = player.tehai[..player.tehai_len as usize]
                .iter()
                .map(|p| p.pai_num)
                .collect();

            let log_json = serde_json::to_string(&ActionLog {
                turn,
                player_idx: current_player,
                tehai: tehai_nums,
                tsumohai: player.tsumohai.pai_num,
                shanten: -1,
                discard: player.tsumohai.pai_num,
                action_type: "TSUMO_AGARI".to_string(),
            })?;
            writeln!(log_file, "{}", log_json)?;
            break;
        }

        let current_player = game_state.teban as usize;

        let player = &game_state.players[current_player];

        let mut tehai: Vec<PaiT> = player
            .tehai
            .iter()
            .take(player.tehai_len as usize)
            .cloned()
            .collect();
        let tehai_nums: Vec<u8> = tehai.iter().map(|p| p.pai_num).collect();

        tehai.push(player.tsumohai.clone());

        // Use mahjong_ai to decide the discard
        let best_discard = match eval_sutehai(game_state) {
            Ok((pai, _score)) => pai as u32,
            Err(_) => {
                if player.is_tsumo {
                    player.tsumohai.pai_num as u32
                } else {
                    tehai[0].pai_num as u32
                }
            }
        };

        let mut is_tsumogiri = false;
        let discard_idx = if player.is_tsumo && player.tsumohai.pai_num as u32 == best_discard {
            is_tsumogiri = true;
            13
        } else {
            let idx = player
                .tehai
                .iter()
                .take(player.tehai_len as usize)
                .position(|p| p.pai_num as u32 == best_discard);
            idx.ok_or_else(|| {
                anyhow::anyhow!(
                    "eval_sutehai returned tile {} not present in tehai/tsumohai",
                    best_discard
                )
            })?
        };

        let discard_action = if is_tsumogiri {
            (discard_idx as u32) | 0x8000
        } else {
            discard_idx as u32
        };

        // Compute shanten for logging
        let mut pstate = PaiState::from(&tehai);
        let shanten = pstate.get_shanten(player.mentsu_len as usize);

        let action_log = ActionLog {
            turn,
            player_idx: current_player,
            tehai: tehai_nums,
            tsumohai: player.tsumohai.pai_num,
            shanten,
            discard: best_discard as u8,
            action_type: "SUTEHAI".to_string(),
        };

        game_state.action(
            play_log,
            ActionType::ACTION_SUTEHAI,
            current_player,
            discard_action,
        )?;

        let log_json = serde_json::to_string(&action_log)?;
        writeln!(log_file, "{}", log_json)?;

        turn += 1;
        if turn > 1000 {
            // fallback safety
            break;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_validate_players() {
        assert!(validate_players("1").is_ok());
        assert!(validate_players("4").is_ok());
        assert!(validate_players("2").is_err());
        assert!(validate_players("3").is_err());
        assert!(validate_players("abc").is_err());
    }

    #[test]
    fn test_simulation_ryuukyoku() -> Result<()> {
        let mut game_state: GameStateT = Default::default();
        let mut play_log = PlayLog::new();
        game_state.create(b"test", 1, &mut play_log);
        game_state.shuffle();
        // 重複なしモード（taku_cursol を使用）に設定
        game_state.is_non_duplicate = true;
        game_state.start(&mut play_log);

        // 山を最後まで進める
        game_state.taku_cursol = game_state.taku.length;

        let mut output = Cursor::new(Vec::new());
        run_simulation(&mut game_state, &mut play_log, &mut output)?;

        let output_str = String::from_utf8(output.into_inner())?;
        assert!(output_str.contains("RYUUKYOKU"));
        Ok(())
    }

    #[test]
    fn test_simulation_tsumo_agari() -> Result<()> {
        let mut game_state: GameStateT = Default::default();
        let mut play_log = PlayLog::new();
        game_state.create(b"test", 1, &mut play_log);

        // テンパイ済みの手牌をロード (1112223334445m)
        // 5m(ID: 16)を引けばアガリ
        let tehai_ids: Vec<u32> = vec![
            0, 1, 2, // 1m
            4, 5, 6, // 2m
            8, 9, 10, // 3m
            12, 13, 14, // 4m
            16, // 5m
        ];
        let mut full_wall = vec![0u32; 136];
        // 1プレイヤーの場合、配牌はインデックス 14 から始まる
        full_wall[14..14 + tehai_ids.len()].copy_from_slice(&tehai_ids);
        // 次に引く牌（ツモ）はインデックス 27
        full_wall[27] = 17; // 5m (ID 16-19 は 5m)

        game_state.load(&full_wall);
        game_state.start(&mut play_log);

        let mut output = Cursor::new(Vec::new());
        run_simulation(&mut game_state, &mut play_log, &mut output)?;

        let output_str = String::from_utf8(output.into_inner())?;
        assert!(output_str.contains("TSUMO_AGARI"));
        Ok(())
    }

    #[test]
    fn test_tsumogiri_bit() -> Result<()> {
        let mut game_state: GameStateT = Default::default();
        let mut play_log = PlayLog::new();
        game_state.create(b"test", 1, &mut play_log);

        // 適当な手牌をロード
        let mut full_wall = vec![0u32; 136];
        for (i, val) in full_wall.iter_mut().enumerate().take(13) {
            *val = i as u32;
        }
        // 次に引く牌を特定
        full_wall[13] = 30; // 北

        game_state.load(&full_wall);
        game_state.start(&mut play_log);

        // ツモ
        game_state.tsumo(&mut play_log)?;

        // ツモ牌を捨てる (ツモ切り)
        game_state.action(&mut play_log, ActionType::ACTION_SUTEHAI, 0, 13)?;

        let player = &game_state.players[0];
        assert!(player.kawahai_len > 0);
        let last_discard = &player.kawahai[player.kawahai_len as usize - 1];
        assert!(last_discard.is_tsumogiri);

        Ok(())
    }
}
