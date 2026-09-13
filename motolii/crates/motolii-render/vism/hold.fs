/*{
  "ID": "motolii.hold",
  "LABEL": "Hold",
  "STAGE": "pass",
  "DESCRIPTION": "Freeze frame: the layer as it looked at one moment (seconds from the layer's in point). The host hands over that picture; the effect remembers nothing",
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "shot", "TYPE": "image", "TIME_AT": "moment" },
    { "NAME": "moment", "LABEL": "Moment", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 600.0, "SUBTYPE": "TIME", "HERO": true }
  ]
}*/

// AE の Freeze Frame(時間の hold)を、効果の 1 枚として。「いつの絵か」は host(TIME_AT)、絵をどうするかは shader。
void main() {
    gl_FragColor = IMG_THIS_PIXEL(shot);
}
