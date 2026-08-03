//! M4-P03-C1: foyer-memoryのfeature、weight、handle、eviction、resize、並行APIを確認する。

use std::sync::Arc;

use foyer_memory::Cache;

fn cache(capacity: usize) -> Cache<u64, Vec<u8>> {
    Cache::builder(capacity)
        .with_shards(1)
        .with_weighter(|_, value: &Vec<u8>| value.len())
        .build()
}

#[test]
fn weighted_capacity_and_external_handle_lifetime_are_observable() {
    let cache = cache(4);
    let held = cache.insert(1, vec![1; 3]);
    assert_eq!(held.weight(), 3);
    assert_eq!(held.refs(), 1);
    assert_eq!(cache.usage(), 3);

    let clone = held.clone();
    assert_eq!(clone.refs(), 2);
    drop(clone);
    assert_eq!(held.refs(), 1);

    let evicted = cache.insert(2, vec![2; 3]);
    assert_eq!(cache.usage(), 3);
    assert!(held.is_outdated());
    assert_eq!(held.refs(), 1);
    drop(evicted);
    drop(held);
    assert_eq!(cache.usage(), 3);
}

#[test]
fn usage_drops_when_an_entry_leaves_the_cache_even_if_a_handle_is_held() {
    let cache = cache(8);
    let held = cache.insert(7, vec![7; 5]);
    let removed = cache.remove(&7).expect("entry should be removable");

    assert!(held.is_outdated());
    assert_eq!(held.refs(), 2);
    assert_eq!(removed.refs(), 2);
    assert_eq!(cache.usage(), 0);

    drop(removed);
    drop(held);
}

#[test]
fn resize_evicts_unheld_entries_and_filter_rejects_without_usage() {
    let cache = cache(8);
    let first = cache.insert(1, vec![1; 3]);
    drop(first);
    let second = cache.insert(2, vec![2; 3]);
    drop(second);
    assert_eq!(cache.usage(), 6);

    cache
        .resize(3)
        .expect("resize should accept a lower weighted cap");
    assert!(cache.usage() <= 3);
    assert!(cache.entries() <= 1);

    let filtered: Cache<u64, Vec<u8>> = Cache::builder(8)
        .with_shards(1)
        .with_weighter(|_, value: &Vec<u8>| value.len())
        .with_filter(|key: &u64, _| *key != 9)
        .build();
    let rejected = filtered.insert(9, vec![9; 5]);
    assert_eq!(rejected.weight(), 5);
    assert_eq!(filtered.usage(), 0);
    assert!(!filtered.contains(&9));
    drop(rejected);
}

#[test]
fn concurrent_get_insert_remove_has_no_external_synchronization_requirement() {
    let cache = Arc::new(cache(64));
    let writers = (0..4).map(|worker| {
        let cache = Arc::clone(&cache);
        std::thread::spawn(move || {
            for index in 0..256 {
                let key = worker * 256 + index;
                let entry = cache.insert(key, vec![key as u8; 2]);
                assert_eq!(entry.key(), &key);
                drop(entry);
                let _ = cache.get(&key);
                let _ = cache.remove(&key);
            }
        })
    });
    for writer in writers {
        writer
            .join()
            .expect("foyer concurrent operations should not panic");
    }
}
