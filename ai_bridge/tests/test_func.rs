#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use mahjong_core::{mahjong_generated::open_mahjong::GameStateT, shanten::PaiState};

    #[test]
    fn test_haipai_to_agari() {
        // loaddata
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/pais.txt");
        let pais = std::fs::read_to_string(path).unwrap();
        let pais_vec: Vec<u32> = pais.split(",").map(|s| s.parse().unwrap()).collect();
        let mut state: GameStateT = Default::default();
        
        let mut array: [u32; 136] = [0; 136];
        array[..pais_vec.len()].copy_from_slice(&pais_vec);

        {
            state.create(b"test", 1);
            state.load(&array);
            state.start();
            state.tsumo();
        }
        
        {
            {
                let player = &state.players[state.teban as usize];
                for p in &player.tehai {
                    print!("{}", p);
                }

                println!("\r");

                let shanten = PaiState::from(&player.tehai).get_shanten(0);

                assert_eq!(shanten, 1);
            }
        }
    }
}
