use iced::Command;
use ai_bridge::interface::{MJPInterfaceFuncP, G_STATE};
use ai_bridge::bindings::{MJPI_SUTEHAI, MJPIR_SUTEHAI};
use crate::types::Message;
use std::ffi::c_void;
use std::thread::sleep;
use std::time::Duration;

pub trait Agent: Send + Sync {
    fn name(&self) -> String;
    fn decide(&mut self, player_idx: usize) -> Command<Message>;
    fn as_any(&self) -> &dyn std::any::Any;
}

#[derive(Clone)]
pub struct DllAgent {
    pub name: String,
    pub symbol: MJPInterfaceFuncP,
    pub inst: *mut c_void,
}

unsafe impl Send for DllAgent {}
unsafe impl Sync for DllAgent {}

impl Agent for DllAgent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn decide(&mut self, _player_idx: usize) -> Command<Message> {
        let symbol = self.symbol;
        let inst = self.inst;

        let tsumohai_num = unsafe {
            let state = &G_STATE;
            let p = &state.players[state.teban as usize];
            if p.is_tsumo {
                p.tsumohai.pai_num as usize
            } else {
                0
            }
        };

        let inst_ptr = inst as usize;

        Command::perform(async move {
            let inst = inst_ptr as *mut c_void;
            sleep(Duration::from_millis(100));
            (symbol)(inst, MJPI_SUTEHAI.try_into().unwrap(), tsumohai_num, 0)
                .try_into()
                .unwrap()
        }, |r| Message::AICommand(r))
    }
}

pub struct BuiltInAgent;

impl Agent for BuiltInAgent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn name(&self) -> String {
        "BuiltIn".to_string()
    }

    fn decide(&mut self, player_idx: usize) -> Command<Message> {
        let (is_tsumo, tehai_len) = unsafe {
            let p = &G_STATE.players[player_idx];
            (p.is_tsumo, p.tehai_len)
        };

        Command::perform(async move {
            sleep(Duration::from_millis(500));
            let index = if is_tsumo { 13 } else {
                tehai_len
            };

            (index as u32) | (MJPIR_SUTEHAI as u32)
        }, |r| Message::AICommand(r))
    }
}
