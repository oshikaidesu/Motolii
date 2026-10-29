/*{
    "DESCRIPTION": "The key colour that spilled onto what is kept taken out of it (Alight Motion's Key Spill Cleaner, after a Chroma Key): gl-chromakey's ProcessChromaKey (MIT, Copyright (c) 2011 Mozilla Foundation, Copyright (c) 2020-2025 RadRoot LLC; https://github.com/bhj/gl-chromakey/blob/d8e616f6f61f72f4f7354c771f07e00e99d842d5/src/lib/shaders/fragmentShaderPaint.js), its lines as they are, its source read by ISF's IMG_NORM_PIXEL: a pixel as near the key colour as its tolerance and spill say is made as grey as its own luma. Its colour alone is taken: what is seen stays as it was, the key itself left to the Chroma Key before it.",
    "CREDIT": "gl-chromakey (MIT)",
    "CATEGORIES": [
        "Key"
    ],
    "ISFVSN": "2",
    "INPUTS": [
        { "NAME": "inputImage", "TYPE": "image" },
        { "NAME": "keyColor", "LABEL": "Key Color", "TYPE": "color", "DEFAULT": [ 0, 1, 0, 1 ] },
        { "NAME": "tolerance", "LABEL": "Tolerance", "TYPE": "float", "DEFAULT": 0.1, "MIN": 0, "MAX": 1 },
        { "NAME": "spill", "LABEL": "Spill", "TYPE": "float", "DEFAULT": 0.1, "MIN": 0, "MAX": 1 }
    ]
}*/

  // Standard BT.709 RGB to YUV conversion
  vec3 RGBtoYUV(vec3 rgb) {
    return vec3(
      rgb.r *  0.2126 + rgb.g *  0.7152 + rgb.b *  0.0722,           // Y (luma)
      rgb.r * -0.1146 + rgb.g * -0.3854 + rgb.b *  0.5,             // U (chroma)
      rgb.r *  0.5    + rgb.g * -0.4542 + rgb.b * -0.0458           // V (chroma)
    );
  }

  vec4 ProcessChromaKey(vec2 texCoord, vec3 keyColor, float tolerance, float smoothness, float spill) {
    // Remap 0-1 parameters to effective ranges
    float mappedTolerance = tolerance * 0.2;  // 0-0.2 range
    float mappedSmoothness = smoothness * 0.2 + 0.01;  // 0.01-0.21 range
    float mappedSpill = spill * 0.5 + 0.01;  // 0.01-0.51 range

    vec4 rgba = IMG_NORM_PIXEL(inputImage, texCoord);

    // Convert to YUV for both pixel and key color
    vec3 pixelYUV = RGBtoYUV(rgba.rgb);
    vec3 keyYUV = RGBtoYUV(keyColor);

    // Use improved distance calculation that includes luminance for achromatic colors
    vec2 pixelUV = pixelYUV.yz;
    vec2 keyUV = keyYUV.yz;
    float chromaDist = distance(pixelUV, keyUV);

    // For achromatic colors (low chrominance), also consider luminance difference
    float chromaLength = length(keyUV);
    if (chromaLength < 0.1) {  // Key color is achromatic (black, white, gray)
      float lumaDiff = abs(pixelYUV.x - keyYUV.x);
      chromaDist = max(chromaDist, lumaDiff);
    }

    float baseMask = chromaDist - mappedTolerance;
    float fullMask = pow(clamp(max(baseMask, 0.0) / mappedSmoothness, 0., 1.), 1.5);
    float spillVal = pow(clamp(max(baseMask, 0.0) / mappedSpill, 0., 1.), 1.5);
    rgba.a = fullMask;

    // Use the Y component from YUV for more accurate desaturation
    vec3 yuv = RGBtoYUV(rgba.rgb);
    float desat = yuv.x; // Use luma component instead of manual calculation
    rgba.rgb = mix(vec3(desat), rgba.rgb, spillVal);

    return rgba;
  }

void main() {
    gl_FragColor = vec4(ProcessChromaKey(isf_FragNormCoord, keyColor.rgb, tolerance, 0.1, spill).rgb, IMG_THIS_PIXEL(inputImage).a);   // (its colour cleaned, what is seen kept)
}
