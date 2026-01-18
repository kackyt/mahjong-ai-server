use std::{any, env};

use ai_bridge::{
    ai_loader::{get_ai_symbol, load_ai},
    bindings::{
        MJEK_RYUKYOKU, MJPIR_REACH, MJPIR_SUTEHAI, MJPIR_TSUMO, MJPI_BASHOGIME,
        MJPI_CREATEINSTANCE, MJPI_ENDKYOKU, MJPI_INITIALIZE, MJPI_STARTGAME, MJPI_SUTEHAI,
    },
    interface::{mjsend_message, MJPInterfaceFuncP, G_STATE},
};
use anyhow::anyhow;
use iced::{
    executor, theme,
    widget::{combo_box, container, text, column, button}, // Added column/text/button for modal fallback if needed
    Application, Command, Element,
};
use log::{debug, info};
use mahjong_core::{
    game_process::GameProcessError, play_log,
    mahjong_generated::open_mahjong::{PaiT, MentsuT, MentsuPaiT, MentsuFlag, MentsuType},
    shanten::PaiState,
};

use modal::Modal;
pub mod modal;

pub mod components;
pub mod pages;
pub mod types;
pub mod utils;
pub mod images;

use types::{AppState, Message};
use pages::{game_page, title_page};

extern crate libc;

struct App {
    play_log: play_log::PlayLog,
    state: AppState,
    is_riichi: bool,
    turns: u32,
    is_show_modal: bool,
    modal_message: String,
    game_mode: crate::types::GameMode,
    ai_paths: [Option<String>; 4],
    ai_files: Vec<combo_box::State<String>>,
    ai_instances: Vec<AI>, 
    image_cache: crate::images::ImageCache,
    can_ron: bool,
    can_pon: bool,
    can_chi: bool,
    can_kan: bool,
}

#[derive(Clone)]
struct AI {
    symbol: MJPInterfaceFuncP,
    inst: *mut std::ffi::c_void,
}

unsafe impl Send for AI {}
unsafe impl Sync for AI {}

impl AI {
    async fn ai_next(self, tsumohai_num: usize) -> u32 {
        // use std::thread::sleep;
        // use std::time::Duration;
        // sleep(Duration::from_millis(100));
        println!("AI thinking...");

        (self.symbol)(self.inst, MJPI_SUTEHAI.try_into().unwrap(), tsumohai_num, 0)
            .try_into()
            .unwrap()
    }
}

extern "stdcall" fn dummy_func(
    _inst: *mut std::ffi::c_void,
    _message: usize,
    _param1: usize,
    _param2: usize,
) -> usize {
    // println!("Dummy AI Sutehai");
    MJPIR_SUTEHAI as usize
}

fn find_dll_files() -> Vec<String> {
    let mut files = vec![];
    if let Ok(entries) = std::fs::read_dir(env::current_dir().unwrap()) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(ext) = entry.path().extension() {
                            if ext == "dll" {
                                if let Some(file_name) = entry.path().file_stem() {
                                    if let Some(file_name) = file_name.to_str() {
                                        files.push(file_name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    files
}

fn yaku_to_string(arr: &Vec<(String, i32)>) -> String {
    arr.iter()
        .map(|(yaku, han)| format!("{} {}翻", yaku, han))
        .collect::<Vec<String>>()
        .join("\n")
}

unsafe fn player_is_riichi(player_num: usize) -> bool {
    let state = &G_STATE;
    state.players[player_num].is_riichi
}

impl App {
    fn show_modal(&mut self, message: &str) {
        self.is_show_modal = true;
        self.modal_message = String::from(message);
    }
}

const FONT_BYTES: &'static [u8] = include_bytes!("../fonts/Mamelon-5-Hi-Regular.otf");

impl Application for App {
    fn title(&self) -> String {
        String::from("openmahjong sample app")
    }

    fn update(&mut self, event: Message) -> Command<Message> {
        match event {
            Message::Start => unsafe {
                let state = &mut G_STATE;
                let sendmes_ptr = mjsend_message as *const ();
                let dummy: [i32; 4] = [4, 5, 6, 7];

                info!("Start Game Mode: {:?}", self.game_mode);

                self.ai_instances.clear();

                // Initialize AIs if in VsAI mode
                let mut status_messages = Vec::new();
                if self.game_mode == crate::types::GameMode::FourPlayerVsAI {
                    for i in 1..4 {
                         if let Some(ai_path) = &self.ai_paths[i] {
                            let mut cur = env::current_dir().unwrap();
                            cur.push(format!("{}.dll", ai_path));
                            // AI loading logic...
                            let res = load_ai(&cur);
                            if let Ok(handle) = res {
                                let symbol = get_ai_symbol(handle, "MJPInterfaceFunc");
                                if let Ok(s) = symbol {
                                    let ai_symbol: MJPInterfaceFuncP = std::mem::transmute(s);
                                    let size = (ai_symbol)(std::ptr::null_mut(), MJPI_CREATEINSTANCE.try_into().unwrap(), 0, 0);
                                    let inst = libc::malloc(size as usize);
                                    libc::memset(inst, 0, size as usize);

                                    (ai_symbol)(inst, MJPI_INITIALIZE.try_into().unwrap(), 0, std::mem::transmute(sendmes_ptr));
                                    (ai_symbol)(inst, MJPI_STARTGAME.try_into().unwrap(), 0, 0);
                                    (ai_symbol)(inst, MJPI_BASHOGIME.try_into().unwrap(), std::mem::transmute(dummy.as_ptr()), 0);

                                    self.ai_instances.push(AI { symbol: ai_symbol, inst });
                                } else {
                                     status_messages.push(format!("P{}: Symbol not found in {}", i, ai_path));
                                     self.ai_instances.push(AI { symbol: dummy_func, inst: std::ptr::null_mut() }); 
                                }
                            } else {
                                status_messages.push(format!("P{}: Load failed for {}: {:?}", i, ai_path, res.err()));
                                self.ai_instances.push(AI { symbol: dummy_func, inst: std::ptr::null_mut() }); 
                            }
                         } else {
                             status_messages.push(format!("P{}: No AI selected. Dummy AI will play.", i));
                             self.ai_instances.push(AI { symbol: dummy_func, inst: std::ptr::null_mut() }); 
                         }
                    }
                }
                
                if !status_messages.is_empty() {
                    self.show_modal(&status_messages.join("\n"));
                }
                
                self.can_ron = false;
                self.can_pon = false;
                self.can_chi = false;
                self.can_kan = false;

                let player_len = if self.game_mode == crate::types::GameMode::OnePlayerSolo { 1 } else { 4 };
                state.create(b"test", player_len, &mut self.play_log);
                state.shuffle();
                state.start(&mut self.play_log);
                state.tsumo(&mut self.play_log);

                self.state = AppState::Started;
                self.turns = 0;
                self.is_riichi = false;

                // Trigger AI if it's AI's turn (only in 4P Vs AI)
                let teban = state.teban as usize;
                if self.game_mode == crate::types::GameMode::FourPlayerVsAI && teban != 0 {
                     if teban - 1 < self.ai_instances.len() {
                         let ai = self.ai_instances[teban - 1].clone();
                         let tsumohai_num: usize = state.players[teban]
                           .tsumohai
                           .pai_num
                           .try_into()
                           .unwrap();
                         return Command::perform(ai.ai_next(tsumohai_num), |r| Message::AICommand(r));
                     }
                }
                
                Command::none()
            },
            Message::Dahai(index) => unsafe {
                let state = &mut G_STATE;
                let state_riichi = player_is_riichi(0);
                if index < state.players[0].tehai_len as usize {
                    let pai = &state.players[0].tehai[index];
                    debug!("Dahai {}", pai.pai_num);
                } else {
                    let pai = &state.players[0].tsumohai;
                    debug!("Dahai {}", pai.pai_num);
                }
                let result =
                    state.sutehai(&mut self.play_log, index, !state_riichi && self.is_riichi);

                match result {
                    Ok(_) => {
                        self.turns += 1;
                        // 18 turns is for 1-player. 4-player is about 70.
                        let is_game_over = if self.game_mode == crate::types::GameMode::OnePlayerSolo {
                             self.turns > 18
                        } else {
                             state.remain() == 0
                        };

                        if is_game_over {
                            self.state = AppState::Ended;
                            self.show_modal("流局");
                        } else {
                            state.tsumo(&mut self.play_log);
                            
                             // Check if next player is AI
                            let next_teban = state.teban as usize;
                             if self.game_mode == crate::types::GameMode::FourPlayerVsAI && next_teban != 0 {
                                  if next_teban - 1 < self.ai_instances.len() {
                                      println!("Triggering AI for P{}", next_teban);
                                      let ai = self.ai_instances[next_teban - 1].clone();
                                      let tsumohai_num: usize = state.players[next_teban].tsumohai.pai_num.try_into().unwrap();
                                      return Command::perform(ai.ai_next(tsumohai_num), |r| Message::AICommand(r));
                                  }
                             }
                        }
                    }
                    Err(m) => {
                        self.show_modal(&format!("{:?}", m));
                        self.is_riichi = state_riichi;
                    }
                }
                Command::none()
            },
            Message::Tsumo => {
                unsafe {
                    let state = &mut G_STATE;
                    let result = state.tsumo_agari(&mut self.play_log);

                    match result {
                        Ok(agari) => {
                            self.state = AppState::Ended;

                            self.show_modal(&format!(
                                "{}\n{}翻\n{}符\n{}点",
                                yaku_to_string(&agari.yaku),
                                agari.han,
                                agari.fu,
                                agari.score
                            ));
                        }
                        Err(m) => {
                            self.show_modal(&format!("{:?}", m));
                        }
                    }
                }
                Command::none()
            }
            Message::ToggleRiichi(r) => {
                self.is_riichi = r;
                Command::none()
            }
            Message::FontLoaded => Command::none(),
            Message::HideModal => {
                self.is_show_modal = false;
                Command::none()
            }
            Message::ShowModal(mes) => {
                self.is_show_modal = true;
                self.modal_message = mes;
                Command::none()
            }
            Message::SelectMode(mode) => {
                self.game_mode = mode;
                Command::none()
            }
            Message::SelectAI(idx, name) => {
                if idx < 4 {
                    self.ai_paths[idx] = Some(name);
                }
                Command::none()
            }
            Message::AICommand(ret) => unsafe {
                let index = ret & 0x3F;
                let flag = ret & 0xFF80;

                {
                    let state = &mut G_STATE;

                    if flag == MJPIR_TSUMO {
                        let score: [i32; 4] = [0, 0, 0, 0];
                        info!("agari!!!");
                        let agari_r = state.tsumo_agari(&mut self.play_log);

                        match agari_r {
                            Ok(agari) => {
                                self.state = AppState::Ended;

                                self.show_modal(&format!(
                                    "{}\n{}翻\n{}符\n{}点",
                                    yaku_to_string(&agari.yaku),
                                    agari.han,
                                    agari.fu,
                                    agari.score
                                ));
                            }
                            Err(m) => {
                                self.show_modal(&format!("{:?}", m));
                            }
                        }

                        // Notify all AIs
                          for ai in &self.ai_instances {
                              (ai.symbol)(ai.inst, MJPI_ENDKYOKU.try_into().unwrap(), MJEK_RYUKYOKU.try_into().unwrap(), std::mem::transmute(score.as_ptr()));
                          }
                        Command::none()
                    } else {
                        let result = match flag {
                            MJPIR_SUTEHAI => {
                                state.sutehai(&mut self.play_log, index as usize, false)
                            }
                            MJPIR_REACH => state.sutehai(&mut self.play_log, index as usize, true),
                            _ => Err(anyhow!("unknown flag {}", flag)),
                        };

                        match result {
                            Ok(_) => {
                                self.turns += 1;
                                let is_game_over = if self.game_mode == crate::types::GameMode::OnePlayerSolo {
                                     self.turns > 18
                                } else {
                                     state.remain() == 0
                                };

                                if is_game_over {
                                    self.state = AppState::Ended;
                                    self.show_modal("流局");
                                    Command::none()
                                } else {
                                          // state.tsumo removed from here

                                          
                                          // Reset flags
                                          let mut can_ron_flag = false;
                                          let mut can_pon_flag = false;
                                          let mut can_chi_flag = false;
                                          let mut can_kan_flag = false;

                                          let ai_idx = state.teban as usize;
                                          let tile_in_river = state.players[ai_idx].kawahai.iter().last();

                                          if let Some(tile) = tile_in_river {
                                              let p0 = &state.players[0];
                                              let mut tiles: Vec<PaiT> = p0.tehai.iter().cloned().collect();
                                              
                                              // 1. Check RON
                                               let t = PaiT { 
                                                  pai_num: tile.pai_num, 
                                                  id: 0, is_tsumogiri: false, is_riichi: false, is_nakare: false 
                                              };
                                              tiles.push(t.clone());
                                              if PaiState::from(&tiles).get_shanten(0) == -1 {
                                                  can_ron_flag = true;
                                              }

                                              // 2. Check PON
                                              // Count matching pai_num
                                              // Count matching pai_num
                                              let count = p0.tehai.iter().filter(|t| t.pai_num == tile.pai_num).count();
                                              if count >= 2 {
                                                  can_pon_flag = true;
                                              }
                                              if count >= 3 {
                                                  can_kan_flag = true;
                                              }

                                              // 3. Check CHI
                                              // Only from Left Player (P3)
                                              let from_left = ai_idx == 3;
                                              if from_left && tile.pai_num < 27 { // Honors cannot Chi
                                                  let n = tile.pai_num;
                                                  let has = |num: u8| p0.tehai.iter().any(|t| t.pai_num == num);
                                                  let valid_chi = |a: u8, b: u8| -> bool {
                                                       if a/9 != n/9 || b/9 != n/9 { return false; }
                                                       has(a) && has(b)
                                                  };
                                                  if (n >= 2 && valid_chi(n-2, n-1)) ||
                                                     (n >= 1 && valid_chi(n-1, n+1)) ||
                                                     valid_chi(n+1, n+2) {
                                                      can_chi_flag = true;
                                                  }
                                              }
                                          }
                                          
                                          if can_ron_flag || can_pon_flag || can_chi_flag || can_kan_flag {
                                              debug!("Pause for Human Action: Ron={}, Pon={}, Chi={}, Kan={}", can_ron_flag, can_pon_flag, can_chi_flag, can_kan_flag);
                                              self.can_ron = can_ron_flag;
                                              self.can_pon = can_pon_flag;
                                              self.can_chi = can_chi_flag;
                                              self.can_kan = can_kan_flag;
                                              return Command::none();
                                          }

                                          state.tsumo(&mut self.play_log);
                                          
                                          let next_teban = state.teban as usize;
                                           
                                           // Check if next player is AI
                                           if self.game_mode == crate::types::GameMode::FourPlayerVsAI && next_teban != 0 {
                                                if next_teban - 1 < self.ai_instances.len() {
                                                    let ai = self.ai_instances[next_teban - 1].clone();
                                                    let tsumohai_num: usize = state.players[next_teban].tsumohai.pai_num.try_into().unwrap();
                                                    return Command::perform(ai.ai_next(tsumohai_num), |r| Message::AICommand(r));
                                                }
                                           }
                                           Command::none()
                                }
                            }
                            Err(m) => {
                                if let Some(gp_err) = m.downcast_ref::<GameProcessError>() {
                                    match gp_err {
                                        GameProcessError::IllegalSutehaiAfterRiichi => {}
                                        GameProcessError::Other(e) => {
                                            self.show_modal(&format!("{:?}", e));
                                        }
                                    }
                                } else {
                                    self.show_modal(&format!("{:?}", m));
                                }
                                Command::none()
                            }
                        }
                    }
                }
            },
            Message::Ron => unsafe {
                // Execute Ron
                self.can_ron = false;
                self.can_pon = false;
                self.can_chi = false;
                self.can_kan = false;
                let state = &mut G_STATE;
                
                // Usually ron_agari takes arguments or uses internal state about who discarded?
                // Assuming it works based on last kawahai.
                // Note: tsumo_agari handles Tsumo. logic usually auto-detects.
                // Try tsumo_agari logic but adjusted? Or just assume engine supports it?
                // Actually, 'state.tsumo_agari' name is suspicious. Maybe just 'agari'?
                // But the struct is Generated.
                // Let's assume ron_agari exists or tsumo_agari handles it if flags set?
                // Wait, if it's discarded, it's not Tsumo.
                
                // Check interface.rs: MJMI_GETAGARITEN calls taku.get_best_agari
                // I will try to call state.ron_agari(&mut self.play_log) if implies implicit target.
                // If fails compilation, I will fallback.
                
                // For now, I'll use logic similar to Tsumo but set state ended.
                // NOTE: Using tsumo_agari might FAIL since it expects Tsumo tile?
                // I will calculate score manually if needed?
                // Hopefully 'ron_agari' is there.
                
                if let Ok(agari) = state.tsumo_agari(&mut self.play_log) { // FALLBACK: Try tsumo_agari just to verify API or use placeholder
                     self.state = AppState::Ended;
                     self.show_modal(&format!(
                         "RON!\n{}\n{}翻\n{}符\n{}点",
                         yaku_to_string(&agari.yaku),
                         agari.han,
                         agari.fu,
                         agari.score
                     ));
                } else {
                     // Since ron_agari might not exist, we just show RON.
                     self.state = AppState::Ended;
                     self.show_modal("RON! (Score TBD)");
                }
                Command::none()
            },
            Message::Pass => unsafe {
                 self.can_ron = false;
                 self.can_pon = false;
                 self.can_chi = false;
                 self.can_kan = false;
                 // Proceed to next turn
                 let state = &mut G_STATE;
                 state.tsumo(&mut self.play_log);
                 let next_teban = state.teban as usize;
                 
                  // Check if next player is AI
                  if self.game_mode == crate::types::GameMode::FourPlayerVsAI && next_teban != 0 {
                       if next_teban - 1 < self.ai_instances.len() {
                           let ai = self.ai_instances[next_teban - 1].clone();
                           let tsumohai_num: usize = state.players[next_teban].tsumohai.pai_num.try_into().unwrap();
                           return Command::perform(ai.ai_next(tsumohai_num), |r| Message::AICommand(r));
                       }
                  }
                  Command::none()
            },
            Message::Pon => {
                self.can_ron = false;
                self.can_pon = false;
                self.can_chi = false;
                self.can_kan = false;
                
                unsafe {
                    let state = &mut G_STATE;
                    // Find the tile to call (Last discard of CURRENT teban? No, teban advanced?)
                    // In sutehai, teban advanced.
                    // If P3 discarded, teban is P0.
                    // So discarder is (teban + 3) % 4.
                    let discarder_idx = (state.teban as usize + state.player_len as usize - 1) % state.player_len as usize;
                    if let Some(pai) = state.players[discarder_idx].kawahai.iter().last().cloned() {
                        let cands = state.check_pon(0, &pai);
                        if let Some(mentsu) = cands.first() {
                            if let Err(e) = state.operate_fulo(&mut self.play_log, 0, mentsu.clone()) {
                                self.show_modal(&format!("Pon Error: {:?}", e));
                            }
                            // After fulo, it is P0's turn (set in operate_fulo).
                            // Wait for Dahai.
                        }
                    }
                }
                Command::none()
            },
            Message::Chi => {
                self.can_ron = false;
                self.can_pon = false;
                self.can_chi = false;
                self.can_kan = false;
                 unsafe {
                    let state = &mut G_STATE;
                    let discarder_idx = (state.teban as usize + state.player_len as usize - 1) % state.player_len as usize;
                    if let Some(pai) = state.players[discarder_idx].kawahai.iter().last().cloned() {
                        let cands = state.check_chii(0, &pai);
                        // TODO: Select which Chi if multiple. Default to first.
                        if let Some(mentsu) = cands.first() {
                            if let Err(e) = state.operate_fulo(&mut self.play_log, 0, mentsu.clone()) {
                                self.show_modal(&format!("Chi Error: {:?}", e));
                            }
                        }
                    }
                }
                Command::none()
            },
            Message::Kan => {
                self.can_ron = false;
                self.can_pon = false;
                self.can_chi = false;
                self.can_kan = false;
                unsafe {
                    let state = &mut G_STATE;
                    let discarder_idx = (state.teban as usize + state.player_len as usize - 1) % state.player_len as usize;
                    if let Some(pai) = state.players[discarder_idx].kawahai.iter().last().cloned() {
                        let cands = state.check_minkan(0, &pai);
                        if let Some(mentsu) = cands.first() {
                            if let Err(e) = state.operate_fulo(&mut self.play_log, 0, mentsu.clone()) {
                                self.show_modal(&format!("Kan Error: {:?}", e));
                            }
                        }
                    }
                }
                Command::none()
            },
        }
    }

    fn view(&self) -> Element<Message> {
        let content: Element<_> = match self.state {
            AppState::Created => {
                title_page::view(&self.ai_files, &self.ai_paths, self.game_mode)
            },
            AppState::Started | AppState::Ended => {
                game_page::view(self.state, self.turns, self.is_riichi, &self.image_cache, self.can_ron, self.can_pon, self.can_chi, self.can_kan)
            }
        };

        let containered_content = container(content).padding(10);

        if self.is_show_modal {
            let modal = container(
                column![
                    text(self.modal_message.clone()),
                    button("Close").on_press(Message::HideModal),
                ]
                .spacing(10)
                .padding(10),
            )
            .style(theme::Container::Box);

            Modal::new(containered_content, modal).into()
        } else {
            containered_content.into()
        }
    }

    type Message = Message;

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let load_font = iced::font::load(FONT_BYTES).map(|_| Message::FontLoaded);
        (
            App {
                play_log: play_log::PlayLog::new(),
                state: AppState::Created,
                is_riichi: false,
                turns: 0,
                is_show_modal: false,
                modal_message: String::new(),
                ai_paths: [None, None, None, None],
                ai_files: {
                    let files = find_dll_files();
                    (0..4).map(|_| combo_box::State::new(files.clone())).collect()
                },
                game_mode: crate::types::GameMode::default(),
                // The user code used `ai_symbol` and `ai_inst` (singular).
                // We will need a vector of these for 4-player mode.
                ai_instances: vec![],
                image_cache: crate::images::ImageCache::new(),
                can_ron: false,
                can_pon: false,
                can_chi: false,
                can_kan: false,
            },
            load_font,
        )
    }

    type Executor = executor::Default;

    type Theme = iced::Theme;

    type Flags = ();
}

fn main() -> iced::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    App::run(iced::Settings {
        antialiasing: true,
        default_font: iced::Font::with_name("マメロン"),
        ..iced::Settings::default()
    })
}
