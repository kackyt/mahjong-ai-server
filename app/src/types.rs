#[derive(Debug, Clone)]
pub enum Message {
    Start,
    Dahai(usize),
    Tsumo,
    ToggleRiichi(bool),
    FontLoaded,
    ShowModal(String),
    HideModal,
    SelectAI(usize, String),
    AICommand(u32),
    SelectMode(GameMode),
    Ron,
    Pass,
    Pon,
    Chi,
    Kan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GameMode {
    #[default]
    FourPlayerVsAI,
    OnePlayerSolo,
}

impl std::fmt::Display for GameMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameMode::FourPlayerVsAI => write!(f, "4-Player (Vs AI)"),
            GameMode::OnePlayerSolo => write!(f, "1-Player (Solo)"),
        }
    }
}

impl GameMode {
    pub const ALL: [GameMode; 2] = [
        GameMode::FourPlayerVsAI,
        GameMode::OnePlayerSolo,
    ];
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AppState {
    Created,
    Started,
    Ended,
}
