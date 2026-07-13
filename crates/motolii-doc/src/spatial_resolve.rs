//! LookAt/Follow 用の world position 事前解決(F-3: 参照先を先に評価)。
//!
//! 描画順とは独立に、parent / Group 継承を含む共通座標へ落としてから
//! `ResolvedLayerParams` と各レイヤーの world アフィンを埋める。

use std::collections::{HashMap, HashSet};

use motolii_core::RationalTime;
use motolii_eval::DataTracks;

use crate::affine::{compose_local, compose_transform, Affine2D};
use crate::command::find_envelope;
use crate::param::DocParam;
use crate::param_eval::{
    eval_f64, eval_look_at_rotation, eval_vec2, ParamEvalError, ResolvedLayerParams,
};
use crate::schema::{TrackItem, Transform2D};
use crate::{Document, LayerId};

/// ドキュメント全レイヤーの world 位置と world アフィンを依存順で解決する。
pub fn resolve_document_spaces(
    doc: &Document,
    t: RationalTime,
    tracks: &DataTracks,
) -> Result<(ResolvedLayerParams, HashMap<u64, Affine2D>), ParamEvalError> {
    let mut group_of: HashMap<u64, LayerId> = HashMap::new();
    let mut layer_ids = Vec::new();
    for track in &doc.tracks {
        collect_layers(&track.items, None, &mut group_of, &mut layer_ids);
    }

    let mut ctx = ResolveCtx {
        doc,
        t,
        tracks,
        group_of: &group_of,
        resolved: ResolvedLayerParams::default(),
        resolve_affine: HashMap::new(),
        world_affine: HashMap::new(),
        visiting: HashSet::new(),
    };

    for id in layer_ids {
        ctx.ensure_world_affine(id)?;
    }

    Ok((ctx.resolved, ctx.world_affine))
}

struct ResolveCtx<'a> {
    doc: &'a Document,
    t: RationalTime,
    tracks: &'a DataTracks,
    group_of: &'a HashMap<u64, LayerId>,
    resolved: ResolvedLayerParams,
    /// `resolve_transform` 相当(Group 継承なし・Transform2D.parent のみ)。
    resolve_affine: HashMap<u64, Affine2D>,
    /// Group 継承込みの world アフィン。
    world_affine: HashMap<u64, Affine2D>,
    visiting: HashSet<u64>,
}

impl<'a> ResolveCtx<'a> {
    fn xform(&self, id: LayerId) -> Result<&'a Transform2D, ParamEvalError> {
        find_envelope(self.doc, id)
            .map(|e| &e.transform)
            .ok_or(ParamEvalError::DanglingParent { parent: id.get() })
    }

    fn ensure_world_pos(&mut self, id: LayerId) -> Result<[f64; 2], ParamEvalError> {
        if let Some(p) = self.resolved.position(id) {
            return Ok(p);
        }
        self.ensure_resolve_affine(id)?;
        self.resolved
            .position(id)
            .ok_or(ParamEvalError::UnresolvedLookAt(id.get()))
    }

    fn ensure_world_affine(&mut self, id: LayerId) -> Result<Affine2D, ParamEvalError> {
        if let Some(m) = self.world_affine.get(&id.get()).copied() {
            return Ok(m);
        }
        let group_m = match self.group_of.get(&id.get()).copied() {
            Some(g) => self.ensure_world_affine(g)?,
            None => Affine2D::IDENTITY,
        };
        let local_chain = self.ensure_resolve_affine(id)?;
        let world = compose_transform(group_m, local_chain);
        self.world_affine.insert(id.get(), world);
        Ok(world)
    }

    fn ensure_resolve_affine(&mut self, id: LayerId) -> Result<Affine2D, ParamEvalError> {
        if let Some(m) = self.resolve_affine.get(&id.get()).copied() {
            return Ok(m);
        }
        let raw = id.get();
        if !self.visiting.insert(raw) {
            return Err(ParamEvalError::SpatialLinkCycle { layer: raw });
        }

        let result = self.resolve_affine_uncached(id);
        self.visiting.remove(&raw);
        let resolve_m = result?;
        self.resolve_affine.insert(raw, resolve_m);
        Ok(resolve_m)
    }

    fn resolve_affine_uncached(&mut self, id: LayerId) -> Result<Affine2D, ParamEvalError> {
        let xform = self.xform(id)?.clone();
        let parent_m = match xform.parent {
            Some(p) => self.ensure_resolve_affine(p)?,
            None => Affine2D::IDENTITY,
        };
        let group_m = match self.group_of.get(&id.get()).copied() {
            Some(g) => self.ensure_world_affine(g)?,
            None => Affine2D::IDENTITY,
        };
        let placement_space = compose_transform(group_m, parent_m);

        let (local_pos, world_pos) = self.eval_placement(&xform.position, placement_space, id)?;
        self.resolved.insert_position(id, world_pos);

        let anchor = eval_vec2(&xform.anchor, self.t, self.tracks, &self.resolved)?;
        let scale = eval_vec2(&xform.scale, self.t, self.tracks, &self.resolved)?;
        let rotation = self.eval_rotation_world(&xform.rotation, world_pos)?;
        let local = compose_local(local_pos, anchor, scale, rotation);
        Ok(compose_transform(parent_m, local))
    }

    fn eval_placement(
        &mut self,
        position: &DocParam,
        placement_space: Affine2D,
        self_id: LayerId,
    ) -> Result<([f64; 2], [f64; 2]), ParamEvalError> {
        match position {
            DocParam::Follow { target, offset } => {
                let tw = self.ensure_world_pos(*target)?;
                let world = [tw[0] + offset[0], tw[1] + offset[1]];
                let local = match placement_space.try_invert() {
                    Some(inv) => inv.transform_point(world[0], world[1]),
                    None => {
                        return Err(ParamEvalError::SingularPlacementSpace {
                            layer: self_id.get(),
                        })
                    }
                };
                Ok((local, world))
            }
            other => {
                let local = eval_vec2(other, self.t, self.tracks, &self.resolved)?;
                let world = placement_space.transform_point(local[0], local[1]);
                Ok((local, world))
            }
        }
    }

    fn eval_rotation_world(
        &mut self,
        param: &DocParam,
        self_world: [f64; 2],
    ) -> Result<f64, ParamEvalError> {
        match param {
            DocParam::LookAt { target, axis } => {
                let _ = self.ensure_world_pos(*target)?;
                eval_look_at_rotation(self_world, *target, *axis, &self.resolved)
            }
            other => eval_f64(other, self.t, self.tracks, &self.resolved),
        }
    }
}

fn collect_layers(
    items: &[TrackItem],
    group: Option<LayerId>,
    group_of: &mut HashMap<u64, LayerId>,
    out: &mut Vec<LayerId>,
) {
    for item in items {
        let id = match item {
            TrackItem::Clip(c) => c.envelope.layer_id,
            TrackItem::Group(g) => g.envelope.layer_id,
        };
        if let Some(g) = group {
            group_of.insert(id.get(), g);
        }
        out.push(id);
        if let TrackItem::Group(g) = item {
            collect_layers(&g.children, Some(id), group_of, out);
        }
    }
}
