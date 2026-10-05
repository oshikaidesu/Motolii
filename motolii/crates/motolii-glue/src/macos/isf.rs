//! A card's header: `/*{ ...json... }*/` before the WGSL, as the old shelf host read it.
//! Only what the post host needs, and pure: the meaning of a card file stays one.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum InputType {
    Image,
    Float,
    Long,
    Bool,
    Point2D,
    Color,
}

impl InputType {
    fn parse(name: &str) -> Option<Self> {
        match name {
            "image" => Some(Self::Image),
            "float" => Some(Self::Float),
            "long" => Some(Self::Long),
            "bool" => Some(Self::Bool),
            "point2D" => Some(Self::Point2D),
            "color" => Some(Self::Color),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct Input {
    pub name: String,
    pub ty: InputType,
    pub default: [f32; 4],
}

/// A pass target's side: a constant, or the output size over a constant (`"$WIDTH/2"`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Dimension {
    Fixed(u32),
    Divided(u32),
}

impl Dimension {
    pub(super) fn resolve(self, full: u32) -> u32 {
        match self {
            Self::Fixed(n) => n,
            Self::Divided(n) => full.div_ceil(n).max(1),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct Pass {
    pub target: Option<String>,
    pub width: Option<Dimension>,
    pub height: Option<Dimension>,
    pub float: bool,
    pub persistent: bool,
}

#[derive(Clone, Debug, Default)]
pub(super) struct Manifest {
    pub label: Option<String>,
    pub linear: bool,
    pub inputs: Vec<Input>,
    pub passes: Vec<Pass>,
}

impl Manifest {
    pub(super) fn images(&self) -> impl Iterator<Item = &Input> {
        self.inputs.iter().filter(|input| input.ty == InputType::Image)
    }

    pub(super) fn params(&self) -> impl Iterator<Item = &Input> {
        self.inputs.iter().filter(|input| input.ty != InputType::Image)
    }

    /// Named targets in first-appearance order: one texture each, however many passes write it.
    pub(super) fn targets(&self) -> Vec<&str> {
        let mut out: Vec<&str> = Vec::new();
        for name in self.passes.iter().filter_map(|pass| pass.target.as_deref()) {
            if !out.contains(&name) {
                out.push(name);
            }
        }
        out
    }

    /// The pass that first declares a target decides its size and format.
    pub(super) fn declaring(&self, target: &str) -> Option<&Pass> {
        self.passes.iter().find(|pass| pass.target.as_deref() == Some(target))
    }
}

/// The header as a manifest. The WGSL is the whole file: the header is a block comment to
/// naga, so the line numbers in its errors are the file's.
pub(super) fn parse(source: &str) -> Result<Manifest, String> {
    let trimmed = source.trim_start();
    let end = trimmed
        .strip_prefix("/*")
        .and_then(|rest| rest.find("*/"))
        .ok_or("no `/*{ ... }*/` header")?;
    let value: serde_json::Value =
        serde_json::from_str(&trimmed[2..2 + end]).map_err(|error| format!("header: {error}"))?;
    let mut manifest = Manifest {
        label: value.get("LABEL").and_then(|v| v.as_str()).map(str::to_owned),
        linear: value.get("FILTER").and_then(|v| v.as_str()) == Some("linear"),
        ..Default::default()
    };
    for entry in value.get("INPUTS").and_then(|v| v.as_array()).into_iter().flatten() {
        let (Some(name), Some(ty)) = (
            entry.get("NAME").and_then(|v| v.as_str()),
            entry.get("TYPE").and_then(|v| v.as_str()).and_then(InputType::parse),
        ) else {
            continue;
        };
        manifest.inputs.push(Input { name: name.to_owned(), ty, default: components(entry.get("DEFAULT")) });
    }
    for entry in value.get("PASSES").and_then(|v| v.as_array()).into_iter().flatten() {
        let flag = |key: &str| entry.get(key).is_some_and(|v| v.as_bool().unwrap_or(v.as_i64().unwrap_or(0) != 0));
        manifest.passes.push(Pass {
            target: entry.get("TARGET").and_then(|v| v.as_str()).map(str::to_owned),
            width: dimension(entry.get("WIDTH"), "$WIDTH/")?,
            height: dimension(entry.get("HEIGHT"), "$HEIGHT/")?,
            float: flag("FLOAT"),
            persistent: flag("PERSISTENT"),
        });
    }
    Ok(manifest)
}

fn dimension(value: Option<&serde_json::Value>, divided: &str) -> Result<Option<Dimension>, String> {
    let Some(value) = value else { return Ok(None) };
    if let Some(n) = value.as_str().and_then(|s| s.strip_prefix(divided)).and_then(|s| s.parse::<u32>().ok()) {
        if n > 0 {
            return Ok(Some(Dimension::Divided(n)));
        }
    }
    match value.as_f64().or_else(|| value.as_str()?.parse().ok()) {
        Some(n) if n >= 1.0 && n <= 16384.0 && n.fract() == 0.0 => Ok(Some(Dimension::Fixed(n as u32))),
        _ => Err(format!("{} must be `{divided}n` or a whole number", &divided[1..divided.len() - 1])),
    }
}

fn components(value: Option<&serde_json::Value>) -> [f32; 4] {
    let mut out = [0.0f32; 4];
    match value {
        Some(serde_json::Value::Array(items)) => {
            for (slot, item) in out.iter_mut().zip(items) {
                *slot = item.as_f64().unwrap_or(0.0) as f32;
            }
        }
        Some(serde_json::Value::Number(n)) => out[0] = n.as_f64().unwrap_or(0.0) as f32,
        Some(serde_json::Value::Bool(b)) => out[0] = f32::from(u8::from(*b)),
        _ => {}
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{parse, Dimension, InputType};

    /// The glow card as shipped: one image, seven params, eleven float targets and the output.
    #[test]
    fn the_glow_card_header_is_read_as_the_old_host_read_it() {
        let path = super::super::post::vism_dir().join("glow.wgsl");
        let source = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let manifest = parse(&source).unwrap();
        assert_eq!(manifest.label.as_deref(), Some("Glow"));
        assert!(manifest.linear);
        assert_eq!(manifest.images().map(|i| i.name.as_str()).collect::<Vec<_>>(), ["source"]);
        assert_eq!(manifest.params().count(), 7);
        assert_eq!(manifest.params().next().map(|p| (p.ty, p.default[0])), Some((InputType::Float, 0.6)));
        assert_eq!(manifest.targets().len(), 11);
        assert_eq!(manifest.passes.len(), 12);
        assert_eq!(manifest.passes[5].width, Some(Dimension::Divided(64)));
        assert_eq!(manifest.passes[5].width.unwrap().resolve(2560), 40);
        assert!(manifest.passes[11].target.is_none() && manifest.passes[11].width.is_none());
        assert!(!manifest.passes.iter().any(|pass| pass.persistent));
    }
}
