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
}
