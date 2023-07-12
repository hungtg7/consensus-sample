pub struct RaftCore {
    // id of this node
    id: u64,

    // current election term
    pub term: u64,

    // vote for node id
    pub vote: u64,

    // Curent raft state
    pub state: StateRole,

    // leader id
    pub leader_id: u64,
}

pub enum StateRole {
    Follower,
    Candidate,
    Leader,
}

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
