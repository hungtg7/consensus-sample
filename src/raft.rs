use slog::{info, debug, Logger};
use anyhow::Result;
use std::ops::{Deref, DerefMut};
use rand::{self, Rng};

use crate::config::Config;
use raftpb::proto::{ Message, MessageType };

/// A constant represents invalid id of raft.
pub const INVALID_ID: u64 = 0;


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

    /// Randomize election timeout
    /// will in range [min_election_timeout, max_election_timeout]
    randomized_election_timeout: usize,
    min_election_timeout: usize,
    max_election_timeout: usize,
    pub logger: Logger
}


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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


pub struct Raft {
    pub r: RaftCore,
    msg: Vec<Message>
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

        let mut r = Raft{
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
            },
            msg: Default::default(),
        };
        r.become_follower(r.term);
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

    pub fn become_follower(&mut self, term: u64) {
        self.reset_term(term);
        self.state = StateRole::Follower;
        info!(
            self.logger,
            "became follower at term {term}",
            term = self.term;
        );
    }

    pub fn become_candidate(&mut self) {
        assert_ne!(self.state, StateRole::Leader, "Can not transitted Leader -> Candidate");
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
    pub fn step(&mut self, msg: Message) {
        // Handle message term
        if msg.term == 0 {
            // Local message
        } else if msg.term > self.term {
            // TODO: continue on this
            if msg.msg_type == MessageType::MsgRequestVote as i32
                || msg.msg_type == MessageType::MsgRequestPreVote as i32
            {
                // let force = m.context == CAMPAIGN_TRANSFER;
                // let in_lease = self.leader_id != INVALID_ID
                //     && self.election_elapsed < self.election_timeout;
                // if !force && in_lease {
                //     // if a server receives RequestVote request within the minimum election
                //     // timeout of hearing from a current leader, it does not update its term
                //     // or grant its vote
                //     //
                //     // This is included in the 3rd concern for Joint Consensus, where if another
                //     // peer is removed from the cluster it may try to hold elections and disrupt
                //     // stability.
                //     info!(
                //         self.logger,
                //         "[logterm: {log_term}, index: {log_index}, vote: {vote}] ignored vote from \
                //          {from} [logterm: {msg_term}, index: {msg_index}]: lease is not expired",
                //         log_term = self.raft_log.last_term(),
                //         log_index = self.raft_log.last_index(),
                //         vote = self.vote,
                //         from = m.from,
                //         msg_term = m.log_term,
                //         msg_index = m.index;
                //         "term" => self.term,
                //         "remaining ticks" => self.election_timeout - self.election_elapsed,
                //         "msg type" => ?m.get_msg_type(),
                //     );
                //
                //     return Ok(());
                }
            }


        }
    }
}
