pub use crate::proto::*;

// Include the `items` module, which is generated from items.proto.
// It is important to maintain the same structure as in the proto.
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/raftpb.rs"));
}

// pub mod prelude {
//     pub use crate::raftpb::{
//         Message, MessageType
//     };
// }

