//! M4-P06-C1: `rangemap`のhalf-open区間集合採択probe。
//!
//! これは製品のinvalidation意味を実装せず、映像frame index／音声sample indexへ
//! privateに写す前のライブラリ機構だけを、独立した整数oracleで検証する。

use rangemap::RangeSet;

fn ranges(set: &RangeSet<i64>) -> Vec<std::ops::Range<i64>> {
    set.iter().cloned().collect()
}

fn guarded_insert(set: &mut RangeSet<i64>, range: std::ops::Range<i64>) -> bool {
    if range.start >= range.end {
        return false;
    }
    set.insert(range);
    true
}

#[test]
fn adjacent_and_overlapping_ranges_coalesce() {
    let mut set = RangeSet::new();
    set.insert(0..10);
    set.insert(10..20);
    set.insert(5..15);

    assert_eq!(ranges(&set), vec![0..20]);
    assert!(set.contains(&0));
    assert!(set.contains(&19));
    assert!(!set.contains(&20), "RangeSet must remain end-exclusive");
}

#[test]
fn removal_preserves_two_half_open_fragments() {
    let mut set = RangeSet::from([0..20]);
    set.remove(4..16);

    assert_eq!(ranges(&set), vec![0..4, 16..20]);
    assert!(!set.contains(&4));
    assert!(!set.contains(&15));
    assert!(set.contains(&16));
}

#[test]
fn gaps_are_bounded_by_the_requested_outer_range() {
    let set = RangeSet::from([0..4, 16..20]);
    let outer = 0..25;

    assert_eq!(set.gaps(&outer).collect::<Vec<_>>(), vec![4..16, 20..25]);
    assert_eq!(set.gaps(&(2..18)).collect::<Vec<_>>(), vec![4..16]);
}

#[test]
fn empty_ranges_require_a_private_guard_before_insert() {
    let mut set = RangeSet::new();

    let raw_api_panics = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut raw = RangeSet::new();
        raw.insert(5..5);
    }))
    .is_err();
    assert!(
        raw_api_panics,
        "the raw API must not receive an empty range"
    );

    assert!(!guarded_insert(&mut set, 5..5));
    assert!(set.is_empty());
    assert_eq!(set.len(), 0);
    assert_eq!(set.gaps(&(0..8)).collect::<Vec<_>>(), vec![0..8]);
}

#[test]
fn extreme_i64_bounds_are_compared_without_index_arithmetic() {
    let mut set = RangeSet::new();
    set.insert(i64::MIN..i64::MAX);

    assert!(set.contains(&i64::MIN));
    assert!(set.contains(&(i64::MAX - 1)));
    assert!(!set.contains(&i64::MAX));
}

#[test]
fn frame_and_sample_sets_are_separate_owners() {
    let mut video_frames = RangeSet::new();
    let mut audio_samples = RangeSet::new();

    video_frames.insert(0..30);
    audio_samples.insert(0..48_000);

    assert!(video_frames.contains(&29));
    assert!(!video_frames.contains(&30));
    assert!(audio_samples.contains(&47_999));
    assert!(!audio_samples.contains(&48_000));
}
