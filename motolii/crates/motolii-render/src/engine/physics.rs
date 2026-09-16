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

/// 1 つの物(comp の座標、描く時と同じ置き方)。
#[derive(Clone, Debug)]
pub(crate) struct Body {
    pub layer: LayerId,
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
        }
    }

    fn signature(bodies: &[Body]) -> Vec<(LayerId, [f32; 2], [f32; 2])> {
        bodies.iter().map(|b| (b.layer, b.centre, b.half)).collect()
    }

    /// 世界を組み直す(顔ぶれか始まりの場所が変わった時、または時刻が飛んだ時)。
    fn build(&mut self, rooms: &[Room], bodies: &[Body]) {
        // 何が Rapier に渡ったかを見る口(`MOTOLII_PHYSICS_DEBUG=1`)。
        if std::env::var("MOTOLII_PHYSICS_DEBUG").is_ok() {
            eprintln!("physics build: rooms {rooms:?}, bodies {}", bodies.len());
        }
        *self = Self::new();
        self.shape = Self::signature(bodies);
        for room in rooms {
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
                            .build(),
                    );
                }
                continue;
            }
            // 蓋はしない: 物は上から入って来る(注ぐ・落とす)。床と左右だけが壁。
            let thick = 0.5;
            for (x, y, hx, hy) in [
                (cx, hi[1] + thick, w + thick, thick),
                (lo[0] - thick, cy, thick, h * 3.0),
                (hi[0] + thick, cy, thick, h * 3.0),
            ] {
                self.colliders.insert(ColliderBuilder::cuboid(hx, hy).translation(Vec2::new(x, y)).friction(0.6).restitution(0.0).build());
            }
        }
        for body in bodies {
            let soft = body.hardness.clamp(0.0, 1.0);
            // 重さ 0 は動かない(留め具・引力の元)。周りがそれに当たり、それは動かない。
            let builder = if body.weight <= 0.0 { RigidBodyBuilder::fixed() } else { RigidBodyBuilder::dynamic() };
            let handle = self.bodies.insert(
                builder
                    .translation(Vec2::new(body.centre[0] * PX, body.centre[1] * PX))
                    .ccd_enabled(true)
                    .linear_damping(0.1 + soft * 1.6)
                    .angular_damping(0.2 + soft * 2.4)
                    .build(),
            );
            let margin = body.margin.max(0.0) * PX;
            // 形そのもので当たる。凹みも効くように凸の塊へ分ける(星の谷に物が乗る)。
            // 分けられない形は凸包で、輪郭が無い物だけ箱で当たる。
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
            self.colliders.insert_with_parent(
                collider
                    .friction(0.15 + soft * 1.05)
                    .restitution((0.55 * (1.0 - soft * 2.0)).max(0.0))
                    .density(body.weight.max(0.05))
                    .build(),
                handle,
                &mut self.bodies,
            );
            self.handles.insert(body.layer, (handle, body.centre));
        }
    }

    /// そのコマまで世界を進める。前のコマの続きなら 1 歩、飛んだら組み直して始めから。
    pub(crate) fn solve(&mut self, rooms: &[Room], bodies: &[Body], frame: i64, fps: f64) {
        if bodies.is_empty() {
            self.frame = None;
            return;
        }
        self.last_rooms = rooms.to_vec();
        let same = self.shape == Self::signature(bodies);
        let step_from = match (same, self.frame) {
            (true, Some(last)) if frame == last => return,
            (true, Some(last)) if frame > last && frame - last <= 240 => last + 1,
            _ => {
                self.build(rooms, bodies);
                0
            }
        };
        // 取説(rapier.rs, docs.rs 0.32): 既定は反復 4・CCD の刻み 1。積み上げと壁のめり込みには
        // 反復を増やし、速い物は CCD で刻む。長さの単位は m(PX で写している)のまま。
        self.params.dt = (1.0 / fps) as f32;
        self.params.num_solver_iterations = 8;
        self.params.max_ccd_substeps = 4;
        self.params.normalized_allowed_linear_error = 0.0005;
        for _ in step_from..=frame.max(0) {
            self.push(rooms, bodies);
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
        }
        self.frame = Some(frame.max(0));
    }

    /// 場を力にする: 一様な分は重力、点の分は引き寄せ・押し出し・渦、家に留める分はばね。
    fn push(&mut self, rooms: &[Room], bodies: &[Body]) {
        for body in bodies {
            let Some(&(handle, home)) = self.handles.get(&body.layer) else { continue };
            let Some(rb) = self.bodies.get_mut(handle) else { continue };
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
        let &(handle, home) = self.handles.get(&layer)?;
        let rb = self.bodies.get(handle)?;
        let at = rb.translation();
        Some(([at.x / PX - home[0], at.y / PX - home[1]], rb.rotation().angle().to_degrees()))
    }
}
