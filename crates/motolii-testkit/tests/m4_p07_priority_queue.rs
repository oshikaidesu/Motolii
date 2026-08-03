//! M4-P07-C1: `priority-queue`のbounded job collection採択probe。
//!
//! queueの上限、generation、cancel、worker lifecycleは製品ownerに残し、ここでは
//! priority変更・削除・最大値popと、同順位を製品側のsequenceで全順序化できることだけを検証する。

use priority_queue::PriorityQueue;

type Job = (u64, u64); // (generation, job id)
type Priority = (u8, u64); // (urgency, monotonic tie-break)

#[test]
fn composite_priority_provides_deterministic_ties() {
    let mut queue: PriorityQueue<Job, Priority> = PriorityQueue::new();
    queue.push((1, 10), (2, 10));
    queue.push((1, 11), (2, 11));
    queue.push((1, 12), (1, 12));

    assert_eq!(queue.pop(), Some(((1, 11), (2, 11))));
    assert_eq!(queue.pop(), Some(((1, 10), (2, 10))));
    assert_eq!(queue.pop(), Some(((1, 12), (1, 12))));
    assert!(queue.is_empty());
}

#[test]
fn change_priority_and_remove_are_indexed_operations() {
    let mut queue: PriorityQueue<Job, Priority> = PriorityQueue::new();
    queue.push((3, 30), (1, 30));
    queue.push((3, 31), (1, 31));

    assert_eq!(queue.change_priority(&(3, 30), (4, 30)), Some((1, 30)));
    assert_eq!(queue.pop(), Some(((3, 30), (4, 30))));
    assert_eq!(queue.remove(&(3, 31)), Some(((3, 31), (1, 31))));
    assert!(queue.pop().is_none());
}

#[test]
fn duplicate_item_does_not_expand_the_queue() {
    let mut queue: PriorityQueue<Job, Priority> = PriorityQueue::new();
    assert_eq!(queue.push((4, 40), (1, 40)), None);
    assert_eq!(queue.push((4, 40), (9, 40)), Some((1, 40)));

    assert_eq!(queue.len(), 1);
    assert_eq!(queue.pop(), Some(((4, 40), (9, 40))));
}

#[test]
fn bounded_admission_and_lazy_generation_filter_are_product_guards() {
    let mut queue: PriorityQueue<Job, Priority> = PriorityQueue::new();
    let max_pending = 2;
    let current_generation = 7;

    for job in [(7, 70), (6, 60)] {
        assert!(queue.len() < max_pending);
        queue.push(job, (1, job.1));
    }
    let rejected = (7, 71);
    assert!(
        queue.len() >= max_pending,
        "the queue itself does not own the bound"
    );

    let popped = queue.pop().expect("one pending job");
    assert_eq!(popped.0, (7, 70));
    assert_eq!(popped.0 .0, current_generation);
    assert_eq!(rejected, (7, 71));

    let stale = queue.pop().expect("stale job remains for lazy filtering");
    assert_ne!(stale.0 .0, current_generation);
}
