use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct LayoutManifest {
    pub source: String,
    pub viewport: Size,
    pub elements: Vec<LayoutElement>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LayoutElement {
    pub id: String,
    pub rect: ManifestRect,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Size {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub(crate) struct ManifestRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub(crate) fn react_1440x900() -> LayoutManifest {
    serde_json::from_str(include_str!("../fixtures/react-layout-1440x900.json"))
        .expect("checked-in React layout manifest must be valid")
}

impl LayoutManifest {
    pub(crate) fn rect(&self, id: &str) -> Option<ManifestRect> {
        self.elements
            .iter()
            .find(|element| element.id == id)
            .map(|element| element.rect)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_manifest_identifies_fixed_react_fixture() {
        let manifest = react_1440x900();
        assert_eq!(manifest.source, "docs/mocks-ui/#plugin-browser-candidate");
        assert_eq!(
            (manifest.viewport.width, manifest.viewport.height),
            (1440.0, 900.0)
        );
        assert_eq!(manifest.rect("effects_rail").unwrap().width, 105.9983);
    }
}
