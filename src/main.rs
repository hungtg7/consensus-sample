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
mod tracker;

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
