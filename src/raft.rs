/// contain raft core component
pub struct RaftCore {
    /// id of this node
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
    pub election_timeout: usize,

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
