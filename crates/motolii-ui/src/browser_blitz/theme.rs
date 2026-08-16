//! Browser パネルの見た目の値。ここで新しい値は決めない。
//!
//! すべて `ui/motolii-rn/src/productStyles.ts:27-61` の Browser 用 style
//! (`browser` から `effectTags`) から写した。Browser の旧 probe 格子は製品の
//! 見た目ではなかったので、ここには残さない。

/// productStyles.ts:27 `browser`
pub(super) const BACKGROUND: &str = "#202326";
/// productStyles.ts:29, 32, 34 `borderBottomColor`
pub(super) const BORDER: &str = "#3a3d41";
/// productStyles.ts:33 `tabActive.borderBottomColor`
pub(super) const ACTIVE: &str = "#b4a66a";
/// productStyles.ts:33 `tabActive.backgroundColor`
pub(super) const ACTIVE_BACKGROUND: &str = "#191b1e";
/// productStyles.ts:35 `search.backgroundColor`
pub(super) const INPUT_BACKGROUND: &str = "#17191b";
/// productStyles.ts:35, 36 `borderColor`
pub(super) const CONTROL_BORDER: &str = "#44484d";
/// productStyles.ts:37 `iconButtonActive.borderColor`
pub(super) const MODE_BORDER: &str = "#c6b975";
/// productStyles.ts:37 `iconButtonActive.backgroundColor`
pub(super) const MODE_BACKGROUND: &str = "#38372f";
/// productStyles.ts:39 `sourceRail.backgroundColor`
pub(super) const RAIL_BACKGROUND: &str = "#181a1d";
/// productStyles.ts:40 `railItem.color`
pub(super) const RAIL_TEXT: &str = "#b9bcbd";
/// productStyles.ts:41 `railHeading.color`
pub(super) const MUTED: &str = "#74797c";
/// productStyles.ts:43 `resultTitle.color`
pub(super) const TITLE: &str = "#d5d6d3";
/// productStyles.ts:49 `browserThumb.borderColor`
pub(super) const THUMB_BORDER: &str = "#3c4044";
/// Browser.tsx:43 `MEDIA_COLORS`。Browser.tsx:301 が item index で循環させる。
pub(super) const MEDIA_COLORS: [&str; 5] = ["#5d7899", "#746398", "#88704e", "#557f6d", "#8b5962"];
/// productStyles.ts:52 `effectName.color`
pub(super) const ITEM_TEXT: &str = "#ededeb";
/// productStyles.ts:53 `effectTags.color`
pub(super) const ITEM_DETAIL: &str = "#9ca0a2";

/// productStyles.ts:49 `browserThumb.height`。
pub(super) const THUMB_H: f64 = 52.0;
/// Browser の GRID card は `width: '50%'` で、画像の表示幅は親から来る。
/// これは旧 Browser が使っていた縮小実体の resource 上限を維持する値で、視覚値ではない。
pub(super) const THUMB_W: f64 = 124.0;
