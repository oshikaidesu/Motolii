#import <types.wgsl>
#import <global_bindings.wgsl>

struct UniformBuffer {
    world_from_obj: mat4x4f,
    gradient_line: vec4f,
    start_color: vec4f,
    end_color: vec4f,
    picking_object_id: vec2u,
    picking_instance_id: vec2u,
    outline_mask: vec2u,
};

@group(1) @binding(0)
var<uniform> ubo: UniformBuffer;

struct VertexIn {
    @location(0) position: vec2f,
};

struct VertexOut {
    @builtin(position) position: vec4f,
    @location(0) object_position: vec2f,
};

@vertex
fn vs_main(input: VertexIn) -> VertexOut {
    var output: VertexOut;
    output.position = frame.projection_from_world * ubo.world_from_obj * vec4f(input.position, 0.0, 1.0);
    output.object_position = input.position;
    return output;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4f {
    let start = ubo.gradient_line.xy;
    let direction = ubo.gradient_line.zw - start;
    let length_squared = dot(direction, direction);
    var t = 0.0;
    if length_squared > 0.0000001 {
        t = clamp(dot(input.object_position - start, direction) / length_squared, 0.0, 1.0);
    }
    return mix(ubo.start_color, ubo.end_color, t);
}

@fragment
fn fs_main_picking_layer() -> @location(0) vec4u {
    return vec4u(ubo.picking_object_id, ubo.picking_instance_id);
}

@fragment
fn fs_main_outline_mask() -> @location(0) vec2u {
    return ubo.outline_mask;
}
