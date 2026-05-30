use barqflow_core::types::IDataObject;
use serde_json::{json, Value};
use std::collections::{HashSet, VecDeque};

#[derive(Debug, Clone, PartialEq)]
pub enum DeduplicationMode {
    ArrayOfIds,
    IncrementedKey,
    ContentHash,
}

pub struct DeduplicationManager;

impl DeduplicationManager {
    /// Filters a list of events down to only those that are "new" according to the deduplication mode.
    /// Updates `previous_state` in-place with the new watermark or new list of IDs.
    pub fn filter_new_events(
        events: Vec<IDataObject>,
        mode: DeduplicationMode,
        key_path: &str,
        previous_state: &mut IDataObject,
    ) -> Vec<IDataObject> {
        let mut new_events = Vec::new();

        match mode {
            DeduplicationMode::ArrayOfIds => {
                // Fetch the previous array of IDs
                let mut seen_ids: VecDeque<String> = VecDeque::new();
                let map = &previous_state.0;
                if let Some(Value::Array(arr)) = map.get("seen_ids") {
                    for val in arr {
                        if let Value::String(s) = val {
                            seen_ids.push_back(s.clone());
                        }
                    }
                }

                let mut hash_set: HashSet<String> = seen_ids.iter().cloned().collect();

                for event in events {
                    let map = &event.0;
                    if let Some(id_val) = map.get(key_path) {
                        let id_str = match id_val.as_str() {
                            Some(s) => s.to_string(),
                            None => id_val.to_string(),
                        };

                        if !hash_set.contains(&id_str) {
                            hash_set.insert(id_str.clone());
                            seen_ids.push_back(id_str);
                            new_events.push(event);

                            // Keep max 1000 items in memory to prevent unbounded growth
                            if seen_ids.len() > 1000 {
                                if let Some(oldest) = seen_ids.pop_front() {
                                    hash_set.remove(&oldest);
                                }
                            }
                        }
                    }
                }

                // Save back to state
                let map = &mut previous_state.0;
                let mut arr = Vec::new();
                for id in seen_ids {
                    arr.push(Value::String(id));
                }
                map.insert("seen_ids".to_string(), Value::Array(arr));
            }
            DeduplicationMode::ContentHash => {
                // Fetch the previous array of hashes (same logic as ArrayOfIds but ID is self-generated hash)
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};

                let mut seen_hashes: VecDeque<String> = VecDeque::new();
                let map = &previous_state.0;
                if let Some(Value::Array(arr)) = map.get("seen_hashes") {
                    for val in arr {
                        if let Value::String(s) = val {
                            seen_hashes.push_back(s.clone());
                        }
                    }
                }

                let mut hash_set: HashSet<String> = seen_hashes.iter().cloned().collect();

                for event in events {
                    let mut hasher = DefaultHasher::new();
                    // Hash the JSON string representation
                    let json_str = serde_json::to_string(&event.0).unwrap_or_default();
                    json_str.hash(&mut hasher);
                    let hash_str = format!("{:x}", hasher.finish());

                    if !hash_set.contains(&hash_str) {
                        hash_set.insert(hash_str.clone());
                        seen_hashes.push_back(hash_str);
                        new_events.push(event);

                        // Keep max 1000 items in memory to prevent unbounded growth
                        if seen_hashes.len() > 1000 {
                            if let Some(oldest) = seen_hashes.pop_front() {
                                hash_set.remove(&oldest);
                            }
                        }
                    }
                }

                // Save back to state
                let map = &mut previous_state.0;
                let mut arr = Vec::new();
                for h in seen_hashes {
                    arr.push(Value::String(h));
                }
                map.insert("seen_hashes".to_string(), Value::Array(arr));
            }

            DeduplicationMode::IncrementedKey => {
                let mut max_key_seen = 0.0;

                let map = &previous_state.0;
                if let Some(val) = map.get("max_key") {
                    if let Some(num) = val.as_f64() {
                        max_key_seen = num;
                    }
                }

                let mut highest_new_key = max_key_seen;

                for event in events {
                    let map = &event.0;
                    if let Some(val) = map.get(key_path) {
                        if let Some(num) = val.as_f64() {
                            if num > max_key_seen {
                                new_events.push(event);
                                if num > highest_new_key {
                                    highest_new_key = num;
                                }
                            }
                        }
                    }
                }

                if highest_new_key > max_key_seen {
                    let map = &mut previous_state.0;
                    map.insert("max_key".to_string(), json!(highest_new_key));
                }
            }
        }

        new_events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array_of_ids() {
        let mut state = IDataObject::default();

        let mut events = Vec::new();
        let mut e1 = IDataObject::default();
        e1.0.insert("id".to_string(), json!("A"));
        let mut e2 = IDataObject::default();
        e2.0.insert("id".to_string(), json!("B"));

        events.push(e1.clone());
        events.push(e2.clone());

        let new_evs = DeduplicationManager::filter_new_events(
            events,
            DeduplicationMode::ArrayOfIds,
            "id",
            &mut state,
        );
        assert_eq!(new_evs.len(), 2);

        // Push A, B, C. Should only return C.
        let mut events2 = Vec::new();
        events2.push(e1.clone());
        events2.push(e2.clone());
        let mut e3 = IDataObject::default();
        e3.0.insert("id".to_string(), json!("C"));
        events2.push(e3.clone());

        let new_evs2 = DeduplicationManager::filter_new_events(
            events2,
            DeduplicationMode::ArrayOfIds,
            "id",
            &mut state,
        );
        assert_eq!(new_evs2.len(), 1);
        assert_eq!(new_evs2[0].0.get("id").unwrap().as_str().unwrap(), "C");
    }

    #[test]
    fn test_incremented_key() {
        let mut state = IDataObject::default();

        let mut events = Vec::new();
        let mut e1 = IDataObject::default();
        e1.0.insert("ts".to_string(), json!(100));
        let mut e2 = IDataObject::default();
        e2.0.insert("ts".to_string(), json!(200));

        events.push(e1.clone());
        events.push(e2.clone());

        let new_evs = DeduplicationManager::filter_new_events(
            events,
            DeduplicationMode::IncrementedKey,
            "ts",
            &mut state,
        );
        assert_eq!(new_evs.len(), 2);

        // Push 150, 200, 250
        let mut events2 = Vec::new();
        let mut e1_5 = IDataObject::default();
        e1_5.0.insert("ts".to_string(), json!(150));
        let mut e3 = IDataObject::default();
        e3.0.insert("ts".to_string(), json!(250));

        events2.push(e1_5.clone());
        events2.push(e2.clone());
        events2.push(e3.clone());

        let new_evs2 = DeduplicationManager::filter_new_events(
            events2,
            DeduplicationMode::IncrementedKey,
            "ts",
            &mut state,
        );
        assert_eq!(new_evs2.len(), 1); // Only 250
        assert_eq!(new_evs2[0].0.get("ts").unwrap().as_u64().unwrap(), 250);
    }

    #[test]
    fn test_content_hash() {
        let mut state = IDataObject::default();

        let mut events = Vec::new();
        let mut e1 = IDataObject::default();
        e1.0.insert("title".to_string(), json!("News 1"));
        e1.0.insert("body".to_string(), json!("Content 1"));

        let mut e2 = IDataObject::default();
        e2.0.insert("title".to_string(), json!("News 2"));
        e2.0.insert("body".to_string(), json!("Content 2"));

        events.push(e1.clone());
        events.push(e2.clone());

        let new_evs = DeduplicationManager::filter_new_events(
            events,
            DeduplicationMode::ContentHash,
            "", // key doesn't matter for hash mode
            &mut state,
        );
        assert_eq!(new_evs.len(), 2);

        // Push identical events again plus a new one
        let mut events2 = Vec::new();
        events2.push(e1.clone());
        let mut e3 = IDataObject::default();
        e3.0.insert("title".to_string(), json!("News 3"));
        e3.0.insert("body".to_string(), json!("Content 3"));
        events2.push(e3.clone());

        let new_evs2 = DeduplicationManager::filter_new_events(
            events2,
            DeduplicationMode::ContentHash,
            "",
            &mut state,
        );

        // e1 should be dropped. Only e3 is new.
        assert_eq!(new_evs2.len(), 1);
        assert_eq!(
            new_evs2[0].0.get("title").unwrap().as_str().unwrap(),
            "News 3"
        );

        // Ensure state captured hashes
        assert!(state.0.contains_key("seen_hashes"));
        let hashes = state.0.get("seen_hashes").unwrap().as_array().unwrap();
        assert_eq!(hashes.len(), 3); // e1, e2, e3
    }
}
