// For the PIXEL part of a material that has dropped its corners with recvDrop (recv_drop.glsl), after recv_lands.glsl.
// light: rgb and how far it flew (as the vertex part handed it over, already cut by whatever else the material
// knows); ok, owner, start: as recvDrop gave them. Returns what to write:
//   without a stage  rgb = the light, a = light * distance flown (as beam_light.mat always wrote)
//   on a stage       rgb = the light, a = light * how far the softening pass is to spread it, in pixels; for an
//                    outline: red = 1 where the sun is cut off, a = the spread (green is recv_mask.mat's)
// On a stage a pixel is kept only where its receiver really is (recvOn), not behind where the light started, and for
// light: where no block stands between its start and here, or between the sun and its start (recvBlocked). What a
// block hides from the camera is left out by the depth buffer: the blocks are drawn into it first, and the material
// instance is set to test against it (see recvNearer in recv_drop.glsl for light that lies ON a block).
// Needs the parameter pxScale besides those of recv_drop.glsl.
vec4 recvFinish(vec4 light, float ok, float owner, highp vec3 start, bool outline) {
    if (materialParams.roomCount < 0.5) return vec4(light.rgb, (light.r + light.g + light.b) / 3.0 * light.a);
    vec4 c = light * step(0.9995, ok);
    highp vec3 X = getUserWorldPosition(), eye = getWorldCameraPosition() + (X - getWorldPosition());
    if (owner > -0.5) X = eye + (X - eye) / 0.996;   // (light on a block was pulled a hair towards the camera: recvNearer)
    bool blocks = materialParams.blockCount > 0.5;
    if (c.a < 0.0 || !recvOn(X, owner)) c = vec4(0.0);
    else if (!outline && blocks && ok > 1.01 && recvBlocked(X, start, materialParams.recvSun)) c = vec4(0.0);
    float px = c.a * materialParams.pxScale / length(X - eye);
    if (outline) return vec4(c.r, 0.0, 0.0, c.r * px);
    return vec4(c.rgb, (c.r + c.g + c.b) / 3.0 * px);
}
