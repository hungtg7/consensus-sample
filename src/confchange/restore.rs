use super::changer::Changer;
use crate::tracker::ProgressTracker;
use crate::Result;
use raftpb::{ConfChangeSingle, ConfChangeType, ConfState};

/// Before 2 config state before joint transition
fn to_conf_change_single(cs: &ConfState) -> (Vec<ConfChangeSingle>, Vec<ConfChangeSingle>) {
    // Example to follow along this code:
    // voters=(1 2 3) learners=(5) outgoing=(1 2 4 6) learners_next=(4)
    //
    // This means that before entering the joint config, the configuration
    // had voters (intitial state) (1 2 4 6) and perhaps some learners that are already gone.
    // The new set of voters is (1 2 3), i.e. (1 2) were kept around, and (4 6)
    // are no longer voters; however 4 is poised to become a learner upon leaving
    // the joint state.
    // We can't tell whether 5 was a learner before entering the joint config,
    // but it doesn't matter (we'll pretend that it wasn't).
    //
    // The code below will construct
    // outgoing = add 1; add 2; add 4; add 6
    // incoming = remove 1; remove 2; remove 4; remove 6
    //            add 1;    add 2;    add 3;
    //            add-learner 5;
    //            add-learner 4;
    //
    // So, when starting with an empty config, after applying 'outgoing' we have
    //
    //   quorum=(1 2 4 6)
    //
    // From which we enter a joint state via 'incoming'
    //
    //   quorum=(1 2 3)&&(1 2 4 6) learners=(5) learners_next=(4)
    //
    // as desired.
    let mut incoming = Vec::new();
    let mut outgoing = Vec::new();
    for id in cs.get_voters_outgoing() {
        // If there are outgoing voters, first add them one by one so that the
        // (non-joint) config has them all.
        outgoing.push(raft_proto::new_conf_change_single(
            *id,
            ConfChangeType::AddNode,
        ));
    }

    // We're done constructing the outgoing slice, now on to the incoming one
    // (which will apply on top of the config created by the outgoing slice).

    // First, we'll remove all of the outgoing voters.
    for id in cs.get_voters_outgoing() {
        incoming.push(raft_proto::new_conf_change_single(
            *id,
            ConfChangeType::RemoveNode,
        ));
    }
    // Then we'll add the incoming voters and learners.
    for id in cs.get_voters() {
        incoming.push(raft_proto::new_conf_change_single(
            *id,
            ConfChangeType::AddNode,
        ));
    }
    for id in cs.get_learners() {
        incoming.push(raft_proto::new_conf_change_single(
            *id,
            ConfChangeType::AddLearnerNode,
        ));
    }
    // Same for LearnersNext; these are nodes we want to be learners but which
    // are currently voters in the outgoing config.
    for id in cs.get_learners_next() {
        incoming.push(raft_proto::new_conf_change_single(
            *id,
            ConfChangeType::AddLearnerNode,
        ));
    }
    (outgoing, incoming)
}
