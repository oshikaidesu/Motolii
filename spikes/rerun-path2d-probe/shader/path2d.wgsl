#import <types.wgsl>
#import <global_bindings.wgsl>

struct UniformBuffer {
    world_from_obj: mat4x4f,
    color: vec4f,
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
};

@vertex
fn vs_main(input: VertexIn) -> VertexOut {
    var output: VertexOut;
    output.position = frame.projection_from_world * ubo.world_from_obj * vec4f(input.position, 0.0, 1.0);
    return output;
}

@fragment
fn fs_main() -> @location(0) vec4f {
    return ubo.color;
}

@fragment
fn fs_main_picking_layer() -> @location(0) vec4u {
    return vec4u(ubo.picking_object_id, ubo.picking_instance_id);
}

@fragment
fn fs_main_outline_mask() -> @location(0) vec2u {
    return ubo.outline_mask;
}
