// find.h: part of motolii.mm, read before motolii_calls.h. The shelves' search (motolii_find): RapidFuzz for words typed
// wrong or in part, Japanese WordNet and SQLite's FTS5 for Japanese words (the user's Mercari: read, not asked; golf
// (91), (96)). Cut out of motolii_calls.h as it was, by size.
// A text's letters (a byte 10xxxxxx goes with the one before it, as drawn_made.h splits a text), in small letters as
// RapidFuzz's utils.default_process; and back to UTF-8.
static std::u32string find_letters(const std::string& s) {
    std::u32string out;
    for (size_t i = 0; i < s.size();) {
        const unsigned char b = (unsigned char)s[i];
        const int more = b >= 0xF0 ? 3 : b >= 0xE0 ? 2 : b >= 0xC0 ? 1 : 0;
        char32_t c = more ? b & (0x3F >> more) : b;
        for (int k = 1; k <= more && i + k < s.size(); ++k) c = (c << 6) | ((unsigned char)s[i + k] & 0x3F);
        out += c < 128 ? (char32_t)std::tolower((int)c) : c;
        i += 1 + more;
    }
    return out;
}
static std::string find_utf8(const std::u32string& s) {
    std::string out;
    for (const char32_t c : s) {
        if (c < 0x80) out += (char)c;
        else if (c < 0x800) out += (char)(0xC0 | (c >> 6)), out += (char)(0x80 | (c & 0x3F));
        else if (c < 0x10000) out += (char)(0xE0 | (c >> 12)), out += (char)(0x80 | ((c >> 6) & 0x3F)), out += (char)(0x80 | (c & 0x3F));
        else out += (char)(0xF0 | (c >> 18)), out += (char)(0x80 | ((c >> 12) & 0x3F)), out += (char)(0x80 | ((c >> 6) & 0x3F)), out += (char)(0x80 | (c & 0x3F));
    }
    return out;
}
// Japanese words a hand types, said in English: Japanese WordNet (NICT; words/ja_en.tsv, cut from its wnjpn.db as
// words/SOURCE.txt says; a maker's own words/ja_en.tsv adds to it), its kana as hiragana (katakana and hiragana are
// Unicode's two blocks 0x60 apart), the longest word it has at each place (a dictionary's longest match); a kana
// alone (a particle: が, を) is no word, nor letters of other scripts (an English word typed is RapidFuzz's). Each
// word found, its English words. None: not Japanese, or no word of it known.
static std::vector<std::vector<std::string>> find_english(const std::string& query) {
    static std::unordered_map<std::string, std::string> words;
    static size_t longest = 0;
    static bool read = false;
    if (!read) {
        read = true;
        for (const auto& dir : doc_folders("words")) {
            std::ifstream in(dir / "ja_en.tsv");
            for (std::string line; std::getline(in, line);)
                if (const size_t tab = line.find('\t'); tab != std::string::npos) {
                    const std::string ja = line.substr(0, tab);
                    std::string& en = words[ja];
                    en += (en.empty() ? "" : ",") + line.substr(tab + 1);
                    longest = std::max(longest, find_letters(ja).size());
                }
        }
    }
    std::u32string q = find_letters(query);
    for (char32_t& c : q) if (c >= 0x30A1 && c <= 0x30F6) c -= 0x60;
    std::vector<std::vector<std::string>> said;
    const auto japanese = [](char32_t c) { return c >= 0x3000 && c <= 0x9FFF; };   // (kana, kanji and their marks)
    for (size_t i = 0; i < q.size();) {
        size_t took = 0;
        for (size_t n = std::min(longest, q.size() - i); n >= 1 && !took && japanese(q[i]); --n) {
            if (n == 1 && q[i] >= 0x3041 && q[i] <= 0x3096) break;   // (a kana alone: a particle)
            if (!std::all_of(q.begin() + (long)i, q.begin() + (long)(i + n), japanese)) continue;
            if (const auto w = words.find(find_utf8(q.substr(i, n))); w != words.end()) {
                std::vector<std::string> one;
                std::istringstream each(w->second);
                for (std::string e; std::getline(each, e, ',');) if (!e.empty()) one.push_back(e);
                said.push_back(one);
                took = n;
            }
        }
        i += took ? took : 1;
    }
    return said;
}
// The words a hand types, against a shelf's items ("id<TAB>its name and label<TAB>what it does" a line), nearest first,
// the best `limit` (5 when 0). Japanese words said in English (find_english) are looked for as a search engine looks
// for a query widened by its synonyms: each word's English OR'd, the words AND'd (each word of the query found in
// some way; none found so, any of them), SQLite's FTS5 (its porter tokenizer: melt finds melts) ranking by bm25, an
// English word in many items (mix, level) weighing less than one in few (melt, afterimage). Any other: RapidFuzz
// (MIT, third_party/rapidfuzz, the user's choice), as its author's process.extract goes as it comes, no cut: a name
// by WRatio, extract's own scorer; what it does, a longer text, by partial_ratio, RapidFuzz's way to find a short
// string in a long one (WRatio counts a text 8 times as long at 0.6 of its score); the higher of the two. Nothing of
// the document read.
const char* motolii_find(Motolii*, const char* query, const char* items, int limit) {
    static thread_local std::string said;
    said.clear();
    const size_t most = limit > 0 ? (size_t)limit : 5;
    if (const std::vector<std::vector<std::string>> english = find_english(query); !english.empty()) {
        static thread_local sqlite3* db = nullptr;
        static thread_local size_t built = 0;
        const size_t these = std::hash<std::string>{}(items);
        if (!db) sqlite3_open(":memory:", &db);
        if (built != these) {   // (the items indexed once while they are the same)
            sqlite3_exec(db, "drop table if exists t; create virtual table t using fts5(id unindexed, name, about, tokenize='porter unicode61'); begin", nullptr, nullptr, nullptr);
            sqlite3_stmt* put = nullptr;
            sqlite3_prepare_v2(db, "insert into t values(?, ?, ?)", -1, &put, nullptr);
            std::istringstream lines(items);
            for (std::string line; std::getline(lines, line);) {
                const size_t a = line.find('\t'), b = a == std::string::npos ? a : line.find('\t', a + 1);
                if (a == std::string::npos) continue;
                sqlite3_bind_text(put, 1, line.substr(0, a).c_str(), -1, SQLITE_TRANSIENT);
                sqlite3_bind_text(put, 2, line.substr(a + 1, b == std::string::npos ? std::string::npos : b - a - 1).c_str(), -1, SQLITE_TRANSIENT);
                sqlite3_bind_text(put, 3, b == std::string::npos ? "" : line.substr(b + 1).c_str(), -1, SQLITE_TRANSIENT);
                sqlite3_step(put), sqlite3_reset(put);
            }
            sqlite3_finalize(put);
            sqlite3_exec(db, "commit", nullptr, nullptr, nullptr);
            built = these;
        }
        std::vector<std::string> each;   // (a word of the query: its English OR'd)
        for (const auto& one : english) {
            std::string any;
            for (std::string w : one) {
                for (size_t at = 0; (at = w.find('"', at)) != std::string::npos; at += 2) w.insert(at, "\"");   // (a quote in a word said twice, as FTS5 reads a string)
                any += (any.empty() ? "\"" : " OR \"") + w + "\"";
            }
            each.push_back("(" + any + ")");
        }
        for (const char* join : {" AND ", " OR "}) {   // (each word found; none so, any)
            std::string match;
            for (const std::string& e : each) match += (match.empty() ? "" : join) + e;
            sqlite3_stmt* ask = nullptr;
            if (sqlite3_prepare_v2(db, "select id, -bm25(t) from t where t match ? order by bm25(t) limit ?", -1, &ask, nullptr) == SQLITE_OK) {
                sqlite3_bind_text(ask, 1, match.c_str(), -1, SQLITE_TRANSIENT);
                sqlite3_bind_int(ask, 2, (int)most);
                char score[32];
                while (sqlite3_step(ask) == SQLITE_ROW) std::snprintf(score, sizeof score, "\t%.2f\n", sqlite3_column_double(ask, 1)), said += std::string((const char*)sqlite3_column_text(ask, 0)) + score;
            }
            sqlite3_finalize(ask);
            if (!said.empty()) return said.c_str();
        }
    }
    const std::u32string q = find_letters(query);
    const rapidfuzz::fuzz::CachedWRatio<char32_t> name(q);
    const rapidfuzz::fuzz::CachedPartialRatio<char32_t> does(q);
    std::vector<std::pair<double, std::string>> found;
    std::istringstream lines(items);
    for (std::string line; std::getline(lines, line);) {
        const size_t a = line.find('\t'), b = a == std::string::npos ? a : line.find('\t', a + 1);
        if (a == std::string::npos) continue;
        const std::u32string about = b == std::string::npos ? U"" : find_letters(line.substr(b + 1));
        found.push_back({std::max(name.similarity(find_letters(line.substr(a + 1, b == std::string::npos ? std::string::npos : b - a - 1))), about.empty() ? 0.0 : does.similarity(about)), line.substr(0, a)});
    }
    std::stable_sort(found.begin(), found.end(), [](const auto& a, const auto& b) { return a.first > b.first; });
    found.resize(std::min(found.size(), most));
    char score[32];
    for (const auto& [at, id] : found) std::snprintf(score, sizeof score, "\t%.1f\n", at), said += id + score;
    return said.c_str();
}
