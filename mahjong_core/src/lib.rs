#[cfg(feature = "std")]
pub mod agari;
#[cfg(feature = "std")]
pub mod fbs_utils;
#[cfg(feature = "std")]
pub mod game_process;
#[cfg(feature = "load-pailist")]
pub mod load_pailist;
pub mod mahjong_generated;
#[cfg(feature = "std")]
pub mod play_log;
pub mod shanten;

#[cfg(feature = "ecs")]
pub mod components;
#[cfg(feature = "ecs")]
pub mod systems;
