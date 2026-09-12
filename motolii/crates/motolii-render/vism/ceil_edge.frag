void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    vec2 px = 1.0 / iResolution.xy;
    vec2 uv = fragCoord / iResolution.xy;
    float gx = 0.0, gy = 0.0;
    for (int j = -1; j <= 1; j++) {
        for (int i = -1; i <= 1; i++) {
            vec3 c = texture(iChannel0, uv + vec2(float(i), float(j)) * px).rgb;
            float l = dot(c, vec3(0.299, 0.587, 0.114));
            gx += l * float(i) * (j == 0 ? 2.0 : 1.0);
            gy += l * float(j) * (i == 0 ? 2.0 : 1.0);
        }
    }
    float e = clamp(sqrt(gx * gx + gy * gy) * 2.0, 0.0, 1.0);
    fragColor = vec4(vec3(e) * vec3(0.4, 0.9, 1.4), texture(iChannel0, uv).a);
}
