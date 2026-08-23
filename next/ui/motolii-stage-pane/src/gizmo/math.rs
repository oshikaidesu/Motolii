use glam::Vec2;

use motolii_core::LayerPlacement;

use super::*;

/// move: 親空間での cursor 変位をそのまま position へ足す(position は親空間の値)。
pub fn move_value(start: &GizmoTarget, start_cursor_parent: Vec2, cursor_parent: Vec2) -> [f64; 2] {
    let delta = cursor_parent - start_cursor_parent;
    [
        (start.position[0] + delta.x) as f64,
        (start.position[1] + delta.y) as f64,
    ]
}

/// scale: anchor 不動(position 不変)で、掴んだハンドルが cursor に来る scale を
/// 閉じた式で解く。
///
/// **導出**: 局所→親の写像は `M(S) = T(pos)·R·K·S·T(-anchor)`
/// ([`LayerPlacement::from_transform`] の適用順)。ハンドルのローカル点 `h` が
/// cursor(親空間)に来る条件は `S'·(h-a) = (R·K)⁻¹·(cursor - pos)` — 右辺 `q` を
/// 出せば各軸独立に `s_i = q_i / (h_i - a_i)`。`R·K` は
/// `from_transform(anchor=0, pos=0, scale=1, rotation, skew, skew_axis)` で正本から
/// そのまま組む(skew の式をここへ複製しない)。分母が 0(anchor がハンドルの
/// 線上)の軸と、辺ハンドルの動かさない軸は開始時の値を保つ。負の解は許す
/// (AE と同じ: ハンドルを反対側へ引き抜くと反転)。
///
/// Shift(比率固定、map 680): 変化率 `f_i = s_i' / s_i(開始)` の**変化が大きい方**
/// を両軸へ適用する(辺ハンドルは解けた1軸の f を両軸へ)。開始 scale が 0 の軸は
/// 率が立たないので相手側の f を使う。
pub fn scale_value(
    start: &GizmoTarget,
    handle: ScaleHandle,
    cursor_parent: Vec2,
    shift: bool,
) -> [f64; 2] {
    let rot_skew = LayerPlacement::from_transform(
        [0.0, 0.0],
        [0.0, 0.0],
        [1.0, 1.0],
        start.rotation_degrees,
        start.skew_degrees,
        start.skew_axis_degrees,
    );
    // SAFE-INVERSE: 回転+skew(shear、scale は固定で [1,1])は det=1 で常に可逆
    // (回転行列・shear 行列は共に det=1 — shear は `1*1 - 0*tan(skew)` で
    // tan(skew) がどんな有限値でも打ち消える)。`checked_inverse` を経由しない
    // 唯一の生 `.inverse()`(`tests/inverse_fence.rs` がこの注記を要求する)。
    let q = rot_skew
        .inverse()
        .transform_vector2(cursor_parent - Vec2::new(start.position[0], start.position[1]));

    let h = handle.local_point(start.size);
    let denom = [h[0] - start.anchor[0], h[1] - start.anchor[1]];
    let (affects_x, affects_y) = handle.affects();

    let solve = |affects: bool, q_i: f32, denom_i: f32| -> Option<f32> {
        (affects && denom_i.abs() > SOLVE_EPS).then(|| q_i / denom_i)
    };
    let solved_x = solve(affects_x, q.x, denom[0]);
    let solved_y = solve(affects_y, q.y, denom[1]);

    if !shift {
        return [
            solved_x.unwrap_or(start.scale[0]) as f64,
            solved_y.unwrap_or(start.scale[1]) as f64,
        ];
    }

    let factor_of = |solved: Option<f32>, start_scale: f32| -> Option<f32> {
        let solved = solved?;
        (start_scale.abs() > SOLVE_EPS).then(|| solved / start_scale)
    };
    let fx = factor_of(solved_x, start.scale[0]);
    let fy = factor_of(solved_y, start.scale[1]);
    let factor = match (fx, fy) {
        (Some(fx), Some(fy)) => {
            if (fx - 1.0).abs() >= (fy - 1.0).abs() {
                fx
            } else {
                fy
            }
        }
        (Some(f), None) | (None, Some(f)) => f,
        (None, None) => 1.0,
    };
    [
        (start.scale[0] * factor) as f64,
        (start.scale[1] * factor) as f64,
    ]
}

/// rotate: anchor(親空間では position そのもの — `M(anchor) = position`)を
/// 中心に、開始 cursor と今の cursor の親空間での角度差を開始 rotation へ足す。
/// 親空間で測るので観測カメラのズーム/パン/レンダリングカメラの roll に依らない。
/// y-down 座標では `atan2` の増加方向=時計回り = store の rotation の符号と一致。
///
/// Shift(map 679): 結果の絶対角を最寄りの 15° 刻みへスナップ。
pub fn rotation_value(
    start: &GizmoTarget,
    start_cursor_parent: Vec2,
    cursor_parent: Vec2,
    shift: bool,
) -> f64 {
    let pivot = Vec2::new(start.position[0], start.position[1]);
    let v0 = start_cursor_parent - pivot;
    let v1 = cursor_parent - pivot;
    let mut degrees = start.rotation_degrees as f64;
    if v0.length_squared() > SOLVE_EPS * SOLVE_EPS && v1.length_squared() > SOLVE_EPS * SOLVE_EPS {
        let mut delta = (v1.y.atan2(v1.x) - v0.y.atan2(v0.x)).to_degrees() as f64;
        // atan2 の分枝跨ぎ((-360,360) に散る)を1回転内へ畳む。
        if delta > 180.0 {
            delta -= 360.0;
        } else if delta < -180.0 {
            delta += 360.0;
        }
        degrees += delta;
    }
    if shift {
        degrees = (degrees / 15.0).round() * 15.0;
    }
    degrees
}

/// anchor drag(第2切片、AE pan-behind 型): anchor を cursor の下へ移し、
/// **見た目は不動**になるよう position を補償する。
///
/// **導出**: 局所→親の写像は `M = T(pos)·R·K·S·T(-a)`
/// ([`LayerPlacement::from_transform`] の適用順)。任意の局所点 `p` の像
/// `M(p) = pos + RKS(p - a)` を全 `p` で不変に保ったまま `a0 → a1` へ変えると
/// `pos1 = pos0 + RKS·(a1 - a0)` が唯一解。新しい anchor は「cursor の真下の
/// 局所点」 `a1 = M0⁻¹(cursor_parent)` — このとき anchor の親空間の像は
/// `M1(a1) = pos1 = pos0 + RKS(a1 - a0) = M0(a1) = cursor_parent`、つまり
/// **⊕ が cursor に吸い付き、絵は1px も動かない**(AE の pan-behind と同じ
/// 不変量)。`RKS` は正本 [`LayerPlacement::from_transform`]
/// (anchor=0, pos=0)で組む — 行列をここへ複製しない。
///
/// `M0` が退化(scale 0)して逆行列が立たないなら開始時の値をそのまま返す
/// (解けない drag は値を動かさない — [`GizmoDragState::begin`] の
/// 「掴めない物は掴めないまま」と同じ判断)。修飾キーは第1弾では持たない
/// (AE の pan-behind にも Shift の標準挙動は無い)。
pub fn anchor_value(start: &GizmoTarget, cursor_parent: Vec2) -> ([f64; 2], [f64; 2]) {
    let unchanged = (
        [start.anchor[0] as f64, start.anchor[1] as f64],
        [start.position[0] as f64, start.position[1] as f64],
    );
    let placement = LayerPlacement::from_transform(
        start.anchor,
        start.position,
        start.scale,
        start.rotation_degrees,
        start.skew_degrees,
        start.skew_axis_degrees,
    );
    // 退化(scale 0 等)なら inverse を呼ばずに開始時の値を返す
    // ([`checked_inverse`] doc 参照: glam の `Mat2::inverse()` は結果を返す前に
    // 自己アサートするため、「呼んでから `is_finite()` で後始末する」は書けない)。
    let Some(local_from_parent) = checked_inverse(placement) else {
        return unchanged;
    };
    let new_anchor = local_from_parent.transform_point2(cursor_parent);
    let rks = LayerPlacement::from_transform(
        [0.0, 0.0],
        [0.0, 0.0],
        start.scale,
        start.rotation_degrees,
        start.skew_degrees,
        start.skew_axis_degrees,
    );
    let compensation =
        rks.transform_vector2(new_anchor - Vec2::new(start.anchor[0], start.anchor[1]));
    (
        [new_anchor.x as f64, new_anchor.y as f64],
        [
            (start.position[0] + compensation.x) as f64,
            (start.position[1] + compensation.y) as f64,
        ],
    )
}

