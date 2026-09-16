//! 物理の解き手は借りる(提案 2026-09-16、利用者「物理演算を 1 から作るんじゃなくてこれも外部に揃ったやつがあるだろ」)。
//! Rapier(dimforge、Apache-2.0)が積み上げの warm start・摩擦・joint・状態の保存を持っている。
//! 先例: Cavalry は Box2D。ここが持つのは**意図から Rapier への訳**だけで、解き方は持たない。
//!
//! 覚え方: 物理は時刻の純関数ではなく列(利用者「前のコマの力を覚えるべきです」)。1 コマ進める度に
//! 世界を進め、飛んだ時は近い所から解き直す。狙い(場・着地・吊り)は純関数のまま。

use std::collections::HashMap;

use glam::Vec2;
use rapier2d::prelude::*;

use crate::doc::store::LayerId;

/// comp の px と Rapier の長さの比(Rapier の既定はメートル基準で、その方が接触が落ち着く)。
const PX: f32 = 0.01;
/// 壁の厚み・長さ(m)。長い壁は箱が伸び縮みしても位置を追うだけで済む。
const WALL_THICK: f32 = 0.5;
const WALL_REACH: f32 = 60.0;
/// めり込み・抜けの法は解き手の物(取説 rapier 0.32 narrow_phase.rs:943): 速く動く側が壁(kinematic)でも
/// 速度から予測接触を張る soft-CCD が効く。shape-cast の CCD は dynamic だけなので、壁と留め具はこちら。
/// 1 コマにこの距離(m)まで動く壁を追う(= 200px/コマ)。
/// 棚の札(vism の `"PHYSICS"`)から読む嘘と解き手の欄。世界は部品だけ、組み方はここ。
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Lies {
    pub room_ancestor: bool,
    pub walls_follow: bool,
    pub keep_box: bool,
    pub gather: bool,
    pub substeps: usize,
    /// 以下 3 つは Rapier の IntegrationParameters と同じ名前・同じ既定(取説 rapier 0.32 integration_parameters.rs:311-321)。
    pub num_solver_iterations: usize,
    pub max_ccd_substeps: usize,
    pub normalized_allowed_linear_error: f32,
    pub soft_ccd: f32,
    pub corrective: f32,
}

impl Default for Lies {
    fn default() -> Self {
        let rapier = IntegrationParameters::default();
        Self {
            room_ancestor: true, walls_follow: true, keep_box: true, gather: true, substeps: 4,
            num_solver_iterations: rapier.num_solver_iterations,
            max_ccd_substeps: rapier.max_ccd_substeps,
            normalized_allowed_linear_error: rapier.normalized_allowed_linear_error,
            soft_ccd: 2.0, corrective: 60.0,
        }
    }
}

impl Lies {
    pub(crate) fn from_manifest(physics: &std::collections::BTreeMap<String, serde_json::Value>) -> Self {
        let word = |k: &str| physics.get(k).and_then(|v| v.as_str());
        let num = |k: &str, d: f64| physics.get(k).and_then(|v| v.as_f64()).unwrap_or(d);
        let base = Self::default();
        Self {
            room_ancestor: word("ROOM") != Some("parent"),
            walls_follow: word("WALLS") != Some("fixed"),
            keep_box: word("KEEP") != Some("none"),
            gather: word("TIME") != Some("forward"),
            substeps: num("SUBSTEPS", base.substeps as f64).clamp(1.0, 16.0) as usize,
            num_solver_iterations: num("NUM_SOLVER_ITERATIONS", base.num_solver_iterations as f64).clamp(1.0, 64.0) as usize,
            max_ccd_substeps: num("MAX_CCD_SUBSTEPS", base.max_ccd_substeps as f64).clamp(1.0, 16.0) as usize,
            normalized_allowed_linear_error: num("NORMALIZED_ALLOWED_LINEAR_ERROR", base.normalized_allowed_linear_error as f64) as f32,
            soft_ccd: num("SOFT_CCD", base.soft_ccd as f64) as f32,
            corrective: num("CORRECTIVE", base.corrective as f64) as f32,
        }
    }
}

/// 1 つの物(comp の座標、描く時と同じ置き方)。
#[derive(Clone, Debug)]
pub(crate) struct Body {
    pub layer: LayerId,
    /// どの箱の中か(Room::group)。
    pub group: u32,
    /// この物の時刻(コマ)。箱の順番の札でずれる。集まる時はここから逆に読む。
    pub frame: i64,
    /// 箱の真ん中と半分の大きさ(comp の px)。
    pub centre: [f32; 2],
    pub half: [f32; 2],
    /// 丸い物は半径で当たる。
    pub round: bool,
    /// 間合い(CSS の margin)。当たりの外側に足す。
    pub margin: f32,
    /// 重さ(0 なら動かない)。
    pub weight: f32,
    /// 手触り(提案 2026-09-16 の 1 本の軸): 0 返す ↔ 0.5 吸う ↔ 1 引きずる。
    /// 解き手の欄(摩擦・反発・減衰)はここから作る — 人は摩擦係数を操作しない。
    pub hardness: f32,
    /// 形そのものの輪郭(comp の px)。当たりは四角ではなくこの形で見る。
    pub outline: Option<std::sync::Arc<Vec<[f32; 2]>>>,
}

/// 1 つの住む箱(壁)と、その中に効く場。
#[derive(Clone, Debug)]
pub(crate) struct Room {
    /// どの箱か(層の群)。箱ごとに世界を分ける: 隣の升目の物とは当たらない。
    pub group: u32,
    pub rect: [f32; 4],
    pub round: bool,
    /// 下の向きと強さ(px/秒²)。場の一様な分から。
    pub gravity: [f32; 2],
    /// 点の場: 元の位置・引く強さ(負なら押す)・渦の分・届く距離(0 なら箱じゅう)。
    pub wells: Vec<Well>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Well {
    pub at: [f32; 2],
    pub pull: f32,
    pub swirl: f32,
    pub reach: f32,
    /// 家(レイアウトの場所)へ戻す強さ。0 なら戻らない。
    pub hold: f32,
}

/// Rapier の世界。1 コマ進める度に覚えている(前のコマの力を覚える = warm start)。
pub(crate) struct Physics {
    bodies: RigidBodySet,
    colliders: ColliderSet,
    islands: IslandManager,
    broad: DefaultBroadPhase,
    narrow: NarrowPhase,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd: CCDSolver,
    pipeline: PhysicsPipeline,
    params: IntegrationParameters,
    /// 層 → 剛体と、始まりの場所(ずれはここからの差)。
    handles: HashMap<LayerId, (RigidBodyHandle, [f32; 2])>,
    /// 今どのコマまで進めたか。飛んだらここから解き直す。
    pub(crate) frame: Option<i64>,
    /// 組んだ時の顔ぶれ(変わったら世界を組み直す)。
    shape: Vec<(LayerId, [f32; 2], [f32; 2])>,
    /// 直近に渡された箱と場(可視のモードが読む)。
    last_rooms: Vec<Room>,
    /// 四角い箱の壁(床・左・右)。箱は鍵で伸び縮みするので、壁は kinematic で毎歩、箱に付いて行く。
    walls: Vec<(u32, usize, RigidBodyHandle)>,
    /// コマごとのずれ(焼き)。集まる時は終わりから逆に読む。
    baked: Vec<HashMap<LayerId, ([f32; 2], f32)>>,
    /// 棚の札から読んだ嘘と欄。
    pub(crate) lies: Lies,
    /// 集まる時の終わりのコマ。あれば物ごとの時刻から逆に焼きを読む。無ければ生の世界を読む。
    reading: Option<i64>,
    /// 物ごとの時刻(順番の札でずれたコマ)。
    frames: HashMap<LayerId, i64>,
    /// 組んだ時の輪郭(動く抜きの当たりを差し替える時に比べる)。
    shapes: HashMap<LayerId, Option<std::sync::Arc<Vec<[f32; 2]>>>>,
}

impl Default for Physics {
    fn default() -> Self {
        Self::new()
    }
}

impl Physics {
    pub(crate) fn new() -> Self {
        Self {
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            islands: IslandManager::new(),
            broad: DefaultBroadPhase::new(),
            narrow: NarrowPhase::new(),
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            ccd: CCDSolver::new(),
            pipeline: PhysicsPipeline::new(),
            params: IntegrationParameters::default(),
            handles: HashMap::new(),
            frame: None,
            shape: Vec::new(),
            last_rooms: Vec::new(),
            walls: Vec::new(),
            baked: Vec::new(),
            reading: None,
            frames: HashMap::new(),
            shapes: HashMap::new(),
            lies: Lies::default(),
        }
    }

    /// 顔ぶれの印。重さ 0 の物(留め具・引力の元)は鍵で動くので、場所は印に入れない —
    /// 動く度に世界を組み直すと前のコマの力が消える。場所は毎歩 kinematic として渡す。
    fn signature(bodies: &[Body]) -> Vec<(LayerId, [f32; 2], [f32; 2])> {
        bodies.iter().map(|b| (b.layer, if b.weight <= 0.0 { [0.0, 0.0] } else { b.centre }, b.half)).collect()
    }

    /// 箱ごとの当たりの組(Rapier の collision group)。箱が違えば当たらない。
    fn lane(rooms: &[Room], group: u32) -> InteractionGroups {
        let i = rooms.iter().position(|r| r.group == group).unwrap_or(0) % 32;
        let bit = Group::from_bits_truncate(1 << i);
        InteractionGroups::new(bit, bit, InteractionTestMode::And)
    }

    /// 四角い箱の壁の中心(m): 床・左・右。
    fn wall_places(room: &Room) -> [(f32, f32); 3] {
        let (lo, hi) = ([room.rect[0] * PX, room.rect[1] * PX], [room.rect[2] * PX, room.rect[3] * PX]);
        let (cx, cy) = ((lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5);
        [(cx, hi[1] + WALL_THICK), (lo[0] - WALL_THICK, cy), (hi[0] + WALL_THICK, cy)]
    }

    /// 世界を組み直す(顔ぶれか始まりの場所が変わった時、または時刻が飛んだ時)。
    fn build(&mut self, rooms: &[Room], bodies: &[Body]) {
        // 何が Rapier に渡ったかを見る口(`MOTOLII_PHYSICS_DEBUG=1`)。
        if std::env::var("MOTOLII_PHYSICS_DEBUG").is_ok() {
            eprintln!("physics build: rooms {rooms:?}, bodies {}", bodies.len());
        }
        let lies = self.lies.clone();
        *self = Self::new();
        self.lies = lies;
        self.shape = Self::signature(bodies);
        for room in rooms {
            let lane = Self::lane(rooms, room.group);
            let (lo, hi) = ([room.rect[0] * PX, room.rect[1] * PX], [room.rect[2] * PX, room.rect[3] * PX]);
            let (w, h) = ((hi[0] - lo[0]) * 0.5, (hi[1] - lo[1]) * 0.5);
            let (cx, cy) = ((lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5);
            if room.round {
                // 丸い箱は丸い壁(器)。薄い線だと速い物が抜けるので、厚い円弧を並べて囲う。
                let r = w.min(h);
                let n = 48;
                let thick = 0.3;
                for i in 0..n {
                    let a = std::f32::consts::TAU * i as f32 / n as f32;
                    // 上は開ける(注ぎ口)。物は上から入って来る。
                    if a.sin() < -0.72 {
                        continue;
                    }
                    let half = r * (std::f32::consts::PI / n as f32).tan() * 1.2;
                    self.colliders.insert(
                        ColliderBuilder::cuboid(half, thick)
                            .translation(Vec2::new(cx + (r + thick) * a.cos(), cy + (r + thick) * a.sin()))
                            .rotation(a + std::f32::consts::FRAC_PI_2)
                            .friction(0.6)
                            .restitution(0.0)
                            .collision_groups(lane)
                            .build(),
                    );
                }
                continue;
            }
            // 蓋はしない: 物は上から入って来る(注ぐ・落とす)。床と左右だけが壁。
            // 壁は長めに取り、箱が伸び縮みしても位置を追うだけで済ませる(箱ごとに世界が分かれるので隣に届かない)。
            let _ = (cx, cy, w, h);
            for (which, (x, y)) in Self::wall_places(room).into_iter().enumerate() {
                let (hx, hy) = if which == 0 { (WALL_REACH, WALL_THICK) } else { (WALL_THICK, WALL_REACH) };
                let builder = if self.lies.walls_follow { RigidBodyBuilder::kinematic_position_based() } else { RigidBodyBuilder::fixed() };
                let wall = self.bodies.insert(builder.translation(Vec2::new(x, y)).soft_ccd_prediction(self.lies.soft_ccd).build());
                self.colliders.insert_with_parent(
                    ColliderBuilder::cuboid(hx, hy).friction(0.6).restitution(0.0).collision_groups(lane).build(),
                    wall,
                    &mut self.bodies,
                );
                self.walls.push((room.group, which, wall));
            }
        }
        for body in bodies {
            let soft = body.hardness.clamp(0.0, 1.0);
            let _ = soft;
            // 重さ 0 は押されない(留め具・引力の元)。周りがそれに当たり、それは押されない。
            // 鍵で動かせるように kinematic(場所は毎歩、書類から渡す)。
            let builder = if body.weight <= 0.0 { RigidBodyBuilder::kinematic_position_based() } else { RigidBodyBuilder::dynamic() };
            let handle = self.bodies.insert(
                builder
                    .translation(Vec2::new(body.centre[0] * PX, body.centre[1] * PX))
                    .ccd_enabled(true)
                    .soft_ccd_prediction(self.lies.soft_ccd)
                    .linear_damping(0.1 + soft * 1.6)
                    .angular_damping(0.2 + soft * 2.4)
                    .build(),
            );
            self.colliders.insert_with_parent(Self::collider_for(body, Self::lane(rooms, body.group)), handle, &mut self.bodies);
            self.shapes.insert(body.layer, body.outline.clone());
            self.handles.insert(body.layer, (handle, body.centre));
        }
    }

    /// 物の当たり: 形そのもの(凹みも効くように凸の塊へ分ける — 星の谷に物が乗る)。分けられない形は凸包、
    /// 輪郭が無い物だけ箱か丸で当たる。摩擦・反発は硬さから、重さは密度へ。
    fn collider_for(body: &Body, lane: InteractionGroups) -> Collider {
        let soft = body.hardness.clamp(0.0, 1.0);
        let margin = body.margin.max(0.0) * PX;
        let hull = body.outline.as_ref().and_then(|points| {
            let pts: Vec<Vec2> = points.iter().map(|p| Vec2::new((p[0] - body.centre[0]) * PX, (p[1] - body.centre[1]) * PX)).collect();
            if pts.len() < 3 {
                return None;
            }
            let indices: Vec<[u32; 2]> = (0..pts.len() as u32).map(|i| [i, (i + 1) % pts.len() as u32]).collect();
            Some(ColliderBuilder::convex_decomposition(&pts, &indices)).filter(|_| pts.len() >= 4).or_else(|| ColliderBuilder::convex_hull(&pts))
        });
        let collider = match hull {
            Some(hull) => hull,
            None if body.round => ColliderBuilder::ball((body.half[0].min(body.half[1]) * PX + margin).max(1e-3)),
            None => ColliderBuilder::cuboid((body.half[0] * PX + margin).max(1e-3), (body.half[1] * PX + margin).max(1e-3)),
        };
        collider
            .friction(0.15 + soft * 1.05)
            .restitution((0.55 * (1.0 - soft * 2.0)).max(0.0))
            .density(body.weight.max(0.05))
            .collision_groups(lane)
            .build()
    }

    /// 動く抜き(重さ 0 の動画・絵)の輪郭がコマで変わったら、当たりをそのコマの形に差し替える。
    /// 手が動けば当たりも動く — 抜いた後の透過が当たり、はコマごとに真。
    fn refresh_shapes(&mut self, rooms: &[Room], bodies: &[Body]) {
        for body in bodies.iter().filter(|b| b.weight <= 0.0) {
            let same = match (self.shapes.get(&body.layer), &body.outline) {
                (Some(Some(a)), Some(b)) => std::sync::Arc::ptr_eq(a, b) || **a == **b,
                (Some(None), None) => true,
                _ => false,
            };
            if same {
                continue;
            }
            let Some(&(handle, _)) = self.handles.get(&body.layer) else { continue };
            let old: Vec<ColliderHandle> = self.bodies.get(handle).map(|rb| rb.colliders().to_vec()).unwrap_or_default();
            for c in old {
                self.colliders.remove(c, &mut self.islands, &mut self.bodies, false);
            }
            self.colliders.insert_with_parent(Self::collider_for(body, Self::lane(rooms, body.group)), handle, &mut self.bodies);
            self.shapes.insert(body.layer, body.outline.clone());
        }
    }

    /// そのコマまで世界を進める。前のコマの続きなら 1 歩、飛んだら組み直して始めから。
    pub(crate) fn solve(&mut self, rooms: &[Room], bodies: &[Body], frame: i64, fps: f64, gather_last: Option<i64>) {
        if bodies.is_empty() {
            self.frame = None;
            return;
        }
        // 集まる = 同じ解きを終わりから逆に読む(終わりは構図)。終わりまで焼いてから読む番号を決める。
        if let Some(last) = gather_last {
            self.reading = None;
            self.solve(rooms, bodies, last, fps, None);
            let _ = frame;
            self.frames = bodies.iter().map(|b| (b.layer, b.frame)).collect();
            self.reading = Some(last);
            return;
        }
        self.reading = None;
        let same = self.shape == Self::signature(bodies);
        let step_from = match (same, self.frame) {
            (true, Some(last)) if frame == last => return,
            (true, Some(last)) if frame > last && frame - last <= 240 => last + 1,
            _ => {
                self.build(rooms, bodies);
                0
            }
        };
        // 解き手の欄は棚の札(field.wgsl の PHYSICS)から。積み上げと壁のめり込みには反復を増やし、
        // 速い物は CCD で刻む。長さの単位は m(PX で写している)のまま。
        self.params.num_solver_iterations = self.lies.num_solver_iterations;
        self.params.max_ccd_substeps = self.lies.max_ccd_substeps;
        self.params.normalized_allowed_linear_error = self.lies.normalized_allowed_linear_error;
        // 箱が中身より狭くなった時、壁 2 枚に挟まれた物は 1 歩で大きく押し戻して上へ逃がす
        // (既定 10 では字が壁を押し抜けた)。
        self.params.normalized_max_corrective_velocity = self.lies.corrective;
        // 1 コマを SUB 歩に割る(ゲームの定石の固定刻み)。壁と留め具は歩ごとに前のコマから補間するので、
        // 速く動く壁でも 1 歩の動きは小さく、挟まれた物が積み上がる余地ができる。
        let prev_rooms = if step_from == 0 { rooms.to_vec() } else { std::mem::take(&mut self.last_rooms) };
        self.last_rooms = rooms.to_vec();
        let prev_anchors: HashMap<LayerId, [f32; 2]> = bodies.iter().filter(|b| b.weight <= 0.0)
            .map(|b| (b.layer, if step_from == 0 { b.centre } else { self.handles.get(&b.layer).map_or(b.centre, |h| h.1) }))
            .collect();
        let sub = self.lies.substeps.max(1);
        self.params.dt = (1.0 / fps / sub as f64) as f32;
        for step in step_from..=frame.max(0) {
            if step == frame.max(0) {
                self.refresh_shapes(rooms, bodies);
            }
            for i in 1..=sub {
                // 最後のコマだけ壁を補間する(それより前のコマは既に目的地に着いている)。
                let k = if step == frame.max(0) { i as f32 / sub as f32 } else { 1.0 };
                self.push(rooms, &prev_rooms, &prev_anchors, k, bodies);
                self.pipeline.step(
                    Vec2::ZERO,
                    &self.params,
                    &mut self.islands,
                    &mut self.broad,
                    &mut self.narrow,
                    &mut self.bodies,
                    &mut self.colliders,
                    &mut self.impulse_joints,
                    &mut self.multibody_joints,
                    &mut self.ccd,
                    &(),
                    &(),
                );
                if self.lies.keep_box {
                    self.keep_inside(rooms, bodies);
                }
            }
            let snapshot: HashMap<LayerId, ([f32; 2], f32)> = bodies.iter().filter_map(|b| Some((b.layer, self.live_offset(b.layer)?))).collect();
            self.baked.truncate(step as usize);
            self.baked.push(snapshot);
        }
        self.frame = Some(frame.max(0));
    }

    /// 嘘: 物は箱から出ない。解いた後、箱の外に出た分は箱の中へ戻し、その向きの速さを消す
    /// (先例: PBD の位置への射影、Box2D の world bounds)。蓋は無いので上へは出られる。
    /// 中身が箱より大きい時は物同士の重なりを許し、解き手が上へ逃がす。
    fn keep_inside(&mut self, rooms: &[Room], bodies: &[Body]) {
        for body in bodies {
            if body.weight <= 0.0 {
                continue;
            }
            let Some(room) = rooms.iter().find(|r| r.group == body.group) else { continue };
            let Some(&(handle, _)) = self.handles.get(&body.layer) else { continue };
            let Some(rb) = self.bodies.get(handle) else { continue };
            let mut aabb: Option<Aabb> = None;
            for &c in rb.colliders() {
                if let Some(co) = self.colliders.get(c) {
                    let a = co.compute_aabb();
                    aabb = Some(aabb.map_or(a, |b| b.merged(&a)));
                }
            }
            let Some(aabb) = aabb else { continue };
            let (lo, hi) = ([room.rect[0] * PX, room.rect[1] * PX], [room.rect[2] * PX, room.rect[3] * PX]);
            let mut fix = Vec2::ZERO;
            if room.round {
                // 丸い箱: 中心からの距離で戻す(外接の半径で見る)。
                let r = (hi[0] - lo[0]).min(hi[1] - lo[1]) * 0.5;
                let centre = Vec2::new((lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5);
                let mid = Vec2::new((aabb.mins.x + aabb.maxs.x) * 0.5, (aabb.mins.y + aabb.maxs.y) * 0.5);
                let own = (aabb.maxs.x - aabb.mins.x).max(aabb.maxs.y - aabb.mins.y) * 0.5;
                let away = mid - centre;
                let far = away.length();
                if far + own > r && far > 1e-6 && away.y > -own {
                    fix = away / far * (r - own - far);
                }
            } else {
                if aabb.mins.x < lo[0] { fix.x = lo[0] - aabb.mins.x; }
                if aabb.maxs.x > hi[0] { fix.x = hi[0] - aabb.maxs.x; }
                if aabb.maxs.y > hi[1] { fix.y = hi[1] - aabb.maxs.y; }
            }
            if fix == Vec2::ZERO {
                continue;
            }
            let Some(rb) = self.bodies.get_mut(handle) else { continue };
            let at = rb.translation() + fix;
            rb.set_translation(at, true);
            let mut v = rb.linvel();
            if fix.x != 0.0 { v.x = 0.0; }
            if fix.y != 0.0 { v.y = 0.0; }
            rb.set_linvel(v, true);
        }
    }

    /// 場を力にする: 一様な分は重力、点の分は引き寄せ・押し出し・渦、家に留める分はばね。
    fn push(&mut self, rooms: &[Room], prev_rooms: &[Room], prev_anchors: &HashMap<LayerId, [f32; 2]>, k: f32, bodies: &[Body]) {
        // 壁は箱に付いて行く(箱は鍵・レイアウトで伸び縮みする — 環境は書き上がった構図そのもの)。
        for &(group, which, wall) in self.walls.iter().filter(|_| self.lies.walls_follow) {
            let Some(room) = rooms.iter().find(|r| r.group == group) else { continue };
            let (x, y) = Self::wall_places(room)[which];
            let (px, py) = prev_rooms.iter().find(|r| r.group == group).map_or((x, y), |r| Self::wall_places(r)[which]);
            if let Some(rb) = self.bodies.get_mut(wall) {
                rb.set_next_kinematic_translation(Vec2::new(px + (x - px) * k, py + (y - py) * k));
            }
        }
        for body in bodies {
            let Some(&(handle, home)) = self.handles.get(&body.layer) else { continue };
            let Some(rb) = self.bodies.get_mut(handle) else { continue };
            if body.weight <= 0.0 {
                // 押されない物は書類の場所へ(鍵で動く留め具・元)。ずれは 0 のまま描かれる。
                let from = prev_anchors.get(&body.layer).copied().unwrap_or(body.centre);
                let at = [from[0] + (body.centre[0] - from[0]) * k, from[1] + (body.centre[1] - from[1]) * k];
                rb.set_next_kinematic_translation(Vec2::new(at[0] * PX, at[1] * PX));
                if k >= 1.0 {
                    self.handles.insert(body.layer, (handle, body.centre));
                }
                continue;
            }
            let mass = rb.mass().max(1e-4);
            let at = rb.translation();
            let mut force = Vec2::ZERO;
            for room in rooms {
                force += Vec2::new(room.gravity[0] * PX, room.gravity[1] * PX) * mass;
                for well in &room.wells {
                    let away = Vec2::new(at.x - well.at[0] * PX, at.y - well.at[1] * PX);
                    let far = away.length();
                    let core = (body.half[0].max(body.half[1]) * PX).max(1e-3);
                    let soft = (far * far + core * core).sqrt();
                    let fall = if well.reach > 0.0 { (1.0 - far / (well.reach * PX)).max(0.0) } else { 1.0 };
                    if fall <= 0.0 {
                        continue;
                    }
                    let dir = away / soft;
                    force += -dir * well.pull * PX * mass * fall;
                    force += Vec2::new(-dir.y, dir.x) * well.swirl * PX * mass * fall;
                    if well.hold > 0.0 {
                        // 家(レイアウトの場所)へのばね。留まる物は家から離れない。
                        let back = Vec2::new(home[0] * PX - at.x, home[1] * PX - at.y);
                        force += back * well.hold * 60.0 * mass;
                    }
                }
            }
            rb.reset_forces(true);
            rb.add_force(force, true);
        }
    }

    /// 場の元・届く距離・一様な向き(可視のモードが読む)。
    pub(crate) fn wells(&self) -> Vec<([f32; 2], f32, [f32; 2])> {
        self.last_rooms.iter().flat_map(|room| {
            if room.wells.is_empty() {
                // 元の無い場(一様な重力)は、箱の真ん中から向きの矢だけ出す。
                let at = [(room.rect[0] + room.rect[2]) * 0.5, (room.rect[1] + room.rect[3]) * 0.5];
                return vec![(at, 0.0, room.gravity)];
            }
            room.wells.iter().map(|w| (w.at, w.reach, room.gravity)).collect::<Vec<_>>()
        }).collect()
    }

    /// 触れ合っている点と、その法線(comp の px)。可視のモードが読む — 真ん中どうしを結ぶより、
    /// どこで当たっているかが見える方が説明になる。
    pub(crate) fn contact_marks(&self) -> Vec<([f32; 2], [f32; 2])> {
        let mut out = Vec::new();
        for pair in self.narrow.contact_pairs().filter(|pair| pair.has_any_active_contact()) {
            let Some(first) = self.colliders.get(pair.collider1) else { continue };
            let iso = first.position();
            for manifold in &pair.manifolds {
                let normal = iso.rotation * manifold.local_n1;
                for point in &manifold.points {
                    if point.dist > 0.01 {
                        continue;
                    }
                    let at = iso * point.local_p1;
                    out.push(([at.x / PX, at.y / PX], [normal.x, normal.y]));
                }
            }
        }
        out
    }

    /// 物の真ん中と、今の速さ(comp の px/秒)。
    pub(crate) fn velocities(&self) -> Vec<([f32; 2], [f32; 2])> {
        self.handles.values().filter_map(|&(handle, _)| {
            let rb = self.bodies.get(handle)?;
            let at = rb.translation();
            let v = rb.linvel();
            Some(([at.x / PX, at.y / PX], [v.x / PX, v.y / PX]))
        }).collect()
    }

    /// 触れ合っている組の線(comp の px、真ん中どうし)。可視のモードが読む。
    pub(crate) fn contact_lines(&self) -> Vec<([f32; 2], [f32; 2])> {
        self.narrow.contact_pairs().filter(|pair| pair.has_any_active_contact()).filter_map(|pair| {
            let a = self.colliders.get(pair.collider1)?.parent()?;
            let b = self.colliders.get(pair.collider2)?.parent()?;
            let (a, b) = (self.bodies.get(a)?.translation(), self.bodies.get(b)?.translation());
            Some(([a.x / PX, a.y / PX], [b.x / PX, b.y / PX]))
        }).collect()
    }

    /// 触れ合っている組の数(測り用)。
    pub(crate) fn contacts(&self) -> usize {
        self.narrow.contact_pairs().filter(|p| p.has_any_active_contact()).count()
    }

    /// 物ごとの、始まりの場所からのずれ(comp の px)と回り(度)。
    pub(crate) fn offset(&self, layer: LayerId) -> Option<([f32; 2], f32)> {
        match self.reading {
            Some(last) => {
                let frame = self.frames.get(&layer).copied().unwrap_or(0).clamp(0, last);
                self.baked.get((last - frame) as usize)?.get(&layer).copied()
            }
            None => self.live_offset(layer),
        }
    }

    fn live_offset(&self, layer: LayerId) -> Option<([f32; 2], f32)> {
        let &(handle, home) = self.handles.get(&layer)?;
        let rb = self.bodies.get(handle)?;
        let at = rb.translation();
        Some(([at.x / PX - home[0], at.y / PX - home[1]], rb.rotation().angle().to_degrees()))
    }
}
