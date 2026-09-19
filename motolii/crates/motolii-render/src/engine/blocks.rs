//! 箱のブロック(`STAGE: block`)を 1 コマに掛ける: 層を組む時に物の箱を集め、組み終えたら GPU で効果の順に解いて
//! 描く側の motion の buffer に書く。読み戻さない(利用者 2026-09-15「この天井を作るべきでない」)。

use std::collections::HashMap;

use crate::doc::core::CompSpec;
use crate::doc::store::{LayerId, MaskFrame, PropertyId, RationalTime, ResolvedLayer, ResolvedMask, StoreView, Value};
use crate::render::compositor::effects::block_program::{BlockItem, BlockProgram, BlockWorld, FollowPass, WorldPass};
use crate::render::compositor::effects::isf::IsfStage;
use crate::render::compositor::Layer;
use crate::render::engine::{Engine, EngineError};

/// 効果の列の何番目のブロックか・どのブロックか・欄の値が同じ物を、1 回の計算に束ねる。
pub(crate) struct BlockBatch {
    stage: usize,
    plugin: String,
    params: Vec<f32>,
    members: Vec<u32>,
    /// 場(`SCOPE: room`)の元の物の番号。掛かった層そのものが動くブロックは `u32::MAX`。
    source: u32,
    /// 名指しの深さ(親・anchor を辿った段数)。同じ stage では浅い方(読まれる側)が先に走る。
    rank: usize,
}

/// 形の層の輪郭(素材座標)。当たりは四角ではなく、この形そのもので見る
/// (利用者 2026-09-16「今のコリジョンの当たり判定は四角で変です。2d も 3d もシルエットが算出できるはず」)。
/// 2D は書類の形(`vector::resolve`)、3D の網・粒は `media::silhouette_points`(まだ箱のまま)。
fn outline_of(shapes: &[crate::doc::vector::ShapeNode], stretch: [f32; 2]) -> Option<Vec<[f32; 2]>> {
    let shapes = if stretch == [1.0, 1.0] { shapes.to_vec() } else { crate::doc::vector::stretch_outline(shapes, stretch) };
    let leaves = crate::doc::vector::flatten(&shapes).ok()?;
    let canvas = crate::doc::vector::content_canvas(&shapes).ok().flatten()?;
    let (ox, oy) = (canvas.origin_x as f32, canvas.origin_y as f32);
    let mut points = Vec::new();
    for shape in &leaves {
        if shape.fill.is_none() {
            continue;
        }
        for instance in crate::doc::vector::resolve(shape).ok()?.iter() {
            flatten_contours(&instance.path, [ox, oy], &mut points);
        }
    }
    (points.len() >= 3).then_some(points)
}

/// 輪郭の列を点に割る(曲がりに合わせて、lyon の適応分割)。文字も形も同じ。
fn flatten_contours(contours: &[crate::doc::vector::Contour], offset: [f32; 2], points: &mut Vec<[f32; 2]>) {
    let (ox, oy) = (offset[0], offset[1]);
    for contour in contours {
        let vs = &contour.vertices;
        if vs.len() < 2 {
            continue;
        }
        let last = if contour.closed { vs.len() } else { vs.len() - 1 };
        for i in 0..last {
            let (a, b) = (&vs[i], &vs[(i + 1) % vs.len()]);
            let p = |x: f64, y: f64| lyon_geom::point(x as f32, y as f32);
            let curve = lyon_geom::CubicBezierSegment {
                from: p(a.point.x, a.point.y),
                ctrl1: p(a.point.x + a.out_tangent.x, a.point.y + a.out_tangent.y),
                ctrl2: p(b.point.x + b.in_tangent.x, b.point.y + b.in_tangent.y),
                to: p(b.point.x, b.point.y),
            };
            points.push([curve.from.x + ox, curve.from.y + oy]);
            curve.for_each_flattened(0.6, &mut |line| {
                points.push([line.to.x + ox, line.to.y + oy]);
            });
        }
    }
}

/// 名指しの深さ: 相手(親・anchor)の無い物は 0、相手が居れば相手の深さ + 1。輪は 0 で切る。
/// 同じ stage の batch はこの順に走るので、読む側は読まれる側の今を見る。
fn reference_depth(items: &[BlockItem]) -> Vec<usize> {
    fn go(items: &[BlockItem], k: usize, memo: &mut [Option<usize>], visiting: &mut Vec<usize>) -> usize {
        if let Some(d) = memo[k] {
            return d;
        }
        if visiting.contains(&k) {
            return 0;
        }
        visiting.push(k);
        let d = [items[k].parent_slot, items[k].anchor_slot].into_iter()
            .filter(|j| (*j as usize) < items.len())
            .map(|j| go(items, j as usize, memo, visiting) + 1)
            .max().unwrap_or(0);
        visiting.pop();
        memo[k] = Some(d);
        d
    }
    let mut memo = vec![None; items.len()];
    (0..items.len()).map(|k| go(items, k, &mut memo, &mut Vec::new())).collect()
}

/// 書類から先に読む、物ごとの住む箱と箱。
#[derive(Clone)]
struct Placed {
    room: [f32; 4],
    radius: f32,
    own: [f32; 4],
    group: u32,
    weight: f32,
    /// 書類の間合い(CSS の margin)。物同士はこれだけ空けて当たる。
    margin: f32,
    /// 手触り(0 返す ↔ 0.5 吸う ↔ 1 引きずる)。
    hardness: f32,
    /// 形そのものの輪郭(素材座標)。無ければ箱で当たる。
    outline: Option<std::sync::Arc<Vec<[f32; 2]>>>,
}

#[derive(Default)]
pub(crate) struct BlockState {
    /// ブロックごとの GPU の道と、組んだ時の本文(棚が読み直されたら組み直す)。
    programs: HashMap<String, (String, BlockProgram)>,
    world: Option<BlockWorld>,
    world_pass: Option<WorldPass>,
    follow_pass: Option<FollowPass>,
    placed: HashMap<LayerId, Placed>,
    /// 付いて置く札 → 相手(相手がブロックを持つ時だけ)。
    follows: HashMap<LayerId, LayerId>,
    /// 物の番号 → 層(付いて行く対を番号にする)。
    object_layers: Vec<LayerId>,
    /// 場が立っている住む箱の組(そこに居る物は、効果を持たなくても動く物として並べる)。
    field_rooms: std::collections::HashSet<u32>,
    /// comp の最後のコマ(集まる時の「終わり」)。
    last_frame: i64,
    /// 物ごとの時刻(順番の札でずれたコマ)。object_layers と同じ並び。
    object_frames: Vec<i64>,
    /// 場の元(物の番号)と、その効果の順・欄。動く相手は物が揃ってから決める。
    fields: Vec<(usize, String, Vec<f32>, u32)>,
    objects: Vec<BlockItem>,
    /// 外の解き手(Rapier)。前のコマの力を覚えている。
    physics: crate::render::engine::physics::Physics,
    /// このコマの秒あたりのコマ数と時刻(可視の層が描く直前に解くために覚える)。
    fps: f64,
    now: Option<RationalTime>,
    bases: Vec<([f32; 3], [f32; 3], [f32; 3])>,
    /// 物ごとの輪郭(comp の px、`objects` と同じ順)。
    outlines: Vec<Option<std::sync::Arc<Vec<[f32; 2]>>>>,
    /// 層 → 物の番号(層を組む時に motion の番号を配るため)。
    slots: HashMap<LayerId, u32>,
    /// つなぐ線 → (元の物, 先の物)。物の後ろに並ぶ motion(kind 1)で両端に付いて行く。
    connectors: Vec<(LayerId, u32, u32)>,
    /// なぞる形 → 相手の物。相手の motion をそのまま読む。
    traces: HashMap<LayerId, u32>,
    /// 紐(Line Path = Rope)のつなぐ線: (connectors の番号, Slack %, 硬さ, 減衰)。
    ropes: Vec<(u32, f32, f32, f32)>,
    rope_pass: Option<crate::render::compositor::effects::block_program::RopePass>,
    batches: Vec<BlockBatch>,
}

impl BlockState {
    /// 解き手が動かす物か(描く前に間引かないため)。
    pub(crate) fn moves(&self, layer: LayerId) -> bool {
        self.slots.contains_key(&layer)
    }
}

impl Engine {
    /// 今のコマの物ごとのずれ(震えを測る道具のため。読み戻すので描画では使わない)。
    /// 測り用: 当たりに使っている輪郭の外接(comp の px、解き手が動かした後)。輪郭の無い物は箱。
    pub fn physics_outline_bounds(&self) -> Vec<[f32; 4]> {
        let hulls = self.physics_hulls();
        let mut out = Vec::new();
        for (k, it) in self.blocks.objects.iter().enumerate() {
            let layer = self.blocks.object_layers[k];
            let Some((shift, _)) = self.blocks.physics.offset(layer) else { continue };
            let bounds = match hulls.get(k).filter(|h| h.len() >= 3) {
                Some(h) => h.iter().fold([f32::MAX, f32::MAX, f32::MIN, f32::MIN], |b, p| [b[0].min(p[0]), b[1].min(p[1]), b[2].max(p[0]), b[3].max(p[1])]),
                None => [it.lo[0] + shift[0], it.lo[1] + shift[1], it.hi[0] + shift[0], it.hi[1] + shift[1]],
            };
            out.push(bounds);
        }
        out
    }

    /// 可視のモードが読む: 物理の物の箱(comp の px、解き手が動かした後)。
    pub(crate) fn physics_marks(&self) -> Vec<crate::doc::store::analysis::BlobMark> {
        self.blocks.objects.iter().enumerate().filter_map(|(k, it)| {
            let layer = *self.blocks.object_layers.get(k)?;
            let (shift, _) = self.blocks.physics.offset(layer)?;
            Some(crate::doc::store::analysis::BlobMark {
                id: k as u32,
                center: [(it.lo[0] + it.hi[0]) * 0.5 + shift[0], (it.lo[1] + it.hi[1]) * 0.5 + shift[1]],
                size: [it.hi[0] - it.lo[0], it.hi[1] - it.lo[1]],
                age: 0,
            })
        }).collect()
    }

    /// 可視のモードが読む: 物ごとのずれ、触れ合いの線、場の元と届く輪。
    pub(crate) fn physics_offset(&self, layer: LayerId) -> Option<([f32; 2], f32)> {
        self.blocks.physics.offset(layer)
    }

    pub(crate) fn physics_links(&self) -> Vec<([f32; 2], [f32; 2])> {
        self.blocks.physics.contact_lines()
    }

    pub(crate) fn physics_wells(&self) -> Vec<([f32; 2], f32, [f32; 2])> {
        self.blocks.physics.wells()
    }

    pub(crate) fn physics_contacts_at(&self) -> Vec<([f32; 2], [f32; 2])> {
        self.blocks.physics.contact_marks()
    }

    pub(crate) fn physics_velocities(&self) -> Vec<([f32; 2], [f32; 2])> {
        self.blocks.physics.velocities()
    }

    /// 当たりに使っている輪郭(解き手が動かした後の場所へ移したもの)。
    pub(crate) fn physics_hulls(&self) -> Vec<Vec<[f32; 2]>> {
        self.blocks.objects.iter().enumerate().filter_map(|(k, _)| {
            let layer = *self.blocks.object_layers.get(k)?;
            let outline = self.blocks.outlines.get(k)?.as_ref()?;
            let (shift, turn) = self.blocks.physics.offset(layer)?;
            let (sin, cos) = turn.to_radians().sin_cos();
            let it = self.blocks.objects.get(k)?;
            let mid = glam::vec2((it.lo[0] + it.hi[0]) * 0.5, (it.lo[1] + it.hi[1]) * 0.5);
            Some(outline.iter().map(|p| {
                let d = glam::Vec2::from(*p) - mid;
                let turned = glam::vec2(d.x * cos - d.y * sin, d.x * sin + d.y * cos);
                (mid + turned + glam::Vec2::from(shift)).to_array()
            }).collect())
        }).collect()
    }

    /// 触れ合っている組の数(測り用)。
    pub fn physics_contacts(&self) -> usize {
        self.blocks.physics.contacts()
    }

    pub fn block_states(&self) -> Vec<[f32; 3]> {
        let Some(world) = self.blocks.world.as_ref() else { return Vec::new() };
        let ctx = &self.compositor.ctx;
        let encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("motolii-block-read") });
        crate::render::compositor::effects::block_program::read_state(&ctx.device, &ctx.queue, world, encoder)
            .iter().map(|o| [o.translate[0], o.translate[1], o.rotate]).collect()
    }

    /// 計測の口: 物ごとの今のずれを全部(位置 2・回転・大きさ・色 3・不透明)。描く道は読み戻さない。
    pub fn block_offsets(&self) -> Vec<[f32; 8]> {
        let Some(world) = self.blocks.world.as_ref() else { return Vec::new() };
        let ctx = &self.compositor.ctx;
        let encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("motolii-block-read") });
        crate::render::compositor::effects::block_program::read_state(&ctx.device, &ctx.queue, world, encoder)
            .iter().map(|o| [o.translate[0], o.translate[1], o.rotate, o.scale, o.tint[0], o.tint[1], o.tint[2], o.tint[3]]).collect()
    }

    /// ブロックを持つ層の住む箱(書類の親の Group の箱、無ければ comp の枠)と、物の箱・組・譲る比を読む。
    pub(super) fn prepare_blocks(&mut self, view: &StoreView<'_>, comp: CompSpec, t: RationalTime, resolved: &[ResolvedLayer]) -> Result<(), EngineError> {
        let store = |e: crate::doc::store::StoreError| EngineError::Store(e.to_string());
        let blocks: Vec<String> = self.compositor.catalog.definitions.iter().filter(|d| d.manifest.stage == IsfStage::Block).map(|d| d.plugin_id().to_owned()).collect();
        // ブロックの宣言(id・場かどうか・欄の名前と既定)を先に写す(借りの重なりを避ける)。
        let definitions: Vec<(String, bool, Vec<(String, f32)>)> = self.compositor.catalog.definitions.iter()
            .filter(|d| d.manifest.stage == IsfStage::Block)
            .map(|d| (
                d.plugin_id().to_owned(),
                d.manifest.scope == crate::render::compositor::effects::isf::IsfScope::Room,
                d.manifest.param_inputs().map(|p| (p.name.clone(), p.default[0])).collect(),
            ))
            .collect();
        let field_blocks: Vec<String> = self.compositor.catalog.definitions.iter()
            .filter(|d| d.manifest.stage == IsfStage::Block && d.manifest.scope == crate::render::compositor::effects::isf::IsfScope::Room)
            .map(|d| d.plugin_id().to_owned()).collect();
        // 嘘と解き手の欄は棚の札から(場の vism の `"PHYSICS"`)。Rust は部品だけ。
        let lies = self.compositor.catalog.definitions.iter()
            .find(|d| d.manifest.stage == IsfStage::Block && d.manifest.scope == crate::render::compositor::effects::isf::IsfScope::Room && !d.manifest.physics.is_empty())
            .map(|d| crate::render::engine::physics::Lies::from_manifest(&d.manifest.physics)).unwrap_or_default();
        if self.blocks.physics.lies != lies {
            self.blocks.physics = Default::default();
            self.blocks.physics.lies = lies.clone();
        }
        // 抜いた後の形(効果を通した後の透過)は解析の段で取ってある。物理は「描かれた物の形」で当たる。
        let mut extents: HashMap<LayerId, [f32; 4]> = HashMap::new();
        for layer in resolved.iter().filter(|l| l.copy == 0 && !l.ghost) {
            if matches!(view.meta(layer.id).map_err(store)?.map(|m| m.source), Some(crate::doc::store::LayerSource::Shape)) {
                continue;
            }
            let extent = match crate::doc::store::layout::boxes::layer_box(view, layer.id, t).map_err(store)? {
                Some(own) => Some(own),
                // 絵・動画の寸法は、書類の解析の口に入る前は描く側だけが知っている。物理は待たずに読む。
                None => match view.meta(layer.id).map_err(store)?.map(|m| m.source) {
                    Some(crate::doc::store::LayerSource::File { path, .. }) => self.material_extent(&path, comp).map(|e| [0.0, 0.0, e[0], e[1]]),
                    _ => None,
                },
            };
            if let Some(extent) = extent {
                extents.insert(layer.id, extent);
            }
        }
        let keyed = self.keyed_outlines.clone();
        let state = &mut self.blocks;
        state.fps = view.composition().ok().flatten().map_or(30.0, |c| c.fps.as_f64());
        state.last_frame = view.composition().ok().flatten().map_or(0, |c| c.duration_frames.max(1) - 1);
        state.now = Some(t);
        state.placed.clear();
        state.follows.clear();
        state.object_layers.clear();
        state.object_frames.clear();
        state.slots.clear();
        state.connectors.clear();
        state.traces.clear();
        state.ropes.clear();
        state.field_rooms.clear();
        state.fields.clear();
        state.objects.clear();
        state.bases.clear();
        state.outlines.clear();
        state.batches.clear();
        if blocks.is_empty() {
            return Ok(());
        }
        // 場が立っている住む箱を先に見る: その箱に居る物は、効果を持たなくても場に動かされる。
        for layer in resolved.iter().filter(|l| l.copy == 0 && !l.ghost && l.effects.iter().any(|e| field_blocks.contains(&e.plugin_id))) {
            let parent = view.attrs(layer.id).map_err(store)?.unwrap_or_default().parent;
            state.field_rooms.insert(parent.map_or(0, |p| p.0 as u32));
        }
        let field_rooms = state.field_rooms.clone();
        let solved = view.layout_frame(t).map_err(store)?;
        // 部屋 = 場の立っている一番近い先祖の箱(提案 2026-09-16「場のある箱が部屋」)。入れ子の中の字も、
        // ポスター全体に立った場で散れる。箱(Group)そのものは物にならず、中の葉が物。
        let room_ancestor = lies.room_ancestor;
        let room_of = |view: &StoreView<'_>, id: LayerId| -> Result<Option<u32>, EngineError> {
            let mut at = view.attrs(id).map_err(store)?.unwrap_or_default().parent;
            loop {
                let key = at.map_or(0, |p| p.0 as u32);
                if field_rooms.contains(&key) {
                    return Ok(Some(key));
                }
                if !room_ancestor {
                    return Ok(None);
                }
                let Some(p) = at else { return Ok(None) };
                at = view.attrs(p).map_err(store)?.unwrap_or_default().parent;
            }
        };
        let mut room_ids: HashMap<LayerId, u32> = HashMap::new();
        let mut wanted = Vec::new();
        // つなぐ線の両端は、ブロックを持たなくても物として並べる(線は両端の motion を読むので、動かない端にも項が要る)。
        let mut needed: std::collections::HashSet<LayerId> = std::collections::HashSet::new();
        for layer in resolved.iter().filter(|l| l.copy == 0 && !l.ghost) {
            if let Some((from, to)) = view.connection(layer.id, t).map_err(store)? {
                needed.insert(from);
                needed.insert(to);
            }
            if let Some((target, _)) = view.tracing(layer.id, t).map_err(store)? {
                needed.insert(target);
            }
        }
        // 名指しの相手(親の層、Position Anchor の層)は、ブロックがその今を読むので物として並べる(`parent(k)` / `anchor(k)`)。
        // 番号は最後(名指しだけで物になる層を先に並べると、既存の物の `k` がずれて絵が変わる)。
        let mut named: std::collections::HashSet<LayerId> = std::collections::HashSet::new();
        let anchor_row = PropertyId::new(crate::doc::store::layout::POSITION_ANCHOR).map_err(store)?;
        let anchor_of = |view: &StoreView<'_>, id: LayerId| -> Result<Option<LayerId>, EngineError> {
            Ok(match view.value_at(id, &anchor_row, t).map_err(store)? {
                Some(Value::LayerId(id)) if id != 0 => Some(LayerId(id)),
                Some(Value::F64(v)) if v >= 1.0 => Some(LayerId(v.round() as u64)),
                _ => None,
            })
        };
        for layer in resolved.iter().filter(|l| l.copy == 0 && !l.ghost && l.effects.iter().any(|e| blocks.contains(&e.plugin_id))) {
            if let Some(parent) = view.attrs(layer.id).map_err(store)?.unwrap_or_default().parent {
                named.insert(parent);
            }
            if let Some(target) = anchor_of(view, layer.id)? {
                named.insert(target);
            }
        }
        for layer in resolved.iter().filter(|l| l.copy == 0 && !l.ghost) {
            // 見せるための層(可視の重ね、つなぐ線)は物にしない。物理の相手は画の中身だけ。
            if layer.effects.iter().any(|e| crate::extensions::overlay::is_track_overlay(&e.plugin_id)) {
                continue;
            }
            let connects = |name: &str| -> Result<bool, EngineError> {
                Ok(matches!(view.value_at(layer.id, &PropertyId::new(name).map_err(store)?, t).map_err(store)?,
                    Some(Value::LayerId(id)) if id != 0) || matches!(view.value_at(layer.id, &PropertyId::new(name).map_err(store)?, t).map_err(store)?,
                    Some(Value::F64(v)) if v >= 1.0))
            };
            let has_block = layer.effects.iter().any(|e| blocks.contains(&e.plugin_id));
            // つなぐ線となぞる形は相手の motion を読む側。ただし自分がブロックを持つなら物として並ぶ(輪に札を掛ける Concentrick)。
            if !has_block && (connects(crate::doc::store::layout::CONNECT_FROM)? || connects(crate::doc::store::layout::CONNECT_TO)?) {
                continue;
            }
            let is_group = matches!(view.meta(layer.id).map_err(store)?.map(|m| m.source), Some(crate::doc::store::LayerSource::Group));
            match room_of(view, layer.id)? {
                Some(room) if has_block || !is_group || !room_ancestor => {
                    room_ids.insert(layer.id, room);
                    wanted.push(layer.id);
                }
                None if has_block || needed.contains(&layer.id) || named.contains(&layer.id) => wanted.push(layer.id),
                _ => {}
            }
        }
        let state = &mut self.blocks;
        for layer in resolved.iter().filter(|l| wanted.contains(&l.id)) {
            let frame = ([0.0, 0.0, comp.width as f32, comp.height as f32], 0.0);
            // 住む箱: 場の部屋(先祖)か、無ければ直の親。
            let parent = match room_ids.get(&layer.id) {
                Some(0) => None,
                Some(room) => Some(LayerId(*room as u64)),
                None => view.attrs(layer.id).map_err(store)?.unwrap_or_default().parent,
            };
            let room = match parent {
                Some(parent) => match (resolved.iter().find(|l| l.id == parent && l.copy == 0 && !l.ghost), crate::doc::store::layout::boxes::layer_box(view, parent, t).map_err(store)?) {
                    (Some(owner), Some(b)) => {
                        let m = owner.placement.transform;
                        let corners = [[b[0], b[1]], [b[2], b[1]], [b[0], b[3]], [b[2], b[3]]].map(|c| m.transform_point2(glam::Vec2::from(c)));
                        let lo = corners.iter().fold(glam::Vec2::MAX, |a, p| a.min(*p));
                        let hi = corners.iter().fold(glam::Vec2::MIN, |a, p| a.max(*p));
                        let radius = match view.value_at(parent, &PropertyId::new(crate::doc::store::layout::BORDER_RADIUS).map_err(store)?, t).map_err(store)? {
                            Some(Value::F64(r)) => r.max(0.0) as f32 * m.matrix2.x_axis.length(),
                            _ => 0.0,
                        };
                        ([lo.x, lo.y, hi.x, hi.y], radius)
                    }
                    _ => frame,
                },
                None => frame,
            };
            // 絵・動画の寸法は、書類の解析の口に入る前は描く側だけが知っている。物理は待たずに読む。
            // 箱の無い相手(null の層)は置き方の点 1 つ。
            let Some(own) = crate::doc::store::layout::boxes::layer_box(view, layer.id, t).map_err(store)?.or_else(|| extents.get(&layer.id).copied()).or_else(|| named.contains(&layer.id).then_some([0.0; 4])) else { continue };
            // 並べた結果、形が伸びていればその分(Fill の升目は輪郭を伸ばして解く)。伸びを見ないと、
            // 当たりが元の形の大きさのままになる。
            let stretch = solved.slots.get(&layer.id).map_or([1.0, 1.0], |slot| slot.stretch);
            let outline = match view.meta(layer.id).map_err(store)?.map(|m| m.source) {
                Some(crate::doc::store::LayerSource::Shape) => {
                    let shapes = view.shapes_at(layer.id, t).map_err(store)?;
                    outline_of(&shapes, stretch).map(std::sync::Arc::new)
                }
                // 文字はベクター: 字形の輪郭を形の層と同じ道で(絵の透過は読まない)。
                Some(crate::doc::store::LayerSource::Text) => crate::doc::store::layout::text::text_outline(view, layer.id, t).map_err(store)?.and_then(|contours| {
                    let mut points = Vec::new();
                    flatten_contours(&contours, [0.0, 0.0], &mut points);
                    (points.len() >= 3).then(|| std::sync::Arc::new(points))
                }),
                // それ以外(絵・動画)は描かれた後の透過が形。抜き方を物理は知らない。
                _ => keyed.get(&layer.id).cloned(),
            };
            let own = match solved.slots.get(&layer.id).map(|slot| slot.stretch) {
                Some([sx, sy]) if sx > 0.0 && sy > 0.0 => [own[0] * sx, own[1] * sy, own[2] * sx, own[3] * sy],
                _ => own,
            };
            let weight = match view.value_at(layer.id, &PropertyId::new(crate::doc::store::layout::FLEX_SHRINK).map_err(store)?, t).map_err(store)? {
                Some(Value::F64(v)) => v.max(0.0) as f32,
                _ => 1.0,
            };
            let margin = match view.value_at(layer.id, &PropertyId::new(crate::doc::store::layout::MARGIN).map_err(store)?, t).map_err(store)? {
                Some(Value::F64(v)) => v.max(0.0) as f32,
                _ => 0.0,
            };
            // 手触りの軸(提案 2026-09-16): 返す ↔ 吸う ↔ 引きずる。解き手の摩擦・反発へ訳す。
            let hardness = match view.value_at(layer.id, &PropertyId::new(crate::doc::store::layout::HARDNESS).map_err(store)?, t).map_err(store)? {
                Some(Value::F64(v)) => v.clamp(0.0, 1.0) as f32,
                _ => 0.5,
            };
            let weight = match view.value_at(layer.id, &PropertyId::new(crate::doc::store::layout::HEAVINESS).map_err(store)?, t).map_err(store)? {
                Some(Value::F64(v)) => v.max(0.0) as f32,
                _ => weight,
            };
            state.placed.insert(layer.id, Placed { room: room.0, radius: room.1, own, group: parent.map_or(0, |p| p.0 as u32), weight, margin, hardness, outline });
        }
        // 付いて置く札の相手がブロックで動くなら、札も物として並べて付いて行かせる(CSS の transform を読まない anchor() とは違う、利用者 2026-09-15「付いていく方が自然」)。
        let area_row = PropertyId::new(crate::doc::store::layout::POSITION_AREA).map_err(store)?;
        for layer in resolved.iter().filter(|l| l.copy == 0 && !l.ghost) {
            if !matches!(view.value_at(layer.id, &area_row, t).map_err(store)?, Some(Value::Enum(a)) if a > 0) {
                continue;
            }
            let Some(target) = anchor_of(view, layer.id)? else { continue };
            if state.placed.contains_key(&target) {
                state.follows.insert(layer.id, target);
                if !state.placed.contains_key(&layer.id) {
                    let own = crate::doc::store::layout::boxes::layer_box(view, layer.id, t).map_err(store)?.unwrap_or([0.0; 4]);
                    state.placed.insert(layer.id, Placed { room: [0.0; 4], radius: 0.0, own, group: u32::MAX, weight: 0.0, margin: 0.0, hardness: 0.5, outline: None });
                }
            }
        }
        // 物を先に決める: 層を組む前に箱・輪郭・場を揃えて解く。こうすると、つなぐ線も札も可視も
        // 同じコマの結果を読める(利用者 2026-09-16 の穴「つなぐ線が付いて来ない」)。
        let mut pending: Vec<(u32, Vec<(String, Vec<f32>, bool)>)> = Vec::new();
        let (mut order, mut late): (Vec<&ResolvedLayer>, Vec<&ResolvedLayer>) = (Vec::new(), Vec::new());
        for layer in resolved.iter().filter(|l| l.copy == 0 && !l.ghost) {
            let Some(placed) = state.placed.get(&layer.id) else { continue };
            let has_block = layer.effects.iter().any(|e| blocks.contains(&e.plugin_id));
            if has_block || state.follows.contains_key(&layer.id) || state.field_rooms.contains(&placed.group) || needed.contains(&layer.id) {
                order.push(layer);
            } else if named.contains(&layer.id) {
                late.push(layer);
            }
        }
        order.extend(late);
        for layer in order {
            let Some(placed) = state.placed.get(&layer.id).cloned() else { continue };
            let blocks_here: Vec<(String, Vec<f32>, bool)> = layer.effects.iter().filter_map(|e| {
                let d = definitions.iter().find(|d| d.0 == e.plugin_id)?;
                let params = d.2.iter().map(|(name, default)| {
                    e.params.iter().find(|(n, _)| n == name).and_then(|(_, v)| match v {
                        Value::F64(v) => Some(*v as f32),
                        Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
                        _ => None,
                    }).unwrap_or(*default)
                }).collect();
                Some((e.plugin_id.clone(), params, d.1))
            }).collect();
            let m = layer.placement.transform;
            let own = placed.own;
            let corners = [[own[0], own[1]], [own[2], own[1]], [own[0], own[3]], [own[2], own[3]]].map(|c| m.transform_point2(glam::Vec2::from(c)));
            let lo = corners.iter().fold(glam::Vec2::MAX, |a, p| a.min(*p));
            let hi = corners.iter().fold(glam::Vec2::MIN, |a, p| a.max(*p));
            let k = state.objects.len() as u32;
            state.slots.insert(layer.id, k);
            state.objects.push(BlockItem {
                lo: lo.to_array(),
                hi: hi.to_array(),
                room_lo: [placed.room[0], placed.room[1]],
                room_size: [placed.room[2] - placed.room[0], placed.room[3] - placed.room[1]],
                radius: placed.radius,
                group: placed.group,
                margin: placed.margin,
                weight: placed.weight,
                ..Default::default()
            });
            state.outlines.push(placed.outline.as_ref().map(|points| {
                std::sync::Arc::new(points.iter().map(|p| m.transform_point2(glam::Vec2::from(*p)).to_array()).collect::<Vec<[f32; 2]>>())
            }));
            state.bases.push(([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 0.0]));
            state.object_layers.push(layer.id);
            state.object_frames.push((view.layer_time(layer.id, t).map_err(store)?.as_seconds_f64() * state.fps).round() as i64);
            pending.push((k, blocks_here));
        }
        // 名指しの相手の番号は物が揃ってから引く。深さ = 親・anchor を辿った段数: 同じ stage では浅い方(読まれる側)が
        // 先に走るので、子の block は親の block の結果を同じコマで読める(1 段ずつ、輪は 0)。
        for k in 0..state.objects.len() {
            let layer = state.object_layers[k];
            let slot = |id: Option<LayerId>| id.and_then(|id| state.slots.get(&id).copied()).unwrap_or(crate::render::compositor::effects::block_program::NO_OBJECT);
            state.objects[k].parent_slot = slot(view.attrs(layer).map_err(store)?.unwrap_or_default().parent);
            state.objects[k].anchor_slot = slot(anchor_of(view, layer)?);
        }
        let depth = reference_depth(&state.objects);
        for (k, blocks_here) in pending {
            let rank = depth[k as usize];
            for (stage, (plugin, params, is_field)) in blocks_here.into_iter().enumerate() {
                if is_field {
                    state.fields.push((stage, plugin, params, k));
                    continue;
                }
                match state.batches.iter_mut().find(|b| b.stage == stage && b.rank == rank && b.plugin == plugin && b.params == params && b.source == u32::MAX) {
                    Some(batch) => batch.members.push(k),
                    None => state.batches.push(BlockBatch { stage, plugin, params, members: vec![k], source: u32::MAX, rank }),
                }
            }
        }
        state.batches.sort_by_key(|b| (b.stage, b.rank));
        // つなぐ線となぞる形は物にならないが、相手の motion を描く側で読む(利用者 2026-09-18「位置は毎コマ変わるのに
        // GPU じゃないの変すぎ」)。CPU の道は動く前の箱から引き、動いた分は頂点で足す。
        for layer in resolved.iter().filter(|l| l.copy == 0 && !l.ghost) {
            if let Some((from, to)) = view.connection(layer.id, t).map_err(store)? {
                if let (Some(&a), Some(&b)) = (state.slots.get(&from), state.slots.get(&to)) {
                    let path = view.value_at(layer.id, &PropertyId::new(crate::doc::store::layout::LINE_PATH).map_err(store)?, t).map_err(store)?;
                    if matches!(path, Some(Value::Enum(4))) {
                        let slack = match view.value_at(layer.id, &PropertyId::new(crate::doc::store::layout::SLACK).map_err(store)?, t).map_err(store)? { Some(Value::F64(v)) => v as f32, _ => 20.0 };
                        // 硬さと減衰は今は定数(紐らしさの 1 点: 遅れて、揺れて、止まる)。欄にするかは絵を見てから。
                        state.ropes.push((state.connectors.len() as u32, slack.max(0.0), 60.0, 6.0));
                    }
                    state.connectors.push((layer.id, a, b));
                    continue;
                }
            }
            if let Some((target, _)) = view.tracing(layer.id, t).map_err(store)? {
                if let Some(&k) = state.slots.get(&target) {
                    state.traces.insert(layer.id, k);
                }
            }
        }
        self.solve_physics(t);
        Ok(())
    }

    /// 祖先の箱の切り(`MaskFrame::Box`)を、ブロックのずれの後に箱の枠で掛ける層か: 解き手が動かす、平らに置かれた、
    /// 本番の組み(辿り直しの中でない)。
    pub(super) fn cut_after_motion(&self, layer: &ResolvedLayer) -> bool {
        !self.feedback_replaying
            && self.blocks.moves(layer.id)
            && layer.placement.z == 0.0 && layer.placement.rotation_x == 0.0 && layer.placement.rotation_y == 0.0
            && layer.masks.iter().any(|m| m.frame == MaskFrame::Box)
    }

    /// 箱の切りを comp の px へ(素材座標 → 置き方)。ずれの後の絵は comp 大なので、そこで掛ける。
    pub(super) fn box_cut_in_comp(&self, layer: &ResolvedLayer, built: &Layer) -> Vec<ResolvedMask> {
        if !self.cut_after_motion(layer) {
            return Vec::new();
        }
        let origin = built.frame.as_ref().map_or(glam::Vec2::ZERO, |f| glam::Vec2::from(f.origin));
        let to = built.placement.transform * glam::Affine2::from_translation(-origin);
        let scale = (to.matrix2.x_axis.length() + to.matrix2.y_axis.length()) * 0.5;
        layer.masks.iter().filter(|m| m.frame == MaskFrame::Box).map(|m| {
            let mut mask = m.clone();
            for vertex in &mut mask.shape.vertices {
                let p = to.transform_point2(glam::vec2(vertex.point[0] as f32, vertex.point[1] as f32));
                vertex.point = [f64::from(p.x), f64::from(p.y)];
                for tangent in [&mut vertex.in_tangent, &mut vertex.out_tangent] {
                    let d = to.transform_vector2(glam::vec2(tangent[0] as f32, tangent[1] as f32));
                    *tangent = [f64::from(d.x), f64::from(d.y)];
                }
            }
            mask.expansion *= f64::from(scale);
            mask
        }).collect()
    }

    /// 組んだ 1 枚にブロックが掛かっていれば、描く時と同じ置き方の箱を物として並べ、motion の番号を最後の欄に入れる。
    pub(super) fn attach_block(&mut self, layer: &ResolvedLayer, built: &mut Layer, comp: CompSpec) {
        // 物は既に決まっている(層を組む前に揃えて解いた)。ここでするのは、描く側へ渡す番号と、
        // その物の comp → world の向き(組んだ素材の大きさが要るのでここでしか作れない)。
        let Some(&k) = self.blocks.slots.get(&layer.id) else {
            // 物でない層: つなぐ線は物の後ろの自分の番号、なぞる形は相手の番号を読む。
            if let Some(c) = self.blocks.connectors.iter().position(|(id, _, _)| *id == layer.id) {
                built.shading.params[crate::render::compositor::effects::surface_program::PARAM_SLOTS - 1] = (self.blocks.objects.len() + 2 * c + 1) as f32;
                if self.blocks.ropes.iter().any(|(index, ..)| *index as usize == c) {
                    // 紐は曲線に沿って頂点が動くので板を刻む。
                    built.shading.grid_hint = 48;
                }
            } else if let Some(&target) = self.blocks.traces.get(&layer.id) {
                built.shading.params[crate::render::compositor::effects::surface_program::PARAM_SLOTS - 1] = (target + 1) as f32;
            }
            return;
        };
        let size = glam::Vec2::from(built.size);
        if size.x <= 0.0 || size.y <= 0.0 {
            return;
        }
        built.shading.params[crate::render::compositor::effects::surface_program::PARAM_SLOTS - 1] = (k + 1) as f32;
        let m = built.placement.transform;
        let (origin, u, v) = crate::render::compositor::projected_placement_corners(comp, built.projection_camera, built.projection, built.placement, glam::Vec2::ZERO, size);
        let (per_x, per_y) = (u / size.x, v / size.y);
        let inverse = if m.matrix2.determinant().abs() > 1e-12 { m.matrix2.inverse() } else { glam::Mat2::IDENTITY };
        let world = |comp_step: glam::Vec2| { let material = inverse * comp_step; (per_x * material.x + per_y * material.y).to_array() };
        let own = self.blocks.placed.get(&layer.id).map(|p| p.own).unwrap_or([0.0; 4]);
        let middle = glam::Vec2::new((own[0] + own[2]) * 0.5, (own[1] + own[3]) * 0.5);
        let centre = origin + u * (middle.x / size.x) + v * (middle.y / size.y);
        if let Some(slot) = self.blocks.bases.get_mut(k as usize) {
            *slot = (world(glam::Vec2::X), world(glam::Vec2::Y), centre.to_array());
        }
    }

    /// 意図(場・箱)を外の解き手へ渡してこのコマまで進める。同じコマなら何度呼んでも進まない。
    /// 可視の層は、絵を組む途中でこれを呼んでから中身を読む。
    pub(crate) fn solve_physics_now(&mut self) {
        if let Some(t) = self.blocks.now {
            self.solve_physics(t);
        }
    }

    pub(crate) fn solve_physics(&mut self, t: RationalTime) {
        let fps = self.blocks.fps.max(1.0);
        let state = &mut self.blocks;
        if state.objects.is_empty() {
            return;
        }
        // 場と箱は外の解き手(Rapier)に渡す。ここが持つのは意図からの訳だけで、解き方は持たない
        // (利用者 2026-09-16「物理演算を 1 から作るんじゃなくてこれも外部に揃ったやつがあるだろ」)。
        use crate::render::engine::physics::{Body, Room, Well};
        let mut rooms: HashMap<u32, Room> = HashMap::new();
        let mut gathering = false;
        for (stage, plugin, params, source) in std::mem::take(&mut state.fields) {
            let _ = (stage, &plugin);
            let item = state.objects[source as usize];
            let (turn, spread, angle, strength, reach, hold, gather) = (
                params.first().copied().unwrap_or(0.0).to_radians(),
                params.get(1).copied().unwrap_or(0.0).clamp(0.0, 1.0),
                params.get(2).copied().unwrap_or(90.0).to_radians(),
                params.get(3).copied().unwrap_or(0.0),
                params.get(4).copied().unwrap_or(0.0),
                params.get(5).copied().unwrap_or(0.0).clamp(0.0, 1.0),
                params.get(6).copied().unwrap_or(0.0) >= 0.5 && state.physics.lies.gather,
            );
            gathering |= gather;
            let room = rooms.entry(item.group).or_insert_with(|| Room {
                group: item.group,
                rect: [item.room_lo[0], item.room_lo[1], item.room_lo[0] + item.room_size[0], item.room_lo[1] + item.room_size[1]],
                round: item.radius * 2.0 >= item.room_size[0].min(item.room_size[1]) - 1e-3,
                gravity: [0.0, 0.0],
                wells: Vec::new(),
            });
            room.gravity[0] += angle.cos() * strength * spread;
            room.gravity[1] += angle.sin() * strength * spread;
            if spread < 1.0 {
                room.wells.push(Well {
                    at: [(item.lo[0] + item.hi[0]) * 0.5, (item.lo[1] + item.hi[1]) * 0.5],
                    pull: turn.cos() * strength * (1.0 - spread),
                    swirl: turn.sin() * strength * (1.0 - spread),
                    reach,
                    hold,
                });
            }
        }
        let frame = (t.as_seconds_f64() * fps).round() as i64;
        let bodies: Vec<Body> = state.objects.iter().enumerate()
            .filter(|(_, it)| rooms.contains_key(&it.group))
            .map(|(k, it)| Body {
                layer: state.object_layers[k],
                group: it.group,
                frame: state.object_frames.get(k).copied().unwrap_or(frame),
                centre: [(it.lo[0] + it.hi[0]) * 0.5, (it.lo[1] + it.hi[1]) * 0.5],
                half: [(it.hi[0] - it.lo[0]) * 0.5, (it.hi[1] - it.lo[1]) * 0.5],
                round: false,
                margin: it.margin,
                weight: it.weight,
                hardness: state.placed.get(&state.object_layers[k]).map_or(0.5, |p| p.hardness),
                outline: state.outlines.get(k).cloned().flatten(),
            })
            .collect();
        // 箱の順は毎コマ同じに(当たりの組が箱の順で決まる)。
        let mut rooms: Vec<Room> = rooms.into_values().collect();
        rooms.sort_by_key(|r| r.group);
        // 集まる: 終わりは構図。comp の最後まで前向きに解いて焼き、最後から逆に読む。
        let last = state.last_frame;
        state.physics.solve(&rooms, &bodies, frame, fps, gathering.then_some(last));
    }

    /// 集めた物を GPU で効果の順に解き、world のずれにして描く側へ渡す。ブロックが無ければ外す。
    pub(super) fn run_blocks(&mut self, t: RationalTime, fps: f64) {
        let state = &mut self.blocks;
        if state.objects.is_empty() {
            self.compositor.motion = None;
            return;
        }
        self.solve_physics(t);
        let state = &mut self.blocks;
        let seed: Vec<crate::render::compositor::effects::block_program::BlockOffset> = state.object_layers.iter()
            .map(|layer| match state.physics.offset(*layer) {
                Some((translate, rotate)) => crate::render::compositor::effects::block_program::BlockOffset { translate, rotate, ..Default::default() },
                None => crate::render::compositor::effects::block_program::BlockOffset::default(),
            })
            .collect();
        let ctx = &self.compositor.ctx;
        // 近くの物の升目は、ブロックが宣言した届く距離(`REACH` の欄)の一番大きい値だけ広げる。
        let reach = state.batches.iter().filter_map(|batch| {
            let d = self.compositor.catalog.definitions.iter().find(|d| d.plugin_id() == batch.plugin)?;
            let name = d.manifest.reach.as_ref()?;
            d.manifest.param_inputs().position(|p| &p.name == name).and_then(|i| batch.params.get(i).copied())
        }).fold(0.0f32, f32::max);
        let world = state.world.get_or_insert_with(|| BlockWorld::new(&ctx.device));
        world.begin_from(&ctx.device, &ctx.queue, &state.objects, reach, &seed);
        let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("motolii-blocks") });
        let stages = state.batches.iter().map(|b| b.stage + 1).max().unwrap_or(0);
        for stage in 0..stages {
            for batch in state.batches.iter().filter(|b| b.stage == stage) {
                let Some(definition) = self.compositor.catalog.definitions.iter().find(|d| d.plugin_id() == batch.plugin) else { continue };
                let (source, program) = state.programs.entry(batch.plugin.clone())
                    .or_insert_with(|| (definition.vertex_text.clone(), BlockProgram::new(&ctx.device, &batch.plugin, &definition.vertex_text, definition.manifest.rounds)));
                if *source != definition.vertex_text {
                    *source = definition.vertex_text.clone();
                    *program = BlockProgram::new(&ctx.device, &batch.plugin, source, definition.manifest.rounds);
                }
                program.record_from(&ctx.device, &ctx.queue, &mut encoder, world, t.as_seconds_f64() as f32, &batch.members, &batch.params, batch.source);
            }
        }
        let index_of = |id: LayerId| state.object_layers.iter().position(|l| *l == id).map(|k| k as u32);
        let pairs: Vec<(u32, u32)> = state.object_layers.iter().enumerate()
            .filter_map(|(k, id)| state.follows.get(id).and_then(|target| index_of(*target)).map(|target| (k as u32, target)))
            .collect();
        state.follow_pass.get_or_insert_with(|| FollowPass::new(&ctx.device)).record(&ctx.device, &ctx.queue, &mut encoder, world, &pairs);
        let links: Vec<(u32, u32)> = state.connectors.iter().map(|(_, a, b)| (*a, *b)).collect();
        let motion = re_renderer::MotionBuffer::new(ctx, (state.objects.len() + 2 * links.len()) as u64);
        let bases_buffer = state.world_pass.get_or_insert_with(|| WorldPass::new(&ctx.device)).record(&ctx.device, &ctx.queue, &mut encoder, world, &state.bases, &links, motion.buffer());
        if let Some(bases_buffer) = bases_buffer.as_ref() {
            let frame = (t.as_seconds_f64() * state.fps).round() as i64;
            state.rope_pass.get_or_insert_with(|| crate::render::compositor::effects::block_program::RopePass::new(&ctx.device))
                .record(&ctx.device, &ctx.queue, &mut encoder, world, bases_buffer, &links, &state.ropes, motion.buffer(), frame, state.fps as f32);
        }
        ctx.queue.submit([encoder.finish()]);
        self.compositor.motion = Some(motion);
    }
}

#[cfg(test)]
mod tests {
    use crate::doc::eval::Keyframe;
    use crate::doc::store::{layout, property, Composition, Document, EffectId, EffectInstance, Fps, Intent, Interp, KeyframeTrack, LayerAttrsPatch, LayerId, LayerMeta, LayerProjection, LayerSource, LayerTiming, PathSource, PropertyId, RationalTime, Shape, ShapeNode, Value};
    use crate::doc::vector::{Brush, Fill, Point, Rgb};
    use crate::render::engine::Engine;

    const W: u32 = 160;
    const H: u32 = 100;

    /// 灰色でない住む箱(Display の Group、角丸 0)の中を、白い四角が右下へまっすぐ漂う。`gpu` なら子に Bounce のブロック、
    /// そうでなければ親の Overflow = Bounce(書類の CPU の法)。
    fn scene(gpu: bool) -> Document {
        let mut doc = Document::new().with_programs(crate::extensions::bundled());
        doc.apply(Intent::SetComposition(Composition { width: W, height: H, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 90, background: [0.0; 4] })).unwrap();
        let (group, child) = (LayerId(1), LayerId(2));
        let two_d = LayerAttrsPatch { projection: Some(LayerProjection::TwoD), ..Default::default() };
        doc.apply_all([
            Intent::AddLayer(group),
            Intent::SetMeta { layer: group, meta: LayerMeta { source: LayerSource::Group, order: 0, timing: LayerTiming::place(0, None, 90) } },
            Intent::SetAttrs { layer: group, patch: two_d.clone() },
            Intent::AddLayer(child),
            Intent::SetMeta { layer: child, meta: LayerMeta { source: LayerSource::Shape, order: 1, timing: LayerTiming::place(0, None, 90) } },
            Intent::SetAttrs { layer: child, patch: LayerAttrsPatch { parent: Some(Some(group)), ..two_d } },
            Intent::SetShapes { layer: child, shapes: vec![ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: 16.0, y: 16.0 } }, ops: Vec::new(), stroke: None, fill: Some(Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 1.0, b: 1.0 }), ..Default::default() }) })] },
        ]).unwrap();
        let put = |doc: &mut Document, layer, name: &str, value: Value| doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap();
        put(&mut doc, group, property::POSITION, Value::Vec2([10.0, 10.0]));
        put(&mut doc, group, layout::DISPLAY, Value::Enum(1));
        put(&mut doc, group, layout::HORIZONTAL_SIZING, Value::Enum(2));
        put(&mut doc, group, layout::VERTICAL_SIZING, Value::Enum(2));
        put(&mut doc, group, layout::WIDTH, Value::F64(120.0));
        put(&mut doc, group, layout::HEIGHT, Value::F64(70.0));
        put(&mut doc, child, layout::POSITION_TYPE, Value::Enum(1));
        let mut track = KeyframeTrack::new();
        track.insert(Keyframe { t: RationalTime::ZERO, value: Value::Vec2([30.0, 20.0]), interp: Interp::Linear, spatial: None });
        track.insert(Keyframe { t: RationalTime::try_new(3, 1).unwrap(), value: Value::Vec2([30.0 + 390.0, 20.0 + 240.0]), interp: Interp::Linear, spatial: None });
        doc.apply(Intent::SetTrack { layer: child, property: PropertyId::new(property::POSITION).unwrap(), track }).unwrap();
        if gpu {
            doc.apply(Intent::SetEffects { layer: child, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.bounce_block".into() }] }).unwrap();
        } else {
            put(&mut doc, group, layout::OVERFLOW, Value::Enum(2));
        }
        doc
    }

    /// 住む箱(Display の Group、(10,10) から 120×70、`clip` なら Overflow Clip)の底の白い四角(16 px、箱の (30, 50))が、
    /// `start` コマ目から Arrive で下から来る(From 90・Distance 80・Arrive 0.7・Bounce 0・Stagger 0・Spin 0)。
    fn arrive_scene(start: i64, clip: bool) -> Document {
        let mut doc = Document::new().with_programs(crate::extensions::bundled());
        doc.apply(Intent::SetComposition(Composition { width: W, height: H, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 90, background: [0.0; 4] })).unwrap();
        let (group, child) = (LayerId(1), LayerId(2));
        let two_d = LayerAttrsPatch { projection: Some(LayerProjection::TwoD), ..Default::default() };
        doc.apply_all([
            Intent::AddLayer(group),
            Intent::SetMeta { layer: group, meta: LayerMeta { source: LayerSource::Group, order: 0, timing: LayerTiming::place(0, None, 90) } },
            Intent::SetAttrs { layer: group, patch: two_d.clone() },
            Intent::AddLayer(child),
            Intent::SetMeta { layer: child, meta: LayerMeta { source: LayerSource::Shape, order: 1, timing: LayerTiming::place(start, None, 90) } },
            Intent::SetAttrs { layer: child, patch: LayerAttrsPatch { parent: Some(Some(group)), ..two_d } },
            Intent::SetShapes { layer: child, shapes: vec![ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: 16.0, y: 16.0 } }, ops: Vec::new(), stroke: None, fill: Some(Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 1.0, b: 1.0 }), ..Default::default() }) })] },
            Intent::SetEffects { layer: child, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.arrive".into() }] },
        ]).unwrap();
        let put = |doc: &mut Document, layer, name: &str, value: Value| doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap();
        put(&mut doc, group, property::POSITION, Value::Vec2([10.0, 10.0]));
        put(&mut doc, group, layout::DISPLAY, Value::Enum(1));
        put(&mut doc, group, layout::HORIZONTAL_SIZING, Value::Enum(2));
        put(&mut doc, group, layout::VERTICAL_SIZING, Value::Enum(2));
        put(&mut doc, group, layout::WIDTH, Value::F64(120.0));
        put(&mut doc, group, layout::HEIGHT, Value::F64(70.0));
        if clip {
            put(&mut doc, group, layout::OVERFLOW, Value::Enum(1));
        }
        put(&mut doc, child, layout::POSITION_TYPE, Value::Enum(1));
        put(&mut doc, child, property::POSITION, Value::Vec2([30.0, 50.0]));
        for (name, value) in [("side", 90.0), ("distance", 80.0), ("arrive", 0.7), ("bounce", 0.0), ("stagger", 0.0), ("spin", 0.0)] {
            put(&mut doc, child, &format!("{}0.param.{name}", property::EFFECT_PREFIX), Value::F64(value));
        }
        doc
    }

    /// 描かれた行(alpha > 128 の画素の y、重複なし・昇順)。
    fn lit_rows(pixels: &[u8]) -> Vec<u32> {
        let mut rows: Vec<u32> = pixels.chunks_exact(4).enumerate().filter(|(_, c)| c[3] > 128).map(|(i, _)| (i as u32) / W).collect();
        rows.sort_unstable();
        rows.dedup();
        rows
    }

    /// 箱の切りは箱の枠で(CSS の overflow: clip は要素の箱に掛かり、中で transform した子は箱で切れる): Overflow Clip の箱の
    /// 子が Arrive で下から来る途中、箱の外(下)の画素は透明。着いた後は箱の中に丸ごと見える。
    #[test]
    fn the_boxs_clip_cuts_what_arrive_moves_in_the_boxs_frame() {
        let doc = arrive_scene(0, true);
        let mut engine = Engine::new().unwrap();
        let fps = Fps::try_new(30, 1).unwrap();
        let at = |engine: &mut Engine, frame: i64| lit_rows(&engine.render_frame(&doc.view(), RationalTime::try_from_frame(frame, fps).unwrap()).unwrap());
        let landed = at(&mut engine, 30);
        let bottom = 10 + 70 + 1;
        assert!(landed.len() >= 15 && *landed.last().unwrap() < bottom, "landed: the whole square sits inside the box: {landed:?}");
        // frame 9 = 0.3 s of 0.7: left = (1 − 3/7)^3 ≈ 0.19 → 15 px below its slot → the square straddles the box's bottom.
        let mid = at(&mut engine, 9);
        assert!(!mid.is_empty() && mid[0] > landed[0], "mid-arrive: the square is on its way up (rows {mid:?} vs landed {landed:?})");
        assert!(*mid.last().unwrap() < bottom && mid.len() < landed.len(), "mid-arrive: the box cuts it at its bottom edge, the offset moved only the content: rows {mid:?}");
    }

    /// つなぐ線は、ブロックが動かした相手に付いて行く(線の端が GPU で相手の motion を読む、利用者 2026-09-18
    /// 「位置は毎コマ変わるのに GPU じゃないの変すぎ」): 静かな白い四角と、Arrive で下から来る白い四角を赤い線で結ぶ。
    /// 来る途中(frame 9)の線の下端は、着いた後(frame 30)より下にある。
    #[test]
    fn a_connector_follows_what_a_block_moved() {
        let mut doc = arrive_scene(0, false);
        let (still, line) = (LayerId(3), LayerId(4));
        let two_d = LayerAttrsPatch { projection: Some(LayerProjection::TwoD), ..Default::default() };
        let white = Some(Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 1.0, b: 1.0 }), ..Default::default() });
        doc.apply_all([
            Intent::AddLayer(still),
            Intent::SetMeta { layer: still, meta: LayerMeta { source: LayerSource::Shape, order: 2, timing: LayerTiming::place(0, None, 90) } },
            Intent::SetAttrs { layer: still, patch: LayerAttrsPatch { parent: Some(Some(LayerId(1))), ..two_d.clone() } },
            Intent::SetShapes { layer: still, shapes: vec![ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: 16.0, y: 16.0 } }, ops: Vec::new(), stroke: None, fill: white })] },
            Intent::AddLayer(line),
            Intent::SetMeta { layer: line, meta: LayerMeta { source: LayerSource::Shape, order: 3, timing: LayerTiming::place(0, None, 90) } },
            Intent::SetAttrs { layer: line, patch: LayerAttrsPatch { parent: Some(Some(LayerId(1))), ..two_d } },
            Intent::SetShapes { layer: line, shapes: vec![ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: 4.0, y: 4.0 } }, ops: Vec::new(), fill: None,
                stroke: Some(crate::doc::vector::Stroke { brush: Brush::Solid(Rgb { r: 1.0, g: 0.0, b: 0.0 }), width: 3.0, ..Default::default() }) })] },
        ]).unwrap();
        let put = |doc: &mut Document, layer, name: &str, value: Value| doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap();
        put(&mut doc, still, layout::POSITION_TYPE, Value::Enum(1));
        put(&mut doc, still, property::POSITION, Value::Vec2([90.0, 8.0]));
        put(&mut doc, line, layout::CONNECT_FROM, Value::LayerId(still.0));
        put(&mut doc, line, layout::CONNECT_TO, Value::LayerId(LayerId(2).0));
        let mut engine = Engine::new().unwrap();
        let fps = Fps::try_new(30, 1).unwrap();
        let red_bottom = |engine: &mut Engine, frame: i64| -> Option<u32> {
            let pixels = engine.render_frame(&doc.view(), RationalTime::try_from_frame(frame, fps).unwrap()).unwrap();
            pixels.chunks_exact(4).enumerate().filter(|(_, c)| c[0] > 150 && c[1] < 90 && c[2] < 90 && c[3] > 128).map(|(i, _)| (i as u32) / W).max()
        };
        let landed = red_bottom(&mut engine, 30).expect("the line is drawn once the square has landed");
        let mid = red_bottom(&mut engine, 9).expect("the line is drawn while the square is on its way");
        assert!(mid > landed + 6, "mid-arrive the line's lower end is with the square below its slot: mid {mid} vs landed {landed}");
    }

    /// GPU のブロックが描く位置は、書類の CPU の Bounce と同じ(読み戻さずに描く側の頂点で動く)。
    #[test]
    fn the_bounce_block_draws_where_the_cpu_bounce_does() {
        let (cpu, gpu) = (scene(false), scene(true));
        let mut engine = Engine::new().unwrap();
        for frame in [0, 20, 45, 70] {
            let t = RationalTime::try_from_frame(frame, Fps::try_new(30, 1).unwrap()).unwrap();
            let expected = engine.render_frame(&cpu.view(), t).unwrap();
            let actual = engine.render_frame(&gpu.view(), t).unwrap();
            let lit = |p: &[u8]| p.chunks_exact(4).enumerate().filter(|(_, c)| c[3] > 128).map(|(i, _)| ((i as u32) % W, (i as u32) / W)).collect::<Vec<_>>();
            let (e, a) = (lit(&expected), lit(&actual));
            let centre = |v: &[(u32, u32)]| { let n = v.len().max(1) as f32; (v.iter().map(|p| p.0 as f32).sum::<f32>() / n, v.iter().map(|p| p.1 as f32).sum::<f32>() / n) };
            assert!(!e.is_empty() && (e.len() as i64 - a.len() as i64).abs() < 40, "frame {frame}: the square is drawn once in both ({} vs {} px)", e.len(), a.len());
            let (ce, ca) = (centre(&e), centre(&a));
            assert!((ce.0 - ca.0).abs() < 1.0 && (ce.1 - ca.1).abs() < 1.0, "frame {frame}: cpu centre {ce:?}, gpu centre {ca:?}");
        }
    }

    /// 場(`SCOPE: room`)は、掛かった層でなく**同じ住む箱に居る他の全員**を動かす。相手は効果を 1 枚も持たない。
    #[test]
    fn a_field_moves_everyone_in_its_room_who_carries_no_effect() {
        let fps = Fps::try_new(30, 1).unwrap();
        let mut doc = Document::new().with_programs(crate::extensions::bundled());
        doc.apply(Intent::SetComposition(Composition { width: W, height: H, fps, duration_frames: 90, background: [0.0; 4] })).unwrap();
        let (group, ball, field) = (LayerId(1), LayerId(2), LayerId(3));
        let two_d = LayerAttrsPatch { projection: Some(LayerProjection::TwoD), ..Default::default() };
        let square = |size: f32, fill: Rgb| ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: size as f64, y: size as f64 } }, ops: Vec::new(), stroke: None, fill: Some(Fill { brush: Brush::Solid(fill), ..Default::default() }) });
        doc.apply_all([
            Intent::AddLayer(group),
            Intent::SetMeta { layer: group, meta: LayerMeta { source: LayerSource::Group, order: 0, timing: LayerTiming::place(0, None, 90) } },
            Intent::SetAttrs { layer: group, patch: two_d.clone() },
            Intent::AddLayer(ball),
            Intent::SetMeta { layer: ball, meta: LayerMeta { source: LayerSource::Shape, order: 1, timing: LayerTiming::place(0, None, 90) } },
            Intent::SetAttrs { layer: ball, patch: LayerAttrsPatch { parent: Some(Some(group)), ..two_d.clone() } },
            Intent::SetShapes { layer: ball, shapes: vec![square(16.0, Rgb { r: 1.0, g: 1.0, b: 1.0 })] },
            Intent::AddLayer(field),
            Intent::SetMeta { layer: field, meta: LayerMeta { source: LayerSource::Shape, order: 2, timing: LayerTiming::place(0, None, 90) } },
            Intent::SetAttrs { layer: field, patch: LayerAttrsPatch { parent: Some(Some(group)), ..two_d } },
            Intent::SetShapes { layer: field, shapes: vec![square(4.0, Rgb { r: 1.0, g: 0.0, b: 0.0 })] },
        ]).unwrap();
        let put = |doc: &mut Document, layer, name: &str, value: Value| doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap();
        put(&mut doc, group, property::POSITION, Value::Vec2([10.0, 10.0]));
        put(&mut doc, group, layout::DISPLAY, Value::Enum(1));
        put(&mut doc, group, layout::HORIZONTAL_SIZING, Value::Enum(2));
        put(&mut doc, group, layout::VERTICAL_SIZING, Value::Enum(2));
        put(&mut doc, group, layout::WIDTH, Value::F64(120.0));
        put(&mut doc, group, layout::HEIGHT, Value::F64(70.0));
        for layer in [ball, field] {
            put(&mut doc, layer, layout::POSITION_TYPE, Value::Enum(1));
        }
        put(&mut doc, ball, property::POSITION, Value::Vec2([30.0, 15.0]));
        put(&mut doc, field, property::POSITION, Value::Vec2([100.0, 15.0]));
        // 一様(Spread 1)の場を下(+90°)へ。落ちる速さは外の解き手(Rapier)が決めるので、
        // ここで見るのは「効果を持たない隣人が場だけで下へ動き、箱の中で止まる」という法の意味。
        doc.apply(Intent::SetEffects { layer: field, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.field".into() }] }).unwrap();
        for (name, value) in [("spread", 1.0), ("turn", 0.0), ("angle", 90.0), ("strength", 40.0), ("reach", 0.0)] {
            doc.apply(Intent::SetConstant { layer: field, property: PropertyId::effect_param(EffectId(0), name).unwrap(), value: Value::F64(value) }).unwrap();
        }
        let mut engine = Engine::new().unwrap();
        // 白い四角(場の元は赤)の真ん中。
        let centre = |pixels: &[u8]| {
            let hits: Vec<(f32, f32)> = pixels.chunks_exact(4).enumerate()
                .filter(|(_, c)| c[3] > 128 && c[2] > 128)
                .map(|(i, _)| ((i as u32 % W) as f32, (i as u32 / W) as f32)).collect();
            let n = hits.len().max(1) as f32;
            (hits.len(), hits.iter().map(|p| p.0).sum::<f32>() / n, hits.iter().map(|p| p.1).sum::<f32>() / n)
        };
        let (n0, x0, y0) = centre(&engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap());
        let (n1, x1, y1) = centre(&engine.render_frame(&doc.view(), RationalTime::try_from_frame(30, fps).unwrap()).unwrap());
        let (n2, _, y2) = centre(&engine.render_frame(&doc.view(), RationalTime::try_from_frame(60, fps).unwrap()).unwrap());
        assert!(n0 > 100 && n1 > 100 && n2 > 100, "白い四角がどのコマにも在る ({n0} / {n1} / {n2} px)");
        assert!((x1 - x0).abs() < 2.0, "横には動かない ({x0} → {x1})");
        assert!(y1 > y0 + 5.0 && y2 >= y1 - 1.0, "場だけで下へ動く ({y0} → {y1} → {y2})");
        // 箱の床(親の Group の下辺)より下へは行かない。
        assert!(y2 < 88.0, "箱の中で止まる (y {y2})");
    }

    /// 場の動きはコマからコマへ飛ばない(利用者 2026-09-16「動きは離散的にならないように」)。
    /// 同じ画を 30 コマ描いて、真ん中の動いた量の差(加速度)が、動いた量そのものより小さいことを見る。
    #[test]
    fn a_field_moves_without_jumping_between_frames() {
        let fps = Fps::try_new(30, 1).unwrap();
        let mut doc = Document::new().with_programs(crate::extensions::bundled());
        doc.apply(Intent::SetComposition(Composition { width: W, height: H, fps, duration_frames: 40, background: [0.0; 4] })).unwrap();
        let (group, ball, field) = (LayerId(1), LayerId(2), LayerId(3));
        let two_d = LayerAttrsPatch { projection: Some(LayerProjection::TwoD), ..Default::default() };
        let square = |size: f64, fill: Rgb| ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: size, y: size } }, ops: Vec::new(), stroke: None, fill: Some(Fill { brush: Brush::Solid(fill), ..Default::default() }) });
        doc.apply_all([
            Intent::AddLayer(group),
            Intent::SetMeta { layer: group, meta: LayerMeta { source: LayerSource::Group, order: 0, timing: LayerTiming::place(0, None, 40) } },
            Intent::SetAttrs { layer: group, patch: two_d.clone() },
            Intent::AddLayer(ball),
            Intent::SetMeta { layer: ball, meta: LayerMeta { source: LayerSource::Shape, order: 1, timing: LayerTiming::place(0, None, 40) } },
            Intent::SetAttrs { layer: ball, patch: LayerAttrsPatch { parent: Some(Some(group)), ..two_d.clone() } },
            Intent::SetShapes { layer: ball, shapes: vec![square(14.0, Rgb { r: 1.0, g: 1.0, b: 1.0 })] },
            Intent::AddLayer(field),
            Intent::SetMeta { layer: field, meta: LayerMeta { source: LayerSource::Shape, order: 2, timing: LayerTiming::place(0, None, 40) } },
            Intent::SetAttrs { layer: field, patch: LayerAttrsPatch { parent: Some(Some(group)), ..two_d } },
            Intent::SetShapes { layer: field, shapes: vec![square(3.0, Rgb { r: 1.0, g: 0.0, b: 0.0 })] },
        ]).unwrap();
        let put = |doc: &mut Document, layer, name: &str, value: Value| doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap();
        put(&mut doc, group, property::POSITION, Value::Vec2([8.0, 8.0]));
        put(&mut doc, group, layout::DISPLAY, Value::Enum(1));
        put(&mut doc, group, layout::HORIZONTAL_SIZING, Value::Enum(2));
        put(&mut doc, group, layout::VERTICAL_SIZING, Value::Enum(2));
        put(&mut doc, group, layout::WIDTH, Value::F64(140.0));
        put(&mut doc, group, layout::HEIGHT, Value::F64(84.0));
        for layer in [ball, field] {
            put(&mut doc, layer, layout::POSITION_TYPE, Value::Enum(1));
        }
        put(&mut doc, ball, property::POSITION, Value::Vec2([20.0, 20.0]));
        put(&mut doc, field, property::POSITION, Value::Vec2([110.0, 60.0]));
        doc.apply(Intent::SetEffects { layer: field, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.field".into() }] }).unwrap();
        // 元へ寄る場(Spread 0、Turn 0)。一番動きが速い所を含む 30 コマを見る。
        for (name, value) in [("spread", 0.0), ("turn", 0.0), ("strength", 220.0), ("reach", 0.0), ("tumble", 0.0), ("hold", 0.0)] {
            doc.apply(Intent::SetConstant { layer: field, property: PropertyId::effect_param(EffectId(0), name).unwrap(), value: Value::F64(value) }).unwrap();
        }
        let mut engine = Engine::new().unwrap();
        let centre = |pixels: &[u8]| {
            let hits: Vec<(f32, f32)> = pixels.chunks_exact(4).enumerate()
                .filter(|(_, c)| c[3] > 128 && c[2] > 128)
                .map(|(i, _)| ((i as u32 % W) as f32, (i as u32 / W) as f32)).collect();
            let n = hits.len().max(1) as f32;
            (hits.iter().map(|p| p.0).sum::<f32>() / n, hits.iter().map(|p| p.1).sum::<f32>() / n)
        };
        let path: Vec<(f32, f32)> = (0..30).map(|f| centre(&engine.render_frame(&doc.view(), RationalTime::try_from_frame(f, fps).unwrap()).unwrap())).collect();
        let step: Vec<f32> = path.windows(2).map(|w| ((w[1].0 - w[0].0).powi(2) + (w[1].1 - w[0].1).powi(2)).sqrt()).collect();
        let moved: f32 = step.iter().sum();
        assert!(moved > 20.0, "場が動かしている ({moved}px)");
        let jump = step.windows(2).map(|w| (w[1] - w[0]).abs()).fold(0.0f32, f32::max);
        let fastest = step.iter().copied().fold(0.0f32, f32::max);
        assert!(jump <= fastest * 0.5, "コマ間の変わり方が跳ばない(最大の差 {jump}px、一番速いコマ {fastest}px)");
    }

    /// 落ちて積もった後は震えない(利用者 2026-09-16「まだまだガッタガタやぞ」)。
    /// 画素の平均差では物ごとの震えが見えないので、物ごとのずれをコマ順に読んで測る。
    #[test]
    fn things_stop_moving_once_they_have_settled() {
        let fps = Fps::try_new(30, 1).unwrap();
        let mut doc = Document::new().with_programs(crate::extensions::bundled());
        doc.apply(Intent::SetComposition(Composition { width: W, height: H, fps, duration_frames: 90, background: [0.0; 4] })).unwrap();
        let (group, field) = (LayerId(1), LayerId(2));
        let two_d = LayerAttrsPatch { projection: Some(LayerProjection::TwoD), ..Default::default() };
        let square = |size: f64| ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: size, y: size } }, ops: Vec::new(), stroke: None, fill: Some(Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 1.0, b: 1.0 }), ..Default::default() }) });
        doc.apply_all([
            Intent::AddLayer(group),
            Intent::SetMeta { layer: group, meta: LayerMeta { source: LayerSource::Group, order: 0, timing: LayerTiming::place(0, None, 90) } },
            Intent::SetAttrs { layer: group, patch: two_d.clone() },
            Intent::AddLayer(field),
            Intent::SetMeta { layer: field, meta: LayerMeta { source: LayerSource::Shape, order: 1, timing: LayerTiming::place(0, None, 90) } },
            Intent::SetAttrs { layer: field, patch: LayerAttrsPatch { parent: Some(Some(group)), ..two_d.clone() } },
            Intent::SetShapes { layer: field, shapes: vec![square(2.0)] },
        ]).unwrap();
        let put = |doc: &mut Document, layer, name: &str, value: Value| doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap();
        put(&mut doc, group, property::POSITION, Value::Vec2([6.0, 6.0]));
        put(&mut doc, group, layout::DISPLAY, Value::Enum(1));
        put(&mut doc, group, layout::HORIZONTAL_SIZING, Value::Enum(2));
        put(&mut doc, group, layout::VERTICAL_SIZING, Value::Enum(2));
        put(&mut doc, group, layout::WIDTH, Value::F64(148.0));
        put(&mut doc, group, layout::HEIGHT, Value::F64(88.0));
        put(&mut doc, field, layout::POSITION_TYPE, Value::Enum(1));
        put(&mut doc, field, property::POSITION, Value::Vec2([74.0, 44.0]));
        // 12 個の四角を上からばらばらに落とす。
        for i in 0..12u64 {
            let layer = LayerId(10 + i);
            doc.apply_all([
                Intent::AddLayer(layer),
                Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Shape, order: 2 + i as i16, timing: LayerTiming::place(0, None, 90) } },
                Intent::SetAttrs { layer, patch: LayerAttrsPatch { parent: Some(Some(group)), ..two_d.clone() } },
                Intent::SetShapes { layer, shapes: vec![square(16.0)] },
            ]).unwrap();
            put(&mut doc, layer, layout::POSITION_TYPE, Value::Enum(1));
            put(&mut doc, layer, property::POSITION, Value::Vec2([8.0 + (i % 6) as f64 * 22.0, -20.0 - (i / 6) as f64 * 30.0]));
        }
        doc.apply(Intent::SetEffects { layer: field, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.field".into() }] }).unwrap();
        for (name, value) in [("spread", 1.0), ("turn", 0.0), ("angle", 90.0), ("strength", 400.0), ("reach", 0.0), ("tumble", 0.4), ("hold", 0.0)] {
            doc.apply(Intent::SetConstant { layer: field, property: PropertyId::effect_param(EffectId(0), name).unwrap(), value: Value::F64(value) }).unwrap();
        }
        let mut engine = Engine::new().unwrap();
        // 2.0 秒から 2.5 秒(落ちて積もった後)。
        let mut frames = Vec::new();
        for frame in 60..=75 {
            engine.render_frame(&doc.view(), RationalTime::try_from_frame(frame, fps).unwrap()).unwrap();
            frames.push(engine.block_states());
        }
        assert!(frames[0].len() >= 12, "物が並んでいる ({})", frames[0].len());
        let mut worst = 0.0f32;
        for pair in frames.windows(2) {
            for (a, b) in pair[0].iter().zip(pair[1].iter()) {
                worst = worst.max(((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt());
            }
        }
        assert!(worst < 1.0, "積もった後は震えない(1 コマの動きの最大 {worst}px)");
    }

    /// Push Apart を GPU のブロックにしても、書類の間合いの押し合い(CPU、32 回)と同じだけ押す。
    #[test]
    fn the_push_apart_block_pushes_like_the_margin_law() {
        use crate::render::compositor::effects::block_program::{program_for, read_state, BlockItem, BlockWorld};
        let place = |margin: bool| {
            let mut doc = Document::new().with_programs(crate::extensions::bundled());
            doc.apply(Intent::SetComposition(Composition { width: W, height: H, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 1, background: [0.0; 4] })).unwrap();
            let spots = [[40.0, 40.0], [52.0, 44.0], [60.0, 30.0], [100.0, 60.0], [104.0, 64.0], [20.0, 80.0]];
            for (i, at) in spots.iter().enumerate() {
                let layer = LayerId(i as u64 + 1);
                doc.apply_all([
                    Intent::AddLayer(layer),
                    Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Shape, order: i as i16, timing: LayerTiming::place(0, None, 1) } },
                    Intent::SetAttrs { layer, patch: LayerAttrsPatch { projection: Some(LayerProjection::TwoD), ..Default::default() } },
                    Intent::SetShapes { layer, shapes: vec![ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: 20.0, y: 14.0 } }, ops: Vec::new(), stroke: None, fill: Some(Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 1.0, b: 1.0 }), ..Default::default() }) })] },
                    Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2(*at) },
                ]).unwrap();
                if margin {
                    doc.apply(Intent::SetConstant { layer, property: PropertyId::new(layout::MARGIN).unwrap(), value: Value::F64(5.0) }).unwrap();
                }
            }
            doc
        };
        let (plain, pushed) = (place(false), place(true));
        let t = RationalTime::ZERO;
        let frame = pushed.view().layout_frame(t).unwrap();
        let view = plain.view();
        let items: Vec<BlockItem> = (1..=6).map(|i| {
            let id = LayerId(i);
            let b = view.layer_box(id, t).unwrap().unwrap();
            let m = crate::doc::store::view::resolve::transform::local_transform(&view, id, t).unwrap();
            let (lo, hi) = (m.transform_point2(glam::vec2(b[0], b[1])), m.transform_point2(glam::vec2(b[2], b[3])));
            BlockItem { lo: lo.to_array(), hi: hi.to_array(), room_lo: [0.0; 2], room_size: [W as f32, H as f32], radius: 0.0, group: 0, margin: 0.0, weight: 1.0, ..Default::default() }
        }).collect();
        let engine = Engine::new().unwrap();
        let (device, queue) = (&engine.compositor.ctx.device, &engine.compositor.ctx.queue);
        let program = program_for(device, "push_apart");
        let mut world = BlockWorld::new(device);
        world.begin(device, queue, &items, 0.0);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("test") });
        program.record(device, queue, &mut encoder, &mut world, 0.0, &[0, 1, 2, 3, 4, 5], &[5.0]);
        let gpu = read_state(device, queue, &world, encoder);
        let mut moved = 0;
        for i in 0..6 {
            let cpu = frame.nudges.get(&LayerId(i as u64 + 1)).copied().unwrap_or([0.0, 0.0]);
            let g = gpu[i].translate;
            if cpu != [0.0, 0.0] { moved += 1; }
            assert!((g[0] - cpu[0]).abs() < 0.05 && (g[1] - cpu[1]).abs() < 0.05, "object {i}: gpu {g:?} cpu {cpu:?}");
        }
        assert!(moved >= 4, "the overlapping ones were pushed");
    }

    /// ブロックは効果の順につながる: 押し合って(Push Apart)から壁で折り返す(Bounce)と、全員が箱の中に収まる。
    #[test]
    fn blocks_chain_in_effect_order() {
        use crate::render::compositor::effects::block_program::{program_for, read_state, BlockItem, BlockWorld};
        let engine = Engine::new().unwrap();
        let (device, queue) = (&engine.compositor.ctx.device, &engine.compositor.ctx.queue);
        let push = program_for(device, "push_apart");
        let bounce = program_for(device, "bounce");
        let room = [200.0f32, 120.0];
        let mut items = Vec::new();
        for i in 0..40 {
            let x = 60.0 + (i as f32 * 37.0) % 180.0;
            let y = 20.0 + (i as f32 * 23.0) % 110.0;
            items.push(BlockItem { lo: [x, y], hi: [x + 12.0, y + 12.0], room_lo: [0.0; 2], room_size: room, radius: 0.0, group: 7, margin: 0.0, weight: 1.0, ..Default::default() });
        }
        let mut world = BlockWorld::new(device);
        world.begin(device, queue, &items, 0.0);
        let members: Vec<u32> = (0..40).collect();
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("test") });
        push.record(device, queue, &mut encoder, &mut world, 0.0, &members, &[2.0]);
        bounce.record(device, queue, &mut encoder, &mut world, 0.0, &members, &[1.0]);
        let state = read_state(device, queue, &world, encoder);
        for (i, (item, o)) in items.iter().zip(&state).enumerate() {
            let (lo, hi) = ([item.lo[0] + o.translate[0], item.lo[1] + o.translate[1]], [item.hi[0] + o.translate[0], item.hi[1] + o.translate[1]]);
            assert!(lo[0] >= -0.01 && lo[1] >= -0.01 && hi[0] <= room[0] + 0.01 && hi[1] <= room[1] + 0.01, "object {i} ends inside the room: {lo:?} {hi:?}");
        }
        let spread = state.iter().filter(|o| o.translate != [0.0, 0.0]).count();
        assert!(spread > 20, "the crowd was pushed and folded: {spread} moved");
    }

    /// ブロックごとに自分の欄を読む(1 コマに送る前の書き込みで、後のブロックの欄が前のブロックに混ざらない)。
    #[test]
    fn each_block_reads_its_own_params() {
        use crate::render::compositor::effects::block_program::{program_for, read_state, BlockItem, BlockWorld};
        let engine = Engine::new().unwrap();
        let (device, queue) = (&engine.compositor.ctx.device, &engine.compositor.ctx.queue);
        let bounce = program_for(device, "bounce");
        let push = program_for(device, "push_apart");
        let items = [BlockItem { lo: [300.0, 20.0], hi: [310.0, 30.0], room_lo: [0.0; 2], room_size: [200.0, 100.0], radius: 0.0, group: 1, margin: 0.0, weight: 1.0, ..Default::default() }];
        let mut world = BlockWorld::new(device);
        world.begin(device, queue, &items, 0.0);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("test") });
        bounce.record(device, queue, &mut encoder, &mut world, 0.0, &[0], &[0.0]);
        push.record(device, queue, &mut encoder, &mut world, 0.0, &[0], &[9.0]);
        let state = read_state(device, queue, &world, encoder);
        assert_eq!(state[0].translate, [0.0, 0.0], "Bounce at strength 0 does not fold, whatever the next block's margin is");
    }

    /// 調べる口: `MOTOLII_BLOCK_DOC` の書類の `MOTOLII_BLOCK_FRAME` コマの物の箱とブロックの結果を出す。
    #[test]
    #[ignore]
    fn dump_blocks() {
        use crate::render::compositor::effects::block_program::read_state;
        let doc = Document::load(std::env::var("MOTOLII_BLOCK_DOC").unwrap()).unwrap().with_programs(crate::extensions::bundled());
        let frame: i64 = std::env::var("MOTOLII_BLOCK_FRAME").unwrap().parse().unwrap();
        let view = doc.view();
        let fps = view.composition().unwrap().unwrap().fps;
        let t = RationalTime::try_from_frame(frame, fps).unwrap();
        let mut engine = Engine::new().unwrap();
        let scope = engine.gpu_device().push_error_scope(wgpu::ErrorFilter::Validation);
        engine.render_frame(&view, t).unwrap();
        eprintln!("gpu validation: {:?}", pollster::block_on(scope.pop()));
        eprintln!("layer failures: {:?}", engine.layer_failures());
        for (k, o) in engine.blocks.objects.iter().enumerate() { eprintln!("object {k}: {o:?}"); }
        for b in &engine.blocks.batches { eprintln!("batch stage {} {} {:?} {:?}", b.stage, b.plugin, b.params, b.members); }
        let Some(world) = engine.blocks.world.as_ref() else { return };
        let encoder = engine.compositor.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        for (k, o) in read_state(&engine.compositor.ctx.device, &engine.compositor.ctx.queue, world, encoder).iter().enumerate() { eprintln!("state {k}: {o:?}"); }
    }

    /// 付いて置く札は、相手がブロックで動いた分だけ一緒に動く(GPU の中で、読み戻さずに)。
    #[test]
    fn an_anchored_label_follows_what_a_block_moved() {
        use crate::render::compositor::effects::block_program::read_state;
        let mut doc = scene(true);
        let label = LayerId(3);
        doc.apply_all([
            Intent::AddLayer(label),
            Intent::SetMeta { layer: label, meta: LayerMeta { source: LayerSource::Shape, order: 2, timing: LayerTiming::place(0, None, 90) } },
            Intent::SetAttrs { layer: label, patch: LayerAttrsPatch { projection: Some(LayerProjection::TwoD), ..Default::default() } },
            Intent::SetShapes { layer: label, shapes: vec![ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: 6.0, y: 4.0 } }, ops: Vec::new(), stroke: None, fill: Some(Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 0.0, b: 0.0 }), ..Default::default() }) })] },
            Intent::SetConstant { layer: label, property: PropertyId::new(layout::POSITION_ANCHOR).unwrap(), value: Value::LayerId(2) },
            Intent::SetConstant { layer: label, property: PropertyId::new(layout::POSITION_AREA).unwrap(), value: Value::Enum(2) },
        ]).unwrap();
        let mut engine = Engine::new().unwrap();
        let t = RationalTime::try_from_frame(45, Fps::try_new(30, 1).unwrap()).unwrap();
        engine.render_frame(&doc.view(), t).unwrap();
        let layers = engine.blocks.object_layers.clone();
        let world = engine.blocks.world.as_ref().unwrap();
        let encoder = engine.compositor.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let state = read_state(&engine.compositor.ctx.device, &engine.compositor.ctx.queue, world, encoder);
        let at = |id: u64| state[layers.iter().position(|l| l.0 == id).expect("an object")].translate;
        assert!(at(2) != [0.0, 0.0], "the square was folded by Bounce");
        assert_eq!(at(3), at(2), "its label moved with it");
    }

    /// 名指しの口を書類から: 鍵で動く Hub に Position Anchor で名指しした tile の Effector は、Hub の今の中心で判定する
    /// (鍵は休みの箱を動かすので `anchor_centre` = 休みの箱の中心 + ずれ)。Hub は block を持たなくても物になる。
    #[test]
    fn an_effector_takes_its_centre_from_the_hub_the_tile_is_anchored_to() {
        use crate::render::compositor::effects::block_program::read_state;
        let mut doc = Document::new().with_programs(crate::extensions::bundled());
        doc.apply(Intent::SetComposition(Composition { width: W, height: H, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 90, background: [0.0; 4] })).unwrap();
        let (hub, tile) = (LayerId(1), LayerId(2));
        let two_d = LayerAttrsPatch { projection: Some(LayerProjection::TwoD), ..Default::default() };
        let square = |size: f64| vec![ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: size, y: size } }, ops: Vec::new(), stroke: None, fill: Some(Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 1.0, b: 1.0 }), ..Default::default() }) })];
        doc.apply_all([
            Intent::AddLayer(hub),
            Intent::SetMeta { layer: hub, meta: LayerMeta { source: LayerSource::Shape, order: 0, timing: LayerTiming::place(0, None, 90) } },
            Intent::SetAttrs { layer: hub, patch: two_d.clone() },
            Intent::SetShapes { layer: hub, shapes: square(8.0) },
            Intent::AddLayer(tile),
            Intent::SetMeta { layer: tile, meta: LayerMeta { source: LayerSource::Shape, order: 1, timing: LayerTiming::place(0, None, 90) } },
            Intent::SetAttrs { layer: tile, patch: two_d },
            Intent::SetShapes { layer: tile, shapes: square(16.0) },
            Intent::SetEffects { layer: tile, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.effector".into() }] },
        ]).unwrap();
        let put = |doc: &mut Document, layer, name: &str, value: Value| doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap();
        put(&mut doc, tile, property::POSITION, Value::Vec2([80.0, 50.0]));
        put(&mut doc, tile, layout::POSITION_ANCHOR, Value::LayerId(hub.0));
        for (name, value) in [("shape", 0.0), ("size", 30.0), ("soft", 6.0), ("strength", 1.0), ("lift", -20.0)] {
            put(&mut doc, tile, &format!("{}0.param.{name}", property::EFFECT_PREFIX), Value::F64(value));
        }
        let mut track = KeyframeTrack::new();
        track.insert(Keyframe { t: RationalTime::ZERO, value: Value::Vec2([20.0, 50.0]), interp: Interp::Linear, spatial: None });
        track.insert(Keyframe { t: RationalTime::try_new(3, 1).unwrap(), value: Value::Vec2([140.0, 50.0]), interp: Interp::Linear, spatial: None });
        doc.apply(Intent::SetTrack { layer: hub, property: PropertyId::new(property::POSITION).unwrap(), track }).unwrap();
        let mut engine = Engine::new().unwrap();
        let fps = Fps::try_new(30, 1).unwrap();
        let mut at = |frame: i64| {
            engine.render_frame(&doc.view(), RationalTime::try_from_frame(frame, fps).unwrap()).unwrap();
            let slot = |id: LayerId| engine.blocks.slots[&id] as usize;
            let (h, k) = (slot(hub), slot(tile));
            assert_eq!(engine.blocks.objects[k].anchor_slot, h as u32, "the tile names the hub");
            assert_eq!(engine.blocks.objects[h].anchor_slot, u32::MAX, "the hub names nobody");
            let world = engine.blocks.world.as_ref().unwrap();
            let encoder = engine.compositor.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            let state = read_state(&engine.compositor.ctx.device, &engine.compositor.ctx.queue, world, encoder);
            (engine.blocks.objects[h].lo, state[k].translate)
        };
        let (hub_far, tile_far) = at(0);
        let (hub_over, tile_over) = at(45);
        assert!((hub_over[0] - hub_far[0] - 60.0).abs() < 0.5, "the keys move the hub's rest box: {hub_far:?} -> {hub_over:?}");
        assert_eq!(tile_far, [0.0, 0.0], "hub 60 px away: outside the sphere, the tile sits still");
        assert!((tile_over[1] + 20.0).abs() < 1e-3 && tile_over[0].abs() < 1e-3, "hub over the tile: the tile lifts by Lift: {tile_over:?}");
    }

    /// Wave: 時刻と物の順で縦の正弦波。Wavelength 個離れた物は同じ高さ、半分なら逆。
    #[test]
    fn the_wave_block_travels_along_things_in_order() {
        use crate::render::compositor::effects::block_program::{program_for, read_state, BlockItem, BlockWorld};
        let engine = Engine::new().unwrap();
        let (device, queue) = (&engine.compositor.ctx.device, &engine.compositor.ctx.queue);
        let wave = program_for(device, "wave");
        let items: Vec<BlockItem> = (0..8).map(|i| BlockItem { lo: [i as f32 * 20.0, 0.0], hi: [i as f32 * 20.0 + 10.0, 10.0], weight: 1.0, ..Default::default() }).collect();
        let mut world = BlockWorld::new(device);
        world.begin(device, queue, &items, 0.0);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("test") });
        wave.record(device, queue, &mut encoder, &mut world, 0.125, &(0..8).collect::<Vec<u32>>(), &[10.0, 2.0, 4.0]);
        let y: Vec<f32> = read_state(device, queue, &world, encoder).iter().map(|o| o.translate[1]).collect();
        let expect = |k: f32| 10.0 * (std::f32::consts::TAU * (2.0 * 0.125 - k / 4.0)).sin();
        for (k, v) in y.iter().enumerate() {
            assert!((v - expect(k as f32)).abs() < 1e-3, "object {k}: {v} vs {}", expect(k as f32));
        }
        assert!((y[0] - y[4]).abs() < 1e-3 && (y[0] + y[2]).abs() < 1e-3, "a wavelength apart: same; half: opposite {y:?}");
    }

    /// 近くの物だけ見る押し合い(升目)は、全組を見る同じ手順と同じ結果になる(散らばった 2000 個、4 組)。
    #[test]
    fn push_apart_over_neighbours_matches_all_pairs_at_scale() {
        use crate::render::compositor::effects::block_program::{program_for, read_state, BlockItem, BlockWorld};
        let mut rng = 3u32;
        let mut next = || { rng = rng.wrapping_mul(1664525).wrapping_add(1013904223); (rng >> 8) as f32 / (1u32 << 24) as f32 };
        let items: Vec<BlockItem> = (0..2000).map(|k| {
            let (x, y, w, h) = (next() * 3000.0, next() * 3000.0, 6.0 + next() * 14.0, 6.0 + next() * 14.0);
            BlockItem { lo: [x, y], hi: [x + w, y + h], room_size: [3000.0, 3000.0], group: k % 4, weight: 0.5 + next(), ..Default::default() }
        }).collect();
        let margin = 3.0f32;
        // CPU: 同じ手順を全組で(32 回、全員を同時に測って動かす)。
        let mut lo: Vec<[f32; 2]> = items.iter().map(|i| i.lo).collect();
        let mut hi: Vec<[f32; 2]> = items.iter().map(|i| i.hi).collect();
        for _ in 0..32 {
            let mut step = vec![[0.0f32; 2]; items.len()];
            for k in 0..items.len() {
                for j in 0..items.len() {
                    if j == k || items[j].group != items[k].group { continue; }
                    let (alo, ahi) = ([lo[k][0] - margin, lo[k][1] - margin], [hi[k][0] + margin, hi[k][1] + margin]);
                    let (blo, bhi) = ([lo[j][0] - margin, lo[j][1] - margin], [hi[j][0] + margin, hi[j][1] + margin]);
                    let gap = [(blo[0] + bhi[0]) * 0.5 - (alo[0] + ahi[0]) * 0.5, (blo[1] + bhi[1]) * 0.5 - (alo[1] + ahi[1]) * 0.5];
                    let len = (gap[0] * gap[0] + gap[1] * gap[1]).sqrt();
                    let dir = if len > 1e-4 { [gap[0] / len, gap[1] / len] } else if k < j { [1.0, 0.0] } else { [-1.0, 0.0] };
                    let half = [((ahi[0] - alo[0]) + (bhi[0] - blo[0])) * 0.5, ((ahi[1] - alo[1]) + (bhi[1] - blo[1])) * 0.5];
                    if half[0] - gap[0].abs() <= 0.0 || half[1] - gap[1].abs() <= 0.0 { continue; }
                    let need = |a: usize| if dir[a].abs() >= 1e-6 { ((half[a] - gap[a].abs()) / dir[a].abs()).max(0.0) } else { 1e30 };
                    let depth = need(0).min(need(1));
                    if depth <= 0.0 || depth >= 1e29 { continue; }
                    let w = items[k].weight / (items[k].weight + items[j].weight);
                    step[k][0] -= dir[0] * depth * w * 0.5;
                    step[k][1] -= dir[1] * depth * w * 0.5;
                }
            }
            for k in 0..items.len() {
                for a in 0..2 { lo[k][a] += step[k][a]; hi[k][a] += step[k][a]; }
            }
        }
        let engine = Engine::new().unwrap();
        let (device, queue) = (&engine.compositor.ctx.device, &engine.compositor.ctx.queue);
        let push = program_for(device, "push_apart");
        let mut world = BlockWorld::new(device);
        world.begin(device, queue, &items, 0.0);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("test") });
        push.record(device, queue, &mut encoder, &mut world, 0.0, &(0..items.len() as u32).collect::<Vec<u32>>(), &[margin]);
        let gpu = read_state(device, queue, &world, encoder);
        let mut moved = 0;
        assert_eq!(push_reach(), Some("margin".to_owned()), "Push Apart declares its margin as its reach");
        for k in 0..items.len() {
            let cpu = [lo[k][0] - items[k].lo[0], lo[k][1] - items[k].lo[1]];
            if cpu != [0.0, 0.0] { moved += 1; }
            let g = gpu[k].translate;
            assert!((g[0] - cpu[0]).abs() < 0.05 && (g[1] - cpu[1]).abs() < 0.05, "object {k}: gpu {g:?} cpu {cpu:?}");
        }
        assert!(moved > 20, "some of them overlapped and were pushed: {moved}");
    }

    fn push_reach() -> Option<String> {
        crate::render::compositor::effects::block_program::catalog_definition("push_apart").manifest.reach
    }

    /// Margin が箱よりずっと大きくても、届く距離の分だけ升目を広げるので相手を取りこぼさない。
    #[test]
    fn a_wide_margin_still_finds_its_neighbours() {
        use crate::render::compositor::effects::block_program::{neighbors, BlockItem};
        let items = [
            BlockItem { lo: [0.0, 0.0], hi: [4.0, 4.0], weight: 1.0, ..Default::default() },
            BlockItem { lo: [60.0, 0.0], hi: [64.0, 4.0], weight: 1.0, ..Default::default() },
        ];
        let (_, blind) = neighbors(&items, 0.0);
        assert!(blind.is_empty(), "without the reach, 60 px apart is out of a 8 px cell's 3×3");
        let (starts, list) = neighbors(&items, 40.0);
        assert_eq!((starts, list), (vec![0, 1, 2], vec![1, 0]), "with a 40 px margin they see each other");
    }
}
