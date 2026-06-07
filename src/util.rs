use raftpb::proto::{ConfChangeSingle, ConfChangeType, Entry};

pub fn new_conf_change_single(node_id: u64, change_type: ConfChangeType) -> ConfChangeSingle {
    ConfChangeSingle {
        node_id,
        change_type: change_type as i32,
    }
}

pub fn limit_size(ents: &mut Vec<Entry>, max_size: Option<u64>) {
    if let Some(max_size) = max_size {
        if max_size == 0 {
            ents.clear();
            return;
        }
        let mut size = 0;
        let mut limit = ents.len();
        for (i, e) in ents.iter().enumerate() {
            size += e.data.len() as u64;
            if size > max_size {
                limit = i;
                break;
            }
        }
        if limit == 0 && !ents.is_empty() {
            limit = 1;
        }
        ents.truncate(limit);
    }
}
