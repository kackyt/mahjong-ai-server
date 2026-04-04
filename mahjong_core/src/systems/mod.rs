pub mod agari;
pub mod fulo;
pub mod scoring;
pub mod sutehai;
pub mod tsumo;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::world::MahjongWorld;
    use crate::play_log::PlayLog;

    #[test]
    fn test_tsumo_to_sutehai() {
        let mut world = MahjongWorld::new(4);
        let mut play_log = PlayLog::new();
        assert!(tsumo::run_tsumo(&mut world, &mut play_log).is_ok());
        assert!(sutehai::run_sutehai(&mut world, &mut play_log, 0, false).is_ok());
    }
}
