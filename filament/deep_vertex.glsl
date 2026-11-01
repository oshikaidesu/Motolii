// deep_vertex.glsl: what deep.glsl reads, from the thing's own place: where the eye is in the box's own lengths (Scale Z
// is in them: a ray there is straight however it is scaled) and which way its sides lie in the world, its third way by the first two and how long it is.
void deepVertex(inout MaterialVertexInputs material) {
    highp mat4 place = getWorldFromModelMatrix();
    highp vec3 x = place[0].xyz, y = place[1].xyz, z = place[2].xyz;
    highp vec4 eye = getWorldFromClipMatrix()[2];   // (the one point a camera's every ray goes through, whatever its lens: a shadow's, warped, too)
    bool level = abs(eye.w) < 1e-6 * length(eye.xyz);   // (an orthographic camera has no eye: only a way it looks)
    highp vec3 from = level ? normalize(eye.xyz) : eye.xyz / eye.w - place[3].xyz, a = cross(y, z), b = cross(z, x), c = cross(x, y);   // (the place's inverse, by its cofactors)
    highp float d = dot(x, a);
    material.deepEye = vec4(vec3(dot(a, from), dot(b, from), dot(c, from)) / d, level ? 0.0 : 1.0);
    material.deepAt = vec4(getPosition().xyz, 0.0);
    a /= d, b /= d, c /= d;   // (how a normal of the box turns into the world's: the place's inverse turned over, as Filament's own normals)
    material.deepUp = vec4(b, dot(c, normalize(cross(a, b))));
    material.copyOf.yzw = a;
}
