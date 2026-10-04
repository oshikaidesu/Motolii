use super::*;

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
}
