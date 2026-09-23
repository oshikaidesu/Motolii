// What a Block's motion entry means (the frame's `motion` storage, written by the block programs):
// kind 0 a thing, 1 a connector between two things, 2 a rope.

// The frame's `motion` (re_renderer's motion binding) holds four vec4 per object —
// `(offset.xyz, turn)`, `(centre.xyz, scale)`, `(axis.xyz, kind)`, `(tint.rgb, opacity)`. Kind 0 is a thing;
// kind 1 is a connector between two things (see `motion_offset`). The object whose
// last param is `n` reads entry `n - 1`; 0 = not moved. A vertex is scaled by `scale` and turned by
// `turn` radians about `axis` through `centre`, then offset. A fragment is multiplied by `tint` and
// `opacity` (premultiplied, so opacity scales colour and coverage alike).

const MOTION_STRIDE = 4u;

fn motion_entry(slot: f32) -> u32 {
    let n = u32(max(slot, 0.0) + 0.5);
    if n == 0u || n * MOTION_STRIDE > arrayLength(&motion) {
        return 0u;
    }
    return n;
}

fn cubic(p0: vec3f, p1: vec3f, p2: vec3f, p3: vec3f, t: f32) -> vec3f {
    let u = 1.0 - t;
    return u * u * u * p0 + 3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t * p3;
}

fn motion_offset(slot: f32, world_position: vec3f) -> vec3f {
    let n = motion_entry(slot);
    if n == 0u {
        return vec3f(0.0);
    }
    let base = (n - 1u) * MOTION_STRIDE;
    let move_turn = motion[base];
    let centre_scale = motion[base + 1u];
    let axis_kind = motion[base + 2u];
    // A connector entry (kind 1): `(offset at A, _)`, `(A, _)`, `(B - A, 1)`, `(offset at B, _)`.
    // Each vertex takes the offset of the end it is nearer to, blended along A→B, so a line between two
    // moved things keeps touching both.
    if axis_kind.w == 1.0 {
        let ab = axis_kind.xyz;
        let t = clamp(dot(world_position - centre_scale.xyz, ab) / max(dot(ab, ab), 1e-6), 0.0, 1.0);
        return mix(move_turn.xyz, motion[base + 3u].xyz, t);
    }
    // A rope entry (kind 2) spans two entries: the four above, then `(c0 drawn)`, `(c1 drawn)`, `(c0 now)`, `(c1 now)`.
    // The picture is a cubic through A, c0, c1, B; the vertex moves by the difference between that cubic
    // with the live ends and control points, and the cubic that was drawn.
    if axis_kind.w == 2.0 && (n + 1u) * MOTION_STRIDE <= arrayLength(&motion) {
        let ab = axis_kind.xyz;
        let a = centre_scale.xyz;
        let b = a + ab;
        let t = clamp(dot(world_position - a, ab) / max(dot(ab, ab), 1e-6), 0.0, 1.0);
        let drawn = cubic(a, motion[base + 4u].xyz, motion[base + 5u].xyz, b, t);
        let live = cubic(a + move_turn.xyz, motion[base + 6u].xyz, motion[base + 7u].xyz, b + motion[base + 3u].xyz, t);
        return live - drawn;
    }
    let scale = select(centre_scale.w, 1.0, centre_scale.w == 0.0);
    var placed = vec3f(0.0);
    if move_turn.w != 0.0 || scale != 1.0 {
        let r = (world_position - centre_scale.xyz) * scale;
        let axis = axis_kind.xyz;
        let c = cos(move_turn.w);
        let s = sin(move_turn.w);
        placed = r * c + cross(axis, r) * s + axis * dot(axis, r) * (1.0 - c) - (world_position - centre_scale.xyz);
    }
    return move_turn.xyz + placed;
}

/// `(tint.rgb, opacity)` of the object, `vec4f(1.0)` when it carries no motion entry.
fn motion_tint(slot: f32) -> vec4f {
    let n = motion_entry(slot);
    if n == 0u || motion[(n - 1u) * MOTION_STRIDE + 2u].w != 0.0 {
        return vec4f(1.0);
    }
    return motion[(n - 1u) * MOTION_STRIDE + 3u];
}
