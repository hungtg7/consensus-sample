use slog::Logger;
use anyhow::Result;

use crate::config::Config;


/// contain raft core component
pub struct RaftCore {
    id: u64,
    /// current election term
    pub term: u64,

    /// vote for node id
    pub vote: u64,

    /// Curent raft state
    pub state: StateRole,

    /// leader id
    pub leader_id: u64,

    /// if it doesn't receive message from leader
    /// it will timeout
    /// election timeout must be greater than
    /// HeartbeatTick. We suggest election_timeout = 10 * heartbeat_timeout to avoid
    /// unnecessary leader switching
    election_timeout: usize,
    heartbeat_timeout: usize,

    /// Randomize election timeout
    /// will in range [min_election_timeout, max_election_timeout]
    randomized_election_timeout: usize,
    min_election_timeout: usize,
    max_election_timeout: usize,
}


pub enum StateRole {
    Follower,
    Candidate,
    Leader,
}

// default start up server is Follower
impl Default for StateRole {
    fn default() -> Self {
        Self::Follower
    }
}

// TODO: make it a RPC message
type Message = String;


pub struct Raft {
    core: RaftCore,
    msg: Vec<Message>
}

impl Raft {
    //pub fn new(log: Span) -> Result<Self> {
    pub fn new(conf: &Config, logger: Logger) -> Result<Self> {
        let id = conf.id;

        Ok(Raft{
            core: RaftCore { 
                id,
                term: Default::default(),
                vote: Default::default(),
                state: StateRole::default(),
                leader_id: Default::default(),
                election_timeout: conf.election_tick,
                heartbeat_timeout: conf.heartbeat_tick,
                randomized_election_timeout: Default::default(),
                min_election_timeout: conf.min_election_tick,
                max_election_timeout: conf.max_election_tick
            },
            msg: Default::default(),
        })

    }
}
