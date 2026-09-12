void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    vec2 uv = fragCoord / iResolution.xy;
    float v = sin(uv.x * 14.0 + iTime) + sin((uv.y + uv.x) * 11.0 - iTime * 1.3);
    vec3 col = 0.5 + 0.5 * cos(vec3(0.0, 2.1, 4.2) + v * 1.7);
    fragColor = vec4(col, 1.0);
}
