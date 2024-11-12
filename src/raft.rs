use anyhow::{Ok, Result};
use rand::{self, Rng};
use slog::{debug, info, Logger};
use std::ops::{Deref, DerefMut};

use crate::config::Config;
use raftpb::proto::{Message, MessageType};

/// A constant represents invalid id of raft.
pub const INVALID_ID: u64 = 0;

#[doc(hidden)]
// CAMPAIGN_ELECTION represents a normal (time-based) election (the second phase
// of the election when Config.pre_vote is true).
#[doc(hidden)]
pub const CAMPAIGN_ELECTION: &[u8] = b"CampaignElection";
#[doc(hidden)]
// CAMPAIGN_TRANSFER represents the type of leader transfer.
#[doc(hidden)]
pub const CAMPAIGN_TRANSFER: &[u8] = b"CampaignTransfer";

// Vote result represents the outcome of a vote
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoteResult {
    Won,
    Pending,
    Lost,
}

fn new_message(to: u64, field_type: MessageType, from: Option<u64>) -> Message {
    let mut m = Message::default();
    m.to = to;
    if let Some(id) = from {
        m.from = id;
    }
    m.set_msg_type(field_type);
    m
}

/// contain raft core component
pub struct RaftCore {
    pub id: u64,
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

    /// Whether to check the quorum
    /// The quorum is True when this node believe the current leader still alive with the majority
    pub check_quorum: bool,

    /// Randomize election timeout
    /// will in range [min_election_timeout, max_election_timeout]
    randomized_election_timeout: usize,
    min_election_timeout: usize,
    max_election_timeout: usize,

    /// Ticks since it reached last electionTimeout when it is leader or candidate.
    /// Number of ticks since it reached last electionTimeout or received a
    /// valid message from current leader when it is a follower.
    pub election_elapsed: usize,

    pub logger: Logger,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StateRole {
    Follower,
    Candidate,
    Leader,
    PreCandidate,
}

// default start up server is Follower
impl Default for StateRole {
    fn default() -> Self {
        Self::Follower
    }
}

pub struct Raft {
    prs: 
    pub r: RaftCore,
    pub msg: Vec<Message>,
}

impl RaftCore {
    fn send(&mut self, m: Message, msgs: &mut Vec<Message>) {
        debug!(
            self.logger,
            "Sending from {from} to {to}",
            from = self.id,
            to = m.to;
            "msg" => ?m,
        );
        msgs.push(m);
    }
}

// allows you to use the dot operator (.) directly on your custom type to access the fields of the contained type.
impl Deref for Raft {
    type Target = RaftCore;

    fn deref(&self) -> &Self::Target {
        &self.r
    }
}

// DerefMut allow Raft to modify nested Deref RaftCore
impl DerefMut for Raft {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.r
    }
}

impl Raft {
    //pub fn new(log: Span) -> Result<Self> {
    pub fn new(conf: &Config, logger: &Logger) -> Result<Self> {
        conf.validate()?;

        let mut r = Raft {
            r: RaftCore {
                id: conf.id,
                term: Default::default(),
                vote: Default::default(),
                state: StateRole::default(),
                leader_id: Default::default(),
                election_timeout: conf.election_tick,
                heartbeat_timeout: conf.heartbeat_tick,
                randomized_election_timeout: Default::default(),
                min_election_timeout: conf.min_election_tick,
                max_election_timeout: conf.max_election_tick,
                logger: logger.clone(),
                election_elapsed: Default::default(),
                check_quorum: conf.check_quorum,
            },
            msg: Default::default(),
        };
        r.become_follower(r.term, INVALID_ID);
        info!(
            r.logger,
            "newRaft";
            "term" => r.term,
        );
        info!(
            r.logger,
            "newRaft";
            "term" => r.term,
        );
        Ok(r)
    }

    pub fn reset_term(&mut self, term: u64) {
        if self.term != term {
            self.term = term;
            self.vote = INVALID_ID;
        }
        self.leader_id = INVALID_ID;
        self.randomized_election_timeout()
    }

    pub fn randomized_election_timeout(&mut self) {
        let prev_timeout = self.randomized_election_timeout;
        let timeout =
            rand::thread_rng().gen_range(self.min_election_timeout..self.max_election_timeout);
        debug!(
            self.logger,
            "reset election timeout {prev_timeout} -> {timeout}",
            prev_timeout = prev_timeout,
            timeout = timeout,
        );
        self.randomized_election_timeout = timeout;
    }

    pub fn become_follower(&mut self, term: u64, leader_id: u64) {
        self.reset_term(term);
        self.state = StateRole::Follower;
        self.leader_id = leader_id;
        info!(
            self.logger,
            "became follower at term {term}",
            term = self.term;
        );
    }

    pub fn become_candidate(&mut self) {
        assert_ne!(
            self.state,
            StateRole::Leader,
            "Can not transitted Leader -> Candidate"
        );
        let term = self.term + 1;
        self.reset_term(term);
        let id = self.id;
        self.vote = id;
        self.state = StateRole::Candidate;
        info!(
            self.logger,
            "became candidate at term {term}",
            term = self.term;
        );
    }

    /// This function incharge of steps up or down of Raft node.
    /// Always call this steps when receive a message.
    /// When receve a message there are 3 cases
    /// 1/ msg.term == 0
    pub fn step(&mut self, msg: Message) -> Result<()> {
        // Handle message term
        if msg.term == 0 {
            // Local message
        } else if msg.term > self.term {
            if msg.msg_type() == MessageType::MsgRequestVote
                || msg.msg_type() == MessageType::MsgRequestPreVote
            {
                let force = msg.context == CAMPAIGN_TRANSFER;
                // a period of time during which it expects the leader to send heartbeats
                // onyly when
                // 1/ The node has a valid leader (self.leader_id != INVALID_ID).
                // 2/ The node is still within the timeout period for hearing from the leader (self.election_elapsed < self.election_timeout).
                let in_lease =
                    self.leader_id != INVALID_ID && self.election_elapsed < self.election_timeout;
                if in_lease & force {
                    // if a server receives RequestVote request within the minimum election
                    // timeout of hearing from a current leader, it does not update its term
                    // or grant its vote
                    //
                    // This is included in the 3rd concern for Joint Consensus, where if another
                    // peer is removed from the cluster it may try to hold elections and disrupt
                    // stability.
                    info!(
                        self.logger,
                        "[vote: {vote}] ignored vote from \
                         [logterm: {msg_term},]: lease is not expired",
                        vote = self.vote,
                        msg_term = msg.term;
                        "term" => self.term,
                        "remaining ticks" => self.election_timeout - self.election_elapsed,
                        "msg type" => ?msg.msg_type,
                    );

                    return Ok(());
                }
            }
            if msg.msg_type() == MessageType::MsgRequestPreVote
                || (msg.msg_type() == MessageType::MsgRequestPreVoteResponse && !msg.reject)
            {
                // For a pre-vote request:
                // Never change our term in response to a pre-vote request.
                //
                // For a pre-vote response with pre-vote granted:
                // We send pre-vote requests with a term in our future. If the
                // pre-vote is granted, we will increment our term when we get a
                // quorum. If it is not, the term comes from the node that
                // rejected our vote so we should become a follower at the new
                // term.
            } else {
                info!(
                    self.logger,
                    "received a message with higher term from {from}",
                    from = msg.from;
                    "term" => self.term,
                    "message_term" => msg.term,
                    "msg type" => ?msg.msg_type(),
                );
                if msg.msg_type() == MessageType::MsgHeartbeat {
                    self.become_follower(msg.term, msg.from);
                } else {
                    self.become_follower(msg.term, INVALID_ID);
                }
            }
        } else if msg.term < self.term {
            if self.check_quorum
                && msg.msg_type() == MessageType::MsgHeartbeat
                && msg.msg_type() == MessageType::MsgAppend
            {
                // Recevie msg from a leder with a lower term
                // THere are two posiple cases
                // 1/ Because of delay in network that make the msg arrive late
                // 2/ Because of network partition this node is advance increment the term
                // If check_quorum is True, this node must not increase the term, but sending to
                // the old leader
                let to_send = new_message(msg.from, MessageType::MsgAppendResponse, None);
                // TODO:
                // how the leader handle this
                self.r.send(to_send, &mut self.msg);
            }
        }
        // TODO: This case is not happend with one node example (else if m.get_msg_type() == MessageType::MsgRequestPreVote)
        else {
            // ignore other cases
            info!(
                self.logger,
                "ignored a message with lower term from {from}",
                from = msg.from;
                "term" => self.term,
                "msg type" => ?msg.msg_type(),
                "msg term" => msg.term
            );
            return Ok(());
        }

        match msg.msg_type() {
            MessageType::MsgHup => self.hup(false),
            _ => match self.state {
                StateRole::PreCandidate | StateRole::Candidate => self.step_candidate(msg)?,
                StateRole::Follower => self.step_follower(msg)?,
                StateRole::Leader => self.step_leader(msg)?,
            },
        }
        Ok(())
    }

    fn hup(&mut self, transfer_leader: bool) {
        if self.state == StateRole::Leader {
            info!(
                self.logger,
                "Already a leader";
            );
            return;
        }
        info!(
            self.logger,
            "starting a new election";
            "term" => self.term,
        );
        if transfer_leader {
            self.campaign(CAMPAIGN_TRANSFER);
        } else {
            self.campaign(CAMPAIGN_ELECTION);
        }
    }

    fn campaign(&mut self, campaign_type: &'static [u8]) {
        // TODO: campaign to be a leader
        self.become_candidate();
        let (vote_msg, term) = (MessageType::MsgRequestVote, self.term);
        let self_id = self.id;
        // TODO: implement poll
    }

    pub fn tick(&mut self) -> bool {
        match self.state {
            StateRole::Follower | StateRole::PreCandidate | StateRole::Candidate => {
                self.tick_election()
            }
            StateRole::Leader => self.tick_heartbeat(),
        }
    }

    pub fn tick_election(&mut self) -> bool {
        self.election_elapsed += 1;

        if !self.pass_election_timeout() {
            return false;
        }

        println!("\nstep y'aaaaaa\n");
        let m = new_message(INVALID_ID, MessageType::MsgHup, Some(self.id));
        let _ = self.step(m);
        true
    }

    // tick_heartbeat is run by leaders to send a MsgBeat after self.heartbeat_timeout.
    // Returns true to indicate that there will probably be some readiness need to be handled.
    fn tick_heartbeat(&mut self) -> bool {
        // TODO
        true
    }

    pub fn pass_election_timeout(&self) -> bool {
        self.election_elapsed >= self.randomized_election_timeout
    }

    fn step_candidate(&mut self, _msg: Message) -> Result<()> {
        Ok(())
    }
    fn step_leader(&mut self, _msg: Message) -> Result<()> {
        Ok(())
    }
    fn step_follower(&mut self, _msg: Message) -> Result<()> {
        Ok(())
    }

    // TODO: implement poll
    fn poll(&mut self, from: u64, m_t: MessageType, vote: bool) -> VoteResult {}
}
