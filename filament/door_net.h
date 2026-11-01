// door_net.h: part of doc_door.h. What a thing from the net comes with. curl fetches it, as extensions/hydra/get.sh does,
// with no shell between. A work on OpenProcessing, named by its address, is what the site's public API says it is
// (openprocessing.org/api: its tabs, libraries, files, license), laid out in a folder as a saved page is, so the door reads it
// as it reads any page. The site answers 40 asks a minute; a work is 3 and one for each of its files.
#pragma once
#include <glib.h>
#include <rapidjson/document.h>
#include <filesystem>
#include <fstream>
#include <regex>
#include <sstream>
#include <string>

inline bool door_fetch(const std::string& url, const std::filesystem::path& to) {
    const std::string file = to.string();
    const char* line[] = {"curl", "-fsSL", "--proto", "=http,https", "--proto-redir", "=http,https", "-m", "60", "-o", file.c_str(), "--url", url.c_str(), nullptr};
    gint status = 1;
    return g_spawn_sync(nullptr, (gchar**)line, nullptr, (GSpawnFlags)(G_SPAWN_SEARCH_PATH | G_SPAWN_STDOUT_TO_DEV_NULL | G_SPAWN_STDERR_TO_DEV_NULL), nullptr, nullptr, nullptr, nullptr, &status, nullptr) && status == 0;
}

inline std::string door_site_work(const std::string& word) {   // (the number of a work on OpenProcessing, "": it is not an address of one)
    static const std::regex address(R"(^\s*(?:https?://)?(?:www\.)?openprocessing\.org/sketch/(\d+)[^\s]*\s*$)");
    std::smatch said;
    return word.size() < 200 && std::regex_match(word, said, address) ? said[1].str() : "";
}

// Its page, in `at` with its tabs and its files ("": it did not come, and why).
inline std::filesystem::path door_openprocessing(const std::string& id, const std::filesystem::path& at, std::string& why) {
    const std::string api = "https://openprocessing.org/api/sketch/" + id;
    std::error_code wrong;
    std::filesystem::create_directories(at, wrong);
    const auto asked = [&](const std::string& url, const char* name, rapidjson::Document& out) {
        if (!door_fetch(url, at / name)) return false;
        std::ifstream in(at / name);
        std::stringstream all;
        all << in.rdbuf();
        return !out.Parse(all.str().c_str()).HasParseError();
    };
    const auto word = [](const rapidjson::Value& v, const char* k) { return v.IsObject() && v.HasMember(k) && v[k].IsString() ? std::string(v[k].GetString()) : std::string(); };
    rapidjson::Document about, tabs, files;
    if (!asked(api, "about.json", about) || !about.IsObject() || !asked(api + "/code", "code.json", tabs) || !tabs.IsArray() || !asked(api + "/files", "files.json", files) || !files.IsArray())
        return why = "OpenProcessing gave nothing for the work " + id + ": it may be private, or asked too often (40 a minute)", std::filesystem::path{};
    const auto safe = [](const std::string& name) { return std::regex_replace(name, std::regex(R"([^A-Za-z0-9._\-])"), "_"); };
    std::vector<std::string> scripts;
    bool page = false;
    for (const auto& tab : tabs.GetArray()) {
        std::string name = safe(word(tab, "title"));
        if (name.empty()) continue;
        if (name.find('.') == std::string::npos) name += ".js";
        std::ofstream(at / name) << word(tab, "code");
        if (name == "index.html") page = true;
        else if (name != "mySketch.js" && name.find(".css") == std::string::npos) scripts.push_back(name);
    }
    for (const auto& one : files.GetArray()) {   // (its pictures and the rest, under its address)
        const std::string url = word(one, "url"), name = safe(word(one, "name"));
        if (!url.empty() && !name.empty() && !door_fetch(url, at / name)) return why = "OpenProcessing did not give the file " + name, std::filesystem::path{};
    }
    if (page) return at / "index.html";
    if (std::filesystem::exists(at / "mySketch.js", wrong)) scripts.push_back("mySketch.js");
    std::string head = std::regex_replace(word(about, "title") + " by " + word(about, "username") + ", " + word(about, "license") + ", openprocessing.org/sketch/" + id, std::regex(R"([<>\n\r])"), " ");
    std::ofstream out(at / "index.html");
    out << "<script>// " << head << "</script>\n";   // (what it is and whose: the first line of what is kept)
    if (about.HasMember("libraries") && about["libraries"].IsArray())
        for (const auto& lib : about["libraries"].GetArray()) if (const std::string url = word(lib, "url"); !url.empty()) out << "<script src=\"" << url << "\"></script>\n";
    for (const std::string& name : scripts) out << "<script src=\"" << name << "\"></script>\n";
    return at / "index.html";
}
