#[derive(Debug, Clone)]
pub enum Message {
    Start,
    Dahai(usize),
    Tsumo,
    ToggleRiichi(bool),
    FontLoaded,
    ShowModal(String),
    HideModal,
    SelectAI(String),
    AICommand(u32),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AppState {
    Created,
    Started,
    Ended,
}
