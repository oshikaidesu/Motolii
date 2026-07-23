struct ViewUniform {
    viewport_pan_zoom: vec4<f32>,
    track_origin: vec4<f32>,
}

struct KeyInstance {
    time_seconds: f32,
    track: f32,
    selected: u32,
    _padding: u32,
}

@group(0) @binding(0)
var<uniform> view: ViewUniform;

@group(0) @binding(1)
var<storage, read> keys: array<KeyInstance>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    let corners = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, 1.0),
    );
    let key = keys[instance_index];
    let center = vec2<f32>(
        view.track_origin.y + (key.time_seconds - view.viewport_pan_zoom.z) * view.viewport_pan_zoom.w,
        view.track_origin.z + (key.track + 0.5) * view.track_origin.x,
    );
    let half_size = select(vec2<f32>(2.5, 2.5), vec2<f32>(4.0, 4.0), key.selected == 1u);
    let physical = center + corners[vertex_index] * half_size;
    let clip = vec2<f32>(
        physical.x / view.viewport_pan_zoom.x * 2.0 - 1.0,
        1.0 - physical.y / view.viewport_pan_zoom.y * 2.0,
    );
    var output: VertexOutput;
    output.position = vec4<f32>(clip, 0.0, 1.0);
    output.color = select(
        vec4<f32>(0.72, 0.75, 0.82, 0.88),
        vec4<f32>(0.98, 0.76, 0.25, 1.0),
        key.selected == 1u,
    );
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
