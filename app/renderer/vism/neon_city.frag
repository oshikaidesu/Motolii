// Shadertoy の作法そのまま(manifest 無し・mainImage・iChannel0・iTime)。
void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    vec2 uv = fragCoord / iResolution.xy;
    vec4 src = texture(iChannel0, uv);
    float shift = 0.004 * (1.0 + sin(iTime * 2.0));
    float r = texture(iChannel0, uv + vec2(shift, 0.0)).r;
    float b = texture(iChannel0, uv - vec2(shift, 0.0)).b;
    float scan = 0.88 + 0.12 * sin(uv.y * 220.0 + iTime * 6.0);
    vec3 col = vec3(r, src.g, b) * scan;
    col += vec3(0.12, 0.02, 0.20) * pow(max(1.0 - abs(uv.y - 0.5) * 2.0, 0.0), 3.0) * src.a;
    fragColor = vec4(col, src.a);
}
