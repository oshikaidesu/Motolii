// matte.glsl: what the mattes leave of a point (kit.h: matte, matted). A matte is a picture taken by an eye of its
// own, before all else: white where its shape is, black where it is not, grey between. That eye is the film's camera
// (the matte is in the film's picture) or one that rides on a thing (the matte is on the thing). A material that
// can be cut has the parameters matte0..3 (the pictures), matteEye0..3 (for each: where its eye sees a point of the
// world), matteUse (for each: 1 seen inside it, -1 outside it, 0 not cut by it) and matteSoft (how far its edge is
// blurred, as a level of the picture), is blended, and puts out what it would have, times matteLeft().
// The point is looked up where the MATTE's eye sees it, not where the eye that looks does: so a thing is cut the same
// whoever looks at it, the film's camera or another.
highp vec3 matteSeen(highp mat4 eye, highp vec3 at) {   // where on its picture, and 1 (0 if beside the picture or behind the eye)
    highp vec4 c = eye * vec4(at, 1.0);
    highp vec2 uv = c.xy / max(c.w, 1e-6) * 0.5 + 0.5;
    return vec3(uvToRenderTargetUV(uv), step(1e-6, c.w) * step(0.0, uv.x) * step(uv.x, 1.0) * step(0.0, uv.y) * step(uv.y, 1.0));
}
float matteLeft() {
    if (materialParams.matteUse == vec4(0.0)) return 1.0;   // (nothing cuts it: none of the pictures is looked at)
    highp vec3 at = getUserWorldPosition();
    highp vec3 a = matteSeen(materialParams.matteEye0, at), b = matteSeen(materialParams.matteEye1, at);
    highp vec3 c = matteSeen(materialParams.matteEye2, at), d = matteSeen(materialParams.matteEye3, at);
    vec4 m = vec4(a.z * textureLod(materialParams_matte0, a.xy, materialParams.matteSoft.x).r, b.z * textureLod(materialParams_matte1, b.xy, materialParams.matteSoft.y).r,
                  c.z * textureLod(materialParams_matte2, c.xy, materialParams.matteSoft.z).r, d.z * textureLod(materialParams_matte3, d.xy, materialParams.matteSoft.w).r);
    vec4 use = materialParams.matteUse;
    vec4 left = mix(vec4(1.0), mix(1.0 - m, m, step(0.0, use)), abs(use));
    return left.x * left.y * left.z * left.w;
}
