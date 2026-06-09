use anyhow::{Ok, Result};
use rand::{self, Rng};
use slog::{debug, info, Logger};
use std::ops::{Deref, DerefMut};

use crate::{confchange, config::Config, tracker::ProgressTracker};
use crate::storage::{RaftLog, Storage};
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
pub struct RaftCore<T: Storage> {
    pub id: u64,
    /// current election term
    pub term: u64,

    /// vote for node id
    pub vote: u64,

    pub raft_log: RaftLog<T>,

    /// Curent raft state
    pub state: StateRole,

    /// leader id
    pub leader_id: u64,

    /// if it doesn't receive message from leader
    /// it will timeout
    election_timeout: usize,
    heartbeat_timeout: usize,

    /// Whether to check the quorum
    pub check_quorum: bool,

    /// Randomize election timeout
    randomized_election_timeout: usize,
    min_election_timeout: usize,
    max_election_timeout: usize,

    /// Ticks since it reached last electionTimeout when it is leader or candidate.
    pub election_elapsed: usize,

    /// Ticks since it reached last heartbeatTimeout when it is leader.
    pub heartbeat_elapsed: usize,

    pub logger: Logger,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StateRole {
    Follower,
    Candidate,
    Leader,
    PreCandidate,
}

impl Default for StateRole {
    fn default() -> Self {
        Self::Follower
    }
}

pub struct Raft<T: Storage> {
    prs: ProgressTracker,
    pub r: RaftCore<T>,
    pub msg: Vec<Message>,
}

impl<T: Storage> RaftCore<T> {
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

impl<T: Storage> Deref for Raft<T> {
    type Target = RaftCore<T>;

    fn deref(&self) -> &Self::Target {
        &self.r
    }
}

impl<T: Storage> DerefMut for Raft<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.r
    }
}

impl<T: Storage> Raft<T> {
    pub fn new(conf: &Config, storage: T, logger: &Logger) -> Result<Self> {
        conf.validate()?;
        let raft_state = storage.initial_state()?;
        let conf_state = &raft_state.conf_state;

        let mut r = Raft {
            prs: Default::default(),
            r: RaftCore {
                id: conf.id,
                term: Default::default(),
                vote: Default::default(),
                raft_log: RaftLog::new(storage),
                state: StateRole::default(),
                leader_id: Default::default(),
                election_timeout: conf.election_tick,
                heartbeat_timeout: conf.heartbeat_tick,
                randomized_election_timeout: Default::default(),
                min_election_timeout: conf.min_election_tick,
                max_election_timeout: conf.max_election_tick,
                logger: logger.clone(),
                election_elapsed: Default::default(),
                heartbeat_elapsed: Default::default(),
                check_quorum: conf.check_quorum,
            },
            msg: Default::default(),
        };
        confchange::restore::restore(&mut r.prs, r.r.raft_log.last_index(), conf_state)?;
        r.become_follower(r.term, INVALID_ID);
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
        self.election_elapsed = 0;
        info!(
            self.logger,
            "became follower at term {term}",
            term = self.term;
        );
    }

    pub fn become_leader(&mut self) {
        if self.state == StateRole::Follower {
            panic!("invalid transition [follower -> leader]");
        }
        let term = self.term;
        self.reset_term(term);
        self.leader_id = self.id;
        self.state = StateRole::Leader;
        
        // When becoming leader, reset heartbeat timer
        self.heartbeat_elapsed = 0;

        info!(
            self.logger,
            "became leader at term {term}",
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

    pub fn step(&mut self, msg: Message) -> Result<()> {
        if msg.term == 0 {
            // Local message
        } else if msg.term > self.term {
            if msg.msg_type() == MessageType::MsgRequestVote
                || msg.msg_type() == MessageType::MsgRequestPreVote
            {
                let force = msg.context == CAMPAIGN_TRANSFER;
                let in_lease =
                    self.leader_id != INVALID_ID && self.election_elapsed < self.election_timeout;
                if in_lease && !force {
                    info!(
                        self.logger,
                        "ignored vote request: lease is not expired";
                        "term" => self.term,
                        "msg term" => msg.term,
                    );
                    return Ok(());
                }
            }
            
            info!(
                self.logger,
                "received higher term from {from}",
                from = msg.from;
                "term" => self.term,
                "msg_term" => msg.term,
            );
            if msg.msg_type() == MessageType::MsgHeartbeat {
                self.become_follower(msg.term, msg.from);
            } else {
                self.become_follower(msg.term, INVALID_ID);
            }
        } else if msg.term < self.term {
            if (self.check_quorum || self.state == StateRole::Leader)
                && (msg.msg_type() == MessageType::MsgHeartbeat
                || msg.msg_type() == MessageType::MsgAppend)
            {
                // Respond to old leader with our higher term to make them step down
                let mut m = new_message(msg.from, MessageType::MsgAppendResponse, None);
                m.term = self.term;
                self.r.send(m, &mut self.msg);
                return Ok(());
            } else {
                return Ok(());
            }
        }

        match msg.msg_type() {
            MessageType::MsgHup => self.hup(false),
            MessageType::MsgRequestVote | MessageType::MsgRequestPreVote => {
                let can_vote = (self.vote == msg.from) || 
                               (self.vote == INVALID_ID && self.leader_id == INVALID_ID);
                if can_vote && self.raft_log.is_up_to_date(msg.index, msg.log_term) {
                    self.election_elapsed = 0;
                    self.vote = msg.from;
                    let mut m = new_message(msg.from, MessageType::MsgRequestVoteResponse, None);
                    m.term = self.term;
                    self.r.send(m, &mut self.msg);
                } else {
                    let mut m = new_message(msg.from, MessageType::MsgRequestVoteResponse, None);
                    m.term = self.term;
                    m.reject = true;
                    self.r.send(m, &mut self.msg);
                }
            }
            _ => match self.state {
                StateRole::Candidate | StateRole::PreCandidate => self.step_candidate(msg)?,
                StateRole::Follower => self.step_follower(msg)?,
                StateRole::Leader => self.step_leader(msg)?,
            },
        }
        Ok(())
    }

    fn hup(&mut self, transfer_leader: bool) {
        if self.state == StateRole::Leader {
            return;
        }
        if transfer_leader {
            self.campaign(CAMPAIGN_TRANSFER);
        } else {
            self.campaign(CAMPAIGN_ELECTION);
        }
    }

    fn campaign(&mut self, _campaign_type: &'static [u8]) {
        self.become_candidate();
        let self_id = self.id;
        if VoteResult::Won == self.poll(self_id, MessageType::MsgRequestVote, true) {
            return;
        }

        let last_index = self.raft_log.last_index();
        let last_term = self.raft_log.last_term();
        
        let ids: Vec<u64> = self.prs.voter_ids().into_iter().collect();
        for id in ids {
            if id == self_id {
                continue;
            }
            let mut m = new_message(id, MessageType::MsgRequestVote, Some(self_id));
            m.term = self.term;
            m.index = last_index;
            m.log_term = last_term;
            self.r.send(m, &mut self.msg);
        }
    }

    pub fn tick(&mut self) -> bool {
        match self.state {
            StateRole::Follower | StateRole::PreCandidate | StateRole::Candidate => {
                self.tick_election()
            }
            StateRole::Leader => self.tick_heartbeat(),
        }
    }

    fn tick_election(&mut self) -> bool {
        self.election_elapsed += 1;
        if self.election_elapsed >= self.randomized_election_timeout {
            self.election_elapsed = 0;
            let m = new_message(INVALID_ID, MessageType::MsgHup, Some(self.id));
            let _ = self.step(m);
            true
        } else {
            false
        }
    }

    fn tick_heartbeat(&mut self) -> bool {
        self.heartbeat_elapsed += 1;
        self.election_elapsed += 1;

        if self.election_elapsed >= self.election_timeout {
            self.election_elapsed = 0;
            if self.check_quorum {
                let m = new_message(INVALID_ID, MessageType::MsgCheckQuorum, Some(self.id));
                let _ = self.step(m);
            }
        }

        if self.state != StateRole::Leader {
            return false;
        }

        if self.heartbeat_elapsed >= self.heartbeat_timeout {
            self.heartbeat_elapsed = 0;
            let m = new_message(INVALID_ID, MessageType::MsgBeat, Some(self.id));
            let _ = self.step(m);
        }
        true
    }

    pub fn pass_election_timeout(&self) -> bool {
        self.election_elapsed >= self.randomized_election_timeout
    }

    fn step_candidate(&mut self, msg: Message) -> Result<()> {
        match msg.msg_type() {
            MessageType::MsgRequestVoteResponse | MessageType::MsgRequestPreVoteResponse => {
                self.poll(msg.from, msg.msg_type(), !msg.reject);
            }
            _ => (),
        }
        Ok(())
    }

    fn step_leader(&mut self, msg: Message) -> Result<()> {
        match msg.msg_type() {
            MessageType::MsgBeat => {
                self.bcast_heartbeat();
            }
            MessageType::MsgCheckQuorum => {
                if !self.prs.quorum_recently_active() {
                    let term = self.term;
                    info!(
                        self.logger,
                        "stepping down; lost quorum";
                        "term" => term,
                    );
                    self.become_follower(term, INVALID_ID);
                }
                self.prs.reset_recent_active();
            }
            MessageType::MsgHeartbeatResponse => {
                if let Some(pr) = self.prs.get_mut(msg.from) {
                    pr.recent_active = true;
                }
            }
            _ => (),
        }
        Ok(())
    }

    fn step_follower(&mut self, msg: Message) -> Result<()> {
        match msg.msg_type() {
            MessageType::MsgHeartbeat => {
                self.election_elapsed = 0;
                self.leader_id = msg.from;
                let mut m = new_message(msg.from, MessageType::MsgHeartbeatResponse, None);
                m.term = self.term;
                self.r.send(m, &mut self.msg);
            }
            _ => (),
        }
        Ok(())
    }

    fn bcast_heartbeat(&mut self) {
        let self_id = self.id;
        let ids: Vec<u64> = self.prs.voter_ids().into_iter().collect();
        for id in ids {
            if id == self_id {
                continue;
            }
            let mut m = new_message(id, MessageType::MsgHeartbeat, Some(self_id));
            m.term = self.term;
            self.r.send(m, &mut self.msg);
        }
    }

    fn poll(&mut self, from: u64, _m_t: MessageType, vote: bool) -> VoteResult {
        self.prs.record_vote(from, vote);
        let (gr, rj, res) = self.prs.tally_votes();
        info!(
            self.logger,
            "poll result";
            "from" => from,
            "vote" => vote,
            "approvals" => gr,
            "rejections" => rj,
            "result" => ?res,
        );

        match res {
            VoteResult::Won => {
                self.become_leader();
            }
            VoteResult::Lost => {
                let term = self.term;
                self.become_follower(term, INVALID_ID);
            }
            VoteResult::Pending => (),
        }
        res
    }
}
