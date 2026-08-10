use rerun::Component as _;
use rerun::external::re_sdk_types::try_serialize_field;

#[derive(Default)]
pub struct Path2DFill {
    payload: Option<rerun::SerializedComponentBatch>,
}

impl rerun::Archetype for Path2DFill {
    fn name() -> rerun::ArchetypeName {
        "motolii.Path2DFill".into()
    }

    fn display_name() -> &'static str {
        "Motolii Path2D Fill"
    }

    fn required_components() -> std::borrow::Cow<'static, [rerun::ComponentDescriptor]> {
        vec![Self::descriptor_payload()].into()
    }
}

impl Path2DFill {
    pub fn descriptor_payload() -> rerun::ComponentDescriptor {
        rerun::ComponentDescriptor {
            archetype: Some("motolii.Path2DFill".into()),
            component: "motolii.Path2DFill:payload".into(),
            component_type: Some(rerun::components::Blob::name()),
        }
    }

    pub fn new(payload: impl Into<rerun::components::Blob>) -> Self {
        Self {
            payload: try_serialize_field::<rerun::components::Blob>(
                Self::descriptor_payload(),
                [payload.into()],
            ),
        }
    }
}

impl rerun::AsComponents for Path2DFill {
    fn as_serialized_batches(&self) -> Vec<rerun::SerializedComponentBatch> {
        self.payload.clone().into_iter().collect()
    }
}
