mod progress;
mod state;

use getset::Getters;
use progress::Progress;
use std::collections::{HashMap, HashSet};
use crate::quorum::joint::Configuration as JointConfig;

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

impl Default for ProgressTracker {
    fn default() -> Self {
        let voter = 0;
        let learner = 0;
        
        return ProgressTracker{
            progress: HashMap::new(),
            conf: Configuration::new(voter, learner)
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Getters)]
pub struct Configuration {
    pub voters: JointConfig,
    /// Learners is a set of IDs corresponding to the learners active in the
    /// current configuration.
    ///
    /// Invariant: Learners and Voters does not intersect, i.e. if a peer is in
    /// either half of the joint config, it can't be a learner; if it is a
    /// learner it can't be in either half of the joint config. This invariant
    /// simplifies the implementation since it allows peers to have clarity about
    /// its current role without taking into account joint consensus.
    pub learners: HashSet<u64>,

}

impl Configuration {
    fn new(voter: u64, learner: u64) -> Configuration {
        return Configuration{}
    }
    
}
