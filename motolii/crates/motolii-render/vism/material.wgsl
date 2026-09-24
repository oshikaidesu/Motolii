// Motolii's standard material: split-sum environment light, the sun's light cookie,
// transmission that refracts the backdrop (Standard Glass), and the Views a Vism asked for.
// Appended to a surface program's surface hook; reads the frame's resources (re_renderer's bindings).

// The view's program constants as Motolii lays them out (compositor/light.rs).
/// xyz: world direction toward the sun; w: its share of the diffuse light (0 = no sun, no shadow).
fn sun_direction() -> vec4f { return frame.program_constants[0]; }
/// rgb: the sun's tint; w = 1 while the light cookie is captured (surfaces write what they let through).
fn sun_color() -> vec4f { return frame.program_constants[1]; }
/// World → light cookie uv (orthographic, looking along the sun).
fn light_uv_from_world() -> mat4x4f {
    return mat4x4f(frame.program_constants[2], frame.program_constants[3], frame.program_constants[4], frame.program_constants[5]);
}

/// The Views this surface's Vism asked for (`VIEWS`, in their order), side by side in one picture;
/// 0 when it asked for none or the host drew none.
fn view_count() -> u32 { return u32(frame.program_constants[6].x); }
/// The coarsest mip level the host generated for the Views (`VIEW_BLUR`); reads are clamped to it.
fn view_lod_limit() -> f32 { return frame.program_constants[6].y; }
/// The lod a roughness (0..1) reads the Views at: the backdrop's mapping (`transmitted_backdrop`,
/// mirrored by the host's `backdrop_levels_read`), so a Vism declaring `VIEW_BLUR` reads no
/// level the host did not generate.
fn view_lod(roughness: f32) -> f32 {
    let levels = f32(textureNumLevels(view_capture_texture));
    return pow(clamp(roughness, 0.0, 1.0), 0.8) * max(levels - 1.0, 0.0) * 0.55;
}
/// Where they were taken from: the centre of the requesting layer (world).
fn view_origin() -> vec3f { return frame.program_constants[7].xyz; }
/// View `i` at `uv` (0..1 across that View, +y down), premultiplied: alpha is what the View saw
/// there. `lod` blurs it by mip level, no coarser than the host generated.
fn view_sample(i: u32, uv: vec2f, lod_asked: f32) -> vec4f {
    let lod = min(lod_asked, view_lod_limit());
    let n = f32(max(view_count(), 1u));
    let size = vec2f(textureDimensions(view_capture_texture));
    let margin = min(0.49, exp2(lod) * n / size.x);
    let local = clamp(uv, vec2f(margin), vec2f(1.0 - margin));
    return textureSampleLevel(view_capture_texture, screen_sampler, vec2f((f32(i) + local.x) / n, local.y), lod);
}

/// Split-sum environment BRDF, analytic fit (Karis 2014, "Physically Based Shading on Mobile").
fn env_brdf_approx(f0: vec3f, roughness: f32, n_dot_v: f32) -> vec3f {
    let c0 = vec4f(-1.0, -0.0275, -0.572, 0.022);
    let c1 = vec4f(1.0, 0.0425, 1.04, -0.04);
    let r = roughness * c0 + c1;
    let a004 = min(r.x * r.x, exp2(-9.28 * n_dot_v)) * r.x + r.y;
    let ab = vec2f(-1.04, 1.04) * a004 + r.zw;
    return f0 * ab.x + ab.y;
}

/// Light from the sun that reaches `position`: 1 where nothing blocks it, the blocker's tint × coverage
/// where the cookie says something does. No depth: a blocker shades everything along its ray.
fn sun_light_through(position: vec3f) -> vec3f {
    if sun_direction().w <= 0.0 {
        return vec3f(1.0);
    }
    let uv = (light_uv_from_world() * vec4f(position, 1.0)).xy;
    if any(uv < vec2f(0.0)) || any(uv > vec2f(1.0)) {
        return vec3f(1.0);
    }
    let cookie = textureSampleLevel(coverage_texture, screen_sampler, uv, 0.0);
    return vec3f(1.0 - cookie.a) + cookie.rgb;
}

/// Shadow factor: only the sun's share of the light is taken away, scaled by how much the surface
/// faces the sun. `facing_floor` keeps a flat, unlit picture readable as a receiver.
fn sun_shade(position: vec3f, normal: vec3f, facing_floor: f32) -> vec3f {
    let facing = max(clamp(dot(normal, sun_direction().xyz), 0.0, 1.0), facing_floor);
    let share = sun_direction().w * facing;
    return vec3f(1.0) - share * (vec3f(1.0) - sun_light_through(position));
}

/// What is drawn behind the surface, seen where a ray refracted at `ior` leaves a slab of `thickness`;
/// the environment where nothing is drawn.
fn transmitted_backdrop(view_dir: vec3f, normal: vec3f, reflected: vec3f, world_position: vec3f, thickness: f32, ior: f32, roughness: f32, lod: f32) -> vec3f {
    let refracted = refract(-view_dir, normal, 1.0 / ior);
    let through = select(reflected, refracted, any(refracted != vec3f(0.0)));
    let uv = clamp(projected_surface_uv(world_position + through * thickness), vec2f(0.0), vec2f(1.0));
    let behind = textureSampleLevel(backdrop_texture, screen_sampler, uv, lod);
    return behind.rgb + (1.0 - behind.a) * environment_specular_along(through, roughness);
}

/// Radiance leaving a surface: Lambert diffuse from the irradiance map, glossy reflection from the
/// environment's radiance mip chain (split-sum), and for transmissive surfaces the refracted see-through of what
/// is drawn behind (the backdrop, read where the refracted ray leaves a slab of `thickness`; vgpu's
/// transmission example) with the environment where nothing is drawn.
/// `surface` = (roughness, metallic, transmission, ior). Without an environment, the fixed lights apply.
/// `dispersion` is KHR_materials_dispersion's 20 / Abbe number, read the way three.js does: the IOR
/// spreads by `(ior - 1) * 0.025 * dispersion` to either side and red / blue refract on their own.
fn shade_surface(albedo: vec3f, normal: vec3f, view_dir: vec3f, world_position: vec3f, thickness: f32, surface: vec4f, dispersion: f32) -> vec3f {
    let roughness = clamp(surface.x, 0.0, 1.0);
    let metallic = clamp(surface.y, 0.0, 1.0);
    let transmission = clamp(surface.z, 0.0, 1.0);
    let ior = max(surface.w, 1.0);
    let n_dot_v = clamp(dot(normal, view_dir), 1e-4, 1.0);
    let f0_dielectric = pow((ior - 1.0) / (ior + 1.0), 2.0);
    let f0 = mix(vec3f(f0_dielectric), albedo, metallic);
    let diffuse_weight = (1.0 - metallic) * (1.0 - transmission);

    if sun_color().w > 0.0 {
        // Light cookie capture: what this surface lets through, straight along the sun's ray.
        return albedo * transmission * (1.0 - metallic);
    }
    let shade = sun_shade(world_position, normal, 0.0);

    if frame.environment_present != 1u {
        return albedo * simple_lighting(normal) * shade;
    }

    let diffuse = albedo * diffuse_weight * diffuse_shading(normal);
    let reflected = reflect(-view_dir, normal);
    if FILTER_SURFACE_FOOTPRINT {
        surface_ray_dx = reflect(-view_direction_to_camera(world_position + surface_position_dx), footprint_neighbor_normal(normal,surface_normal_dx)) - reflected;
        surface_ray_dy = reflect(-view_direction_to_camera(world_position + surface_position_dy), footprint_neighbor_normal(normal,surface_normal_dy)) - reflected;
    }
    let specular = environment_specular_along(reflected, roughness) * env_brdf_approx(f0, roughness, n_dot_v);
    var transmitted = vec3f(0.0);
    if transmission > 0.0 {
        let fresnel = f0_dielectric + (1.0 - f0_dielectric) * pow(1.0 - n_dot_v, 5.0);
        let refracted = refract(-view_dir, normal, 1.0 / ior);
        let through = select(reflected, refracted, any(refracted != vec3f(0.0)));
        let exit = world_position + through * thickness;
        let projected_uv = projected_surface_uv(exit);
        let levels = f32(textureNumLevels(backdrop_texture));
        var lod = pow(roughness, 0.8) * max(levels - 1.0, 0.0) * 0.55;
        if FILTER_SURFACE_FOOTPRINT {
            let vx = -view_direction_to_camera(world_position + surface_position_dx);
            let vy = -view_direction_to_camera(world_position + surface_position_dy);
            let nx = footprint_neighbor_normal(normal,surface_normal_dx); let ny = footprint_neighbor_normal(normal,surface_normal_dy);
            let rx = refract(vx,nx,1.0/ior); let ry = refract(vy,ny,1.0/ior);
            let tx = select(reflect(vx,nx),rx,any(rx != vec3f(0.0)));
            let ty = select(reflect(vy,ny),ry,any(ry != vec3f(0.0)));
            surface_ray_dx = tx-through; surface_ray_dy = ty-through;
            let dx = projected_surface_uv(world_position + surface_position_dx + tx*thickness) - projected_uv;
            let dy = projected_surface_uv(world_position + surface_position_dy + ty*thickness) - projected_uv;
            lod = min(levels-1.0,max(lod,footprint_lod(dx,dy,vec2f(textureDimensions(backdrop_texture)))));
        }
        let half_spread = (ior - 1.0) * 0.025 * max(dispersion, 0.0);
        var seen = transmitted_backdrop(view_dir, normal, reflected, world_position, thickness, ior, roughness, lod);
        if half_spread > 0.0 {
            seen = vec3f(
                transmitted_backdrop(view_dir, normal, reflected, world_position, thickness, ior - half_spread, roughness, lod).r,
                seen.g,
                transmitted_backdrop(view_dir, normal, reflected, world_position, thickness, ior + half_spread, roughness, lod).b,
            );
        }
        transmitted = seen * albedo * transmission * (1.0 - metallic) * (1.0 - fresnel);
    }
    return (diffuse + specular) * shade + transmitted;
}
