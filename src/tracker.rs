mod progress;
mod state;

use crate::{quorum::joint::Configuration as JointConfig, raft::VoteResult};
use getset::Getters;
use progress::Progress;
use std::collections::{HashMap, HashSet};

pub type ProgressMap = HashMap<u64, Progress>;
/// `ProgressTracker` contains several `Progress`es,
/// which could be `Leader`, `Follower` and `Learner`.
#[derive(Clone, Getters)]
pub struct ProgressTracker {
    progress: ProgressMap,

    /// current configuration state of the node.
    #[get = "pub"]
    conf: Configuration,
    pub votes: HashMap<u64, bool>,
}

impl Default for ProgressTracker {
    fn default() -> Self {
        let voter = 0;
        let learner = 0;

        return ProgressTracker {
            progress: HashMap::with_capacity(voter + learner),
            conf: Configuration::with_capacity(voter, learner),
            votes: HashMap::with_capacity(voter),

        };
    }
}

impl ProgressTracker {
    /// Records that the node with the given id voted for this Raft
    /// instance if v == true (and declined it otherwise).
    pub fn record_vote(&mut self, id: u64, vote: bool) {
        self.votes.entry(id).or_insert(vote);
    }

    /// TallyVotes returns the number of granted and rejected Votes, and whether the
    /// election outcome is known.
    pub fn tally_votes(&self) -> (usize, usize, VoteResult) {
        // Make sure to populate granted/rejected correctly even if the Votes slice
        // contains members no longer part of the configuration. This doesn't really
        // matter in the way the numbers are used (they're informational), but might
        // as well get it right.
        let (mut granted, mut rejected) = (0, 0);
        for (id, vote) in &self.votes {
            if !self.conf.voters.contains(*id) {
                continue;
            }
            if *vote {
                granted += 1;
            } else {
                rejected += 1;
            }
        }
        let result = self.vote_result(&self.votes);
        (granted, rejected, result)
    }
    /// Returns the Candidate's eligibility in the current election.
    ///
    /// If it is still eligible, it should continue polling nodes and checking.
    /// Eventually, the election will result in this returning either `Elected`
    /// or `Ineligible`, meaning the election can be concluded.
    pub fn vote_result(&self, votes: &HashMap<u64, bool>) -> VoteResult {
        self.conf.voters.vote_result(|id| votes.get(&id).cloned())
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
    /// Create a new configuration with the given configuration.
    pub fn with_capacity(voters: usize, learners: usize) -> Self {
        Self {
            voters: JointConfig::with_capacity(voters),
            learners: HashSet::with_capacity(learners)
        }
    }
}
