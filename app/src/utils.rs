use std::env;

pub fn painum2path(painum: u32) -> String {
    if painum < 9 {
        return format!(
            "{}/images/haiga/man{}.gif",
            env!("CARGO_MANIFEST_DIR"),
            painum + 1
        );
    }

    if painum < 18 {
        return format!(
            "{}/images/haiga/pin{}.gif",
            env!("CARGO_MANIFEST_DIR"),
            painum - 8
        );
    }

    if painum < 27 {
        return format!(
            "{}/images/haiga/sou{}.gif",
            env!("CARGO_MANIFEST_DIR"),
            painum - 17
        );
    }

    if painum < 34 {
        let zihai = ["ton", "nan", "sha", "pei", "haku", "hatu", "tyun"];
        return format!(
            "{}/images/haiga/{}.gif",
            env!("CARGO_MANIFEST_DIR"),
            zihai[(painum - 27) as usize]
        );
    }

    format!("{}/images/haiga/ura.gif", env!("CARGO_MANIFEST_DIR"))
}
