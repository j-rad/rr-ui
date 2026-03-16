use rr_ui::domain::models::ActiveConnection;

#[test]
fn test_aggregation_logic() {
    let connections = [ActiveConnection {
            id: "1".to_string(),
            upload_bytes: 100,
            download_bytes: 200,
            ..Default::default()
        },
        ActiveConnection {
            id: "2".to_string(),
            upload_bytes: 300,
            download_bytes: 400,
            ..Default::default()
        },
        ActiveConnection {
            id: "3".to_string(),
            upload_bytes: 50,
            download_bytes: 50,
            ..Default::default()
        }];

    let total_up: u64 = connections.iter().map(|c| c.upload_bytes).sum();
    let total_down: u64 = connections.iter().map(|c| c.download_bytes).sum();

    assert_eq!(total_up, 100 + 300 + 50);
    assert_eq!(total_down, 200 + 400 + 50);
}

#[test]
fn test_history_pruning_logic() {
    use std::collections::{HashMap, VecDeque};

    // Simulate the state map
    let mut history_map: HashMap<String, VecDeque<(i64, i64)>> = HashMap::new();

    // Initial state: conns 1, 2
    history_map.insert("1".to_string(), VecDeque::new());
    history_map.insert("2".to_string(), VecDeque::new());

    // New stats: conns 2, 3 (1 disconnected)
    let new_stats = vec![
        ActiveConnection { id: "2".to_string(), ..Default::default() },
        ActiveConnection { id: "3".to_string(), ..Default::default() },
    ];

    let mut seen_ids = Vec::new();
    for conn in &new_stats {
        seen_ids.push(conn.id.clone());
        history_map.entry(conn.id.clone()).or_default();
    }

    history_map.retain(|k, _| seen_ids.contains(k));

    assert!(!history_map.contains_key("1"));
    assert!(history_map.contains_key("2"));
    assert!(history_map.contains_key("3"));
}
