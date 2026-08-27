//! Stage 共有面の Host 契約(裁定256)。
//!
//! Makepad 型も wgpu 型もここへ入れない。r7 がサイズ変化時だけ結び、
//! compositor は検査を通した texture へ書く。VISM はこのモジュールを見ない。
//! 通常経路は [`StagePresent::Shared`]。失敗は Stage 上のエラー画面。
//! [`StagePresent::FallbackCpu`] は screenshot / export など明示 fallback だけ。通常表示には使わない。

/// Host が共有面へ要求する画素形式。製品 Stage はこれだけ。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SharedPixelFormat {
    Rgba8Srgb,
}

/// 共有面の仕様。枚数は常に1。寿命はサイズが変わったときだけ作り直す。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SharedSurfaceDesc {
    pub width: u32,
    pub height: u32,
    pub format: SharedPixelFormat,
}

impl SharedSurfaceDesc {
    pub fn from_comp(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            format: SharedPixelFormat::Rgba8Srgb,
        }
    }

    pub fn matches_size(self, width: u32, height: u32) -> bool {
        self.width == width && self.height == height
    }
}

/// OS が渡す共有面の葉。import はこのどれか1つ。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SharedOsHandle {
    IoSurfaceId(u32),
    DxgiSharedHandle(u64),
    DmaBufFd(i32),
}

impl SharedOsHandle {
    pub fn is_usable(self) -> bool {
        match self {
            Self::IoSurfaceId(id) => id != 0,
            Self::DxgiSharedHandle(h) => h != 0,
            Self::DmaBufFd(fd) => fd >= 0,
        }
    }
}

/// 継ぎ目が握る面。サイズが同じなら作り直さない。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StageSurfaceSlot {
    pub desc: SharedSurfaceDesc,
    pub handle: SharedOsHandle,
}

impl StageSurfaceSlot {
    pub fn new(desc: SharedSurfaceDesc, handle: SharedOsHandle) -> Option<Self> {
        handle.is_usable().then_some(Self { desc, handle })
    }

    pub fn needs_recreate(&self, next: SharedSurfaceDesc) -> bool {
        self.desc != next
    }
}

/// 通常経路と fallback を型で分ける。混ぜない。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StagePresent {
    Shared(StageSurfaceSlot),
    FallbackCpu,
}

impl StagePresent {
    pub fn shared(desc: SharedSurfaceDesc, handle: SharedOsHandle) -> Option<Self> {
        StageSurfaceSlot::new(desc, handle).map(Self::Shared)
    }

    pub fn is_zero_copy(self) -> bool {
        matches!(self, Self::Shared(_))
    }

    pub fn needs_recreate(self, next: SharedSurfaceDesc) -> bool {
        match self {
            Self::Shared(slot) => slot.needs_recreate(next),
            Self::FallbackCpu => true,
        }
    }
}

/// 画が出ないときに最初に読む1行 — どの室で止まったか(裁定256の3室)。
///
/// この enum があるので、Stage が黒いときに全部のコードを読む必要はない。
/// 室が分かれば、次に開くファイルは1つに決まる。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StageRoom {
    /// 絵の意味・format・1枚を持つ側(`motolii-engine` / `motolii-compositor`)。
    Host,
    /// 共有面を確保して同じ handle を表示する側(Makepad fork)。
    Leaf,
    /// サイズ変化時だけ結ぶ側(この probe の `stage_import` / `main`)。
    Seam,
}

impl StageRoom {
    /// ログ用の室名。`STAGE room=leaf ...` の形で1行に出す。
    pub fn tag(self) -> &'static str {
        match self {
            Self::Host => "host",
            Self::Leaf => "leaf",
            Self::Seam => "seam",
        }
    }

    /// その室の持ち主。責任の所在をログと Stage 上の文言で名指しする。
    pub fn owner(self) -> &'static str {
        match self {
            Self::Host => "engine/compositor",
            Self::Leaf => "makepad fork",
            Self::Seam => "r7 stage_import",
        }
    }
}

/// present 1回の判定。`Shown` は「書けた」ではなく「**出た**」を意味する。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StageVerdict {
    Shown,
    Stalled {
        room: StageRoom,
        reason: &'static str,
    },
}

impl StageVerdict {
    pub fn stalled(room: StageRoom, reason: &'static str) -> Self {
        Self::Stalled { room, reason }
    }

    /// Stage 上に出す文言。室と持ち主を必ず含める。
    pub fn message(self) -> String {
        match self {
            Self::Shown => String::new(),
            Self::Stalled { room, reason } => {
                format!("{reason}\n[{}] {}", room.tag(), room.owner())
            }
        }
    }
}

/// 通常経路が本当にゼロコピーで、かつ**表示側が同じ寸法を答えた**かを見る。
///
/// ここが今まで欠けていた検査だった: 面が作れて compositor が書いても、
/// 表示側が寸法を答えられなければ 0×0 の quad になり、3室とも "ok" のまま
/// 画だけが出ない。寸法を答えるのは共有面を確保した室(`Leaf`)の責任。
pub fn check_shown(
    present: StagePresent,
    desc: SharedSurfaceDesc,
    displayed: Option<(u32, u32)>,
) -> StageVerdict {
    if !present.is_zero_copy() {
        return StageVerdict::stalled(StageRoom::Seam, "the normal path is not a shared surface");
    }
    match displayed {
        None => StageVerdict::stalled(StageRoom::Leaf, "the shared surface reports no size (drawn 0x0)"),
        Some((width, height)) if !desc.matches_size(width, height) => {
            StageVerdict::stalled(StageRoom::Leaf, "displayed size differs from the shared surface")
        }
        Some(_) => StageVerdict::Shown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resize_only_recreates_the_slot() {
        let desc = SharedSurfaceDesc::from_comp(1920, 1080);
        assert!(desc.matches_size(1920, 1080));
        assert!(!desc.matches_size(1280, 720));
        let slot = StageSurfaceSlot {
            desc,
            handle: SharedOsHandle::IoSurfaceId(1),
        };
        assert!(!slot.needs_recreate(desc));
        assert!(slot.needs_recreate(SharedSurfaceDesc::from_comp(1280, 720)));
    }

    #[test]
    fn accepted_os_handles_are_the_three_surfaces() {
        assert_ne!(
            SharedOsHandle::IoSurfaceId(1),
            SharedOsHandle::DxgiSharedHandle(2)
        );
        assert_ne!(
            SharedOsHandle::DxgiSharedHandle(2),
            SharedOsHandle::DmaBufFd(3)
        );
    }

    #[test]
    fn fallback_is_not_the_normal_path() {
        assert!(!StagePresent::FallbackCpu.is_zero_copy());
        let shared = StagePresent::shared(
            SharedSurfaceDesc::from_comp(64, 64),
            SharedOsHandle::IoSurfaceId(2),
        )
        .expect("usable handle");
        assert!(shared.is_zero_copy());
        assert!(StagePresent::shared(
            SharedSurfaceDesc::from_comp(64, 64),
            SharedOsHandle::IoSurfaceId(0),
        )
        .is_none());
    }

    #[test]
    fn a_surface_that_reports_no_size_names_the_leaf() {
        let desc = SharedSurfaceDesc::from_comp(1920, 1080);
        let present =
            StagePresent::shared(desc, SharedOsHandle::IoSurfaceId(7)).expect("usable handle");
        // 面は作れて書けている。それでも出ないのは寸法を答えない室の責任。
        assert_eq!(
            check_shown(present, desc, None),
            StageVerdict::stalled(StageRoom::Leaf, "the shared surface reports no size (drawn 0x0)")
        );
        assert_eq!(check_shown(present, desc, Some((1920, 1080))), StageVerdict::Shown);
    }

    #[test]
    fn a_stale_display_size_is_not_shown() {
        let desc = SharedSurfaceDesc::from_comp(1280, 720);
        let present =
            StagePresent::shared(desc, SharedOsHandle::IoSurfaceId(7)).expect("usable handle");
        // comp が変わったのに表示側が前の寸法のまま = リサイズ追随の破れ。
        match check_shown(present, desc, Some((1920, 1080))) {
            StageVerdict::Stalled { room, .. } => assert_eq!(room, StageRoom::Leaf),
            StageVerdict::Shown => panic!("寸法違いを Shown にしてはいけない"),
        }
    }

    #[test]
    fn the_cpu_fallback_is_never_reported_as_shown() {
        let desc = SharedSurfaceDesc::from_comp(64, 64);
        match check_shown(StagePresent::FallbackCpu, desc, Some((64, 64))) {
            StageVerdict::Stalled { room, .. } => assert_eq!(room, StageRoom::Seam),
            StageVerdict::Shown => panic!("fallback は通常経路の合格にならない"),
        }
    }

    #[test]
    fn every_room_names_its_owner() {
        for room in [StageRoom::Host, StageRoom::Leaf, StageRoom::Seam] {
            assert!(!room.tag().is_empty());
            assert!(!room.owner().is_empty());
        }
    }
}
