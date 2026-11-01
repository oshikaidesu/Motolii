// door_shadertoy.h: part of door.h. A Shadertoy shader as an ISF file, each tab a pass: VVISF's ShadertoyConverter.cpp
// (BSD-3-Clause, which sorts the buffers first and the Image last) and rust_ae's shadertoy.rs (which keeps each buffer
// PERSISTENT and FLOAT). Each tab's code stays as it was written. What differs between tabs is said in macros around it:
// its iChannels, each a number here that door_read reads (a sampler told to a function goes as that number, which Vulkan's
// GLSL could not do with the picture), and the names two tabs both have: there the tabs are files apart, here one (rust_ae
// refused those; a macro takes them apart). What the site gives are values, not words: a function's own iResolution stands.
#pragma once
#include <algorithm>
#include <map>
#include <regex>
#include <set>
#include <string>
#include <vector>

inline std::string door_bare(const std::string& t) {   // (its comments out)
    std::string out;
    for (size_t i = 0; i < t.size(); ++i) {
        if (t.compare(i, 2, "//") == 0) { while (i + 1 < t.size() && t[i + 1] != '\n') ++i; }
        else if (t.compare(i, 2, "/*") == 0) { if ((i = t.find("*/", i)) == std::string::npos) break; ++i, out += ' '; }
        else out += t[i];
    }
    return out;
}

// What a tab names at its top level, and the macros it defines.
inline void door_tab_names(const std::string& code, std::set<std::string>& names, std::set<std::string>& macros) {
    const std::string t = door_bare(code);
    static const std::regex function(R"((\w+)\s*\()"), one(R"((\w+)\s*(?:\[[^\]]*\]\s*)?$)"), kind(R"(^\s*struct\s+(\w+))"), macro(R"(^\s*#\s*define\s+(\w+))");
    std::string head;
    int deep = 0;
    bool fresh = true;
    const auto said = [&](bool opens) {
        std::smatch m;
        const size_t equals = head.find('='), paren = head.find('(');
        if (std::regex_search(head, m, kind)) names.insert(m[1]);
        else if (head.find("precision") != std::string::npos) {}
        else if (paren != std::string::npos && (opens || paren < equals)) { if (std::regex_search(head, m, function)) names.insert(m[1]); }
        else if (!opens) {   // (each that a line declares: `const vec3 a = vec3(1.), b = vec3(2.);`)
            std::string part;
            int inner = 0;
            const auto take = [&] {
                const std::string name = part.substr(0, part.find('='));
                if (std::regex_search(name, m, one)) names.insert(m[1]);
                part.clear();
            };
            for (const char c : head) {
                inner += (c == '(') - (c == ')');
                if (c == ',' && !inner) take();
                else part += c;
            }
            take();
        }
        head.clear();
    };
    for (size_t i = 0; i < t.size(); ++i) {
        const char c = t[i];
        if (fresh && c == '#') {   // (a line for the preprocessor, however long)
            size_t end = i;
            while ((end = t.find('\n', end)) != std::string::npos && t[end - 1] == '\\') ++end;
            const std::string line = t.substr(i, end == std::string::npos ? end : end - i);
            std::smatch m;
            if (std::regex_search(line, m, macro)) macros.insert(m[1]);
            if (end == std::string::npos) break;
            i = end;
            continue;
        }
        fresh = c == '\n' || (fresh && (c == ' ' || c == '\t' || c == '\r'));
        if (c == '{' && !deep++) said(true);
        else if (c == '}') --deep;
        else if (!deep && c == ';') said(false);
        else if (!deep) head += c;
    }
}

// What another player declares for itself: shader-toy for VS Code, GLSL Sandbox's, a file's own.
inline std::string door_toy_clean(const std::string& text) {
    static const std::regex left(R"(\buniform\s+(?!sampler))");   // (any other uniform of its own: nothing tells it here, so a value at 0, as the site leaves one)
    return std::regex_replace(std::regex_replace(text, std::regex(R"(#iChannel[0-9][^\n]*|#version[^\n]*|#define\s+(?:iTime|iResolution|iMouse)\b[^\n]*|\buniform\s+\w+\s+(?:iTime|iGlobalTime|iResolution|iMouse|iFrame|iDate|iTimeDelta|iFrameRate|iSampleRate|iChannelTime|iChannelResolution|iChannel[0-9]|time|resolution|mouse)\s*(?:\[\s*\d*\s*\])?\s*;)"), ""), left, "");
}

// A tab: what it is called, the picture it draws ("": out), its code, and what each iChannel is: -1 nothing (a keyboard, a
// sound, a cube, nothing there), 0 the picture it is put on, N the Nth buffer; 16 more where it is read repeating.
struct DoorTab {
    std::string name, target, code;
    int channel[4] = {-1, -1, -1, -1};
};

inline std::string door_tabs_isf(std::string common, std::vector<DoorTab> tabs, bool film, const std::string& about = "Shadertoy", const std::string& credit = "") {
    const bool mouse = common.find("iMouse") != std::string::npos || std::any_of(tabs.begin(), tabs.end(), [](const DoorTab& t) { return t.code.find("iMouse") != std::string::npos; });
    static const std::set<std::string> taken = {"time", "resolution", "frame", "passindex", "clip_time", "clip_length", "frameRect", "passRect"};   // (isf.cpp's knobs, names a file cannot have too)
    const bool apart = common.find("iChannel") != std::string::npos;   // (Common reads what each tab is given: it goes in each)
    if (apart) for (DoorTab& t : tabs) t.code = common + t.code;
    std::vector<std::set<std::string>> names(tabs.size()), macros(tabs.size());
    std::map<std::string, int> many;
    std::set<std::string> shared, own;
    door_tab_names(common, own, shared);
    for (size_t k = 0; k < tabs.size(); ++k) {
        door_tab_names(tabs[k].code, names[k], macros[k]);
        names[k].erase("mainImage");
        for (const std::string& n : names[k]) many[n] += taken.count(n) ? 2 : 1;
    }
    std::string first, last;
    for (const std::string& n : own) if (taken.count(n)) first += "#define " + n + " " + n + "_common\n", last += "#undef " + n + "\n";
    if (!shared.count("texelFetch")) first = "#define isf_fetch door_texelFetch\n" + first;
    std::string body;
    for (size_t k = 0; k < tabs.size(); ++k) {
        std::string before = "#define mainImage door_tab_" + std::to_string(k) + "\n", after = "#undef mainImage\n";
        for (int n = 0; n < 4; ++n) before += "#define iChannel" + std::to_string(n) + " " + std::to_string(tabs[k].channel[n]) + "\n", after += "#undef iChannel" + std::to_string(n) + "\n";
        for (const std::string& n : names[k])
            if (many[n] > 1) before += "#define " + n + " " + n + "_" + std::to_string(k) + "\n", after += "#undef " + n + "\n";
        for (const std::string& n : macros[k])
            if (!shared.count(n) && !many.count(n)) after += "#undef " + n + "\n";
        if (macros[k].count("texelFetch") && !shared.count("texelFetch")) before += "#undef isf_fetch\n", after += "#undef isf_fetch\n#define isf_fetch door_texelFetch\n";   // (a file's own, webGL1's: isf.cpp says it under this name)
        body += "\n// ---- " + tabs[k].name + " ----\n" + before + tabs[k].code + "\n" + after;
    }
    std::string passes, reads, pixels;
    for (size_t k = 0; k + 1 < tabs.size(); ++k) {
        passes += "{\"TARGET\":\"" + tabs[k].target + "\",\"PERSISTENT\":true,\"FLOAT\":true},";
        reads += "if(k==" + std::to_string(k + 1) + ")return texture(isf_img_" + tabs[k].target + ",p);";
        pixels += "if(k==" + std::to_string(k + 1) + ")return texelFetch(isf_img_" + tabs[k].target + ",p,0);";
    }
    if (film) reads += "if(k==0)return texture(isf_img_inputImage,p);", pixels += "if(k==0)return texelFetch(isf_img_inputImage,p,0);";
    std::string run;
    for (size_t k = 0; k < tabs.size(); ++k) run += (k ? "else " : "") + std::string(tabs.size() > 1 ? "if(PASSINDEX==" + std::to_string(k) + ")" : "") + "door_tab_" + std::to_string(k) + "(door_out,floor(gl_FragCoord.xy)+.5);";
    return std::string("/*{\"ISFVSN\":\"2\",\"DESCRIPTION\":\"") + about + "\"," + (credit.empty() ? "" : "\"CREDIT\":\"" + credit + "\",") + "\"CATEGORIES\":[\"Shadertoy\"],\"INPUTS\":[" + (film ? "{\"NAME\":\"inputImage\",\"TYPE\":\"image\"}" : "") + (film && mouse ? "," : "") + (mouse ? "{\"NAME\":\"door_mouse\",\"LABEL\":\"Mouse\",\"TYPE\":\"point2D\",\"DEFAULT\":[0,0]}" : "") + "]" + (tabs.size() > 1 ? ",\"PASSES\":[" + passes + "{}]" : "") + "}*/\n"
           "vec3 iResolution,iChannelResolution[4];float iTime,iGlobalTime,iTimeDelta,iFrameRate,iSampleRate,iChannelTime[4];int iFrame;vec4 iMouse,iDate;\n"
           "void door_given(){iResolution=vec3(RENDERSIZE,1.);iChannelResolution=vec3[4](iResolution,iResolution,iResolution,iResolution);iTime=iGlobalTime=TIME;iChannelTime=float[4](TIME,TIME,TIME,TIME);iTimeDelta=TIMEDELTA;iFrameRate=60.;iSampleRate=44100.;iFrame=FRAMEINDEX;iDate=DATE;" + (mouse ? "vec2 m=door_mouse*RENDERSIZE;iMouse=m==vec2(0.)?vec4(0.):vec4(m,m.x,-m.y);" : "iMouse=vec4(0.);") + "}\n"   // (a point on its card that keys can move: down while it is anywhere but its corner, Shadertoy's sign of z)
           "float door_smoothstep(float a,float b,float x){return smoothstep(a,b,x);}\nvec2 door_smoothstep(float a,float b,vec2 x){return smoothstep(a,b,x);}\nvec3 door_smoothstep(float a,float b,vec3 x){return smoothstep(a,b,x);}\nvec4 door_smoothstep(float a,float b,vec4 x){return smoothstep(a,b,x);}\nvec2 door_smoothstep(vec2 a,vec2 b,vec2 x){return smoothstep(a,b,x);}\nvec3 door_smoothstep(vec3 a,vec3 b,vec3 x){return smoothstep(a,b,x);}\nvec4 door_smoothstep(vec4 a,vec4 b,vec4 x){return smoothstep(a,b,x);}\n"   // (Tint refuses equal edges known beforehand; a GPU draws them)
           "vec4 door_read(int k,vec2 p){if(k>=16){k-=16;p=fract(p);}" + reads + "return vec4(0.);}\nvec4 door_pixel(int k,ivec2 p){k=k&15;p.y=int(RENDERSIZE.y)-1-p.y;" + pixels + "return vec4(0.);}\n"
           "vec4 door_texture(int k,vec2 p){return door_read(k,p);}\nvec4 door_texture(int k,vec2 p,float b){return door_read(k,p);}\nvec4 door_textureLod(int k,vec2 p,float l){return door_read(k,p);}\nvec4 door_textureGrad(int k,vec2 p,vec2 a,vec2 b){return door_read(k,p);}\n"
           "vec4 door_textureOffset(int k,vec2 p,ivec2 o){return door_read(k,p+vec2(o)/RENDERSIZE);}\nvec4 door_textureLodOffset(int k,vec2 p,float l,ivec2 o){return door_read(k,p+vec2(o)/RENDERSIZE);}\n"
           "vec4 door_texture(int k,vec3 p){return vec4(0.);}\nvec4 door_texture(int k,vec3 p,float b){return vec4(0.);}\nvec4 door_textureLod(int k,vec3 p,float l){return vec4(0.);}\n"
           "vec4 door_texelFetch(int k,ivec2 p,int l){return door_pixel(k,p);}\nvec4 door_texelFetchOffset(int k,ivec2 p,int l,ivec2 o){return door_pixel(k,p+o);}\nivec2 door_textureSize(int k,int l){return ivec2(RENDERSIZE);}\n"
           "#undef TIME\n#undef texture\n#undef textureLod\n#undef textureSize\n#undef isf_fetch\n#define texture door_texture\n#define textureLod door_textureLod\n#define textureGrad door_textureGrad\n#define textureOffset door_textureOffset\n#define textureLodOffset door_textureLodOffset\n#define texelFetchOffset door_texelFetchOffset\n#define textureSize door_textureSize\n#define sampler2D int\n#define sampler3D int\n#define samplerCube int\n#define sampler door_sampler\n#define smoothstep door_smoothstep\n" + first +
           (apart ? "" : "\n// ---- Common ----\n" + common) + body + "\n" + last +
           "#undef texture\n#undef textureLod\n#undef sampler2D\n#define texture(i, c) textureLod(i, isf_up(c) * frameRect.zw + frameRect.xy, 0.0)\n"   // (as isf.cpp says it: what follows the tabs reads a picture as ever)
           "void main(){door_given();vec4 door_out=vec4(0.,0.,0.,1.);" + run + "gl_FragColor=PASSINDEX==" + std::to_string(tabs.size() - 1) + "?vec4(door_out.rgb,1.):door_out;}\n";
}
