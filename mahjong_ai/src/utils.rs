use mahjong_core::mahjong_generated::open_mahjong::{GameStateT};

pub fn paidistance(tehai_counts: &[u8; 34], target: usize) -> i32 {
    let target = target as i32;
    let mut min_dist = 100;

    if target >= 27 {
        if tehai_counts[target as usize] > 0 {
            return 0;
        } else {
            return -1;
        }
    }

    let suit = target / 9;
    let start = suit * 9;
    let end = start + 9;

    let mut found = false;
    for i in start..end {
        if tehai_counts[i as usize] > 0 {
            let dist = (target - i).abs();
            if dist < min_dist {
                min_dist = dist;
            }
            found = true;
        }
    }

    if found {
        min_dist
    } else {
        -1
    }
}

static DIST_COEF: [f64; 10] = [
    0.988, 0.990, 0.999, 0.998, 0.994, 0.997, 0.993, 0.993, 0.993, 0.993,
];

static KIND_COEF: [f64; 5] = [0.997, 0.998, 0.999, 0.996, 0.995];

pub fn get_dist_coef(dist: i32) -> f64 {
    let idx = dist + 1;
    if idx >= 0 && idx < 10 {
        DIST_COEF[idx as usize]
    } else {
        1.0
    }
}

pub fn get_kind_coef(game_state: &GameStateT, pai: usize) -> f64 {
    let pai = pai as i32;
    if pai >= 31 {
        KIND_COEF[3]
    } else if pai >= 27 {
        // Winds
        // We use game_state.teban as my seat index (0..3)
        // This seems to be the convention in mahjong_core
        let my_seat = game_state.teban as i32;
        let bakaze = game_state.bakaze as i32;
        let oya = game_state.oya as i32;

        let zikaze = (my_seat - oya + 4) % 4; // 0=East, 1=South, 2=West, 3=North
        let pai_wind = pai - 27;

        if pai_wind == bakaze || pai_wind == zikaze {
            KIND_COEF[3] // Value Wind
        } else {
            KIND_COEF[4] // Guest Wind
        }
    } else {
        match pai % 9 {
            0 | 8 => KIND_COEF[0],
            1 | 7 => KIND_COEF[1],
            _ => KIND_COEF[2],
        }
    }
}
