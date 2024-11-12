use std::fmt;
use std::fmt::{Display, Formatter};

/// The state of the progress.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum ProgressState {
    /// Whether it's probing.
    #[default]
    Probe,
    /// Whether it's replicating.
    Replicate,
    /// Whether it's a snapshot.
    Snapshot,
}

impl Display for ProgressState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ProgressState::Probe => write!(f, "StateProbe"),
            ProgressState::Replicate => write!(f, "StateReplicate"),
            ProgressState::Snapshot => write!(f, "StateSnapshot"),
        }
    }
}
