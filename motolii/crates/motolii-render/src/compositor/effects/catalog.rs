use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock, atomic::{AtomicU64, AtomicBool, Ordering}};

use super::{isf, subtype, VismDefinition, VismSource};
use super::subtype::{ParamSubtype, Subtype};

/// 棚の 1 枚が何を返すか(席)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectStage {
    /// texture → texture の 2D pass。
    Pass,
    /// Material-local XY warp; independent of layer projection.
    Warp,
    /// 網の面の hook(fork の `motolii_surface`)。
    Surface,
    /// 網の頂点の hook(fork の `motolii_field`)。点群は CPU の写しで受ける。
    Field,
    /// 世界の平面で切る。板・点群・網が同じ式に従う。
    Clip,
    /// 配置の集合(shader を持たない)。
    Placement,
    /// 形の層の輪郭(shader を持たない)。
    Path,
    /// 平らな素材から立体を起こす(shader を持たない): 押し出し・縁の丸み。
    Solid,
}

#[derive(Clone, Debug)]
pub struct EffectDescriptor {
    pub plugin_id: String,
    pub label: String,
    pub stage: EffectStage,
    pub params: Vec<EffectParamDescriptor>,
    /// 札の絵(PNG): shader の隣の `<plugin_id>_snapshot.png`。VST3 の Plug-in Snapshot の型。
    pub snapshot: Option<Arc<[u8]>>,
    pub(crate) padding: Option<EffectPaddingDescriptor>,
    /// coverage 外の出力を下へ合成する混ぜ方(溢れの法)。無ければ層の Blend のまま。
    pub(crate) spill: Option<crate::render::compositor::BlendMode>,
    pub(crate) output_format: wgpu::TextureFormat,
    /// 2 枚目以降の image が要求する時刻のずれ(秒。負が過去)。宣言順。
    pub(crate) image_time_offsets: Vec<isf::TimeOffset>,
    /// 時計(`TIME` 系)を読む。読む効果だけ、時刻が変われば焼き直す。
    pub(crate) uses_clock: bool,
    /// pass が下の合成(BACKDROP_INPUT)を読む。層の絵へは焼けず、描いた後の窓で効く。
    pub(crate) reads_backdrop: bool,
    /// 2 枚目の image が層を指す欄の名前(`LAYER`)。宣言順。
    pub(crate) image_layer_fields: Vec<String>,
    /// PERSISTENT な target を持つ(feedback)。
    pub(crate) persistent: bool,
}

#[derive(Clone, Debug)]
pub struct EffectParamDescriptor {
    pub name: String,
    /// 窓に出る英語。ISF は name のまま。
    pub label: String,
    pub default: f64,
    /// 点の欄(Vec2)ならその既定。数の欄は None。
    pub point: Option<[f64; 2]>,
    /// 色の欄(RGBA 0..1)ならその既定。
    pub color: Option<[f64; 4]>,
    /// 層を指す欄。値は LayerId(0 = 無し)。窓はカメラの target と同じ選択肢で描く。
    pub layer: bool,
    pub range: Option<(f64, f64)>,
    /// 選択肢。値は番号。
    pub choices: Option<Vec<String>>,
    /// 性格(Blender の subtype 語彙)。manifest の `SUBTYPE` か、shader の使われ方の次元解析から。
    pub subtype: Option<String>,
    /// px / ° / % / ""。
    pub unit: Option<String>,
    /// 同じ点の仲間(先頭の欄の名前)。
    pub group: Option<String>,
    /// 畳んでおく欄。
    pub advanced: bool,
    /// 主役の欄(宣言)。
    pub hero: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct EffectPaddingDescriptor {
    pub(crate) param: String,
    pub(crate) scale: f32,
}

#[derive(Clone, Debug)]
pub struct CatalogRefresh {
    pub generation: u64,
    pub changed: bool,
    pub errors: Vec<String>,
}

pub(crate) struct CatalogSnapshot {
    pub(crate) generation: u64,
    pub(crate) definitions: Arc<[VismDefinition]>,
    pub(crate) descriptors: Arc<[EffectDescriptor]>,
    errors: Vec<String>,
}

#[derive(Default)]
struct CatalogOwner {
    stamps: BTreeMap<PathBuf, (Option<std::time::SystemTime>, u64)>,
    snapshot: Option<Arc<CatalogSnapshot>>,
}

#[derive(Clone)]
pub struct CatalogRuntime(Arc<CatalogRuntimeState>);

struct CatalogRuntimeState {
    owner: Mutex<CatalogOwner>,
    dirty: AtomicBool,
    generation: AtomicU64,
}

impl Default for CatalogRuntime {
    fn default() -> Self {
        Self(Arc::new(CatalogRuntimeState { owner: Mutex::new(CatalogOwner::default()),
            dirty: AtomicBool::new(true), generation: AtomicU64::new(0) }))
    }
}

impl CatalogRuntime {
    pub fn identity(&self) -> usize { Arc::as_ptr(&self.0) as usize }
    pub fn generation(&self) -> u64 { self.0.generation.load(Ordering::Acquire) }
    pub fn is_dirty(&self) -> bool { self.0.dirty.load(Ordering::Acquire) }
}

static ACTIVE_RUNTIME: OnceLock<Mutex<Option<CatalogRuntime>>> = OnceLock::new();

fn runtime_at(slot: &Mutex<Option<CatalogRuntime>>) -> CatalogRuntime {
    slot.lock().unwrap().get_or_insert_with(CatalogRuntime::default).clone()
}

fn active_runtime() -> CatalogRuntime {
    runtime_at(ACTIVE_RUNTIME.get_or_init(Default::default))
}

fn bind_runtime_at(slot: &Mutex<Option<CatalogRuntime>>, runtime: &CatalogRuntime) {
    *slot.lock().unwrap() = Some(runtime.clone());
}

pub fn bind_catalog_runtime(runtime: &CatalogRuntime) {
    let slot = ACTIVE_RUNTIME.get_or_init(Default::default);
    let changed = slot.lock().unwrap().as_ref().is_none_or(|old| old.identity() != runtime.identity());
    if changed {
        bind_runtime_at(slot, runtime);
    }
}

fn directory() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("vism") }

/// 札の絵(shader と同じ dir の PNG)。開発中は disk、配布物は build.rs が埋めた物。
fn picture_bytes(file: &str) -> Option<Arc<[u8]>> {
    #[cfg(load_shaders_from_disk)]
    { std::fs::read(directory().join(file)).ok().map(Arc::from) }
    #[cfg(not(load_shaders_from_disk))]
    { super::VISM_PICTURES.iter().find(|p| p.file == file).map(|p| Arc::from(p.bytes)) }
}

/// `<id>_snapshot_2.0x.png` があればそれ、無ければ `<id>_snapshot.png`。
fn snapshot(plugin_id: &str) -> Option<Arc<[u8]>> {
    picture_bytes(&format!("{plugin_id}_snapshot_2.0x.png")).or_else(|| picture_bytes(&format!("{plugin_id}_snapshot.png")))
}
fn prelude_path() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../reference/vello-blend.wgsl") }

pub fn catalog_source_roots() -> Vec<PathBuf> { vec![directory(), prelude_path()].into_iter().map(|p| p.canonicalize().unwrap_or(p)).collect() }

pub fn catalog_generation() -> u64 { active_runtime().generation() }

/// 最後の読み直しで断った物(名前: 理由)。空なら棚は全部生きている。
/// 窓はこれを status に載せて出す — 効果を書く人が保存した瞬間に、なぜ載らないかを見るため。
pub fn catalog_errors() -> Vec<String> {
    active_runtime().0.owner.lock().unwrap().snapshot.as_ref().map(|s| s.errors.clone()).unwrap_or_default()
}

/// 棚が disk(vism/)を読んでいるか。焼き込み build では false で、保存しても窓は変わらない。
pub fn catalog_reads_disk() -> bool { cfg!(load_shaders_from_disk) }

pub(crate) fn catalog_snapshot() -> Arc<CatalogSnapshot> {
    let runtime = active_runtime();
    let owner = &runtime.0.owner;
    if let Some(snapshot) = owner.lock().unwrap().snapshot.clone() { return snapshot; }
    refresh_runtime(&runtime);
    let snapshot = owner.lock().unwrap().snapshot.as_ref().unwrap().clone();
    snapshot
}

fn schema(manifest: &isf::IsfManifest) -> String {
    format!("{:?}|{:?}|{}|{}|{:?}|{:?}", manifest.id, manifest.stage, manifest.expose, manifest.output_float, (&manifest.passes, manifest.linear_sampling, manifest.specialize_passes),
        manifest.inputs.iter().map(|i| (&i.name, i.ty, &i.maps)).collect::<Vec<_>>())
}

fn type_key(module: &naga::Module, ty: naga::Handle<naga::Type>) -> String {
    use naga::TypeInner;
    match &module.types[ty].inner {
        TypeInner::Array { base, size, stride } => format!("array:{size:?}:{stride}:{}", type_key(module, *base)),
        TypeInner::Struct { members, span } => format!("struct:{span}:{:?}", members.iter().map(|m| (m.offset, &m.binding, type_key(module, m.ty))).collect::<Vec<_>>()),
        TypeInner::Pointer { base, space } => format!("pointer:{space:?}:{}", type_key(module, *base)),
        other => format!("{other:?}"),
    }
}

fn uniform_components(module: &naga::Module, ty: naga::Handle<naga::Type>) -> usize {
    match &module.types[ty].inner {
        naga::TypeInner::Scalar(s) if s.kind == naga::ScalarKind::Float && s.width == 4 => 1,
        naga::TypeInner::Vector { size, scalar } if scalar.kind == naga::ScalarKind::Float && scalar.width == 4 => *size as usize,
        naga::TypeInner::Struct { members, .. } if members.len() == 1 && members[0].offset == 0 => uniform_components(module, members[0].ty),
        _ => 0,
    }
}

/// WGSL の parser の唯一の口(検証も性格の解析もここを通る)。
pub(crate) fn parse_wgsl(source: &str) -> Result<naga::Module, String> {
    naga::front::wgsl::parse_str(source).map_err(|e| e.emit_to_string(source))
}

fn validate_stage(source: &str, entry: &str, stage: naga::ShaderStage, manifest: &isf::IsfManifest) -> Result<String, String> {
    let module = parse_wgsl(source)?;
    naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
        .validate(&module).map_err(|e| e.to_string())?;
    if !module.entry_points.iter().any(|e| e.name == entry && e.stage == stage) {
        return Err(format!("missing {stage:?} entry point {entry}"));
    }
    let params = manifest.param_inputs().collect::<Vec<_>>();
    let images = manifest.image_inputs().count() + manifest.target_slots().len();
    for (_, variable) in module.global_variables.iter() {
        let Some(binding) = variable.binding.as_ref() else { continue };
        let ty = &module.types[variable.ty].inner;
        let accepted = match binding.group {
            0 if binding.binding < (images * 2) as u32 => if binding.binding % 2 == 0 {
                matches!(ty, naga::TypeInner::Image { dim: naga::ImageDimension::D2, arrayed: false, class: naga::ImageClass::Sampled { kind: naga::ScalarKind::Float, multi: false } })
            } else { matches!(ty, naga::TypeInner::Sampler { comparison: false }) },
            1 if variable.space == naga::AddressSpace::Uniform => {
                let components = if (binding.binding as usize) < params.len() { params[binding.binding as usize].ty.component_count() }
                    else if binding.binding as usize == params.len() { 2 }
                    // 段の buffer は (段, TIME, TIMEDELTA, FRAMEINDEX)。同梱の WGSL は先頭の f32 だけ、ISF は vec4 で読む。
                    else if binding.binding as usize == params.len() + 1 { 4 }
                    else if manifest.stage == isf::IsfStage::Warp && binding.binding as usize == params.len() + 2 { 2 }
                    else { 0 };
                let found = uniform_components(&module, variable.ty);
                components > 0 && (found == components || (binding.binding as usize == params.len() + 1 && found == 1))
            }
            _ => false,
        };
        if !accepted { return Err(format!("shader binding {}:{} is incompatible with the manifest layout", binding.group, binding.binding)); }
    }
    let mut bindings = module.global_variables.iter().filter_map(|(_, v)| v.binding.as_ref().map(|b| {
        (b.group, b.binding, format!("{:?}", v.space), type_key(&module, v.ty))
    })).collect::<Vec<_>>();
    bindings.sort();
    let entries = module.entry_points.iter().map(|e| (
        &e.name, e.stage,
        e.function.arguments.iter().map(|a| (&a.binding, type_key(&module, a.ty))).collect::<Vec<_>>(),
        e.function.result.as_ref().map(|r| (&r.binding, type_key(&module, r.ty))),
    )).collect::<Vec<_>>();
    Ok(format!("{bindings:?}|{entries:?}"))
}

fn prepare(source: VismSource, prelude: &str) -> Result<VismDefinition, String> {
    // 貼られた物はここで ISF の言い方へ写し、以後は同梱の効果と同じ道を通る。
    let source = match isf::shadertoy::dialect(&source.source) {
        Some(isf::shadertoy::Dialect::Shadertoy) => {
            let id = format!("import.{}", source.name);
            let text = isf::shadertoy::isf_from_shadertoy(&id, &source.name, &source.source)?;
            VismSource { source: text.into(), extension: "fs".into(), ..source }
        }
        Some(isf::shadertoy::Dialect::Glsl) => {
            return Err(format!("{}: manifest が無い — ISF の /*{{ ... }}*/ を書くか、Shadertoy の mainImage で書く", source.name));
        }
        _ => source,
    };
    if source.extension != "fs" {
        let (manifest, body) = isf::parse_isf_source(&source.source).map_err(|e| e.to_string())?;
        if !matches!(manifest.stage, isf::IsfStage::Pass | isf::IsfStage::Warp) {
            // hook の snippet。型は fork の base と合わせて初めて決まるので、ここでは欄の型だけ縛る。
            for input in manifest.param_inputs() {
                if input.ty.component_count() != 1 {
                    return Err(format!("{}: hook の欄は float / long / bool だけ", input.name));
                }
            }
            if manifest.param_inputs().count() > super::surface_program::PARAM_SLOTS {
                return Err(format!("hook の欄は {} 個まで", super::surface_program::PARAM_SLOTS));
            }
            let interface = schema(&manifest);
            let names = manifest.param_inputs().map(|p| p.name.clone()).collect::<Vec<_>>();
            let stage_params = match manifest.stage {
                isf::IsfStage::Field => "FieldParams",
                isf::IsfStage::Clip => "ClipParams",
                _ => "SurfaceParams",
            };
            let subtypes = parse_wgsl(&subtype::hook_stub(&body, stage_params, &names))
                .map(|m| subtype::analyze_module(&m, names.len())).unwrap_or_else(|_| subtype::unknown(names.len()));
            return Ok(VismDefinition { source, manifest, interface, vertex_text: body.clone(), fragment_text: body,
                vertex_entry: String::new(), fragment_entry: String::new(), subtypes });
        }
    }
    // 棚に出る効果が読める絵は、今は層の絵 1 枚だけ。2 枚目を宣言されても繋ぐ先が無いので、
    // 黙って 1 枚目を二度束ねず(絵は出るが意味が違う)、名指しで断る。内部の効果(blend・matte)は
    // 呼び手が明示的に 2 枚渡すので対象外。
    {
        let manifest = if source.extension == "fs" {
            isf::compiled_stages(&source.source).map_err(|e| e.to_string())?.0
        } else {
            isf::parse_isf_source(&source.source).map_err(|e| e.to_string())?.0
        };
        if manifest.expose {
            let images: Vec<&isf::IsfInput> = manifest.inputs.iter()
                .filter(|i| i.ty == isf::IsfInputType::Image)
                .collect();
            // 2 枚目以降は、ホストが供給できる物だけ — 今は「層の絵を別の時刻で読む」(TIME_OFFSET)。
            if let Some(extra) = images.iter().skip(1).find(|i| i.time_offset.is_none() && i.layer_field.is_none() && manifest.backdrop_input.as_deref() != Some(i.name.as_str())) {
                return Err(format!(
                    "{}: 2 枚目以降の画像を繋ぐ口がまだ無い(層を指す欄が未実装)。\
                     読めるのは層の絵 1 枚、TIME_OFFSET を宣言した別の時刻の絵、\
                     BACKDROP_INPUT の下の合成、LAYER で指した層の絵、PASSES の中間 buffer だけ",
                    extra.name
                ));
            }
        }
    }
    let (manifest, vertex, fragment, vertex_entry, fragment_entry) = if source.extension == "fs" {
        let (manifest, vertex, fragment) = isf::compiled_stages(&source.source).map_err(|e| e.to_string())?;
        (manifest, vertex, fragment, "main", "main")
    } else {
        let manifest = isf::parse_isf_source(&source.source).map_err(|e| e.to_string())?.0;
        let text = if manifest.stage == isf::IsfStage::Warp {
            format!("{}\n{}", re_renderer::noise::WGSL, source.source)
        } else if matches!(source.name.as_str(), "blend" | "matte") {
            format!("{prelude}\n{}", source.source)
        } else { source.source.to_string() };
        (manifest, text.clone(), text, "vs_main", "fs_main")
    };
    let interface = format!("{}|{}|{}", schema(&manifest), validate_stage(&vertex, vertex_entry, naga::ShaderStage::Vertex, &manifest)?, validate_stage(&fragment, fragment_entry, naga::ShaderStage::Fragment, &manifest)?);
    let n = manifest.param_inputs().count();
    let subtypes = parse_wgsl(&fragment).map(|m| subtype::analyze_module(&m, n)).unwrap_or_else(|_| subtype::unknown(n));
    Ok(VismDefinition { source, manifest, interface, vertex_text: vertex, fragment_text: fragment,
        vertex_entry: vertex_entry.into(), fragment_entry: fragment_entry.into(), subtypes })
}

fn descriptors(definitions: &[VismDefinition]) -> Arc<[EffectDescriptor]> {
    // shader を持たない棚の 1 枚(配置・表面・場)は doc の 1 つの表から。棚と Inspector には同じ列で並ぶ。
    let declared = crate::doc::store::kind::all().map(|kind| EffectDescriptor {
        persistent: false,
        plugin_id: kind.plugin_id.to_owned(),
        label: kind.label.to_owned(),
        stage: match kind.family {
            crate::doc::store::kind::Family::Placement => EffectStage::Placement,
            crate::doc::store::kind::Family::Path => EffectStage::Path,
            crate::doc::store::kind::Family::Solid => EffectStage::Solid,
        },
        params: kind.params.iter().map(|p| EffectParamDescriptor {
            name: p.name.to_owned(), label: p.label.to_owned(), default: p.default[0], range: p.range,
            point: matches!(p.kind, crate::doc::store::kind::ParamKind::Vec2).then_some(p.default),
            color: None,
            layer: false,
            choices: p.choices().map(|c| c.iter().map(|s| (*s).to_owned()).collect()),
            subtype: None, unit: None, group: None, advanced: false, hero: false,
        }).collect(),
        snapshot: snapshot(kind.plugin_id),
        padding: None,
        spill: None,
        output_format: wgpu::TextureFormat::Rgba8Unorm,
        image_time_offsets: Vec::new(),
        uses_clock: false,
        reads_backdrop: false,
        image_layer_fields: Vec::new(),
    });
    definitions.iter().filter(|d| d.manifest.expose).map(|d| EffectDescriptor {
        persistent: d.manifest.passes.iter().any(|p| p.persistent),
        image_time_offsets: d.manifest.inputs.iter()
            .filter(|i| i.ty == isf::IsfInputType::Image)
            .skip(1)
            .filter_map(|i| i.time_offset.clone())
            .collect(),
        uses_clock: d.manifest.uses_clock,
        reads_backdrop: d.manifest.stage == isf::IsfStage::Pass && d.manifest.backdrop_input.is_some(),
        image_layer_fields: d.manifest.inputs.iter().filter(|i| i.ty == isf::IsfInputType::Image).skip(1).filter_map(|i| i.layer_field.clone()).collect(),
        plugin_id: d.plugin_id().to_owned(),
        label: d.label(),
        snapshot: snapshot(d.plugin_id()),
        stage: match d.manifest.stage {
            isf::IsfStage::Pass => EffectStage::Pass,
            isf::IsfStage::Warp => EffectStage::Warp,
            isf::IsfStage::Surface => EffectStage::Surface,
            isf::IsfStage::Field => EffectStage::Field,
            isf::IsfStage::Clip => EffectStage::Clip,
        },
        params: d.manifest.param_inputs().enumerate().map(|(i, p)| {
            let range = p.min.zip(p.max).map(|(min, max)| (min[0] as f64, max[0] as f64))
                .or_else(|| p.labels.as_ref().map(|l| (0.0, (l.len().max(1) - 1) as f64)));
            let read = d.subtypes.get(i).cloned().unwrap_or_default();
            let (subtype, unit, group) = character(p, &read, range, d);
            EffectParamDescriptor {
                name: p.name.clone(), label: p.label.clone().unwrap_or_else(|| p.name.clone()), default: p.default[0] as f64,
                // 点(2 と 3 成分)は x,y を窓へ。3 成分目は窓に部品が無く既定のまま。
                point: matches!(p.ty, isf::IsfInputType::Point2D | isf::IsfInputType::Point3D).then(|| [p.default[0] as f64, p.default[1] as f64]),
                color: matches!(p.ty, isf::IsfInputType::Color).then(|| [p.default[0] as f64, p.default[1] as f64, p.default[2] as f64, p.default[3] as f64]),
                layer: p.ty == isf::IsfInputType::Layer,
                range, choices: p.labels.clone(), subtype, unit, group, advanced: p.advanced, hero: p.hero,
            }
        }).collect(),
        padding: d.manifest.padding.as_ref().map(|p| EffectPaddingDescriptor { param: p.param.clone(), scale: p.scale }),
        spill: d.manifest.spill.as_deref().map(|mode| match mode { "add" => crate::render::compositor::BlendMode::Add, "multiply" => crate::render::compositor::BlendMode::Multiply, _ => crate::render::compositor::BlendMode::Screen }),
        output_format: d.output_format(),
    }).chain(declared).collect::<Vec<_>>().into()
}

/// 欄の性格: 宣言(`SUBTYPE`)が先、無ければ使われ方の解析、どちらも無ければ不明のまま。
/// 単位は性格から: 角度は °、長さは px、0〜1 か 0〜100 の量は %。
fn character(p: &isf::IsfInput, read: &ParamSubtype, range: Option<(f64, f64)>, d: &VismDefinition) -> (Option<String>, Option<String>, Option<String>) {
    let declared = p.subtype.as_deref().and_then(Subtype::parse);
    let subtype = declared.or(read.subtype);
    let percent = matches!(range, Some((0.0, max)) if max == 1.0 || max == 100.0);
    let unit = match subtype {
        Some(Subtype::Angle) => Some("°"),
        Some(Subtype::Distance | Subtype::Translation) => if declared.is_some() { Some("px") } else { read.unit },
        Some(Subtype::Factor | Subtype::Opacity) if percent => Some("%"),
        Some(Subtype::Factor | Subtype::Opacity | Subtype::Level) => Some(""),
        _ => None,
    };
    let group = read.group.and_then(|g| d.manifest.param_inputs().nth(g)).map(|g| g.name.clone());
    (subtype.map(|s| s.name().to_owned()), unit.map(str::to_owned), group)
}

pub fn refresh_effect_catalog() -> CatalogRefresh { refresh_runtime(&active_runtime()) }

pub fn refresh_effect_catalog_for(runtime: &CatalogRuntime) -> CatalogRefresh { refresh_runtime(runtime) }

fn refresh_runtime(runtime: &CatalogRuntime) -> CatalogRefresh {
    let mut owner = runtime.0.owner.lock().unwrap();
    let previous = owner.snapshot.clone();
    if !runtime.0.dirty.swap(false, Ordering::AcqRel) {
        if let Some(old) = previous.as_ref() { return CatalogRefresh { generation: old.generation, changed: false, errors: old.errors.clone() }; }
    }
    let mut errors = Vec::new();
    #[cfg(load_shaders_from_disk)]
    let (sources, prelude) = {
        let mut paths = match std::fs::read_dir(directory()) {
            Ok(entries) => entries.filter_map(Result::ok).map(|e| e.path()).filter(|p| matches!(p.extension().and_then(|x| x.to_str()), Some("wgsl" | "fs" | "frag" | "glsl"))).collect::<Vec<_>>(),
            Err(e) => { errors.push(format!("catalog directory: {e}")); Vec::new() }
        };
        paths.sort();
        let stamps = paths.iter().chain(std::iter::once(&prelude_path())).filter_map(|p| {
            std::fs::metadata(p).ok().map(|m| (p.clone(), (m.modified().ok(), m.len())))
        }).collect::<BTreeMap<_, _>>();
        if stamps == owner.stamps {
            if let Some(old) = previous.as_ref() { return CatalogRefresh { generation: old.generation, changed: false, errors: old.errors.clone() }; }
        }
        owner.stamps = stamps;
        let sources = paths.into_iter().filter_map(|path| match std::fs::read_to_string(&path) {
            Ok(text) => Some(VismSource { name: path.file_stem().unwrap().to_string_lossy().into_owned(), extension: path.extension().unwrap().to_string_lossy().into_owned(), source: text.into() }),
            Err(e) => { errors.push(format!("{}: {e}", path.display())); None }
        }).collect::<Vec<_>>();
        let prelude = std::fs::read_to_string(prelude_path()).unwrap_or_else(|e| { errors.push(format!("blend prelude: {e}")); String::new() });
        (sources, prelude)
    };
    #[cfg(not(load_shaders_from_disk))]
    let (sources, prelude) = (super::VISM_SOURCES.iter().map(|s| VismSource {
        name: s.name.into(), extension: s.extension.into(), source: s.source.into(),
    }).collect::<Vec<_>>(), super::VELLO_BLEND_PRELUDE.to_owned());

    let mut definitions = previous.as_ref().map(|p| p.definitions.iter().map(|d| (d.source.name.clone(), d.clone())).collect::<BTreeMap<_, _>>()).unwrap_or_default();
    let mut changed = previous.is_none();
    let present = sources.iter().map(|s| s.name.clone()).collect::<std::collections::BTreeSet<_>>();
    for source in sources {
        let name = source.name.clone();
        match prepare(source, &prelude) {
            Ok(candidate) => {
                if let Some(old) = definitions.get(&name) {
                    if old.interface != candidate.interface { errors.push(format!("{name}: incompatible shader interface; last valid program retained")); continue; }
                    if old.source.source == candidate.source.source && old.vertex_text == candidate.vertex_text && old.fragment_text == candidate.fragment_text { continue; }
                }
                if definitions.iter().any(|(n, d)| n != &name && d.plugin_id() == candidate.plugin_id()) {
                    errors.push(format!("{name}: duplicate effect ID; last valid catalog retained")); continue;
                }
                if let Err(error) = candidate.stage() { errors.push(format!("{name}: {error}")); continue; }
                definitions.insert(name, candidate);
                changed = true;
            }
            Err(error) => errors.push(format!("{name}: {error}; last valid program retained")),
        }
    }
    for name in definitions.keys().filter(|n| !present.contains(*n)) {
        errors.push(format!("{name}: source removed; last valid program retained"));
    }
    let generation = previous.as_ref().map_or(0, |p| p.generation) + u64::from(changed);
    let definitions: Arc<[VismDefinition]> = definitions.into_values().collect::<Vec<_>>().into();
    owner.snapshot = Some(Arc::new(CatalogSnapshot { generation, descriptors: descriptors(&definitions), definitions, errors: errors.clone() }));
    runtime.0.generation.store(generation, Ordering::Release);
    CatalogRefresh { generation, changed, errors }
}

pub struct CatalogWatcher {
    runtime: CatalogRuntime,
    #[cfg(load_shaders_from_disk)]
    _watcher: notify::RecommendedWatcher,
}

impl CatalogWatcher {
    pub fn runtime(&self) -> CatalogRuntime { self.runtime.clone() }
}

pub fn watch_effect_catalog(on_change: impl Fn() + Send + Sync + 'static) -> Result<CatalogWatcher, String> {
    let runtime = active_runtime();
    #[cfg(load_shaders_from_disk)]
    {
        use notify::Watcher;
        let wake_runtime = runtime.clone();
        let mut watcher = notify::recommended_watcher(move |event: Result<notify::Event, notify::Error>| {
            if event.as_ref().is_ok_and(|e| matches!(e.kind, notify::EventKind::Create(_) | notify::EventKind::Modify(_) | notify::EventKind::Remove(_))) { wake_runtime.0.dirty.store(true, Ordering::Release); on_change(); }
        }).map_err(|e| e.to_string())?;
        watcher.watch(&directory(), notify::RecursiveMode::NonRecursive).map_err(|e| e.to_string())?;
        watcher.watch(&prelude_path(), notify::RecursiveMode::NonRecursive).map_err(|e| e.to_string())?;
        Ok(CatalogWatcher { runtime, _watcher: watcher })
    }
    #[cfg(not(load_shaders_from_disk))]
    { let _ = on_change; Ok(CatalogWatcher { runtime }) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blur(text: String) -> Result<VismDefinition, String> {
        prepare(VismSource { name: "blur".into(), extension: "wgsl".into(), source: text.into() }, "")
    }

    /// 貼った Shadertoy が、同梱の効果と同じ道で棚に出る(棚に出る条件 = ここを抜けること)。
    #[test]
    fn a_pasted_shadertoy_becomes_an_effect() {
        let source = "void mainImage(out vec4 c, in vec2 p) { c = vec4(p / iResolution.xy, sin(iTime), 1.0); }";
        let definition = prepare(VismSource { name: "city".into(), extension: "frag".into(), source: source.into() }, "").unwrap();
        assert_eq!(definition.plugin_id(), "import.city");
        assert!(matches!(definition.manifest.stage, isf::IsfStage::Pass));
        // manifest の無い素の GLSL は、理由を名前つきで断る。
        let plain = prepare(VismSource { name: "bare".into(), extension: "frag".into(), source: "void main() { gl_FragColor = vec4(1.0); }".into() }, "").err().expect("manifest の無い GLSL は断る");
        assert!(plain.contains("bare") && plain.contains("manifest"), "{plain}");
    }

    #[test]
    fn default_range_and_body_changes_preserve_the_shader_interface_and_handles() {
        let source = include_str!("../../../vism/blur.wgsl");
        let old = blur(source.into()).unwrap();
        let new = blur(source.replace("\"DEFAULT\": 8.0", "\"DEFAULT\": 12.0")
            .replace("\"MAX\": 128.0", "\"MAX\": 96.0")
            .replace("radius * 0.5", "radius * 0.6")).unwrap();
        assert_eq!(old.interface, new.interface);
        assert_eq!(old.paths(), new.paths());
        assert_eq!(descriptors(&[old])[0].params[0].default, 8.0);
        let descriptors = descriptors(&[new]);
        assert_eq!(descriptors[0].params[0].default, 12.0);
        assert_eq!(descriptors[0].params[0].range, Some((0.0, 96.0)));
    }

    #[test]
    fn incompatible_bindings_and_missing_entry_points_fail_before_staging() {
        let source = include_str!("../../../vism/blur.wgsl");
        assert!(blur(source.replace("@group(1)", "@group(2)")).is_err());
        assert!(blur(source.replace("fn fs_main", "fn other_main")).is_err());
        assert!(blur(source.replace("return blur(blur_h_tex", "return missing(blur_h_tex")).is_err());
        let old = blur(source.into()).unwrap();
        let renamed = blur(source.replace("radius", "spread")).unwrap();
        assert_ne!(old.interface, renamed.interface);
    }

    #[test]
    fn copied_entrypoint_rebinds_the_watcher_runtime_and_last_good_snapshot() {
        let runtime = CatalogRuntime::default();
        let source = include_str!("../../../vism/blur.wgsl");
        let good = blur(source.into()).unwrap();
        let snapshot = Arc::new(CatalogSnapshot {
            generation: 7,
            descriptors: descriptors(std::slice::from_ref(&good)),
            definitions: vec![good].into(),
            errors: vec!["invalid replacement retained the valid program".into()],
        });
        runtime.0.owner.lock().unwrap().snapshot = Some(snapshot.clone());
        runtime.0.generation.store(7, Ordering::Release);
        runtime.0.dirty.store(false, Ordering::Release);
        let copied_slot = Mutex::new(None);
        assert_eq!(runtime_at(&copied_slot).generation(), 0);
        bind_runtime_at(&copied_slot, &runtime);
        let rebound = runtime_at(&copied_slot);
        assert!(blur(source.replace("return blur(blur_h_tex", "return missing(blur_h_tex")).is_err());
        runtime.0.dirty.store(true, Ordering::Release);
        assert_eq!(rebound.identity(), runtime.identity());
        assert_eq!(rebound.generation(), 7);
        assert!(rebound.0.dirty.load(Ordering::Acquire));
        assert!(Arc::ptr_eq(rebound.0.owner.lock().unwrap().snapshot.as_ref().unwrap(), &snapshot));
    }

    /// 貼った ISF に、点・色の**全成分**と時計が届く(実 GPU、program 1 つで 2×2 を描いて読む)。
    /// `name` は shader の本文の置き場になる — 並走する test 同士で同じ名前を使うと上書きし合う。
    fn run_once(name: &str, source: &str, params: &[(&str, f32)]) -> [u8; 4] {
        let mut compositor = crate::render::compositor::Compositor::headless().unwrap();
        let definition = prepare(VismSource { name: name.into(), extension: "fs".into(), source: source.into() }, "").unwrap();
        // shader の本文は棚が書く。ここは棚を通らないので自分で書く(radiance の参照 test と同じ)。
        definition.stage().unwrap();
        let building = compositor.ctx.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let program = super::super::EffectProgram::compile_for(&compositor.ctx, &definition, wgpu::TextureFormat::Rgba8Unorm);
        // pipeline は次の frame の頭で組まれる。組ませてから記録し、転送を流してから submit。
        compositor.ctx.before_submit();
        compositor.ctx.begin_frame();
        let error = pollster::block_on(building.pop());
        assert!(error.is_none(), "pipeline が組めない: {error:?}");
        let ctx = &compositor.ctx;
        let texture = |usage| ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: None, size: wgpu::Extent3d { width: 2, height: 2, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1,
            dimension: wgpu::TextureDimension::D2, format: wgpu::TextureFormat::Rgba8Unorm, usage, view_formats: &[],
        });
        let src = texture(wgpu::TextureUsages::TEXTURE_BINDING);
        let dst = texture(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC);
        let mut scratch = super::super::EffectScratch::default();
        let params: Vec<(String, f32)> = params.iter().map(|(k, v)| ((*k).to_owned(), *v)).collect();
        let scope = ctx.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let mut encoder = ctx.device.create_command_encoder(&Default::default());
        program.record(ctx, &mut encoder, &mut scratch, &[&src.create_view(&Default::default())], &dst.create_view(&Default::default()), &params, [2.0, 2.0]);
        let buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor { label: None, size: 256 * 2, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
        encoder.copy_texture_to_buffer(dst.as_image_copy(), wgpu::TexelCopyBufferInfo { buffer: &buffer, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(256), rows_per_image: Some(2) } }, wgpu::Extent3d { width: 2, height: 2, depth_or_array_layers: 1 });
        let commands = encoder.finish();
        drop(ctx);
        compositor.ctx.before_submit();
        let ctx = &compositor.ctx;
        ctx.queue.submit([commands]);
        let error = pollster::block_on(scope.pop());
        assert!(error.is_none(), "GPU の検証に落ちた: {error:?}");
        let slice = buffer.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        ctx.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        let data = slice.get_mapped_range();
        [data[0], data[1], data[2], data[3]]
    }

    #[test]
    fn a_point_and_a_colour_arrive_with_every_component() {
        let source = "/*{ \"INPUTS\": [{\"NAME\":\"inputImage\",\"TYPE\":\"image\"}, {\"NAME\":\"p\",\"TYPE\":\"point2D\",\"DEFAULT\":[0.0,0.0]}, {\"NAME\":\"c\",\"TYPE\":\"color\",\"DEFAULT\":[0.0,0.0,0.0,0.0]}] }*/\n\
                      void main() { gl_FragColor = vec4(p.x / 64.0, p.y / 64.0, c.b, c.a); }";
        let px = run_once("contract-point", source, &[("p", 16.0), ("p.1", 32.0), ("c.2", 1.0), ("c.3", 0.5)]);
        let near = |a: u8, b: u8| a.abs_diff(b) <= 2;
        assert!(near(px[0], 64) && near(px[1], 128) && near(px[2], 255) && near(px[3], 128), "{px:?}: 成分が欠けている");
    }

    #[test]
    fn the_clock_reaches_the_shader() {
        let source = "/*{ \"INPUTS\": [{\"NAME\":\"inputImage\",\"TYPE\":\"image\"}] }*/\n\
                      void main() { gl_FragColor = vec4(TIME / 10.0, float(FRAMEINDEX) / 255.0, TIMEDELTA * 10.0, 1.0); }";
        let px = run_once("contract-clock", source, &[("TIME", 2.0), ("FRAMEINDEX", 48.0), ("TIMEDELTA", 0.1)]);
        let near = |a: u8, b: u8| a.abs_diff(b) <= 2;
        assert!(near(px[0], 51) && near(px[1], 48) && near(px[2], 255), "{px:?}: 時計が届いていない");
    }

    #[cfg(load_shaders_from_disk)]
    #[test]
    fn catalog_refresh_keeps_the_open_renderer_frame_and_device() {
        let mut compositor = crate::render::compositor::Compositor::headless().unwrap();
        compositor.ctx.before_submit();
        compositor.ctx.begin_frame();
        let current = catalog_snapshot();
        compositor.catalog = Arc::new(CatalogSnapshot {
            generation: current.generation.wrapping_sub(1),
            definitions: current.definitions.clone(),
            descriptors: current.descriptors.clone(),
            errors: Vec::new(),
        });
        let frame = compositor.ctx.active_frame_idx();
        let device = compositor.ctx.device.clone();
        let shaders = compositor.ctx.gpu_resources.shader_modules.num_resources();
        compositor.refresh_catalog_programs();
        assert_eq!(compositor.ctx.active_frame_idx(), frame);
        assert_eq!(compositor.ctx.device, device);
        assert_eq!(compositor.ctx.gpu_resources.shader_modules.num_resources(), shaders);
    }
}

#[cfg(test)]
mod character_oracle {
    /// 棚の全 Vism について、欄の性格が使われ方から読めること(名前は見ていない)。
    /// 期待値は 2026-09-08 の目視で確定した写像。変えるなら理由をここに書く。
    #[test]
    fn every_shader_param_has_its_character_read_from_use() {
        let runtime = super::CatalogRuntime::default();
        super::refresh_effect_catalog_for(&runtime);
        let snapshot = runtime.0.owner.lock().unwrap().snapshot.clone().unwrap();
        let mut seen = std::collections::BTreeMap::new();
        for d in snapshot.descriptors.iter() {
            for p in &d.params {
                seen.insert(format!("{}.{}", d.plugin_id, p.name), (p.subtype.clone(), p.unit.clone(), p.group.clone()));
            }
        }
        let s = |v: &str| Some(v.to_owned());
        let expected: &[(&str, Option<String>, Option<String>, Option<String>)] = &[
            ("motolii.isf_bloom.threshold", s("LEVEL"), s(""), None),
            ("motolii.isf_bloom.intensity", s("FACTOR"), s(""), None),
            ("motolii.isf_bloom.radius", s("DISTANCE"), s("px"), None),
            ("motolii.blur.radius", s("DISTANCE"), s("px"), None),
            ("motolii.gain.gain", s("FACTOR"), s(""), None),
            ("motolii.glass.roughness", s("FACTOR"), s("%"), None),
            ("motolii.glow.threshold", s("LEVEL"), s(""), None),
            ("motolii.glow.intensity", s("FACTOR"), s(""), None),
            ("motolii.glow.radius", s("DISTANCE"), s("px"), None),
            // shader が読まない欄(定数式にだけ使う)は不明のまま — 嘘を付けない。
            ("motolii.tri_led.glow", None, None, None),
            ("motolii.turbulent_displace.size", s("DISTANCE"), s("px"), None),
            ("motolii.turbulent_displace.complexity", s("COUNT"), None, None),
            // 使われ方では平行移動に見える。manifest の SUBTYPE が勝つ(宣言 > 解析)。
            ("motolii.turbulent_displace.evolution", s("TIME"), None, None),
            ("motolii.turbulent_displace.offset_x", s("TRANSLATION"), s("px"), s("offset_x")),
            ("motolii.turbulent_displace.offset_y", s("TRANSLATION"), s("px"), s("offset_x")),
            ("motolii.turbulent_displace.offset_z", s("TRANSLATION"), s("px"), s("offset_x")),
            // shader を持たない棚(配置)は解析の外。
            ("motolii.repeat.seed", None, None, None),
        ];
        for (key, subtype, unit, group) in expected {
            assert_eq!(seen.get(*key), Some(&(subtype.clone(), unit.clone(), group.clone())), "{key}");
        }
    }
}
