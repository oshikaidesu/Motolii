/*{
  "ID": "motolii.linear_wipe",
  "LABEL": "Linear Wipe",
  "STAGE": "pass",
  "DESCRIPTION": "Wipes the layer away along a straight edge at an angle as the completion rises (After Effects' Linear Wipe). 90 degrees wipes from left to right",
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "completion", "LABEL": "Transition Completion", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 100.0, "HERO": true },
    { "NAME": "angle", "LABEL": "Wipe Angle", "TYPE": "float", "DEFAULT": 90.0, "MIN": -360.0, "MAX": 360.0, "SUBTYPE": "ANGLE" },
    { "NAME": "feather", "LABEL": "Feather", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 4000.0, "SUBTYPE": "DISTANCE" }
  ]
}*/

// 角度は AE と同じく 0 が上から下、90 が左から右。拭く線は画面の対角の長さを 0..100% で通り抜ける。
void main() {
    float a = radians(angle);
    vec2 dir = vec2(sin(a), -cos(a));
    vec2 p = gl_FragCoord.xy - RENDERSIZE * 0.5;
    p.y = -p.y;
    float reach = dot(abs(dir), RENDERSIZE * 0.5);
    float along = dot(p, dir);
    float edge = -reach - feather * 0.5 + (2.0 * reach + feather) * completion / 100.0;
    float keep = feather > 0.0 ? smoothstep(edge - feather * 0.5, edge + feather * 0.5, along) : step(edge, along);
    gl_FragColor = IMG_THIS_PIXEL(inputImage) * keep;
}
