//! 立体を作る族 — 平らな素材(形・文字・画)の輪郭から立体を起こす効果。shader を持たず、
//! 値は層の property なので他の効果と同じにキーが打てる。棚の欄は配置効果と同じ契約(`kind.rs`)。
//! Extrude は奥行き(Depth 属性の効果版。属性は互換のため残る)、Bevel は縁の丸み — 法線が縁で回り、
//! ガラスがそこで下の絵を歪める(C4D の Fillet Cap、Blender の Bevel)。2026-09-12 利用者裁定 (b)。

use crate::doc::store::kind::Param;

pub struct SolidKind {
    pub plugin_id: &'static str,
    pub label: &'static str,
    pub params: &'static [Param],
}

pub const EXTRUDE: &str = "motolii.extrude";
pub const BEVEL: &str = "motolii.bevel";
pub const PROFILES: &[&str] = &["Round", "Chamfer"];

pub const KINDS: &[SolidKind] = &[
    SolidKind { plugin_id: EXTRUDE, label: "Extrude", params: &[
        Param::number("depth", "Depth", 20.0, Some((0.0, f64::MAX))),
    ] },
    SolidKind { plugin_id: BEVEL, label: "Bevel", params: &[
        Param::number("radius", "Radius", 8.0, Some((0.0, f64::MAX))),
        Param::number("segments", "Segments", 6.0, Some((1.0, 32.0))),
        Param::choice("profile", "Profile", PROFILES),
    ] },
];
