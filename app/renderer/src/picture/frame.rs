//! 並べた結果 — その時刻の配置を 1 回だけ解き、view の手控えとコマをまたぐ覚えに置く。
//! 解くのは flow、覚えるのはここ。値の読み口(`number`・`choice`・`display`)はコア側に残る。

use super::*;

/// その時刻に並べた結果。Display の Group が無ければ空。
pub fn layout_frame(view: &StoreView<'_>, t: RationalTime) -> Result<std::sync::Arc<Frame>, StoreError> {
    if let Some(hit) = view.layout_memo().borrow().frames.get(&t) {
        return Ok(hit.clone());
    }
    if let Some((cache, revision)) = view.shared_layout_cache() {
        let cache = cache.borrow();
        if cache.revision.as_ref() == Some(revision) {
            if let Some(hit) = cache.frames.get(&t) {
                view.layout_memo().borrow_mut().frames.insert(t, hit.clone());
                return Ok(hit.clone());
            }
        }
    }
    // 解いている間に同じ時刻を問われたら(面の Group の親を辿る時など)、空の結果で答えて巡らない。
    view.layout_memo().borrow_mut().frames.insert(t, std::sync::Arc::new(Frame::default()));
    // 解いている途中の内側の時刻は、巡り止めの空の結果を読んでいるかもしれない。コマをまたいで覚えるのは一番外側だけ。
    let outermost = view.layout_memo().borrow_mut().enter();
    let computed = match view.layout_solver() {
        Some(solver) => solver.compute(view, t),
        None => crate::picture::flow::compute_layout(view, t),
    };
    view.layout_memo().borrow_mut().leave();
    let frame = std::sync::Arc::new(computed?);
    view.layout_memo().borrow_mut().frames.insert(t, frame.clone());
    if let Some((cache, revision)) = view.shared_layout_cache().filter(|_| outermost) {
        let mut cache = cache.borrow_mut();
        if cache.revision.as_ref() != Some(revision) || cache.frames.len() >= LayoutCache::LIMIT {
            cache.revision = Some(revision.clone());
            cache.frames.clear();
        }
        cache.frames.insert(t, frame.clone());
    }
    Ok(frame)
}

/// 並ぶ子なら、層の Position と Scale の代わりに使う値。
pub fn laid_out(view: &StoreView<'_>, layer: LayerId, t: RationalTime) -> Result<Option<Slot>, StoreError> {
    let Some(parent) = view.attrs(layer)?.unwrap_or_default().parent else { return Ok(None) };
    if view.display(parent, t)? == 0 {
        return Ok(None);
    }
    let Some(mut slot) = layout_frame(view, t)?.slots.get(&layer).copied() else { return Ok(None) };
    // 移り方: 少し前の行き先を、区間の重みで混ぜる(位置と大きさ)。
    let mut acc = [[0.0f32; 2]; 4];
    let mut total = 0.0f32;
    for (at, weight) in crate::picture::motion_time::transition_samples(view, layer, t)? {
        if let Some(past) = layout_frame(view, at)?.slots.get(&layer) {
            for (sum, value) in acc.iter_mut().zip([past.position, past.scale, past.stretch, past.anchor]) {
                for axis in 0..2 {
                    sum[axis] += value[axis] * weight;
                }
            }
            total += weight;
        }
    }
    if total > 1e-6 {
        [slot.position, slot.scale, slot.stretch, slot.anchor] = acc.map(|sum| sum.map(|v| v / total));
    }
    Ok(Some(slot))
}
