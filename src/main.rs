use raftpb::proto::Message;
use slog::{info, o, Drain, Logger};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use tokio::{sync::mpsc, time::timeout};

mod config;
mod node;
mod raft;

use node::Node;

type ProposeCallback = Box<dyn Fn() + Send>;

enum Msg {
    Propose {
        id: u8,
        cb: ProposeCallback,
    },
    // Here we don't use Raft Message, so use dead_code to
    // avoid the compiler warning.
    #[allow(dead_code)]
    Raft(Message),
}

#[tokio::main]
async fn main() {
    // Log configuration
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = slog::Logger::root(drain, o!());

    let conf = config::Config {
        id: 1,
        heartbeat_tick: 15,
        election_tick: 20,
        min_election_tick: 25,
        max_election_tick: 30,
        check_quorum: false,
    };
    let mut node = Node::new(&conf, &logger).unwrap();

    let (sender, mut receiver) = mpsc::unbounded_channel();

    // Loop to control the Raft node
    let mut t = Instant::now();
    let mut r_timeout = Duration::from_millis(100);

    // Make another tokio task to make a raft request
    tokio::task::spawn(send_propose(logger, sender));

    // Use a HashMap to hold the `propose` callbacks.
    let mut cbs = HashMap::new();

    loop {
        match timeout(r_timeout, receiver.recv()).await {
            Ok(Some(Msg::Propose { id, cb })) => {
                cbs.insert(id, cb);
                // node.raft.propose(vec![], vec![id]).unwrap();
            }
            Ok(Some(Msg::Raft(m))) => node.raft.step(m).unwrap(),
            Err(_) => (),
            _ => ()
            // Err(RecvTimeoutError::Timeout) => (),
            // Err(RecvTimeoutError::Disconnected) => return,
        }
        let d = t.elapsed();
        t = Instant::now();
        if d >= r_timeout {
            r_timeout = Duration::from_millis(100);
            // We drive Raft every 100ms.
            node.tick();
        } else {
            r_timeout -= d;
        }
        // on_ready(&mut r, &mut cbs);
    }
}

async fn send_propose(logger: Logger, sender: mpsc::UnboundedSender<Msg>) {
    let _ = tokio::task::spawn(async move {
        // Wait some time and send the request to the Raft.
        println!("Hello, tokio!");

        tokio::time::sleep(Duration::from_secs(10)).await;

        let (s1, mut r1) = mpsc::unbounded_channel();

        info!(logger, "propose a request");

        // Send a command to the Raft, wait for the Raft to apply it
        // and get the result.
        sender
            .send(Msg::Propose {
                id: 1,
                cb: Box::new(move || {
                    s1.send(0).unwrap();
                }),
            })
            .unwrap();

        // TODO: in order to make recv we must trigger the callback in the main.rs store in cbs
        // hashmap
        let n = r1.recv().await.unwrap();
        info!(logger, "recv a request");
        assert_eq!(n, 0);

        info!(logger, "receive the propose callback");
    })
    .await;
}

// fn on_ready(raft_group: &mut Node, cbs: &mut HashMap<u8, ProposeCallback>) {
//     if !raft_group.has_ready() {
//         return;
//     }

//     // Get the `Ready` with `RawNode::ready` interface.
//     let mut ready = raft_group.ready();

//     let handle_messages = |msgs: Vec<Message>| {
//         for _msg in msgs {
//             // Send messages to other peers.
//         }
//     };

//     if !ready.messages().is_empty() {
//         // Send out the messages come from the node.
//         handle_messages(ready.take_messages());
//     }

//     if !ready.snapshot().is_empty() {
//         // This is a snapshot, we need to apply the snapshot at first.
//         store.wl().apply_snapshot(ready.snapshot().clone()).unwrap();
//     }

//     let mut _last_apply_index = 0;
//     let mut handle_committed_entries = |committed_entries: Vec<Entry>| {
//         for entry in committed_entries {
//             // Mostly, you need to save the last apply index to resume applying
//             // after restart. Here we just ignore this because we use a Memory storage.
//             _last_apply_index = entry.index;

//             if entry.data.is_empty() {
//                 // Empty entry, when the peer becomes Leader it will send an empty entry.
//                 continue;
//             }

//             if entry.get_entry_type() == EntryType::EntryNormal {
//                 if let Some(cb) = cbs.remove(entry.data.first().unwrap()) {
//                     cb();
//                 }
//             }

//             // TODO: handle EntryConfChange
//         }
//     };
//     handle_committed_entries(ready.take_committed_entries());

//     if !ready.entries().is_empty() {
//         // Append entries to the Raft log.
//         store.wl().append(ready.entries()).unwrap();
//     }

//     if let Some(hs) = ready.hs() {
//         // Raft HardState changed, and we need to persist it.
//         store.wl().set_hardstate(hs.clone());
//     }

//     if !ready.persisted_messages().is_empty() {
//         // Send out the persisted messages come from the node.
//         handle_messages(ready.take_persisted_messages());
//     }

//     // Advance the Raft.
//     let mut light_rd = raft_group.advance(ready);
//     // Update commit index.
//     if let Some(commit) = light_rd.commit_index() {
//         store.wl().mut_hard_state().set_commit(commit);
//     }
//     // Send out the messages.
//     handle_messages(light_rd.take_messages());
//     // Apply all committed entries.
//     handle_committed_entries(light_rd.take_committed_entries());
//     // Advance the apply index.
//     raft_group.advance_apply();
// }
