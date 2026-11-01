// Code found on the net, as it comes: what it is read from itself and run by the door it needs, the file left as it is.
// What is on the page of each is said in a line before it, so a sketch's lines stay its own lines (script.h sketch_marked).
// JavaScript is told as a p5.js sketch on p5.js's canvas (p5_stand.h), a shader as an ISF effect (isf.cpp). The forms are
// each site's own: Dwitter's page (dwitter/templates/dweet/dweet.html: c, x, S, C, T, R, u(t) 60 times a second on a white
// 1920 x 1080), tixy.land's (t, i, x, y, with(Math); 16 x 16, white or red), Turtletoy's (walk(i), turtletoy on npm),
// Two.js's and Paper.js's own canvas renderers, Shadertoy's mainImage and its uniforms, GLSL Sandbox's time and resolution,
// twigl's r m t f s b and FC (src/fragmen.js), Bonzomatic's fGlobalTime, v2Resolution and out_color (its default shader).
#pragma once
#include <filesystem>
#include <fstream>
#include <regex>
#include <sstream>
#include <string>
#include "third_party/js/twigl/noise.h"

inline bool door_has(const std::string& t, const char* word) { return t.find(word) != std::string::npos; }
#include "door_shadertoy.h"

// "" as it is (ISF, a p5.js sketch, a recipe of ours, or not code); or shadertoy sandbox hydra two paper turtle canvas dweet tixy.
inline std::string door_form(const std::string& t) {
    static const std::regex twigl(R"(\bFC\b|\buniform\s+vec2\s+r\s*;)"), sandbox(R"(\buniform\s+\w+\s+(u_)?(time|resolution)\b)"), hydra(R"(\b(osc|noise|voronoi|shape|gradient|solid|src)\s*\()"), member(R"(\b[xc]\.\w|\bwith\s*\(\s*x\s*\))"), tixy(R"(\b[tixy]\b)"), paper(R"(\bnew\s+(Path|Point|Group|Raster|PointText|Layer|Size)\b|\bview\.|\bfunction\s+onFrame\b|\bproject\.)");
    if (door_has(t, "\"ISFVSN\"") || door_has(t, "\"INPUTS\"") || door_has(t, "\"CATEGORIES\"") || door_has(t, "function each")) return "";
    if (door_has(t, "mainImage")) return "shadertoy";
    if (door_has(t, "gl_FragColor") && door_has(t, "void main") && std::regex_search(t, sandbox)) return "sandbox";
    if (door_has(t, ".out(") && std::regex_search(t, hydra)) return "hydra";
    if (door_has(t, "THREE.") || door_has(t, "from 'three'") || door_has(t, "from \"three\"")) return "three";
    if (door_has(t, "new Two(")) return "two";
    if (door_has(t, "new Turtle(") || door_has(t, "extends Turtle") || door_has(t, "Canvas.setpenopacity(")) return "turtle";
    const bool sketch = door_has(t, "function draw") || door_has(t, "function setup") || door_has(t, "createCanvas");
    if (sketch) return "";
    if (door_has(t, "fGlobalTime") || door_has(t, "v2Resolution")) return "bonzomatic";
    if (door_has(t, "vec") && !door_has(t, "=>") && !door_has(t, "function") && (std::regex_search(t, twigl) || door_has(t, "gl_FragCoord") || door_has(t, "gl_FragColor"))) return "twigl";
    if (std::regex_search(t, paper)) return "paper";
    if (door_has(t, "getContext")) return "canvas";
    if (t.size() <= 900 && (door_has(t, "unescape(escape`") || std::regex_search(t, member))) return "dweet";   // (a dweet packed in wide characters: Dwitter's own players unpack it by running it)
    if (t.size() <= 120 && t.find('\n') == t.rfind('\n') && !door_has(t, ";") && !door_has(t, "function") && std::regex_search(t, tixy)) return "tixy";
    return "";
}

// A library a form runs on, from third_party/js ("": none).
inline std::string door_library(const std::string& form) {
    return form == "two" ? "two/two.min.js" : form == "paper" ? "paper/paper-full.min.js" : form == "turtle" ? "turtletoy/turtletoy.umd.cjs" : form == "three" ? "three/three.core.js" : "";
}

// A whole page as it is copied: what its own <script> tags hold; what they load is the form's library.
inline std::string door_page(const std::string& t) {
    std::string out;
    for (size_t at = t.find("<script"); at != std::string::npos; at = t.find("<script", at)) {
        const size_t head = t.find('>', at), end = head == std::string::npos ? head : t.find("</script>", head);
        if (end == std::string::npos) break;
        const std::string tag = t.substr(at, head - at);
        if (!door_has(tag, "src=") && (!door_has(tag, "type=") || door_has(tag, "module") || door_has(tag, "javascript"))) out += t.substr(head + 1, end - head - 1) + "\n";
        at = end;
    }
    return out.empty() ? t : out;
}

inline std::string door_quoted(const std::string& t) {   // (as a JavaScript string)
    std::string out = "\"";
    for (const char c : t) out += c == '"' ? "\\\"" : c == '\\' ? "\\\\" : c == '\n' ? "\\n" : c == '\r' ? "" : std::string(1, c);
    return out + "\"";
}

// p5's loadShader as the two files beside the sketch say it, in the sketch from here on: what is kept is one file, and a
// script has no net to fetch them by. Files not there: as it was.
inline std::string door_shader_files(const std::string& text, const std::filesystem::path& beside) {
    static const std::regex load(R"re(\bloadShader\s*\(\s*['"`]([^'"`]+)['"`]\s*,\s*['"`]([^'"`]+)['"`]\s*(?:,[^)]*)?\))re");
    const auto file = [&](const std::string& name) { std::ifstream in(beside / name); std::stringstream all; all << in.rdbuf(); return all.str(); };
    std::string out;
    size_t at = 0;
    for (std::sregex_iterator m(text.begin(), text.end(), load), end; m != end; ++m) {
        const std::string vert = file((*m)[1]), frag = file((*m)[2]);
        out += text.substr(at, (size_t)m->position() - at) + (vert.empty() || frag.empty() ? m->str() : "createShader(" + door_quoted(vert) + "," + door_quoted(frag) + ")");
        at = (size_t)(m->position() + m->length());
    }
    return out + text.substr(at);
}

// A library as a script runs it: three.js's module, its names one object as three.min.js gave them.
inline std::string door_library_text(const std::string& form, std::string text) {
    if (form != "three") return text;
    const size_t at = text.rfind("export {");
    if (at == std::string::npos) return text;
    const std::string names = text.substr(at + 8, text.rfind('}') - at - 8);
    return text.substr(0, at) + "globalThis.THREE = {" + std::regex_replace(names, std::regex(R"((\w+) as (\w+))"), "$2: $1") + "};\n";
}

// A file's imports, which a script does not have, as what they are here: three.js's names from THREE, any other's stand-ins.
inline std::string door_three_imports(std::string src) {
    src = std::regex_replace(src, std::regex(R"re(import\s+\*\s+as\s+(\w+)\s+from\s+['"]three['"]\s*;?)re"), "var $1=globalThis.THREE;");
    src = std::regex_replace(src, std::regex(R"re(import\s+(\w+)\s+from\s+['"]three['"]\s*;?)re"), "var $1=globalThis.THREE;");
    src = std::regex_replace(src, std::regex(R"re(import\s+(?:\*\s+as\s+)?(\w+)\s+from\s+['"][^'"]+['"]\s*;?)re"), "var $1=__any_names.$1;");
    std::string out;
    static const std::regex named(R"re(import\s*\{([^}]*)\}\s*from\s*['"]([^'"]+)['"]\s*;?)re");
    auto from = std::sregex_iterator(src.begin(), src.end(), named);
    size_t at = 0;
    for (auto m = from; m != std::sregex_iterator(); ++m) {
        out += src.substr(at, (size_t)m->position() - at), at = (size_t)(m->position() + m->length());
        const std::string names = std::regex_replace((*m)[1].str(), std::regex(R"((\w+)\s+as\s+(\w+))"), "$1:$2");
        if ((*m)[2].str() == "three") out += "var {" + names + "}=THREE;";
        else out += "var {" + names + "}=__any_names;";
    }
    return out + src.substr(at);
}

// A JavaScript form as a p5.js sketch: the source's lines stay where they were. Any other: the source as it is.
inline std::string door_js(const std::string& src, const std::string& form) {
    const std::string body = "function __door_body(){" + src + "\n;return typeof walk=='function'?walk:null}\n";
    if (form == "dweet")
        return "var c,x,S=Math.sin,C=Math.cos,T=Math.tan,R=(r,g,b,a)=>'rgba('+(r|0)+','+(g|0)+','+(b|0)+','+(a===undefined?1:a)+')',time=0,frame=0;function setup(){createCanvas(1920,1080);__door_canvas();x=drawingContext;c=x.canvas;c.sheet='#fff';c.width=1920}function draw(){time=(frame=frameCount-1)/60;u(time)}function u(t){" + src + "\n}\n";
    if (form == "tixy")
        return "function setup(){createCanvas(1024,1024);noStroke()}function draw(){background(0);const t=(frameCount-1)/60;for(let i=0,y=0;y<16;y++)for(let x=0;x<16;x++,i++){const v=Math.max(-1,Math.min(1,Number(__tixy(t,i,x,y))||0));fill(v<0?'#f24':'#fff');circle(x*64+32,y*64+32,Math.abs(v)*62)}}function __tixy(t,i,x,y){with(Math){return(" + src + "\n)}}\n";
    if (form == "turtle")
        return "var Canvas,Turtle,__walk,__i=0;function setup(){createCanvas(1080,1080);noFill();background(255);stroke(0);Canvas=turtletoy.Canvas,Turtle=turtletoy.Turtle;Canvas.setpenopacity=v=>{background(v<0?0:255);stroke(v<0?255:0,Math.abs(v)*255)};turtletoy.turtleDraw(()=>{},{onDrawLine:(a,b,p,q)=>line((a+100)*5.4,(b+100)*5.4,(p+100)*5.4,(q+100)*5.4)});__walk=__door_body()}function draw(){for(let n=0;n<400&&__walk;++n)if(__walk(__i++,1)===false)__walk=null}" + body;
    if (form == "two")
        return "function setup(){createCanvas(1920,1080);__door_canvas();const W=Two;Two=class extends W{constructor(o){o=Object.assign({},o,{type:W.Types.canvas});if(o.width&&o.height&&!o.fullscreen&&!o.fitted)resizeCanvas(o.width,o.height);super(o)}};__door_body()}" + body;
    if (form == "canvas")
        return "function setup(){createCanvas(1920,1080);__door_canvas();__door_body();const e=drawingContext.canvas;if(e.width!==width||e.height!==height)resizeCanvas(e.width,e.height,true)}" + body;
    if (form == "three")
        return "function setup(){createCanvas(1920,1080,WEBGL);__door_canvas();__three_stand();__door_body()}function draw(){}function __door_body(){" + door_three_imports(src) + "\n}\n";
    if (form == "paper")
        return "function setup(){createCanvas(1920,1080);__door_canvas();paper.setup(drawingContext.canvas);paper.PaperScript.execute(" + door_quoted(src) + ",paper);paper.view.play&&paper.view.play()}\n";
    return src;
}

// A shader's form as an ISF effect: what it asks of its site said in ISF's words. A shader that reads iChannel takes the
// picture it is put on, a film as much as anything. Any other: the text as it is.
// A shader with a main of its own (or a body with none: twigl's geekest) called from ISF's: `before` what it is given at
// its top, `start` what it is told each time, `done` what comes of it. What it reads of its last frame (`last`, the names
// that picture has) is a pass kept from frame to frame (ISF_Spec's PERSISTENT).
inline std::string door_called(const std::string& name, std::string body, const std::string& last, const std::string& before, const std::string& start, const std::string& done) {
    static const std::regex hasMain(R"(\bvoid\s+main\s*\(\s*(void)?\s*\))");
    const std::regex past(R"(\b(?:texture|texture2D)\s*\(\s*(?:)" + last + R"()\s*,)");
    const bool kept = std::regex_search(body, past);
    body = std::regex_replace(body, past, "texture(isf_img_door_last,");
    body = std::regex_search(body, hasMain) ? std::regex_replace(body, hasMain, "void door_main()") : "void door_main(){\n" + body + "\n}";
    return "/*{\"ISFVSN\":\"2\",\"DESCRIPTION\":\"" + name + "\",\"CATEGORIES\":[\"Shadertoy\"],\"INPUTS\":[]" + (kept ? ",\"PASSES\":[{\"TARGET\":\"door_last\",\"PERSISTENT\":true},{}]" : "") + "}*/\n" + before + body +
           "\nvoid main(){" + (kept ? "if(PASSINDEX==1){gl_FragColor=IMG_NORM_PIXEL(door_last,gl_FragCoord.xy/RENDERSIZE);return;}" : "") + start + "door_main();" + done + "}\n";
}

inline std::string door_isf(const std::string& text) {
    const std::string form = door_form(text);
    if (form == "shadertoy") {   // (one tab, as it is copied: every iChannel the picture it is put on)
        DoorTab tab;
        tab.name = "Image", tab.code = door_toy_clean(text);
        const bool reads = door_has(text, "iChannel");
        if (reads) std::fill(tab.channel, tab.channel + 4, 0);
        return door_tabs_isf("", {tab}, reads);
    }
    if (form == "bonzomatic") {
        static const std::regex output(R"(\blayout\s*\([^)]*\)\s*out\s+vec4\s+(\w+)\s*;)");
        std::smatch said;
        const std::string out = std::regex_search(text, said, output) ? said[1].str() : "out_color";
        static const std::regex picture(R"(\buniform\s+sampler[12]D\s+(\w+)\s*;)");
        std::string none = "texFFT\\w*";   // (its sound and its pictures, whatever their names: none here)
        for (std::sregex_iterator at(text.begin(), text.end(), picture), end; at != end; ++at) if ((*at)[1] != "texPreviousFrame") none += "|" + (*at)[1].str();
        std::string body = std::regex_replace(text, std::regex(R"(#version[^\n]*|\buniform\s+\w+\s+(?:fGlobalTime|fFrameTime|v2Resolution)\s*;)"), "");
        body = std::regex_replace(std::regex_replace(body, output, ""), picture, "");
        body = std::regex_replace(body, std::regex(R"(\b(?:texture|texture2D|texture1D|textureLod|texelFetch)\s*\(\s*(?:)" + none + R"()\s*,)"), "door_none(");
        return door_called("Bonzomatic", body, "texPreviousFrame", "float fGlobalTime,fFrameTime;vec2 v2Resolution;float door_now(){return TIME;}vec2 door_size(){return RENDERSIZE;}\nvec4 " + out + ";\nvec4 door_none(float a){return vec4(0.);}\nvec4 door_none(float a,float b){return vec4(0.);}\nvec4 door_none(vec2 a){return vec4(0.);}\nvec4 door_none(vec2 a,float b){return vec4(0.);}\n", "fGlobalTime=door_now();v2Resolution=door_size();fFrameTime=1./60.;", "gl_FragColor=vec4(" + out + ".rgb,1.);");   // (values, not words: `#define time fGlobalTime` is its usual first line)
    }
    if (form == "twigl") {
        static const std::regex own(R"(\bvoid\s+main\s*\()"), frag(R"(\bFC\b)");
        std::string body = std::regex_replace(text, std::regex(R"(#version[^\n]*|\bprecision\s+\w+\s+\w+\s*;|\buniform\s+\w+\s+(?:r|m|t|f|s|b|resolution|mouse|time|backbuffer)\s*;|\bout\s+vec4\s+(?:o|outColor)\s*;)"), "");
        const std::string given = std::regex_search(text, own) && !std::regex_search(text, frag) ? "" : twigl_noise;   // (its helpers, its geekest mode's alone)
        return door_called("twigl", body, "b|backbuffer", "#define FC gl_FragCoord\nvec2 r,m,mouse;float t,f,s;vec4 o,outColor;\n" + given + "\n", "r=RENDERSIZE;m=mouse=vec2(0.);t=TIME;f=float(FRAMEINDEX);s=0.;",
                           door_has(body, "gl_FragColor") ? "" : std::string("gl_FragColor=vec4(") + (door_has(body, "outColor") ? "outColor" : "o") + ".rgb,1.);");
    }
    if (form == "sandbox") {
        std::string body = std::regex_replace(text, std::regex(R"(\buniform\s+\w+\s+(u_)?(time|resolution|mouse)\s*;|\bvarying\s+vec2\s+surface\w*\s*;|#extension[^\n]*)"), "");
        body = std::regex_replace(body, std::regex(R"(\bvoid\s+main\s*\(\s*void\s*\))"), "void main()");
        if (door_has(body, "backbuffer")) {   // (what it drew last: GLSL Sandbox's own, twigl's classic mode too)
            body = std::regex_replace(body, std::regex(R"(\buniform\s+sampler2D\s+backbuffer\s*;)"), "");
            return door_called("GLSL Sandbox", body, "backbuffer", "#define u_time TIME\n#define u_resolution RENDERSIZE\n#define mouse vec2(.5)\n#define u_mouse vec2(.5)\n", "", "");
        }
        return "/*{\"ISFVSN\":\"2\",\"DESCRIPTION\":\"GLSL Sandbox\",\"CATEGORIES\":[\"Shadertoy\"],\"INPUTS\":[]}*/\n#define u_time TIME\n#define u_resolution RENDERSIZE\n#define mouse vec2(.5)\n#define u_mouse vec2(.5)\n" + body;   // (time and resolution are what ISF's own are called here: isf.cpp)
    }
    return text;
}
