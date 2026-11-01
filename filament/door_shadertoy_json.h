// door_shadertoy_json.h: part of doc_door.h. A Shadertoy shader as the site keeps it, all its tabs, read into door_tabs_isf
// (door_shadertoy.h): rust_ae's isf_from_shadertoy_project, by the JSON's own words.
#pragma once
#include <rapidjson/document.h>
#include "door.h"

// The site's JSON (its API's, its Export's), all its tabs ("": it cannot be, and why): made one file at the door, as
// Hydra's is. A sound's tab is left out, as VVISF's is; a cube's draws what nothing here can read.
inline std::string door_shadertoy(const std::string& json, std::string& why) {
    rapidjson::Document all;
    if (all.Parse(json.c_str()).HasParseError()) return why = "this Shadertoy file does not read as JSON", "";
    const rapidjson::Value* root = &all;
    if (root->IsArray() && !root->Empty()) root = &(*root)[0];
    if (root->IsObject() && root->HasMember("Shader")) root = &(*root)["Shader"];
    if (!root->IsObject() || !root->HasMember("renderpass") || !(*root)["renderpass"].IsArray()) return why = "this Shadertoy file has no renderpass", "";
    const auto text = [](const rapidjson::Value& v, const char* k) { return v.IsObject() && v.HasMember(k) && v[k].IsString() ? std::string(v[k].GetString()) : std::string(); };
    const auto kind = [&](const rapidjson::Value& v) { const std::string t = text(v, "type"); return t.empty() ? text(v, "ctype") : t; };
    const auto id = [](const rapidjson::Value& v) { return !v.IsObject() || !v.HasMember("id") ? std::string() : v["id"].IsString() ? std::string(v["id"].GetString()) : v["id"].IsNumber() ? std::to_string(v["id"].GetInt64()) : std::string(); };
    std::vector<const rapidjson::Value*> passes;
    std::string common;
    const rapidjson::Value* image = nullptr;
    for (const auto& pass : (*root)["renderpass"].GetArray()) {
        const std::string type = kind(pass);
        if (type == "common") common += door_toy_clean(text(pass, "code")) + "\n";
        else if (type == "image") image = &pass;
        else if (type == "buffer") passes.push_back(&pass);
        else if (type == "cubemap") return why = "the tab " + text(pass, "name") + " draws a cube, which Motolii does not take yet", "";
    }
    if (!image) return why = "this Shadertoy file has no Image tab", "";
    std::sort(passes.begin(), passes.end(), [&](const rapidjson::Value* a, const rapidjson::Value* b) { return text(*a, "name") < text(*b, "name"); });   // (A to D, the order the site draws them)
    std::map<std::string, int> picture;   // (each buffer's number, by its output's id and by the picture that stood for it: the API's older word, /media/previz/buffer00.png)
    for (size_t k = 0; k < passes.size(); ++k) {
        const rapidjson::Value& out = (*passes[k])["outputs"];
        if (out.IsArray()) for (const auto& o : out.GetArray()) picture[id(o)] = (int)k + 1;
        picture["buffer0" + std::to_string(k)] = (int)k + 1;
    }
    passes.push_back(image);
    bool film = false;
    std::vector<DoorTab> tabs;
    for (size_t k = 0; k < passes.size(); ++k) {
        DoorTab tab;
        tab.name = text(*passes[k], "name"), tab.code = door_toy_clean(text(*passes[k], "code"));
        if (k + 1 < passes.size()) tab.target = "buffer" + std::string(1, (char)('A' + k));
        if (passes[k]->HasMember("inputs") && (*passes[k])["inputs"].IsArray())
            for (const auto& in : (*passes[k])["inputs"].GetArray()) {
                const int n = in.HasMember("channel") && in["channel"].IsInt() ? in["channel"].GetInt() : -1;
                if (n < 0 || n > 3) continue;
                const std::string type = kind(in), src = text(in, "src").empty() ? text(in, "filepath") : text(in, "src");
                const size_t at = src.find("buffer0");
                int read = -1;
                if (type == "buffer") {
                    auto found = picture.find(id(in));
                    if (found == picture.end() && at != std::string::npos) found = picture.find(src.substr(at, 8));
                    if (found != picture.end()) read = found->second;
                } else if (type == "texture" || type == "video" || type == "webcam") film = true, read = 0;   // (the picture it is put on: a film as much as anything)
                const bool repeats = in.HasMember("sampler") && text(in["sampler"], "wrap") == "repeat";
                tab.channel[n] = read + (read >= 0 && repeats ? 16 : 0);
            }
        tabs.push_back(tab);
    }
    const auto word = [&](const char* k) {   // (its name and maker, as ISF's head says them)
        const std::string said = root->HasMember("info") ? text((*root)["info"], k) : "";
        return std::regex_replace(said, std::regex(R"(["\\\n\r\t])"), " ");
    };
    return door_tabs_isf(common, tabs, film, word("name").empty() ? "Shadertoy" : word("name"), word("username"));
}
