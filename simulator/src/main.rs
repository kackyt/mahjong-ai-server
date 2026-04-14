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
    #[arg(long, default_value_t = 1)]
    players: usize,
    #[arg(short, long, default_value = "simulator_log.jsonl")]
    log_file: String,
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

    let num_players = if args.players == 4 { 4 } else { 1 };
    game_state.create(title, num_players as u32, &mut play_log);

    if let Some(pai_list) = args.pai_list_file {
        let hai_ids = load_pailist(pai_list, args.index)?;
        game_state.load(&hai_ids);
    } else {
        game_state.shuffle();
    }

    game_state.start(&mut play_log);

    let mut log_file = File::create(&args.log_file)?;
    let mut turn = 0;

    loop {
        let current_player = game_state.teban as usize;

        // Next player draw
        let _ = game_state.tsumo(&mut play_log);

        let player = &game_state.players[current_player];

        let mut tehai: Vec<PaiT> = player
            .tehai
            .iter()
            .take(player.tehai_len as usize)
            .cloned()
            .collect();
        let tehai_nums: Vec<u8> = tehai.iter().map(|p| p.pai_num).collect();

        tehai.push(player.tsumohai.clone());

        let shanten = PaiState::from(&tehai).get_shanten(player.mentsu_len as usize);

        // Check for agari
        if shanten == -1 {
            let log_json = serde_json::to_string(&ActionLog {
                turn,
                player_idx: current_player,
                tehai: tehai_nums.clone(),
                tsumohai: player.tsumohai.pai_num,
                shanten,
                discard: player.tsumohai.pai_num,
                action_type: "TSUMO_AGARI".to_string(),
            })?;
            writeln!(log_file, "{}", log_json)?;
            println!("Agari on turn {} by player {}!", turn, current_player);
            break;
        }

        // Check for ryuukyoku (exhausted wall)
        if game_state.taku.length == game_state.taku_cursol {
            let log_json = serde_json::to_string(&ActionLog {
                turn,
                player_idx: current_player,
                tehai: tehai_nums.clone(),
                tsumohai: player.tsumohai.pai_num,
                shanten,
                discard: 255, // dummy
                action_type: "RYUUKYOKU".to_string(),
            })?;
            writeln!(log_file, "{}", log_json)?;
            println!("Ryuukyoku on turn {}!", turn);
            break;
        }

        // Use mahjong_ai to decide the discard
        let best_discard = match eval_sutehai(&game_state) {
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
            idx.unwrap_or(0)
        };

        let discard_action = if is_tsumogiri {
            (discard_idx as u32) | 0x8000
        } else {
            discard_idx as u32
        };

        let action_log = ActionLog {
            turn,
            player_idx: current_player,
            tehai: tehai_nums,
            tsumohai: player.tsumohai.pai_num,
            shanten,
            discard: best_discard as u8,
            action_type: "SUTEHAI".to_string(),
        };

        let log_json = serde_json::to_string(&action_log)?;
        writeln!(log_file, "{}", log_json)?;

        let _ = game_state.action(
            &mut play_log,
            ActionType::ACTION_SUTEHAI,
            current_player,
            discard_action,
        );

        turn += 1;
        if turn > 1000 {
            // fallback safety
            break;
        }
    }

    Ok(())
}
