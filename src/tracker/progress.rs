use super::state::ProgressState;


#[derive(Clone)]
pub struct Progress {
    /// There are 3 state
    ///
    /// Probes is for leader. Leader sends only at most 1 message per heartbeat interval
    /// Also, probe replca actual state
    ///
    /// When in ProgressStateReplicate, leader optimistically increases next
    /// to the latest entry sent after sending replication message. This is
    /// an optimized state for fast replicating log entries to the follower.
    ///
    /// When in ProgressStateSnapshot, leader should have sent out snapshot
    /// before and stop sending any replication message.
    pub state: ProgressState,
}
