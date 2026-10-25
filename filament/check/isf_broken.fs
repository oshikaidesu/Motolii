/*{"ISFVSN":"2","DESCRIPTION":"Not GLSL on purpose: the stage refuses it (check/thing_fx.txt)","INPUTS":[{"NAME":"inputImage","TYPE":"image"}]}*/
void main() { gl_FragColor = this is not glsl; }
