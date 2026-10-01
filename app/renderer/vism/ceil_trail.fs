/*{
  "ID": "import.ceil_trail",
  "LABEL": "ceil_trail",
  "STAGE": "pass",
  "INPUTS": [ { "NAME": "inputImage", "TYPE": "image" } ],
  "PASSES": [ { "TARGET": "history", "PERSISTENT": true }, { } ]
}*/
void main() {
    if (PASSINDEX == 0) {
        gl_FragColor = mix(IMG_THIS_PIXEL(history), IMG_THIS_PIXEL(inputImage), 0.2);
    } else {
        gl_FragColor = IMG_THIS_PIXEL(history);
    }
}
