#[cfg(test)]
mod tests {
    use ai_bridge::{
        bindings::{MJITehai, MJITehai0, MJMI_GETMACHI, MJMI_GETTEHAI, MJMI_GETVISIBLEHAIS},
        interface::{mjsend_message, G_STATE},
    };
    use mahjong_core::{mahjong_generated::open_mahjong::GameStateT, play_log, shanten::PaiState};
    use std::{
        path::PathBuf,
        ptr::{null, null_mut},
    };

    #[test]
    fn test_haipai_to_agari() {
        // loaddata
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/pais.txt");
        let pais = std::fs::read_to_string(path).unwrap();
        let pais_vec: Vec<u32> = pais.split(",").map(|s| s.parse().unwrap()).collect();
        let mut play_log = play_log::PlayLog::new();

        let mut array: [u32; 136] = [0; 136];
        array[..pais_vec.len()].copy_from_slice(&pais_vec);

        unsafe {
            let state = &mut G_STATE;
            state.create(b"test", 1, &mut play_log);
            state.load(&array);
            state.start(&mut play_log);
            state.tsumo(&mut play_log);
        }

        {
            unsafe {
                let state = &G_STATE;
                let player = &state.players[state.teban as usize];
                for p in &player.tehai {
                    print!("{}", p);
                }

                println!("\r");

                let shanten = PaiState::from(&player.tehai).get_shanten(0);

                assert_eq!(shanten, 1);

                let mut tehai = MJITehai::default();

                mjsend_message(
                    null_mut(),
                    MJMI_GETTEHAI.try_into().unwrap(),
                    0,
                    std::mem::transmute(&mut tehai),
                );
                assert_eq!(tehai.tehai_max, 13);
            }

            unsafe {
                let state = &mut G_STATE;

                state.sutehai(&mut play_log, 8, false);
                state.tsumo(&mut play_log);

                let player = &state.players[state.teban as usize];
                for p in &player.tehai {
                    print!("{}", p);
                }

                println!("\r");

                let shanten = PaiState::from(&player.tehai).get_shanten(0);

                assert_eq!(shanten, 0);
            }

            unsafe {
                let mut machi = [0u32; 34];

                mjsend_message(
                    std::ptr::null_mut(),
                    MJMI_GETMACHI.try_into().unwrap(),
                    0,
                    std::mem::transmute(&mut machi),
                );

                assert_eq!(
                    machi,
                    [
                        0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 0, 0
                    ]
                );

                let visible_hais: Vec<usize> = (0..34)
                    .map(|num| {
                        mjsend_message(
                            std::ptr::null_mut(),
                            MJMI_GETVISIBLEHAIS.try_into().unwrap(),
                            num,
                            0,
                        )
                    })
                    .collect();

                assert_eq!(
                    visible_hais,
                    vec![
                        2, 1, 1, 3, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                        2, 1, 3, 0, 0, 0, 0, 0, 0
                    ]
                )
            }
            unsafe {
                let state = &mut G_STATE;

                state.sutehai(&mut play_log, 12, false);
                state.tsumo(&mut play_log);

                let player = &state.players[state.teban as usize];
                for p in &player.tehai {
                    print!("{}", p);
                }

                println!("\r");

                let shanten = PaiState::from(&player.tehai).get_shanten(0);

                assert_eq!(shanten, 0);

                let result = state.tsumo_agari(&mut play_log);

                assert!(result.is_ok());
            }
        }
    }

    #[test]
    fn test_agari_failcase() {
        let mut play_log = play_log::PlayLog::new();
        // loaddata
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/pais.txt");
        let pais = std::fs::read_to_string(path).unwrap();
        let pais_vec: Vec<u32> = pais.split(",").map(|s| s.parse().unwrap()).collect();

        let mut array: [u32; 136] = [0; 136];
        array[..pais_vec.len()].copy_from_slice(&pais_vec);

        unsafe {
            let mut state = GameStateT::default();
            state.create(b"test", 1, &mut play_log);
            state.load(&array);
            state.start(&mut play_log);
            state.tsumo(&mut play_log);
            let mut machi = [1u32; 34];

            mjsend_message(
                std::ptr::null_mut(),
                MJMI_GETMACHI.try_into().unwrap(),
                0,
                std::mem::transmute(&mut machi),
            );

            assert_eq!(
                machi,
                [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0
                ]
            );

            let result = state.tsumo_agari(&mut play_log);

            assert_eq!(result.is_err(), true);
        }
    }
}
