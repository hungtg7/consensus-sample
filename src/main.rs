use slog::{Drain, Logger, o, info};
use std::time::{Duration, Instant};
use std::thread;
use std::sync::mpsc::{self, RecvTimeoutError};

mod node;
mod raft;
mod config;

use node::Node;
use crate::raft::Raft;
use raftpb::proto::Message;

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

fn main() {
    // Log configuration
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = slog::Logger::root(drain, o!());

    let conf = config::Config{
        id: 1,
        heartbeat_tick: 15,
        election_tick: 20,
        min_election_tick: 25,
        max_election_tick: 30,
    };
    let node = Node::new(&conf, &logger);
    // let (sender, receiver) = mpsc::channel();
    println!("Hello, world!");
    // send_propose(logger.clone(), sender);
}

fn send_propose(logger: Logger, sender: mpsc::Sender<Msg>) {
    thread::spawn(move || {
        // Wait some time and send the request to the Raft.
        thread::sleep(Duration::from_secs(10));

        let (s1, r1) = mpsc::channel::<u8>();

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

        let n = r1.recv().unwrap();
        assert_eq!(n, 0);

        info!(logger, "receive the propose callback");
    });
}
