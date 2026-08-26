//! Inspector の device/effect 契約。
//!
//! ここは表示ホストと個別 device の境界だけを持つ。`Document`、`StoreView`、
//! `Intent`、書き込み関数には依存しない。実際の値は [`crate::projection`] が
//! 作り、各 section の view と write route が消費する。
//!
//! `parameters` は「この device が意味を宣言できる範囲」であり、未知 provider の
//! パラメータを推測するための fallback ではない。したがって registry に無い
//! provider は安全に descriptor なしで扱える。

use std::hash::{Hash, Hasher};

/// device を識別する安定した文字列。enum で provider の種類を閉じないため、
/// 将来の Vism/M4L provider は registry へ追加するだけでよい。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DeviceId(&'static str);

impl DeviceId {
    pub const fn new(id: &'static str) -> Self {
        Self(id)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

pub const ATTRS_DEVICE: DeviceId = DeviceId::new("inspector.attrs");
pub const TRANSFORM_DEVICE: DeviceId = DeviceId::new("inspector.transform");
pub const TEXT_DEVICE: DeviceId = DeviceId::new("inspector.text");
pub const AUDIO_DEVICE: DeviceId = DeviceId::new("inspector.audio");
pub const SHAPE_DEVICE: DeviceId = DeviceId::new("inspector.shape");
pub const MASK_DEVICE: DeviceId = DeviceId::new("inspector.mask");
pub const EFFECTS_DEVICE: DeviceId = DeviceId::new("inspector.effects");
pub const GLOW_DEVICE: DeviceId = DeviceId::new("motolii.glow");

/// section/card がいつ表示できるか。意味の有無は projection が決め、ホストは
/// この方針を読むだけに留める。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeviceVisibility {
    Always,
    WhenProjectionPresent,
    ProviderDefined,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ParameterKind {
    Scalar,
    Vector,
    Toggle,
    Color,
    Enum,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ParameterCapabilities {
    pub animatable: bool,
    pub keyframeable: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct ParameterDescriptor {
    pub id: &'static str,
    pub display_name: &'static str,
    pub kind: ParameterKind,
    pub visibility: DeviceVisibility,
    pub capabilities: ParameterCapabilities,
    /// track が無い時に Inspector/Key 列が使う provider 宣言の既定値。
    /// engine の評価側既定値と同期させる責任は provider catalog にある。
    pub default_value: f64,
}

impl PartialEq for ParameterDescriptor {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.display_name == other.display_name
            && self.kind == other.kind
            && self.visibility == other.visibility
            && self.capabilities == other.capabilities
            && self.default_value.to_bits() == other.default_value.to_bits()
    }
}

impl Eq for ParameterDescriptor {}

impl Hash for ParameterDescriptor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.display_name.hash(state);
        self.kind.hash(state);
        self.visibility.hash(state);
        self.capabilities.hash(state);
        self.default_value.to_bits().hash(state);
    }
}

impl ParameterDescriptor {
    pub const fn scalar(
        id: &'static str,
        display_name: &'static str,
        default_value: f64,
        animatable: bool,
        keyframeable: bool,
    ) -> Self {
        Self {
            id,
            display_name,
            kind: ParameterKind::Scalar,
            visibility: DeviceVisibility::WhenProjectionPresent,
            capabilities: ParameterCapabilities {
                animatable,
                keyframeable,
            },
            default_value,
        }
    }

    pub const fn default_value(self) -> f64 {
        self.default_value
    }
}

/// 既存 projection の読み出し境界。文字列は Document の path ではなく、
/// `SelectionProjection` 内の安定した読み口を表す。書き込み経路は各 module の
/// 既存の自由関数/Message に残る。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectionReadBoundary {
    pub source: &'static str,
    pub row: Option<&'static str>,
}

impl ProjectionReadBoundary {
    pub const fn section(source: &'static str) -> Self {
        Self { source, row: None }
    }

    pub const fn row(source: &'static str, row: &'static str) -> Self {
        Self {
            source,
            row: Some(row),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceCapabilities {
    pub animatable: bool,
    pub keyframeable: bool,
}

/// 個別 device/card が宣言する契約。ここには UI chrome の状態や Document の
/// 書き込み処理を置かない。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InspectorDevice {
    pub id: DeviceId,
    pub display_name: &'static str,
    pub visibility: DeviceVisibility,
    pub projection: ProjectionReadBoundary,
    pub capabilities: DeviceCapabilities,
    pub parameters: &'static [ParameterDescriptor],
}

static NO_PARAMETERS: &[ParameterDescriptor] = &[];

static GLOW_PARAMETERS: [ParameterDescriptor; 3] = [
    // engine::translate_glow_params の既定値と同期する。
    ParameterDescriptor::scalar("threshold", "Threshold", 1.0, true, true),
    ParameterDescriptor::scalar("intensity", "Intensity", 0.75, true, true),
    ParameterDescriptor::scalar("radius", "Radius", 1.0, true, true),
];

static CORE_DEVICES: [InspectorDevice; 7] = [
    InspectorDevice {
        id: ATTRS_DEVICE,
        display_name: "Attrs",
        visibility: DeviceVisibility::WhenProjectionPresent,
        projection: ProjectionReadBoundary::section("SelectionProjection.attrs"),
        capabilities: DeviceCapabilities {
            animatable: false,
            keyframeable: false,
        },
        parameters: NO_PARAMETERS,
    },
    InspectorDevice {
        id: TRANSFORM_DEVICE,
        display_name: "Transform",
        visibility: DeviceVisibility::WhenProjectionPresent,
        projection: ProjectionReadBoundary::section("SelectionProjection.transform"),
        capabilities: DeviceCapabilities {
            animatable: true,
            keyframeable: true,
        },
        parameters: NO_PARAMETERS,
    },
    InspectorDevice {
        id: TEXT_DEVICE,
        display_name: "Text",
        visibility: DeviceVisibility::WhenProjectionPresent,
        projection: ProjectionReadBoundary::section("SelectionProjection.text"),
        capabilities: DeviceCapabilities {
            animatable: true,
            keyframeable: true,
        },
        parameters: NO_PARAMETERS,
    },
    InspectorDevice {
        id: AUDIO_DEVICE,
        display_name: "Audio",
        visibility: DeviceVisibility::WhenProjectionPresent,
        projection: ProjectionReadBoundary::section("SelectionProjection.audio"),
        capabilities: DeviceCapabilities {
            animatable: false,
            keyframeable: false,
        },
        parameters: NO_PARAMETERS,
    },
    InspectorDevice {
        id: SHAPE_DEVICE,
        display_name: "Shape",
        visibility: DeviceVisibility::WhenProjectionPresent,
        projection: ProjectionReadBoundary::section("SelectionProjection.shape"),
        capabilities: DeviceCapabilities {
            animatable: true,
            keyframeable: true,
        },
        parameters: NO_PARAMETERS,
    },
    InspectorDevice {
        id: MASK_DEVICE,
        display_name: "Mask",
        visibility: DeviceVisibility::WhenProjectionPresent,
        projection: ProjectionReadBoundary::section("SelectionProjection.masks"),
        capabilities: DeviceCapabilities {
            animatable: true,
            keyframeable: true,
        },
        parameters: NO_PARAMETERS,
    },
    InspectorDevice {
        id: EFFECTS_DEVICE,
        display_name: "Effects",
        visibility: DeviceVisibility::WhenProjectionPresent,
        projection: ProjectionReadBoundary::section("SelectionProjection.effects"),
        capabilities: DeviceCapabilities {
            animatable: true,
            keyframeable: true,
        },
        parameters: NO_PARAMETERS,
    },
];

static GLOW_DESCRIPTOR: InspectorDevice = InspectorDevice {
    id: GLOW_DEVICE,
    display_name: "Glow",
    visibility: DeviceVisibility::ProviderDefined,
    projection: ProjectionReadBoundary::row("SelectionProjection.effects", "EffectRowProjection"),
    capabilities: DeviceCapabilities {
        animatable: true,
        keyframeable: true,
    },
    parameters: &GLOW_PARAMETERS,
};

/// 個別 device の意味を閉じずに列挙する registry。未知 provider はここに現れず、
/// 呼び手は `device_for_provider` の `None` を安全な fallback として扱う。
pub fn device_registry() -> impl Iterator<Item = &'static InspectorDevice> {
    CORE_DEVICES.iter().chain(std::iter::once(&GLOW_DESCRIPTOR))
}

pub fn device_for(id: &str) -> Option<&'static InspectorDevice> {
    device_registry().find(|device| device.id.as_str() == id)
}

pub fn device_for_provider(provider_id: &str) -> Option<&'static InspectorDevice> {
    if provider_id == GLOW_DEVICE.as_str() {
        Some(&GLOW_DESCRIPTOR)
    } else {
        None
    }
}

/// provider が宣言した parameter catalog。未知 provider は空であり、Inspector が
/// plugin 固有の意味を推測して行を捏造しない。
pub fn parameters_for_provider(provider_id: &str) -> &'static [ParameterDescriptor] {
    device_for_provider(provider_id)
        .map(|device| device.parameters)
        .unwrap_or(&[])
}

/// provider catalog の安定 id から descriptor を引く読み口。Inspector section は
/// enum を増やさず、この descriptor 参照を projection/field に渡す。
pub fn parameter_for_provider(
    provider_id: &str,
    parameter_id: &str,
) -> Option<&'static ParameterDescriptor> {
    parameters_for_provider(provider_id)
        .iter()
        .find(|parameter| parameter.id == parameter_id)
}

/// 共通ホストが持つ表示状態。selection/scroll/collapse だけを扱い、device 固有の
/// 値や write route は持たない。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollapseState {
    Expanded,
    Collapsed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InspectorHostState {
    pub selection_count: usize,
    pub selected_device: Option<DeviceId>,
    pub collapse: CollapseState,
    pub scroll_offset: u32,
}

impl Default for InspectorHostState {
    fn default() -> Self {
        Self {
            selection_count: 0,
            selected_device: None,
            collapse: CollapseState::Expanded,
            scroll_offset: 0,
        }
    }
}

impl InspectorHostState {
    pub fn has_selection(self) -> bool {
        self.selection_count > 0
    }

    pub fn is_collapsed(self) -> bool {
        self.collapse == CollapseState::Collapsed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn registry_ids_are_unique_and_projection_boundaries_are_present() {
        let devices: Vec<_> = device_registry().collect();
        let ids: HashSet<_> = devices.iter().map(|device| device.id).collect();
        assert_eq!(ids.len(), devices.len());
        assert!(devices.iter().all(|device| !device.projection.source.is_empty()));
    }

    #[test]
    fn glow_is_a_known_animatable_device_with_keyframeable_parameters() {
        let glow = device_for_provider(GLOW_DEVICE.as_str()).expect("Glow は registry にある");
        assert_eq!(glow.display_name, "Glow");
        assert_eq!(glow.parameters.len(), 3);
        assert!(glow.capabilities.animatable);
        assert!(glow.capabilities.keyframeable);
        assert!(glow
            .parameters
            .iter()
            .all(|parameter| parameter.capabilities.animatable
                && parameter.capabilities.keyframeable));
    }

    #[test]
    fn unknown_provider_has_no_invented_descriptor() {
        assert!(device_for_provider("third-party.sparkle").is_none());
        assert!(device_for("third-party.sparkle").is_none());
    }

    #[test]
    fn host_state_contains_only_selection_and_presentation_state() {
        let state = InspectorHostState::default();
        assert!(!state.has_selection());
        assert!(!state.is_collapsed());
        assert_eq!(state.scroll_offset, 0);
        assert_eq!(state.selected_device, None);
    }
}
