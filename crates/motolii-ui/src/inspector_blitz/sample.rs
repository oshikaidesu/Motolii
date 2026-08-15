//! 絵を出すための**固定サンプル**。`Document` にも `rn_product_host` にも触らない。
//!
//! C7 は「`Inspector.tsx` の見た目が Blitz で出るか」だけを見る回なので、
//! ここの値は**意味を持たない置き**である。色・寸法・間隔は1つも含まない
//! (それらは `theme.rs` = `productStyles.ts` の写しだけが持つ)。
//!
//! key の3状態(`unkeyed` / `animated` / `current`)は `Inspector.tsx:22` の型に対応し、
//! サンプルでは3つとも1回ずつ出るように置いてある。効いているかを絵で見るため。

/// `Inspector.tsx:22` の `KeyAffordanceState` の写し。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    Unkeyed,
    Animated,
    Current,
}

pub struct DialSample {
    /// `Inspector.tsx:332,379,420,479,529` の `label`。
    pub label: &'static str,
    /// `Inspector.tsx:1587-1588` の `draft`(= `formatDialValue` 済みの文字列)。
    pub value: &'static str,
    /// `Inspector.tsx:1590` の `unit`。空なら `dialUnit` を出さない。
    pub unit: &'static str,
    /// `Inspector.tsx:1520` — `addKeyTestID` が無い dial には KeyAffordance が出ない。
    pub key: Option<KeyState>,
}

pub struct ParamSample {
    pub label: &'static str,
    /// `Inspector.tsx:291-296` の f64 param。
    pub value: &'static str,
    /// `Inspector.tsx:284-289` の color param(RGBA4本)。`Some` なら `ColorRgbaEditor` の枝。
    pub color: Option<[&'static str; 4]>,
}

pub struct EffectSample {
    pub name: &'static str,
    pub params: &'static [ParamSample],
}

pub struct InspectorSample {
    /// `PanelHeader` の `detail`(`Inspector.tsx:137`)。
    pub detail: &'static str,
    /// `styles.inspectorTitle` に出る `layerSeat.displayName`(`Inspector.tsx:154`)。
    pub display_name: &'static str,
    /// `Inspector.tsx:155` の `position keys: N`。
    pub position_key_count: u32,
    /// `Inspector.tsx:157-159` の Opacity 行。
    pub opacity: &'static str,
    pub opacity_key: KeyState,
    /// `Inspector.tsx:219-277` の pathOperationGrid のボタン列。
    pub clip_actions: &'static [&'static str],
    /// `Inspector.tsx:279-299`。
    pub source_params: &'static [ParamSample],
    /// `Inspector.tsx:301-329`。
    pub effects: &'static [EffectSample],
    /// `Inspector.tsx:330-592`。
    pub dials: &'static [DialSample],
}

pub const SAMPLE: InspectorSample = InspectorSample {
    detail: "sample.vism",
    display_name: "Sample Layer",
    position_key_count: 3,
    opacity: "1",
    opacity_key: KeyState::Animated,
    // Inspector.tsx:230,242,255,265,275。Mute/Solo は visible/solo で文言が入れ替わる。
    clip_actions: &["Delete", "Duplicate", "Split", "Mute", "Solo"],
    source_params: &[
        ParamSample {
            label: "text",
            value: "MOTOLII",
            color: None,
        },
        ParamSample {
            label: "size",
            value: "48",
            color: None,
        },
        ParamSample {
            label: "fill",
            value: "",
            color: Some(["1", "0.85", "0.4", "1"]),
        },
    ],
    effects: &[EffectSample {
        name: "Gaussian Blur",
        params: &[
            ParamSample {
                label: "radius",
                value: "6",
                color: None,
            },
            ParamSample {
                label: "mix",
                value: "0.5",
                color: None,
            },
        ],
    }],
    dials: &[
        DialSample {
            label: "Position X",
            value: "0.000",
            unit: "",
            key: Some(KeyState::Current),
        },
        DialSample {
            label: "Position Y",
            value: "0.000",
            unit: "",
            key: None,
        },
        DialSample {
            label: "Rotation Z",
            value: "0.0",
            // Inspector.tsx:420 の unit="°"
            unit: "°",
            key: Some(KeyState::Animated),
        },
        DialSample {
            label: "Scale X",
            value: "1.000",
            unit: "",
            key: Some(KeyState::Unkeyed),
        },
        DialSample {
            label: "Scale Y",
            value: "1.000",
            unit: "",
            key: None,
        },
    ],
};
