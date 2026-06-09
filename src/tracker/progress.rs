use super::state::ProgressState;


#[derive(Clone)]
pub struct Progress {
    pub state: ProgressState,
    pub recent_active: bool,
}
