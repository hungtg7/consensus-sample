use crate::tracker::{ProgressTracker, Configuration};
use crate::errors::Result;
use raftpb::proto::{ConfChangeSingle, ConfChangeType};

/// Changer facilitates configuration changes.
pub struct Changer<'a> {
    pub tracker: &'a mut ProgressTracker,
}

impl<'a> Changer<'a> {
    pub fn new(tracker: &'a mut ProgressTracker) -> Self {
        Changer { tracker }
    }

    pub fn simple(&mut self, changes: &[ConfChangeSingle]) -> Result<(Configuration, Vec<(u64, u64)>)> {
        let mut cfg = self.tracker.conf().clone();
        let mut changes_out = Vec::new();
        for cc in changes {
            match cc.change_type() {
                ConfChangeType::AddNode => {
                    cfg.voters.incoming.voters.insert(cc.node_id);
                    cfg.learners.remove(&cc.node_id);
                }
                ConfChangeType::AddLearnerNode => {
                    cfg.voters.incoming.voters.remove(&cc.node_id);
                    cfg.voters.outgoing.voters.remove(&cc.node_id);
                    cfg.learners.insert(cc.node_id);
                }
                ConfChangeType::RemoveNode => {
                    cfg.voters.incoming.voters.remove(&cc.node_id);
                    cfg.voters.outgoing.voters.remove(&cc.node_id);
                    cfg.learners.remove(&cc.node_id);
                }
            }
            changes_out.push((cc.node_id, cc.node_id));
        }
        Ok((cfg, changes_out))
    }

    pub fn enter_joint(&mut self, _auto_leave: bool, changes: &[ConfChangeSingle]) -> Result<(Configuration, Vec<(u64, u64)>)> {
        let mut cfg = self.tracker.conf().clone();
        cfg.voters.outgoing = cfg.voters.incoming.clone();
        // apply changes to incoming
        for cc in changes {
            match cc.change_type() {
                ConfChangeType::AddNode => {
                    cfg.voters.incoming.voters.insert(cc.node_id);
                    cfg.learners.remove(&cc.node_id);
                }
                ConfChangeType::AddLearnerNode => {
                    cfg.voters.incoming.voters.remove(&cc.node_id);
                    cfg.learners.insert(cc.node_id);
                }
                ConfChangeType::RemoveNode => {
                    cfg.voters.incoming.voters.remove(&cc.node_id);
                    cfg.learners.remove(&cc.node_id);
                }
            }
        }
        let mut changes_out = Vec::new();
        for cc in changes {
            changes_out.push((cc.node_id, cc.node_id));
        }

        Ok((cfg, changes_out))
    }
}
