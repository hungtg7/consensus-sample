mod progress;
mod state;

use getset::Getters;
use progress::Progress;
use std::collections::HashMap;

pub type ProgressMap = HashMap<u64, Progress>;
/// `ProgressTracker` contains several `Progress`es,
/// which could be `Leader`, `Follower` and `Learner`.
#[derive(Clone, Getters)]
pub struct ProgressTracker {
    progress: ProgressMap,

    /// current configuration state of the node.
    #[get = "pub"]
    // TODO: implement Configuration
    conf: Configuration,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Getters)]
pub struct Configuration {}
