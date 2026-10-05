//! The shelf: the picture's shaders are files beside the crate, not Rust.
//! Edit one while the app runs; the next frame compiles it on the session thread inside an
//! error scope. A file that fails prints a red ERROR and the last good pipeline stays.
//! The embedded copy of each file is the fallback when the directory is not there.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum Kind {
    Mesh,
    Card,
    Mark,
    Echo,
    Blit,
    Glass,
}

impl Kind {
    pub(super) const ALL: [Kind; 6] = [Kind::Mesh, Kind::Card, Kind::Mark, Kind::Echo, Kind::Blit, Kind::Glass];

    pub(super) fn name(self) -> &'static str {
        match self {
            Kind::Mesh => "mesh",
            Kind::Card => "card",
            Kind::Mark => "mark",
            Kind::Echo => "echo",
            Kind::Blit => "blit",
            Kind::Glass => "glass",
        }
    }

    pub(super) fn embedded(self) -> &'static str {
        match self {
            Kind::Mesh => include_str!("../../shelf/mesh.wgsl"),
            Kind::Card => include_str!("../../shelf/card.wgsl"),
            Kind::Mark => include_str!("../../shelf/mark.wgsl"),
            Kind::Echo => include_str!("../../shelf/echo.wgsl"),
            Kind::Blit => include_str!("../../shelf/blit.wgsl"),
            Kind::Glass => include_str!("../../shelf/glass.wgsl"),
        }
    }
}

pub(super) struct Pipe {
    pub pipeline: wgpu::RenderPipeline,
    pub layout: wgpu::BindGroupLayout,
}

pub(super) type Stamp = Option<(Option<SystemTime>, u64)>;

pub(super) struct Shelf {
    dir: PathBuf,
    stamps: BTreeMap<Kind, Stamp>,
}

/// Where a source came from, for the log line.
pub(super) enum Origin {
    File(PathBuf),
    Embedded,
}

impl std::fmt::Display for Origin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Origin::File(path) => write!(f, "{}", path.display()),
            Origin::Embedded => write!(f, "embedded"),
        }
    }
}

impl Shelf {
    /// `MOTOLII_SHELF`, else the crate's own `shelf/` directory as it was at build time.
    pub(super) fn open() -> Self {
        let dir = std::env::var_os("MOTOLII_SHELF")
            .map(PathBuf::from)
            .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("shelf"));
        let mut shelf = Self { dir, stamps: BTreeMap::new() };
        for kind in Kind::ALL {
            shelf.stamps.insert(kind, stamp(&shelf.path(kind)));
        }
        if shelf.stamps.values().all(Option::is_none) {
            eprintln!("A1: shelf {} not found, drawing from the embedded copies", shelf.dir.display());
        } else {
            eprintln!("A1: shelf {} watched; edit a .wgsl there while this runs", shelf.dir.display());
        }
        shelf
    }

    pub(super) fn dir(&self) -> &Path {
        &self.dir
    }

    pub(super) fn path(&self, kind: Kind) -> PathBuf {
        self.dir.join(format!("{}.wgsl", kind.name()))
    }

    /// The kinds whose file changed (or appeared, or vanished) since the last call.
    pub(super) fn changed(&mut self) -> Vec<Kind> {
        let mut out = Vec::new();
        for kind in Kind::ALL {
            let now = stamp(&self.path(kind));
            if self.stamps.get(&kind) != Some(&now) {
                self.stamps.insert(kind, now);
                out.push(kind);
            }
        }
        out
    }

    /// The file's text when it is there and readable, else the embedded copy.
    pub(super) fn read(&self, kind: Kind) -> (String, Origin) {
        let path = self.path(kind);
        match std::fs::read_to_string(&path) {
            Ok(text) => (text, Origin::File(path)),
            Err(_) => (kind.embedded().to_string(), Origin::Embedded),
        }
    }
}

pub(super) fn stamp(path: &Path) -> Stamp {
    let meta = std::fs::metadata(path).ok()?;
    Some((meta.modified().ok(), meta.len()))
}

/// Compile one shelf file into its slot. wgpu reports shader and layout errors asynchronously,
/// so the whole creation runs inside an error scope; the error text carries naga's diagnostic.
pub(super) fn build(device: &wgpu::Device, kind: Kind, source: &str) -> Result<Pipe, String> {
    let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let pipe = recipe(device, kind, source);
    match pollster::block_on(scope.pop()) {
        None => Ok(pipe),
        Some(error) => Err(error.to_string()),
    }
}

/// The host side of each slot: what the picture binds and how it is drawn. The file decides
/// nothing here; a file that needs a different interface is a different slot.
fn recipe(device: &wgpu::Device, kind: Kind, source: &str) -> Pipe {
    let name = kind.name();
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(name),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let both = wgpu::ShaderStages::VERTEX_FRAGMENT;
    let entries: Vec<wgpu::BindGroupLayoutEntry> = match kind {
        Kind::Mesh => vec![uniform_entry(0, both)],
        Kind::Card => vec![uniform_entry(0, both), texture_entry(1), texture_entry(2), sampler_entry(3)],
        Kind::Mark | Kind::Echo => vec![uniform_entry(0, both), texture_entry(1), sampler_entry(2)],
        Kind::Blit => vec![texture_entry(0), sampler_entry(1)],
        Kind::Glass => vec![texture_entry(0), sampler_entry(1), uniform_entry(2, both)],
    };
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(name),
        entries: &entries,
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(name),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let position_color = [
        wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 0, shader_location: 0 },
        wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 12, shader_location: 1 },
    ];
    let position_uv = [
        wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 0, shader_location: 0 },
        wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 12, shader_location: 1 },
    ];
    let buffers: Vec<wgpu::VertexBufferLayout<'_>> = match kind {
        Kind::Mesh => vec![wgpu::VertexBufferLayout {
            array_stride: 28,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &position_color,
        }],
        Kind::Card | Kind::Mark | Kind::Echo => vec![wgpu::VertexBufferLayout {
            array_stride: 20,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &position_uv,
        }],
        Kind::Blit | Kind::Glass => vec![],
    };
    let over = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    };
    // Text and echo plates write premultiplied color over what is there; the rest are opaque.
    let blend = match kind {
        Kind::Mark | Kind::Echo => Some(wgpu::BlendState { color: over, alpha: over }),
        _ => None,
    };
    let in_world = matches!(kind, Kind::Mesh | Kind::Card | Kind::Mark | Kind::Echo);
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(name),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs"),
            buffers: &buffers,
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs"),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Bgra8Unorm,
                blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: in_world.then(|| wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24Plus,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: if in_world { 4 } else { 1 },
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview_mask: None,
        cache: None,
    });
    Pipe { pipeline, layout }
}

pub(super) fn uniform_entry(binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

pub(super) fn texture_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

pub(super) fn sampler_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    }
}

#[cfg(test)]
mod tests {
    use super::{Kind, Origin, Shelf};
    use std::collections::BTreeMap;

    fn scratch() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("motolii-shelf-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_saved_file_is_seen_once_and_a_missing_file_falls_back() {
        let dir = scratch();
        let path = dir.join("glass.wgsl");
        std::fs::write(&path, "// one").unwrap();
        let mut shelf = Shelf { dir: dir.clone(), stamps: BTreeMap::new() };
        for kind in Kind::ALL {
            shelf.stamps.insert(kind, super::stamp(&shelf.path(kind)));
        }
        assert!(shelf.changed().is_empty());
        std::fs::write(&path, "// two, longer").unwrap();
        assert_eq!(shelf.changed(), vec![Kind::Glass]);
        assert!(shelf.changed().is_empty());
        let (text, origin) = shelf.read(Kind::Glass);
        assert_eq!(text, "// two, longer");
        assert!(matches!(origin, Origin::File(_)));
        std::fs::remove_file(&path).unwrap();
        assert_eq!(shelf.changed(), vec![Kind::Glass]);
        let (text, origin) = shelf.read(Kind::Glass);
        assert!(matches!(origin, Origin::Embedded));
        assert!(text.contains("@fragment"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
