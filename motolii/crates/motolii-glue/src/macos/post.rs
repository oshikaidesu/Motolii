//! Post: looks from the old shelf (`vism/*.wgsl`, an ISF header before WGSL) run over the
//! finished picture, between glass and blit. `shelf/post.txt` says which, one per line:
//! `<stem> [name=value ...]`, values the card's DEFAULT unless named. The list and each card
//! file are watched like the shelf; a bad line or card prints red and the last good stays.
//!
//! The card's contract (group 0: image, then targets, texture 2n + sampler 2n+1; group 1: one
//! uniform per param, then render_size, then pass_index) is the old host's, so a card means
//! the same thing here. Pass targets are Rgba16Float when FLOAT, else the picture's Bgra8Unorm.

use std::path::{Path, PathBuf};

use super::isf::{self, Manifest};
use super::shelf::{sampler_entry, stamp, texture_entry, uniform_entry, Stamp};

const RED: &str = "\x1b[31m";
const PLAIN: &str = "\x1b[0m";

pub(super) struct Post {
    list: PathBuf,
    list_stamp: Stamp,
    dir: PathBuf,
    looks: Vec<Look>,
    /// Nearest, then linear: the card's FILTER picks.
    samplers: [wgpu::Sampler; 2],
    /// Look k writes `out[k % 2]` and look k+1 reads it; blit reads the last one written.
    out: [(wgpu::Texture, wgpu::TextureView); 2],
    size: (u32, u32),
}

struct Look {
    line: Line,
    path: PathBuf,
    stamp: Stamp,
    card: Card,
    /// Group 0 per pass; depends on the look's source, so rebuilt when the chain changes.
    reads: Vec<wgpu::BindGroup>,
}

/// One compiled card: everything that does not depend on where its source comes from.
struct Card {
    manifest: Manifest,
    read_layout: wgpu::BindGroupLayout,
    pipelines: Vec<(wgpu::TextureFormat, wgpu::RenderPipeline)>,
    targets: Vec<wgpu::TextureView>,
    params: Vec<wgpu::Buffer>,
    passes: Vec<Slot>,
}

struct Slot {
    target: Option<usize>,
    format: wgpu::TextureFormat,
    values: wgpu::BindGroup,
}

/// One line of `post.txt`.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Line {
    pub stem: String,
    pub params: Vec<(String, [f32; 4])>,
}

/// `MOTOLII_VISM`, else the render crate's `vism/` beside this one.
pub(super) fn vism_dir() -> PathBuf {
    std::env::var_os("MOTOLII_VISM")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../motolii-render/vism"))
}

/// `<stem> [name=value ...]`; a value is a number or up to four, comma-separated.
/// Blank lines and `#` lines are nothing.
pub(super) fn parse_line(raw: &str) -> Result<Option<Line>, String> {
    let mut words = raw.split_whitespace();
    let Some(stem) = words.next().filter(|stem| !stem.starts_with('#')) else { return Ok(None) };
    let mut params = Vec::new();
    for word in words {
        let (name, text) = word.split_once('=').ok_or_else(|| format!("`{word}` is not name=value"))?;
        if params.iter().any(|(have, _)| have == name) {
            return Err(format!("`{name}` is given twice"));
        }
        let mut value = [0.0f32; 4];
        for (slot, part) in value.iter_mut().zip(text.split(',')) {
            *slot = part.parse().map_err(|_| format!("{name}: `{text}` is not a number"))?;
        }
        params.push((name.to_owned(), value));
    }
    Ok(Some(Line { stem: stem.to_owned(), params }))
}

/// Param values in declaration order: the line's, else the card's DEFAULT.
/// A name the card does not declare is an error, not silence.
pub(super) fn values(manifest: &Manifest, line: &Line) -> Result<Vec<[f32; 4]>, String> {
    for (name, _) in &line.params {
        if !manifest.params().any(|param| &param.name == name) {
            return Err(format!("{} has no param `{name}`", line.stem));
        }
    }
    Ok(manifest
        .params()
        .map(|param| line.params.iter().find(|(name, _)| *name == param.name).map_or(param.default, |(_, v)| *v))
        .collect())
}

impl Post {
    pub(super) fn new(device: &wgpu::Device, shelf: &Path, width: u32, height: u32) -> Self {
        let list = shelf.join("post.txt");
        let dir = vism_dir();
        eprintln!("A1: post list {} watched; cards from {}", list.display(), dir.display());
        let sampler = |filter: wgpu::FilterMode| {
            device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("post"),
                mag_filter: filter,
                min_filter: filter,
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                ..Default::default()
            })
        };
        let out = |label: &str| {
            let texture = target(device, label, (width, height), wgpu::TextureFormat::Bgra8Unorm);
            let view = texture.create_view(&Default::default());
            (texture, view)
        };
        Self {
            list,
            list_stamp: None,
            dir,
            looks: Vec::new(),
            samplers: [sampler(wgpu::FilterMode::Nearest), sampler(wgpu::FilterMode::Linear)],
            out: [out("post-0"), out("post-1")],
            size: (width, height),
        }
    }

    /// Where blit reads from once the looks ran; `None` when there is nothing to run.
    pub(super) fn output(&self) -> Option<&wgpu::Texture> {
        self.looks.last().map(|_| &self.out[(self.looks.len() - 1) % 2].0)
    }

    /// Files saved since the last frame take effect now. Compiles inside an error scope on the
    /// calling thread, so this belongs where the shelf reloads, not inside a render.
    pub(super) fn reload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, source: &wgpu::Texture) {
        let mut rebind = false;
        let now = stamp(&self.list);
        if self.list_stamp != now {
            self.list_stamp = now;
            self.relist(device, queue);
            rebind = true;
        }
        for look in &mut self.looks {
            let now = stamp(&look.path);
            if look.stamp == now {
                continue;
            }
            look.stamp = now;
            match load(device, queue, &look.path, &look.line, self.size) {
                Ok(card) => {
                    look.card = card;
                    rebind = true;
                    eprintln!("A1: shelf reloaded {} from {}", look.line.stem, look.path.display());
                }
                Err(error) => eprintln!(
                    "{RED}ERROR shelf {} ({}): {error}\nlast valid pipeline retained{PLAIN}",
                    look.line.stem,
                    look.path.display()
                ),
            }
        }
        if rebind {
            self.bind(device, source);
        }
    }

    /// The list changed: looks whose card is already compiled keep it and take the new values;
    /// the others compile. A line that fails keeps its old look if it had one.
    fn relist(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let text = std::fs::read_to_string(&self.list).unwrap_or_default();
        let mut old = std::mem::take(&mut self.looks);
        for (number, raw) in text.lines().enumerate() {
            let number = number + 1;
            let line = match parse_line(raw) {
                Ok(Some(line)) => line,
                Ok(None) => continue,
                Err(error) => {
                    eprintln!("{RED}ERROR post.txt line {number}: {error}\nline skipped{PLAIN}");
                    continue;
                }
            };
            let path = self.dir.join(format!("{}.wgsl", line.stem));
            if let Some(index) = old.iter().position(|look| look.line.stem == line.stem) {
                let mut look = old.remove(index);
                match values(&look.card.manifest, &line) {
                    Ok(values) => {
                        look.card.write(queue, &values);
                        look.line = line;
                        eprintln!("A1: post line {number}: {}", raw.trim());
                    }
                    Err(error) => eprintln!("{RED}ERROR post.txt line {number}: {error}\nlast valid look retained{PLAIN}"),
                }
                self.looks.push(look);
                continue;
            }
            match load(device, queue, &path, &line, self.size) {
                Ok(card) => {
                    eprintln!("A1: post line {number}: {} ({} passes from {})", raw.trim(), card.passes.len(), path.display());
                    self.looks.push(Look { line, path: path.clone(), stamp: stamp(&path), card, reads: Vec::new() });
                }
                Err(error) => eprintln!("{RED}ERROR post.txt line {number} ({}): {error}\nline skipped{PLAIN}", path.display()),
            }
        }
        if self.looks.is_empty() {
            eprintln!("A1: post: no look; blit reads the glass pass");
        }
    }

    fn bind(&mut self, device: &wgpu::Device, source: &wgpu::Texture) {
        let mut previous = source.create_view(&Default::default());
        for (index, look) in self.looks.iter_mut().enumerate() {
            look.reads = look.card.reads(device, &previous, &self.samplers[usize::from(look.card.manifest.linear)]);
            previous = self.out[index % 2].1.clone();
        }
    }

    /// Every pass of every look, in order, into the encoder of the frame. Nothing is created here.
    pub(super) fn encode(&self, encoder: &mut wgpu::CommandEncoder) {
        for (index, look) in self.looks.iter().enumerate() {
            for (number, slot) in look.card.passes.iter().enumerate() {
                let view = slot.target.map_or(&self.out[index % 2].1, |target| &look.card.targets[target]);
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("post"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                pass.set_pipeline(look.card.pipeline(slot.format));
                pass.set_bind_group(0, &look.reads[number], &[]);
                pass.set_bind_group(1, &slot.values, &[]);
                pass.draw(0..3, 0..1);
            }
        }
    }
}

/// Read, parse, check, compile. wgpu reports shader and layout errors asynchronously, so the
/// GPU side runs inside an error scope; the error text carries naga's diagnostic.
fn load(device: &wgpu::Device, queue: &wgpu::Queue, path: &Path, line: &Line, size: (u32, u32)) -> Result<Card, String> {
    let source = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    let manifest = isf::parse(&source)?;
    if manifest.images().count() != 1 {
        return Err(format!("the post host feeds one image; the card declares {}", manifest.images().count()));
    }
    if manifest.passes.iter().any(|pass| pass.persistent) {
        return Err("PERSISTENT targets are not kept by the post host".into());
    }
    let values = values(&manifest, line)?;
    let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let card = build(device, queue, manifest, &source, size);
    if let Some(error) = pollster::block_on(scope.pop()) {
        return Err(error.to_string());
    }
    card.write(queue, &values);
    Ok(card)
}

/// The host side of a card: layouts from the manifest, one pipeline per target format,
/// one texture per named target, and the per-pass uniforms (render_size, pass_index).
fn build(device: &wgpu::Device, queue: &wgpu::Queue, manifest: Manifest, source: &str, size: (u32, u32)) -> Card {
    let label = manifest.label.clone().unwrap_or_default();
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(&label),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let names = manifest.targets();
    let mut read_entries = Vec::new();
    for image in 0..=names.len() as u32 {
        read_entries.push(texture_entry(image * 2));
        read_entries.push(sampler_entry(image * 2 + 1));
    }
    let param_count = manifest.params().count() as u32;
    let value_entries: Vec<_> = (0..param_count + 2).map(|b| uniform_entry(b, wgpu::ShaderStages::VERTEX_FRAGMENT)).collect();
    let read_layout = layout(device, &label, &read_entries);
    let value_layout = layout(device, &label, &value_entries);
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(&label),
        bind_group_layouts: &[Some(&read_layout), Some(&value_layout)],
        immediate_size: 0,
    });
    let extent = |pass: &isf::Pass| {
        (
            pass.width.map_or(size.0, |d| d.resolve(size.0)),
            pass.height.map_or(size.1, |d| d.resolve(size.1)),
        )
    };
    let format_of = |pass: &isf::Pass| match &pass.target {
        Some(name) if manifest.declaring(name).is_some_and(|first| first.float) => wgpu::TextureFormat::Rgba16Float,
        _ => wgpu::TextureFormat::Bgra8Unorm,
    };
    let targets: Vec<wgpu::TextureView> = names
        .iter()
        .map(|name| {
            let first = manifest.declaring(name).expect("named target has a declaring pass");
            target(device, name, extent(first), format_of(first)).create_view(&Default::default())
        })
        .collect();
    let mut pipelines: Vec<(wgpu::TextureFormat, wgpu::RenderPipeline)> = Vec::new();
    for format in manifest.passes.iter().map(format_of) {
        if pipelines.iter().any(|(have, _)| *have == format) {
            continue;
        }
        pipelines.push((format, pipeline(device, &label, &module, &pipeline_layout, format)));
    }
    let params: Vec<wgpu::Buffer> = manifest.params().map(|param| uniform(device, &param.name)).collect();
    let passes = manifest
        .passes
        .iter()
        .enumerate()
        .map(|(index, pass)| {
            let target = pass.target.as_deref().and_then(|name| names.iter().position(|have| *have == name));
            let (w, h) = target.map_or(size, |t| extent(manifest.declaring(names[t]).expect("declared")));
            let render_size = uniform(device, "render_size");
            queue.write_buffer(&render_size, 0, &bytes([w as f32, h as f32, 0.0, 0.0]));
            let pass_index = uniform(device, "pass_index");
            queue.write_buffer(&pass_index, 0, &bytes([index as f32, 0.0, 0.0, 0.0]));
            let mut entries: Vec<wgpu::BindGroupEntry<'_>> = params
                .iter()
                .enumerate()
                .map(|(binding, buffer)| wgpu::BindGroupEntry { binding: binding as u32, resource: buffer.as_entire_binding() })
                .collect();
            entries.push(wgpu::BindGroupEntry { binding: param_count, resource: render_size.as_entire_binding() });
            entries.push(wgpu::BindGroupEntry { binding: param_count + 1, resource: pass_index.as_entire_binding() });
            let values = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&label),
                layout: &value_layout,
                entries: &entries,
            });
            Slot { target, format: format_of(pass), values }
        })
        .collect();
    Card { manifest, read_layout, pipelines, targets, params, passes }
}

impl Card {
    fn pipeline(&self, format: wgpu::TextureFormat) -> &wgpu::RenderPipeline {
        &self.pipelines.iter().find(|(have, _)| *have == format).expect("a pipeline per format").1
    }

    fn write(&self, queue: &wgpu::Queue, values: &[[f32; 4]]) {
        for (buffer, value) in self.params.iter().zip(values) {
            queue.write_buffer(buffer, 0, &bytes(*value));
        }
    }

    /// Group 0 per pass: the source, then the targets; a pass never reads the target it writes,
    /// so that slot holds the source instead.
    fn reads(&self, device: &wgpu::Device, source: &wgpu::TextureView, sampler: &wgpu::Sampler) -> Vec<wgpu::BindGroup> {
        self.passes
            .iter()
            .map(|slot| {
                let mut entries = Vec::new();
                for (image, view) in std::iter::once(source).chain(self.targets.iter()).enumerate() {
                    let view = if slot.target == Some(image.wrapping_sub(1)) { source } else { view };
                    entries.push(wgpu::BindGroupEntry { binding: image as u32 * 2, resource: wgpu::BindingResource::TextureView(view) });
                    entries.push(wgpu::BindGroupEntry { binding: image as u32 * 2 + 1, resource: wgpu::BindingResource::Sampler(sampler) });
                }
                device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("post"), layout: &self.read_layout, entries: &entries })
            })
            .collect()
    }
}

fn layout(device: &wgpu::Device, label: &str, entries: &[wgpu::BindGroupLayoutEntry]) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some(label), entries })
}

/// Full-screen triangle, `vs_main` / `fs_main`, opaque, one sample: the card's one shape.
fn pipeline(
    device: &wgpu::Device,
    label: &str,
    module: &wgpu::ShaderModule,
    layout: &wgpu::PipelineLayout,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState { format, blend: None, write_mask: wgpu::ColorWrites::ALL })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

fn target(device: &wgpu::Device, label: &str, (width, height): (u32, u32), format: wgpu::TextureFormat) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

/// One vec4 slot each: a float reads its x, a point2D its xy, a color all four; the pass
/// buffer's x is the pass index whether the card declares f32 or vec4f.
fn uniform(device: &wgpu::Device, label: &str) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: 16,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn bytes(value: [f32; 4]) -> [u8; 16] {
    let mut out = [0u8; 16];
    for (chunk, v) in out.chunks_exact_mut(4).zip(value) {
        chunk.copy_from_slice(&v.to_ne_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{parse_line, values};

    #[test]
    fn a_line_names_a_card_and_overrides_only_what_it_names() {
        let manifest = super::isf::parse(
            "/*{ \"INPUTS\": [{\"NAME\":\"source\",\"TYPE\":\"image\"}, \
             {\"NAME\":\"threshold\",\"TYPE\":\"float\",\"DEFAULT\":0.6}, \
             {\"NAME\":\"tint\",\"TYPE\":\"color\",\"DEFAULT\":[1,0.5,0.25,1]}], \
             \"PASSES\": [{\"TARGET\":\"half\",\"FLOAT\":true,\"WIDTH\":\"$WIDTH/2\",\"HEIGHT\":\"$HEIGHT/2\"}, {}] }*/ fn x() {}",
        )
        .unwrap();
        let line = parse_line("  glow threshold=0.55 tint=0,1,0,1 ").unwrap().unwrap();
        assert_eq!(line.stem, "glow");
        assert_eq!(values(&manifest, &line).unwrap(), vec![[0.55, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 1.0]]);
        let bare = parse_line("glow").unwrap().unwrap();
        assert_eq!(values(&manifest, &bare).unwrap(), vec![[0.6, 0.0, 0.0, 0.0], [1.0, 0.5, 0.25, 1.0]]);
        let wrong = parse_line("glow thresholdd=1").unwrap().unwrap();
        assert!(values(&manifest, &wrong).unwrap_err().contains("thresholdd"));
        assert!(parse_line("# a comment").unwrap().is_none());
        assert!(parse_line("   ").unwrap().is_none());
        assert!(parse_line("glow threshold=abc").unwrap_err().contains("threshold"));
        assert!(parse_line("glow threshold").unwrap_err().contains("name=value"));
        assert!(parse_line("glow threshold=1 threshold=2").unwrap_err().contains("twice"));
    }
}
