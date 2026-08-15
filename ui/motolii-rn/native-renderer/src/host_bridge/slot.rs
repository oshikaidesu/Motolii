use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

#[cfg(test)]
use std::sync::atomic::AtomicU64;

pub(super) const MAX_JSON_BYTES: usize = 16_384;
pub(super) const MAX_SNAPSHOT_JSON_BYTES: usize = 131_072;

#[cfg(test)]
pub(super) static TEST_SELECTION_DISPATCH_COUNT: AtomicU64 = AtomicU64::new(0);

#[cfg(test)]
pub(super) static TEST_KEYMAP_REMOVE_POSITION_KEY_COUNT: AtomicU64 = AtomicU64::new(0);

#[cfg(test)]
pub(super) static TEST_KEYMAP_DELETE_LAYER_COUNT: AtomicU64 = AtomicU64::new(0);

#[cfg(test)]
pub(super) static TEST_MOVE_LAYER_BY_DISPATCH_COUNT: AtomicU64 = AtomicU64::new(0);

#[cfg(test)]
pub(super) static TEST_SNAPSHOT_READ_COUNT: AtomicU64 = AtomicU64::new(0);

#[cfg(test)]
pub(super) static TEST_SET_POSITION_KEY_TIME_DISPATCH_COUNT: AtomicU64 = AtomicU64::new(0);

pub(super) struct HostSlot {
    pub(super) handle: u64,
    /// 同じlive Document writerへ再接続できるproject identity。
    pub(super) project_path: PathBuf,
    /// processに1つだけのStage seat。register後に埋まる。
    pub(super) stage_handle: Option<u64>,
    /// stage_pointer がdown状態かどうか（Rust内状態機械）。
    pub(super) stage_pointer_active: bool,
    /// stage seat が現在mount状態か。
    pub(super) stage_mounted: bool,
    /// stage_pointer の単調 sequence（bridge内部採番）。
    pub(super) pointer_sequence: u64,
    /// 直近の mount/resize 論理寸法（move閾値の logical px 換算用）。
    pub(super) stage_logical_width: f64,
    pub(super) stage_logical_height: f64,
}

static TIMELINE_INTERACTING: AtomicBool = AtomicBool::new(false);
static IME_PREEDIT: AtomicBool = AtomicBool::new(false);

pub(super) fn host_slot() -> &'static Mutex<Option<HostSlot>> {
    static SLOT: OnceLock<Mutex<Option<HostSlot>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

/// Host生成に失敗したprocessでも、RN操作へcoreのtyped rejectを返す。
#[cfg(target_os = "macos")]
pub(super) fn host_startup_reject() -> &'static Mutex<Option<String>> {
    static REJECT: OnceLock<Mutex<Option<String>>> = OnceLock::new();
    REJECT.get_or_init(|| Mutex::new(None))
}

pub(crate) fn is_timeline_interacting() -> bool {
    TIMELINE_INTERACTING.load(Ordering::Acquire)
}

pub(crate) fn set_timeline_interacting(interacting: bool) {
    TIMELINE_INTERACTING.store(interacting, Ordering::Release);
}

pub(crate) fn try_host_handle() -> Option<u64> {
    host_slot().lock().ok()?.as_ref().map(|slot| slot.handle)
}

pub(crate) fn host_slot_present() -> bool {
    try_host_handle().is_some()
}

#[cfg(test)]
pub(crate) fn test_reset_timeline_selection_dispatch_count() {
    TEST_SELECTION_DISPATCH_COUNT.store(0, Ordering::SeqCst);
}

#[cfg(test)]
pub(crate) fn test_timeline_selection_dispatch_count() -> u64 {
    TEST_SELECTION_DISPATCH_COUNT.load(Ordering::SeqCst)
}

#[cfg(test)]
pub(crate) fn test_reset_move_layer_by_dispatch_count() {
    TEST_MOVE_LAYER_BY_DISPATCH_COUNT.store(0, Ordering::SeqCst);
}

#[cfg(test)]
pub(crate) fn test_move_layer_by_dispatch_count() -> u64 {
    TEST_MOVE_LAYER_BY_DISPATCH_COUNT.load(Ordering::SeqCst)
}

#[cfg(test)]
pub(crate) fn test_reset_snapshot_read_count() {
    TEST_SNAPSHOT_READ_COUNT.store(0, Ordering::SeqCst);
}

#[cfg(test)]
pub(crate) fn test_snapshot_read_count() -> u64 {
    TEST_SNAPSHOT_READ_COUNT.load(Ordering::SeqCst)
}

#[cfg(test)]
pub(crate) fn test_reset_keymap_dispatch_counts() {
    TEST_KEYMAP_REMOVE_POSITION_KEY_COUNT.store(0, Ordering::SeqCst);
    TEST_KEYMAP_DELETE_LAYER_COUNT.store(0, Ordering::SeqCst);
}

#[cfg(test)]
pub(crate) fn test_keymap_remove_position_key_count() -> u64 {
    TEST_KEYMAP_REMOVE_POSITION_KEY_COUNT.load(Ordering::SeqCst)
}

#[cfg(test)]
pub(crate) fn test_keymap_delete_layer_count() -> u64 {
    TEST_KEYMAP_DELETE_LAYER_COUNT.load(Ordering::SeqCst)
}

#[cfg(test)]
pub(crate) fn test_clear_host_slot() {
    if let Ok(mut guard) = host_slot().lock() {
        *guard = None;
    }
}

pub(super) fn slice_from_written<'a>(out: &'a [u8], written: i64) -> Option<&'a [u8]> {
    let written = usize::try_from(written).ok()?;
    (written <= out.len()).then_some(&out[..written])
}
