//! M4-K0: private test-only spatial region contract spike (RoD/RoI).

use motolii_core::Quality;
use motolii_render::{RenderStep, TextureId};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Aabb {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SpatialExtent {
    Finite(Aabb),
    Infinite,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Trust {
    Conformed,
    Unverified,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Declaration {
    Source { extent: SpatialExtent },
    Expand { radius: f64 },
    Affine { canonical_inverse: [f64; 6] },
    Union,
    Opaque,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct NodeDecl {
    decl: Declaration,
    trust: Trust,
}

#[derive(Debug, Clone, PartialEq)]
enum NodeStep {
    Real(RenderStep),
    Blur {
        input: TextureId,
        output: TextureId,
        radius_texels: u32,
    },
    Passthrough {
        input: TextureId,
        output: TextureId,
    },
}

#[derive(Debug, Clone, PartialEq)]
struct PlanNode {
    step: NodeStep,
    decl: NodeDecl,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct RequestedInputRegion {
    input: TextureId,
    region: SpatialExtent,
}

#[derive(Debug, Clone, PartialEq)]
struct RegionPlan {
    adopted: SpatialExtent,
    per_node: Vec<(TextureId, Vec<RequestedInputRegion>)>,
}

#[derive(Debug, Clone, PartialEq)]
struct Canvas {
    width: u32,
    height: u32,
    aspect: f64,
    samples: Vec<[f32; 4]>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum DeclarationVerdict {
    Accepted,
    Rejected { first_differing_index: usize },
}

fn apply_affine(m: [f64; 6], x: f64, y: f64) -> (f64, f64) {
    (m[0] * x + m[1] * y + m[2], m[3] * x + m[4] * y + m[5])
}

fn invert_affine(m: [f64; 6]) -> Option<[f64; 6]> {
    let det = m[0] * m[4] - m[1] * m[3];
    if det == 0.0 {
        return None;
    }
    let inv_det = 1.0 / det;
    let a = m[0];
    let b = m[1];
    let c = m[2];
    let d = m[3];
    let e = m[4];
    let f = m[5];
    Some([
        e * inv_det,
        -b * inv_det,
        (b * f - e * c) * inv_det,
        -d * inv_det,
        a * inv_det,
        (d * c - a * f) * inv_det,
    ])
}

fn hull_from_corners(points: &[(f64, f64)]) -> Aabb {
    let mut min_x = points[0].0;
    let mut min_y = points[0].1;
    let mut max_x = points[0].0;
    let mut max_y = points[0].1;
    for &(x, y) in &points[1..] {
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    Aabb {
        min_x,
        min_y,
        max_x,
        max_y,
    }
}

fn hull_of_finite(extents: &[SpatialExtent]) -> SpatialExtent {
    let mut hull: Option<Aabb> = None;
    for ext in extents {
        if let SpatialExtent::Finite(a) = ext {
            hull = Some(match hull {
                None => *a,
                Some(h) => Aabb {
                    min_x: h.min_x.min(a.min_x),
                    min_y: h.min_y.min(a.min_y),
                    max_x: h.max_x.max(a.max_x),
                    max_y: h.max_y.max(a.max_y),
                },
            });
        }
    }
    hull.map(SpatialExtent::Finite)
        .unwrap_or(SpatialExtent::Unknown)
}

fn expand_extent(ext: SpatialExtent, radius: f64) -> SpatialExtent {
    match ext {
        SpatialExtent::Finite(a) => SpatialExtent::Finite(Aabb {
            min_x: a.min_x - radius,
            min_y: a.min_y - radius,
            max_x: a.max_x + radius,
            max_y: a.max_y + radius,
        }),
        SpatialExtent::Infinite => SpatialExtent::Infinite,
        SpatialExtent::Unknown => SpatialExtent::Unknown,
    }
}

fn expand_finite_region(ext: SpatialExtent, radius: f64) -> SpatialExtent {
    expand_extent(ext, radius)
}

fn affine_forward_extent(inv: [f64; 6], input: SpatialExtent) -> SpatialExtent {
    let forward = match invert_affine(inv) {
        Some(f) => f,
        None => return SpatialExtent::Unknown,
    };
    match input {
        SpatialExtent::Finite(a) => {
            let corners = [
                (a.min_x, a.min_y),
                (a.max_x, a.min_y),
                (a.min_x, a.max_y),
                (a.max_x, a.max_y),
            ];
            let mapped: Vec<(f64, f64)> = corners
                .iter()
                .map(|&(x, y)| apply_affine(forward, x, y))
                .collect();
            SpatialExtent::Finite(hull_from_corners(&mapped))
        }
        SpatialExtent::Infinite => SpatialExtent::Infinite,
        SpatialExtent::Unknown => SpatialExtent::Unknown,
    }
}

fn affine_backmap_region(inv: [f64; 6], requested: SpatialExtent) -> SpatialExtent {
    match requested {
        SpatialExtent::Finite(a) => {
            let corners = [
                (a.min_x, a.min_y),
                (a.max_x, a.min_y),
                (a.min_x, a.max_y),
                (a.max_x, a.max_y),
            ];
            let mapped: Vec<(f64, f64)> = corners
                .iter()
                .map(|&(x, y)| apply_affine(inv, x, y))
                .collect();
            SpatialExtent::Finite(hull_from_corners(&mapped))
        }
        other => other,
    }
}

fn output_extent(decl: &NodeDecl, inputs: &[SpatialExtent]) -> SpatialExtent {
    if decl.trust == Trust::Unverified {
        return SpatialExtent::Unknown;
    }
    match decl.decl {
        Declaration::Opaque => SpatialExtent::Unknown,
        Declaration::Union => {
            for ext in inputs {
                if *ext == SpatialExtent::Unknown {
                    return SpatialExtent::Unknown;
                }
            }
            for ext in inputs {
                if *ext == SpatialExtent::Infinite {
                    return SpatialExtent::Infinite;
                }
            }
            hull_of_finite(inputs)
        }
        Declaration::Source { extent } => extent,
        Declaration::Expand { radius } => {
            let input = inputs.first().copied().unwrap_or(SpatialExtent::Unknown);
            expand_extent(input, radius)
        }
        Declaration::Affine { canonical_inverse } => {
            let input = inputs.first().copied().unwrap_or(SpatialExtent::Unknown);
            affine_forward_extent(canonical_inverse, input)
        }
    }
}

fn input_regions(
    decl: &NodeDecl,
    requested_output: SpatialExtent,
    inputs: &[TextureId],
) -> Vec<RequestedInputRegion> {
    if decl.trust == Trust::Unverified {
        return inputs
            .iter()
            .map(|&input| RequestedInputRegion {
                input,
                region: SpatialExtent::Unknown,
            })
            .collect();
    }
    match decl.decl {
        Declaration::Opaque => inputs
            .iter()
            .map(|&input| RequestedInputRegion {
                input,
                region: SpatialExtent::Unknown,
            })
            .collect(),
        Declaration::Expand { radius } => inputs
            .iter()
            .map(|&input| RequestedInputRegion {
                input,
                region: expand_finite_region(requested_output, radius),
            })
            .collect(),
        Declaration::Affine { canonical_inverse } => inputs
            .iter()
            .map(|&input| RequestedInputRegion {
                input,
                region: affine_backmap_region(canonical_inverse, requested_output),
            })
            .collect(),
        Declaration::Union | Declaration::Source { .. } => inputs
            .iter()
            .map(|&input| RequestedInputRegion {
                input,
                region: requested_output,
            })
            .collect(),
    }
}

fn adopt_extent(node_out: SpatialExtent, requested: SpatialExtent) -> SpatialExtent {
    if node_out == SpatialExtent::Unknown || requested == SpatialExtent::Unknown {
        return SpatialExtent::Unknown;
    }
    match (node_out, requested) {
        (SpatialExtent::Infinite, SpatialExtent::Finite(f)) => SpatialExtent::Finite(f),
        (SpatialExtent::Infinite, SpatialExtent::Infinite) => SpatialExtent::Infinite,
        (SpatialExtent::Finite(a), SpatialExtent::Infinite) => SpatialExtent::Finite(a),
        (SpatialExtent::Finite(a), SpatialExtent::Finite(b)) => SpatialExtent::Finite(Aabb {
            min_x: a.min_x.max(b.min_x),
            min_y: a.min_y.max(b.min_y),
            max_x: a.max_x.min(b.max_x),
            max_y: a.max_y.min(b.max_y),
        }),
        _ => SpatialExtent::Unknown,
    }
}

fn propagation_region(required: SpatialExtent, host_requested: SpatialExtent) -> SpatialExtent {
    if required == SpatialExtent::Unknown {
        if let SpatialExtent::Finite(_) = host_requested {
            return host_requested;
        }
    }
    required
}

fn node_output_id(step: &NodeStep) -> TextureId {
    match step {
        NodeStep::Real(RenderStep::VideoSource { output }) => *output,
        NodeStep::Real(RenderStep::AffinePlace { output, .. }) => *output,
        NodeStep::Real(RenderStep::CompositeNormal { output, .. }) => *output,
        NodeStep::Blur { output, .. } => *output,
        NodeStep::Passthrough { output, .. } => *output,
        other => panic!("unsupported node step in K0 spike: {other:?}"),
    }
}

fn node_input_ids(step: &NodeStep) -> Vec<TextureId> {
    match step {
        NodeStep::Real(RenderStep::VideoSource { .. }) => Vec::new(),
        NodeStep::Real(RenderStep::AffinePlace { input, .. }) => vec![*input],
        NodeStep::Real(RenderStep::CompositeNormal {
            background,
            foreground,
            ..
        }) => vec![*background, *foreground],
        NodeStep::Blur { input, .. } => vec![*input],
        NodeStep::Passthrough { input, .. } => vec![*input],
        other => panic!("unsupported node step in K0 spike: {other:?}"),
    }
}

fn region_plan(
    nodes: &[PlanNode],
    requested_output: SpatialExtent,
    output: TextureId,
    _quality: Quality,
) -> RegionPlan {
    let mut extent_by_output: Vec<(TextureId, SpatialExtent)> = Vec::new();
    for node in nodes {
        let inputs = node_input_ids(&node.step);
        let input_extents: Vec<SpatialExtent> = inputs
            .iter()
            .map(|id| {
                extent_by_output
                    .iter()
                    .find(|(tid, _)| tid == id)
                    .map(|(_, e)| *e)
                    .unwrap_or(SpatialExtent::Unknown)
            })
            .collect();
        let out_ext = output_extent(&node.decl, &input_extents);
        let out_id = node_output_id(&node.step);
        if let Some(slot) = extent_by_output.iter_mut().find(|(tid, _)| *tid == out_id) {
            slot.1 = out_ext;
        } else {
            extent_by_output.push((out_id, out_ext));
        }
    }

    let node_out = extent_by_output
        .iter()
        .find(|(tid, _)| *tid == output)
        .map(|(_, e)| *e)
        .unwrap_or(SpatialExtent::Unknown);
    let adopted = adopt_extent(node_out, requested_output);

    let mut required: Vec<(TextureId, SpatialExtent)> = vec![(output, adopted)];
    let mut per_node: Vec<(TextureId, Vec<RequestedInputRegion>)> = Vec::new();

    for node in nodes.iter().rev() {
        let out_id = node_output_id(&node.step);
        let required_out = required
            .iter()
            .find(|(tid, _)| *tid == out_id)
            .map(|(_, r)| *r);
        let Some(required_out) = required_out else {
            continue;
        };
        let effective = if matches!(node.decl.decl, Declaration::Union) {
            required_out
        } else {
            propagation_region(required_out, requested_output)
        };
        let input_ids = node_input_ids(&node.step);
        let regions = input_regions(&node.decl, effective, &input_ids);
        for rir in &regions {
            if let Some(slot) = required.iter_mut().find(|(tid, _)| *tid == rir.input) {
                slot.1 = rir.region;
            } else {
                required.push((rir.input, rir.region));
            }
        }
        per_node.push((out_id, regions));
    }
    per_node.reverse();

    RegionPlan { adopted, per_node }
}

fn pixel_to_canonical(x: u32, y: u32, width: u32, height: u32, aspect: f64) -> (f64, f64) {
    let u = (f64::from(x) + 0.5) / f64::from(width);
    let v = (f64::from(y) + 0.5) / f64::from(height);
    (aspect * (u - 0.5), 0.5 - v)
}

fn canonical_to_uv(xc: f64, yc: f64, aspect: f64) -> (f64, f64) {
    (xc / aspect + 0.5, 0.5 - yc)
}

fn in_aabb(x: f64, y: f64, a: Aabb) -> bool {
    x >= a.min_x && x <= a.max_x && y >= a.min_y && y <= a.max_y
}

fn sample_canvas_nearest(canvas: &Canvas, u: f64, v: f64) -> [f32; 4] {
    if !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) {
        return [0.0, 0.0, 0.0, 0.0];
    }
    let w = canvas.width;
    let h = canvas.height;
    if w == 0 || h == 0 {
        return [0.0, 0.0, 0.0, 0.0];
    }
    let x = ((u * f64::from(w)) - 0.5)
        .round()
        .clamp(0.0, f64::from(w - 1)) as u32;
    let y = ((v * f64::from(h)) - 0.5)
        .round()
        .clamp(0.0, f64::from(h - 1)) as u32;
    canvas.samples[(y * w + x) as usize]
}

fn sample_canvas_canonical(canvas: &Canvas, xc: f64, yc: f64) -> [f32; 4] {
    let (u, v) = canonical_to_uv(xc, yc, canvas.aspect);
    sample_canvas_nearest(canvas, u, v)
}

fn composite_normal_premul(bg: [f32; 4], fg: [f32; 4]) -> [f32; 4] {
    let inv_a = 1.0 - fg[3];
    [
        fg[0] + bg[0] * inv_a,
        fg[1] + bg[1] * inv_a,
        fg[2] + bg[2] * inv_a,
        fg[3] + bg[3] * inv_a,
    ]
}

fn blur_canvas(input: &Canvas, radius_texels: u32) -> Canvas {
    let r = radius_texels;
    let w = input.width;
    let h = input.height;
    let mut out = vec![[0.0_f32; 4]; (w * h) as usize];
    let side = 2 * r + 1;
    let denom = (side * side) as f32;
    for y in 0..h {
        for x in 0..w {
            let mut acc = [0.0_f32; 4];
            for ky in 0..side {
                for kx in 0..side {
                    let sx = x.saturating_sub(r).saturating_add(kx).min(w - 1);
                    let sy = y.saturating_sub(r).saturating_add(ky).min(h - 1);
                    let s = input.samples[(sy * w + sx) as usize];
                    acc[0] += s[0];
                    acc[1] += s[1];
                    acc[2] += s[2];
                    acc[3] += s[3];
                }
            }
            out[(y * w + x) as usize] = [
                acc[0] / denom,
                acc[1] / denom,
                acc[2] / denom,
                acc[3] / denom,
            ];
        }
    }
    Canvas {
        width: w,
        height: h,
        aspect: input.aspect,
        samples: out,
    }
}

fn canvas_from_fn(
    width: u32,
    height: u32,
    aspect: f64,
    f: impl Fn(u32, u32) -> [f32; 4],
) -> Canvas {
    let mut samples = Vec::with_capacity((width * height) as usize);
    for y in 0..height {
        for x in 0..width {
            samples.push(f(x, y));
        }
    }
    Canvas {
        width,
        height,
        aspect,
        samples,
    }
}

fn evaluate(
    nodes: &[PlanNode],
    plan: &RegionPlan,
    sources: &[(TextureId, Canvas)],
    width: u32,
    height: u32,
    aspect: f64,
) -> Canvas {
    let adopted_finite = match plan.adopted {
        SpatialExtent::Finite(a) => Some(a),
        _ => None,
    };

    let mut textures: Vec<(TextureId, Canvas)> = sources.to_vec();

    for node in nodes {
        let out_id = node_output_id(&node.step);
        match &node.step {
            NodeStep::Real(RenderStep::VideoSource { .. }) => {
                if textures.iter().all(|(id, _)| *id != out_id) {
                    if let Some((id, c)) = sources.iter().find(|(id, _)| *id == out_id) {
                        textures.push((*id, c.clone()));
                    }
                }
            }
            NodeStep::Real(RenderStep::AffinePlace { input, .. }) => {
                let inv = match node.decl.decl {
                    Declaration::Affine { canonical_inverse } => canonical_inverse,
                    _ => continue,
                };
                let input_canvas = textures
                    .iter()
                    .find(|(id, _)| *id == *input)
                    .map(|(_, c)| c)
                    .expect("affine input");
                let mut out_samples = vec![[0.0_f32; 4]; (width * height) as usize];
                for y in 0..height {
                    for x in 0..width {
                        let (xc, yc) = pixel_to_canonical(x, y, width, height, aspect);
                        if let Some(a) = adopted_finite {
                            if !in_aabb(xc, yc, a) {
                                continue;
                            }
                        }
                        let (sx, sy) = apply_affine(inv, xc, yc);
                        let (u, v) = canonical_to_uv(sx, sy, aspect);
                        if !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) {
                            continue;
                        }
                        out_samples[(y * width + x) as usize] =
                            sample_canvas_canonical(input_canvas, sx, sy);
                    }
                }
                textures.push((
                    out_id,
                    Canvas {
                        width,
                        height,
                        aspect,
                        samples: out_samples,
                    },
                ));
            }
            NodeStep::Real(RenderStep::CompositeNormal {
                background,
                foreground,
                ..
            }) => {
                let bg = textures
                    .iter()
                    .find(|(id, _)| *id == *background)
                    .map(|(_, c)| c)
                    .expect("bg");
                let fg = textures
                    .iter()
                    .find(|(id, _)| *id == *foreground)
                    .map(|(_, c)| c)
                    .expect("fg");
                let mut out_samples = vec![[0.0_f32; 4]; (width * height) as usize];
                for y in 0..height {
                    for x in 0..width {
                        let (xc, yc) = pixel_to_canonical(x, y, width, height, aspect);
                        if let Some(a) = adopted_finite {
                            if !in_aabb(xc, yc, a) {
                                continue;
                            }
                        }
                        let bi = (y * width + x) as usize;
                        let b = bg.samples[bi];
                        let f = fg.samples[bi];
                        out_samples[bi] = composite_normal_premul(b, f);
                    }
                }
                textures.push((
                    out_id,
                    Canvas {
                        width,
                        height,
                        aspect,
                        samples: out_samples,
                    },
                ));
            }
            NodeStep::Blur {
                input,
                radius_texels,
                ..
            } => {
                let input_canvas = textures
                    .iter()
                    .find(|(id, _)| *id == *input)
                    .map(|(_, c)| c)
                    .expect("blur input");
                let mut blurred = blur_canvas(input_canvas, *radius_texels);
                if let Some(a) = adopted_finite {
                    for y in 0..height {
                        for x in 0..width {
                            let (xc, yc) = pixel_to_canonical(x, y, width, height, aspect);
                            if !in_aabb(xc, yc, a) {
                                blurred.samples[(y * width + x) as usize] = [0.0, 0.0, 0.0, 0.0];
                            }
                        }
                    }
                }
                textures.push((out_id, blurred));
            }
            NodeStep::Passthrough { input, .. } => {
                let input_canvas = textures
                    .iter()
                    .find(|(id, _)| *id == *input)
                    .map(|(_, c)| c.clone())
                    .expect("passthrough input");
                textures.push((out_id, input_canvas));
            }
            other => panic!("unsupported step in evaluate: {other:?}"),
        }
    }

    textures
        .into_iter()
        .find(|(id, _)| {
            *id == {
                nodes
                    .last()
                    .map(|n| node_output_id(&n.step))
                    .unwrap_or(TextureId(0))
            }
        })
        .map(|(_, c)| c)
        .unwrap_or_else(|| Canvas {
            width,
            height,
            aspect,
            samples: vec![[0.0; 4]; (width * height) as usize],
        })
}

fn conformance(
    nodes: &[PlanNode],
    declared_plan: &RegionPlan,
    full_plan: &RegionPlan,
    sources: &[(TextureId, Canvas)],
    width: u32,
    height: u32,
    aspect: f64,
) -> DeclarationVerdict {
    let a = evaluate(nodes, declared_plan, sources, width, height, aspect);
    let b = evaluate(nodes, full_plan, sources, width, height, aspect);
    if a.width != b.width || a.height != b.height || a.samples.len() != b.samples.len() {
        return DeclarationVerdict::Rejected {
            first_differing_index: 0,
        };
    }
    for (i, (sa, sb)) in a.samples.iter().zip(b.samples.iter()).enumerate() {
        if sa != sb {
            return DeclarationVerdict::Rejected {
                first_differing_index: i,
            };
        }
    }
    DeclarationVerdict::Accepted
}

#[test]
fn blur_expands_input_region_by_declared_radius() {
    let decl = NodeDecl {
        decl: Declaration::Expand { radius: 0.25 },
        trust: Trust::Conformed,
    };
    let finite = SpatialExtent::Finite(Aabb {
        min_x: -0.5,
        min_y: -0.25,
        max_x: 0.5,
        max_y: 0.25,
    });
    let regions = input_regions(&decl, finite, &[TextureId(0)]);
    assert_eq!(regions.len(), 1);
    let SpatialExtent::Finite(a) = regions[0].region else {
        panic!("expected finite");
    };
    assert_eq!(a.min_x, -0.75);
    assert_eq!(a.min_y, -0.5);
    assert_eq!(a.max_x, 0.75);
    assert_eq!(a.max_y, 0.5);

    let inf = input_regions(&decl, SpatialExtent::Infinite, &[TextureId(0)])[0].region;
    assert_eq!(inf, SpatialExtent::Infinite);

    let unk = input_regions(&decl, SpatialExtent::Unknown, &[TextureId(0)])[0].region;
    assert_eq!(unk, SpatialExtent::Unknown);
}

#[test]
fn affine_input_region_equals_independent_four_corner_oracle() {
    let m = [0.5, 0.25, 0.125, -0.25, 0.5, -0.125];
    let requested = SpatialExtent::Finite(Aabb {
        min_x: -0.5,
        min_y: -0.25,
        max_x: 0.5,
        max_y: 0.25,
    });
    let SpatialExtent::Finite(a) = requested else {
        panic!();
    };
    let corners = [
        (a.min_x, a.min_y),
        (a.max_x, a.min_y),
        (a.min_x, a.max_y),
        (a.max_x, a.max_y),
    ];
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for &(x, y) in &corners {
        let px = m[0] * x + m[1] * y + m[2];
        let py = m[3] * x + m[4] * y + m[5];
        min_x = f64::min(min_x, px);
        min_y = f64::min(min_y, py);
        max_x = f64::max(max_x, px);
        max_y = f64::max(max_y, py);
    }
    let decl = NodeDecl {
        decl: Declaration::Affine {
            canonical_inverse: m,
        },
        trust: Trust::Conformed,
    };
    let got = input_regions(&decl, requested, &[TextureId(1)])[0].region;
    let SpatialExtent::Finite(g) = got else {
        panic!("expected finite");
    };
    assert_eq!(g.min_x, min_x);
    assert_eq!(g.min_y, min_y);
    assert_eq!(g.max_x, max_x);
    assert_eq!(g.max_y, max_y);
}

#[test]
fn infinite_generator_clamps_to_requested_final_region() {
    let requested = SpatialExtent::Finite(Aabb {
        min_x: -0.25,
        min_y: -0.125,
        max_x: 0.25,
        max_y: 0.125,
    });
    let nodes = vec![PlanNode {
        step: NodeStep::Real(RenderStep::VideoSource {
            output: TextureId(1),
        }),
        decl: NodeDecl {
            decl: Declaration::Source {
                extent: SpatialExtent::Infinite,
            },
            trust: Trust::Conformed,
        },
    }];
    let plan = region_plan(&nodes, requested, TextureId(1), Quality::FINAL);
    let SpatialExtent::Finite(a) = plan.adopted else {
        panic!("expected finite adopted");
    };
    let SpatialExtent::Finite(r) = requested else {
        panic!();
    };
    assert_eq!(a.min_x, r.min_x);
    assert_eq!(a.min_y, r.min_y);
    assert_eq!(a.max_x, r.max_x);
    assert_eq!(a.max_y, r.max_y);
}

#[test]
fn unknown_plan_pixels_match_full_domain_evaluation() {
    let tid = TextureId(1);
    let out = TextureId(2);
    let source = canvas_from_fn(4, 4, 1.0, |x, y| {
        if x == 1 && y == 1 {
            [0.5, 0.25, 0.0, 1.0]
        } else {
            [0.0, 0.0, 0.0, 0.0]
        }
    });
    let nodes = vec![
        PlanNode {
            step: NodeStep::Real(RenderStep::VideoSource { output: tid }),
            decl: NodeDecl {
                decl: Declaration::Source {
                    extent: SpatialExtent::Finite(Aabb {
                        min_x: -0.5,
                        min_y: -0.5,
                        max_x: 0.5,
                        max_y: 0.5,
                    }),
                },
                trust: Trust::Conformed,
            },
        },
        PlanNode {
            step: NodeStep::Passthrough {
                input: tid,
                output: out,
            },
            decl: NodeDecl {
                decl: Declaration::Opaque,
                trust: Trust::Conformed,
            },
        },
    ];
    let requested = SpatialExtent::Finite(Aabb {
        min_x: -0.5,
        min_y: -0.5,
        max_x: 0.5,
        max_y: 0.5,
    });
    let unknown_plan = RegionPlan {
        adopted: SpatialExtent::Unknown,
        per_node: vec![],
    };
    let full_plan = region_plan(&nodes, requested, out, Quality::FINAL);
    let u = evaluate(&nodes, &unknown_plan, &[(tid, source.clone())], 4, 4, 1.0);
    let f = evaluate(&nodes, &full_plan, &[(tid, source)], 4, 4, 1.0);
    assert_eq!(u.width, f.width);
    assert_eq!(u.height, f.height);
    assert_eq!(u.samples, f.samples);
}

#[test]
fn understated_finite_declaration_is_rejected_by_conformance() {
    let src = TextureId(0);
    let mid = TextureId(1);
    let out = TextureId(2);
    let source = canvas_from_fn(5, 5, 1.0, |x, y| {
        if x == 2 && y == 2 {
            [1.0, 0.0, 0.0, 1.0]
        } else {
            [0.0, 0.0, 0.0, 0.0]
        }
    });
    let nodes = vec![
        PlanNode {
            step: NodeStep::Real(RenderStep::VideoSource { output: src }),
            decl: NodeDecl {
                decl: Declaration::Source {
                    extent: SpatialExtent::Finite(Aabb {
                        min_x: -0.5,
                        min_y: -0.5,
                        max_x: 0.5,
                        max_y: 0.5,
                    }),
                },
                trust: Trust::Conformed,
            },
        },
        PlanNode {
            step: NodeStep::Blur {
                input: src,
                output: mid,
                radius_texels: 1,
            },
            decl: NodeDecl {
                decl: Declaration::Expand { radius: 0.25 },
                trust: Trust::Conformed,
            },
        },
        PlanNode {
            step: NodeStep::Passthrough {
                input: mid,
                output: out,
            },
            decl: NodeDecl {
                decl: Declaration::Source {
                    extent: SpatialExtent::Finite(Aabb {
                        min_x: -0.125,
                        min_y: -0.125,
                        max_x: 0.125,
                        max_y: 0.125,
                    }),
                },
                trust: Trust::Unverified,
            },
        },
    ];
    let full_plan = RegionPlan {
        adopted: SpatialExtent::Unknown,
        per_node: vec![],
    };
    let small_aabb = Aabb {
        min_x: -0.125,
        min_y: -0.125,
        max_x: 0.125,
        max_y: 0.125,
    };
    let declared_plan = RegionPlan {
        adopted: SpatialExtent::Finite(small_aabb),
        per_node: vec![],
    };
    let verdict = conformance(
        &nodes,
        &declared_plan,
        &full_plan,
        &[(src, source)],
        5,
        5,
        1.0,
    );
    let DeclarationVerdict::Rejected {
        first_differing_index,
    } = verdict
    else {
        panic!("expected rejection");
    };
    let x = (first_differing_index % 5) as u32;
    let y = (first_differing_index / 5) as u32;
    let (xc, yc) = pixel_to_canonical(x, y, 5, 5, 1.0);
    assert!(!in_aabb(xc, yc, small_aabb));
    assert!(first_differing_index < 25);
    assert!(matches!(nodes[2].decl.trust, Trust::Unverified));
}

#[test]
fn unverified_plugin_output_extent_is_unknown() {
    let decl = NodeDecl {
        decl: Declaration::Source {
            extent: SpatialExtent::Finite(Aabb {
                min_x: -0.5,
                min_y: -0.5,
                max_x: 0.5,
                max_y: 0.5,
            }),
        },
        trust: Trust::Unverified,
    };
    assert!(matches!(output_extent(&decl, &[]), SpatialExtent::Unknown));
}

#[test]
fn region_plan_denies_unverified_plugin_its_declared_finite_extent() {
    let out = TextureId(1);
    let node = PlanNode {
        step: NodeStep::Real(RenderStep::VideoSource { output: out }),
        decl: NodeDecl {
            decl: Declaration::Source {
                extent: SpatialExtent::Finite(Aabb {
                    min_x: -0.25,
                    min_y: -0.25,
                    max_x: 0.25,
                    max_y: 0.25,
                }),
            },
            trust: Trust::Unverified,
        },
    };
    let inputs: [SpatialExtent; 0] = [];
    assert_eq!(output_extent(&node.decl, &inputs), SpatialExtent::Unknown);
    let requested = SpatialExtent::Finite(Aabb {
        min_x: -0.5,
        min_y: -0.5,
        max_x: 0.5,
        max_y: 0.5,
    });
    let plan = region_plan(std::slice::from_ref(&node), requested, out, Quality::FINAL);
    assert_eq!(plan.adopted, SpatialExtent::Unknown);

    let conformed = PlanNode {
        step: node.step.clone(),
        decl: NodeDecl {
            trust: Trust::Conformed,
            decl: node.decl.decl,
        },
    };
    let plan2 = region_plan(&[conformed], requested, out, Quality::FINAL);
    let SpatialExtent::Finite(a) = plan2.adopted else {
        panic!("expected finite");
    };
    assert_eq!(a.min_x, -0.25);
    assert_eq!(a.min_y, -0.25);
    assert_eq!(a.max_x, 0.25);
    assert_eq!(a.max_y, 0.25);
}

#[test]
fn extents_are_canonical_and_independent_of_pixel_dimensions() {
    let out = TextureId(2);
    let nodes = vec![
        PlanNode {
            step: NodeStep::Real(RenderStep::VideoSource {
                output: TextureId(0),
            }),
            decl: NodeDecl {
                decl: Declaration::Source {
                    extent: SpatialExtent::Finite(Aabb {
                        min_x: -0.5,
                        min_y: -0.5,
                        max_x: 0.5,
                        max_y: 0.5,
                    }),
                },
                trust: Trust::Conformed,
            },
        },
        PlanNode {
            step: NodeStep::Real(RenderStep::AffinePlace {
                input: TextureId(0),
                output: out,
                inverse_uv: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            }),
            decl: NodeDecl {
                decl: Declaration::Affine {
                    canonical_inverse: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
                },
                trust: Trust::Conformed,
            },
        },
    ];
    let requested = SpatialExtent::Finite(Aabb {
        min_x: -0.25,
        min_y: -0.25,
        max_x: 0.25,
        max_y: 0.25,
    });
    let plan = region_plan(&nodes, requested, out, Quality::FINAL);
    let SpatialExtent::Finite(a) = plan.adopted else {
        panic!();
    };
    let SpatialExtent::Finite(r) = requested else {
        panic!();
    };
    assert_eq!(a.min_x, r.min_x);
    assert_eq!(a.min_y, r.min_y);
    assert_eq!(a.max_x, r.max_x);
    assert_eq!(a.max_y, r.max_y);
    let canvas8 = evaluate(
        &nodes,
        &plan,
        &[(
            TextureId(0),
            canvas_from_fn(8, 8, 1.0, |_, _| [1.0, 1.0, 1.0, 1.0]),
        )],
        8,
        8,
        1.0,
    );
    let canvas32 = evaluate(
        &nodes,
        &plan,
        &[(
            TextureId(0),
            canvas_from_fn(32, 32, 1.0, |_, _| [1.0, 1.0, 1.0, 1.0]),
        )],
        32,
        32,
        1.0,
    );
    assert_eq!(canvas8.width, 8);
    assert_eq!(canvas8.height, 8);
    assert_eq!(canvas32.width, 32);
    assert_eq!(canvas32.height, 32);
    let mut count8 = 0_usize;
    for y in 0..8_u32 {
        for x in 0..8_u32 {
            let (xc, yc) = pixel_to_canonical(x, y, 8, 8, 1.0);
            let sample = canvas8.samples[(y * 8 + x) as usize];
            let in_extent = in_aabb(xc, yc, a);
            if in_extent {
                count8 += 1;
                assert_eq!(sample, [1.0, 1.0, 1.0, 1.0]);
            } else {
                assert_eq!(sample, [0.0, 0.0, 0.0, 0.0]);
            }
        }
    }
    let mut count32 = 0_usize;
    for y in 0..32_u32 {
        for x in 0..32_u32 {
            let (xc, yc) = pixel_to_canonical(x, y, 32, 32, 1.0);
            let sample = canvas32.samples[(y * 32 + x) as usize];
            let in_extent = in_aabb(xc, yc, a);
            if in_extent {
                count32 += 1;
                assert_eq!(sample, [1.0, 1.0, 1.0, 1.0]);
            } else {
                assert_eq!(sample, [0.0, 0.0, 0.0, 0.0]);
            }
        }
    }
    assert_eq!(count8 * 32 * 32, count32 * 8 * 8);
    assert_eq!(count8, 16);
    assert_eq!(count32, 256);
}

#[test]
fn region_plan_is_independent_of_pixel_content() {
    let out = TextureId(1);
    let nodes = vec![PlanNode {
        step: NodeStep::Real(RenderStep::VideoSource { output: out }),
        decl: NodeDecl {
            decl: Declaration::Source {
                extent: SpatialExtent::Finite(Aabb {
                    min_x: -0.5,
                    min_y: -0.5,
                    max_x: 0.5,
                    max_y: 0.5,
                }),
            },
            trust: Trust::Conformed,
        },
    }];
    let requested = SpatialExtent::Finite(Aabb {
        min_x: -0.25,
        min_y: -0.25,
        max_x: 0.25,
        max_y: 0.25,
    });
    let transparent = canvas_from_fn(4, 4, 1.0, |_, _| [0.0, 0.0, 0.0, 0.0]);
    let opaque = canvas_from_fn(4, 4, 1.0, |_, _| [1.0, 1.0, 1.0, 1.0]);
    let p1 = region_plan(&nodes, requested, out, Quality::FINAL);
    let p2 = region_plan(&nodes, requested, out, Quality::FINAL);
    assert_eq!(p1, p2);
    assert_ne!(transparent.samples, opaque.samples);
    let a = evaluate(&nodes, &p1, &[(out, transparent)], 4, 4, 1.0);
    let b = evaluate(&nodes, &p1, &[(out, opaque)], 4, 4, 1.0);
    assert_ne!(a.samples, b.samples);
    assert_eq!(a.width, b.width);
    assert_eq!(a.height, b.height);
}

#[test]
fn draft_and_final_share_one_region_function() {
    let out = TextureId(1);
    let nodes = vec![PlanNode {
        step: NodeStep::Real(RenderStep::VideoSource { output: out }),
        decl: NodeDecl {
            decl: Declaration::Source {
                extent: SpatialExtent::Finite(Aabb {
                    min_x: -0.5,
                    min_y: -0.5,
                    max_x: 0.5,
                    max_y: 0.5,
                }),
            },
            trust: Trust::Conformed,
        },
    }];
    let requested = SpatialExtent::Finite(Aabb {
        min_x: -0.25,
        min_y: -0.125,
        max_x: 0.25,
        max_y: 0.125,
    });
    let draft = region_plan(&nodes, requested, out, Quality::DRAFT);
    let fin = region_plan(&nodes, requested, out, Quality::FINAL);
    assert_eq!(draft, fin);
}

#[test]
fn uv_matrix_is_not_interchangeable_with_canonical_inverse() {
    let canonical_inverse = [0.5, 0.25, 0.125, -0.25, 0.5, -0.125];
    let aspect = 2.0_f64;
    // `Affine2D::to_inverse_uv_matrix` は内部で逆行列を取るため、
    // この式は入力として「canonical逆行列」を受ける。
    let m = canonical_inverse;
    let a = aspect;
    let n0 = m[0] * a;
    let n1 = -m[1];
    let n2 = m[0] * (-0.5 * a) + m[1] * 0.5 + m[2];
    let n3 = m[3] * a;
    let n4 = -m[4];
    let n5 = m[3] * (-0.5 * a) + m[4] * 0.5 + m[5];
    let inv_a = 1.0 / a;
    let inverse_uv = [
        inv_a * n0,
        inv_a * n1,
        inv_a * n2 + 0.5,
        -n3,
        -n4,
        -n5 + 0.5,
    ];
    assert_eq!(inverse_uv, [0.5, -0.125, 0.375, 0.5, 0.5, 0.125]);

    assert_eq!(inverse_uv[0], canonical_inverse[0]);
    assert_ne!(inverse_uv[1], canonical_inverse[1]);
    assert_ne!(inverse_uv[2], canonical_inverse[2]);
    assert_ne!(inverse_uv[3], canonical_inverse[3]);
    assert_eq!(inverse_uv[4], canonical_inverse[4]);
    assert_ne!(inverse_uv[5], canonical_inverse[5]);
    assert_ne!(inverse_uv, canonical_inverse);

    let src = TextureId(0);
    let out = TextureId(1);
    let divergent_uv: [f32; 6] = [
        inverse_uv[0] as f32 + 0.25,
        inverse_uv[1] as f32,
        inverse_uv[2] as f32,
        inverse_uv[3] as f32,
        inverse_uv[4] as f32,
        inverse_uv[5] as f32,
    ];
    let make_nodes = |uv: [f32; 6]| {
        vec![
            PlanNode {
                step: NodeStep::Real(RenderStep::VideoSource { output: src }),
                decl: NodeDecl {
                    decl: Declaration::Source {
                        extent: SpatialExtent::Finite(Aabb {
                            min_x: -0.5,
                            min_y: -0.5,
                            max_x: 0.5,
                            max_y: 0.5,
                        }),
                    },
                    trust: Trust::Conformed,
                },
            },
            PlanNode {
                step: NodeStep::Real(RenderStep::AffinePlace {
                    input: src,
                    output: out,
                    inverse_uv: uv,
                }),
                decl: NodeDecl {
                    decl: Declaration::Affine { canonical_inverse },
                    trust: Trust::Conformed,
                },
            },
        ]
    };
    let requested = SpatialExtent::Finite(Aabb {
        min_x: -0.25,
        min_y: -0.25,
        max_x: 0.25,
        max_y: 0.25,
    });
    let ref_nodes = make_nodes([
        inverse_uv[0] as f32,
        inverse_uv[1] as f32,
        inverse_uv[2] as f32,
        inverse_uv[3] as f32,
        inverse_uv[4] as f32,
        inverse_uv[5] as f32,
    ]);
    let div_nodes = make_nodes(divergent_uv);
    let ref_plan = region_plan(&ref_nodes, requested, out, Quality::FINAL);
    let div_plan = region_plan(&div_nodes, requested, out, Quality::FINAL);
    assert_eq!(ref_plan, div_plan);

    let source = canvas_from_fn(4, 4, aspect, |x, y| {
        [(x as f32 + 1.0) / 8.0, (y as f32 + 1.0) / 8.0, 0.25, 1.0]
    });
    let ref_canvas = evaluate(
        &ref_nodes,
        &ref_plan,
        &[(src, source.clone())],
        4,
        4,
        aspect,
    );
    let div_canvas = evaluate(
        &div_nodes,
        &div_plan,
        &[(src, source.clone())],
        4,
        4,
        aspect,
    );
    assert_eq!(ref_canvas.width, div_canvas.width);
    assert_eq!(ref_canvas.height, div_canvas.height);
    assert_eq!(ref_canvas.samples, div_canvas.samples);

    let wrong_decl_nodes = vec![
        PlanNode {
            step: NodeStep::Real(RenderStep::VideoSource { output: src }),
            decl: NodeDecl {
                decl: Declaration::Source {
                    extent: SpatialExtent::Finite(Aabb {
                        min_x: -0.5,
                        min_y: -0.5,
                        max_x: 0.5,
                        max_y: 0.5,
                    }),
                },
                trust: Trust::Conformed,
            },
        },
        PlanNode {
            step: NodeStep::Real(RenderStep::AffinePlace {
                input: src,
                output: out,
                inverse_uv: [
                    inverse_uv[0] as f32,
                    inverse_uv[1] as f32,
                    inverse_uv[2] as f32,
                    inverse_uv[3] as f32,
                    inverse_uv[4] as f32,
                    inverse_uv[5] as f32,
                ],
            }),
            decl: NodeDecl {
                decl: Declaration::Affine {
                    canonical_inverse: inverse_uv,
                },
                trust: Trust::Conformed,
            },
        },
    ];
    let wrong_plan = region_plan(&wrong_decl_nodes, requested, out, Quality::FINAL);
    let wrong_canvas = evaluate(
        &wrong_decl_nodes,
        &wrong_plan,
        &[(src, source.clone())],
        4,
        4,
        aspect,
    );
    assert_ne!(wrong_plan, ref_plan);
    assert!(ref_canvas
        .samples
        .iter()
        .any(|s| *s != [0.0, 0.0, 0.0, 0.0]));
    assert_ne!(wrong_canvas.samples, ref_canvas.samples);
}

#[test]
fn out_of_range_finite_source_samples_transparent_black() {
    let src = TextureId(0);
    let out = TextureId(1);
    let canonical_inverse = [2.0, 0.0, 0.0, 0.0, 2.0, 0.0];
    let nodes = vec![
        PlanNode {
            step: NodeStep::Real(RenderStep::VideoSource { output: src }),
            decl: NodeDecl {
                decl: Declaration::Source {
                    extent: SpatialExtent::Finite(Aabb {
                        min_x: -0.5,
                        min_y: -0.5,
                        max_x: 0.5,
                        max_y: 0.5,
                    }),
                },
                trust: Trust::Conformed,
            },
        },
        PlanNode {
            step: NodeStep::Real(RenderStep::AffinePlace {
                input: src,
                output: out,
                inverse_uv: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            }),
            decl: NodeDecl {
                decl: Declaration::Affine { canonical_inverse },
                trust: Trust::Conformed,
            },
        },
    ];
    let source = canvas_from_fn(4, 4, 1.0, |x, y| {
        [0.25 + 0.125 * x as f32, 0.25 + 0.125 * y as f32, 0.5, 1.0]
    });
    let plan = RegionPlan {
        adopted: SpatialExtent::Finite(Aabb {
            min_x: -0.5,
            min_y: -0.5,
            max_x: 0.5,
            max_y: 0.5,
        }),
        per_node: vec![],
    };
    let canvas = evaluate(&nodes, &plan, &[(src, source)], 4, 4, 1.0);
    assert_eq!(canvas.width, 4);
    assert_eq!(canvas.height, 4);
    let mut oor_count = 0_usize;
    let mut in_range_count = 0_usize;
    for (i, s) in canvas.samples.iter().enumerate() {
        let x = (i as u32) % 4;
        let y = (i as u32) / 4;
        let (xc, yc) = pixel_to_canonical(x, y, 4, 4, 1.0);
        let (sx, sy) = apply_affine(canonical_inverse, xc, yc);
        let (u, v) = canonical_to_uv(sx, sy, 1.0);
        if !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) {
            oor_count += 1;
            assert_eq!(*s, [0.0, 0.0, 0.0, 0.0]);
        } else {
            in_range_count += 1;
            assert_ne!(*s, [0.0, 0.0, 0.0, 0.0]);
        }
    }
    assert_eq!(oor_count, 12);
    assert_eq!(in_range_count, 4);
}

#[test]
fn union_with_unknown_branch_adopts_literal_unknown() {
    let finite_branch = SpatialExtent::Finite(Aabb {
        min_x: -0.25,
        min_y: -0.25,
        max_x: 0.25,
        max_y: 0.25,
    });
    let t0 = TextureId(0);
    let t1 = TextureId(1);
    let tout = TextureId(2);
    let nodes_unknown = vec![
        PlanNode {
            step: NodeStep::Real(RenderStep::VideoSource { output: t0 }),
            decl: NodeDecl {
                decl: Declaration::Source {
                    extent: finite_branch,
                },
                trust: Trust::Conformed,
            },
        },
        PlanNode {
            step: NodeStep::Real(RenderStep::VideoSource { output: t1 }),
            decl: NodeDecl {
                decl: Declaration::Source {
                    extent: SpatialExtent::Finite(Aabb {
                        min_x: -0.5,
                        min_y: -0.5,
                        max_x: 0.5,
                        max_y: 0.5,
                    }),
                },
                trust: Trust::Unverified,
            },
        },
        PlanNode {
            step: NodeStep::Real(RenderStep::CompositeNormal {
                background: t0,
                foreground: t1,
                output: tout,
            }),
            decl: NodeDecl {
                decl: Declaration::Union,
                trust: Trust::Conformed,
            },
        },
    ];
    let inputs = [
        output_extent(&nodes_unknown[0].decl, &[]),
        output_extent(&nodes_unknown[1].decl, &[]),
    ];
    assert!(matches!(
        output_extent(&nodes_unknown[2].decl, &inputs),
        SpatialExtent::Unknown
    ));
    let requested = SpatialExtent::Finite(Aabb {
        min_x: -0.5,
        min_y: -0.5,
        max_x: 0.5,
        max_y: 0.5,
    });
    let plan = region_plan(&nodes_unknown, requested, tout, Quality::FINAL);
    assert!(matches!(plan.adopted, SpatialExtent::Unknown));
    let union_entry = plan
        .per_node
        .iter()
        .find(|(id, _)| *id == tout)
        .expect("union node");
    for rir in &union_entry.1 {
        assert!(matches!(rir.region, SpatialExtent::Unknown));
    }

    let nodes_finite = vec![
        PlanNode {
            step: NodeStep::Real(RenderStep::VideoSource { output: t0 }),
            decl: NodeDecl {
                decl: Declaration::Source {
                    extent: finite_branch,
                },
                trust: Trust::Conformed,
            },
        },
        PlanNode {
            step: NodeStep::Real(RenderStep::VideoSource { output: t1 }),
            decl: NodeDecl {
                decl: Declaration::Source {
                    extent: SpatialExtent::Finite(Aabb {
                        min_x: -0.125,
                        min_y: -0.125,
                        max_x: 0.125,
                        max_y: 0.125,
                    }),
                },
                trust: Trust::Conformed,
            },
        },
        PlanNode {
            step: NodeStep::Real(RenderStep::CompositeNormal {
                background: t0,
                foreground: t1,
                output: tout,
            }),
            decl: NodeDecl {
                decl: Declaration::Union,
                trust: Trust::Conformed,
            },
        },
    ];
    let plan_f = region_plan(&nodes_finite, requested, tout, Quality::FINAL);
    let SpatialExtent::Finite(h) = plan_f.adopted else {
        panic!("expected finite hull");
    };
    assert_eq!(h.min_x, -0.25);
    assert_eq!(h.min_y, -0.25);
    assert_eq!(h.max_x, 0.25);
    assert_eq!(h.max_y, 0.25);
}

#[test]
fn unknown_declaration_does_not_shrink_adopted_region_below_requested() {
    let src = TextureId(0);
    let out = TextureId(1);
    let nodes = vec![
        PlanNode {
            step: NodeStep::Real(RenderStep::VideoSource { output: src }),
            decl: NodeDecl {
                decl: Declaration::Source {
                    extent: SpatialExtent::Finite(Aabb {
                        min_x: -0.5,
                        min_y: -0.5,
                        max_x: 0.5,
                        max_y: 0.5,
                    }),
                },
                trust: Trust::Conformed,
            },
        },
        PlanNode {
            step: NodeStep::Passthrough {
                input: src,
                output: out,
            },
            decl: NodeDecl {
                decl: Declaration::Source {
                    extent: SpatialExtent::Unknown,
                },
                trust: Trust::Conformed,
            },
        },
    ];
    let requested = SpatialExtent::Finite(Aabb {
        min_x: -0.25,
        min_y: -0.125,
        max_x: 0.25,
        max_y: 0.125,
    });
    let plan = region_plan(&nodes, requested, out, Quality::FINAL);
    assert!(matches!(plan.adopted, SpatialExtent::Unknown));
    let entry = plan
        .per_node
        .iter()
        .find(|(id, _)| *id == out)
        .expect("out");
    let SpatialExtent::Finite(r) = requested else {
        panic!();
    };
    for rir in &entry.1 {
        let SpatialExtent::Finite(a) = rir.region else {
            panic!("region must not shrink to empty/non-finite");
        };
        assert_eq!(a.min_x, r.min_x);
        assert_eq!(a.min_y, r.min_y);
        assert_eq!(a.max_x, r.max_x);
        assert_eq!(a.max_y, r.max_y);
    }
}

#[test]
fn blur_after_affine_composes_expanded_then_backmapped_region() {
    let m = [0.5, 0.0, 0.0, 0.0, 1.0, 0.0];
    let radius = 0.25;
    let requested = SpatialExtent::Finite(Aabb {
        min_x: -0.25,
        min_y: -0.25,
        max_x: 0.25,
        max_y: 0.25,
    });
    let expand = NodeDecl {
        decl: Declaration::Expand { radius },
        trust: Trust::Conformed,
    };
    let affine = NodeDecl {
        decl: Declaration::Affine {
            canonical_inverse: m,
        },
        trust: Trust::Conformed,
    };
    let expanded = input_regions(&expand, requested, &[TextureId(0)])[0].region;
    let composed = input_regions(&affine, expanded, &[TextureId(0)])[0].region;

    let affine_first = input_regions(&affine, requested, &[TextureId(0)])[0].region;
    let reversed = input_regions(&expand, affine_first, &[TextureId(0)])[0].region;
    assert_ne!(composed, reversed);

    let SpatialExtent::Finite(c) = composed else {
        panic!("expected finite composed");
    };
    let SpatialExtent::Finite(e) = expanded else {
        panic!();
    };
    let corners = [
        (e.min_x, e.min_y),
        (e.max_x, e.min_y),
        (e.min_x, e.max_y),
        (e.max_x, e.max_y),
    ];
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for &(x, y) in &corners {
        let px = m[0] * x + m[1] * y + m[2];
        let py = m[3] * x + m[4] * y + m[5];
        min_x = f64::min(min_x, px);
        min_y = f64::min(min_y, py);
        max_x = f64::max(max_x, px);
        max_y = f64::max(max_y, py);
    }
    assert_eq!(c.min_x, min_x);
    assert_eq!(c.min_y, min_y);
    assert_eq!(c.max_x, max_x);
    assert_eq!(c.max_y, max_y);
}
