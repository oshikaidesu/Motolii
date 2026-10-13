// matte.glsl: what the mattes leave of a point of the screen (kit.h: matte, matted). A matte is a picture of the
// screen, drawn before all else: white where its shape is, black where it is not, grey between. A material that
// can be cut has the parameters matte0..3 (the pictures), matteUse (for each: 1 seen inside it, -1 outside it, 0 not
// cut by it) and matteSoft (how far its edge is blurred, as a level of the picture), is blended, and puts out what
// it would have, times matteLeft().
float matteLeft() {
    highp vec2 at = uvToRenderTargetUV(getNormalizedPhysicalViewportCoord().xy);
    vec4 m = vec4(textureLod(materialParams_matte0, at, materialParams.matteSoft.x).r, textureLod(materialParams_matte1, at, materialParams.matteSoft.y).r,
                  textureLod(materialParams_matte2, at, materialParams.matteSoft.z).r, textureLod(materialParams_matte3, at, materialParams.matteSoft.w).r);
    vec4 use = materialParams.matteUse;
    vec4 left = mix(vec4(1.0), mix(1.0 - m, m, step(0.0, use)), abs(use));
    return left.x * left.y * left.z * left.w;
}
