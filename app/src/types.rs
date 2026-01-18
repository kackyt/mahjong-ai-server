use mahjong_core::mahjong_generated::open_mahjong::MentsuT;

#[derive(Debug, Clone)]
pub enum Message {
    Start,
    Dahai(usize),
    Tsumo,
    ToggleRiichi(bool),
    FontLoaded,
    ShowModal(String),
    HideModal,
    SelectAI(usize, String), // index, name
    AICommand(u32),
    SelectMode(bool), // true: 1P, false: 4P
    // Action messages
    Chii(usize), // index in candidates
    Pon(usize),
    Kan(usize),
    Ron,
    Pass,
}

#[derive(Clone, Debug)]
pub enum AppState {
    Created,
    Started,
    Ended(Option<mahjong_core::agari::Agari>),
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub is_1p_mode: bool,
    pub ai_names: [Option<String>; 4],
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            is_1p_mode: true,
            ai_names: [None, None, None, None],
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ActionState {
    pub ron_candidate: Option<mahjong_core::agari::Agari>,
    pub pon_candidates: Vec<MentsuT>,
    pub chii_candidates: Vec<MentsuT>,
    pub kan_candidates: Vec<MentsuT>, // Minkan
    pub ankan_candidates: Vec<MentsuT>,
    pub kakan_candidates: Vec<MentsuT>,
}

impl ActionState {
    pub fn has_any(&self) -> bool {
        self.ron_candidate.is_some() ||
        !self.pon_candidates.is_empty() ||
        !self.chii_candidates.is_empty() ||
        !self.kan_candidates.is_empty() ||
        !self.ankan_candidates.is_empty() ||
        !self.kakan_candidates.is_empty()
    }
}
