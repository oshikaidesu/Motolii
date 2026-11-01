// deep_vertex.glsl: what deep.glsl reads, from the thing's own place: where the eye is in the box's own lengths (Scale Z
// is in them: a ray there is straight however it is scaled) and which way its sides lie in the world.
void deepVertex(inout MaterialVertexInputs material) {
    highp mat4 place = getWorldFromModelMatrix(), eye = getWorldFromViewMatrix();
    highp vec3 x = place[0].xyz, y = place[1].xyz, z = place[2].xyz;
    bool level = getClipFromViewMatrix()[3][3] > 0.5;   // (an orthographic camera has no eye: only a way it looks)
    highp vec3 from = level ? eye[2].xyz : eye[3].xyz - place[3].xyz, a = cross(y, z), b = cross(z, x), c = cross(x, y);   // (the place's inverse, by its cofactors)
    material.deepEye = vec4(vec3(dot(a, from), dot(b, from), dot(c, from)) / dot(x, a), level ? 0.0 : 1.0);
    material.deepAt = vec4(getPosition().xyz, 0.0);
    material.deepUp = vec4(normalize(y), 0.0);
    material.copyOf.yzw = normalize(x);
}
