// SPDX-License-Identifier: MIT
#pragma once
#include <map>
#include <string>
#include <swofty/systems/career.hpp>

// A host stand-in for the core's career saves table (SwoftyCareerSaves): the career index and the per-career addon data
// kept in memory, so an addon's tests run career::saves calls without the game.
//
//   swofty::fake::Saves saves;
//   saves.install();
//   saves.load("f1_career_progressive:7731", "driver_career", "f1_career_progressive");
//   career::saves::data("my.addon").set_current(json::parse(R"({"x":1})"));
//   saves.save();                                 // last_saved moves
//   saves.drop("f1_career_progressive:7731");    // the next index lists it as missing

namespace swofty::fake {

class Saves {
public:
    Saves() { self() = this; }
    ~Saves()
    {
        if (self() == this)
            self() = nullptr;
        career::detail::set_saves_table(nullptr);
    }

    void install()
    {
        table_ = {SWOFTY_CAREER_SAVES_VERSION, &Saves::available, &Saves::index, &Saves::current,
                  &Saves::describe,            &Saves::need,      &Saves::data_get, &Saves::data_set,
                  &Saves::data_remove,         &Saves::missing};
        career::detail::set_saves_table(&table_);
    }

    void load(const std::string &id, const std::string &mode = "driver_career", const std::string &content = {})
    {
        json::Value &c = careers_[id];
        c["career"] = json::Value(id);
        c["mode"] = json::Value(mode);
        c["content"] = json::Value(content);
        c["status"] = json::Value("present");
        if (!c.contains("addons"))
            c["addons"] = json::Value::array();
        current_ = id;
    }

    void unload() { current_.clear(); }

    void save()
    {
        if (!current_.empty())
            careers_[current_]["last_saved"] = json::Value(++clock_);
    }

    void drop(const std::string &id)
    {
        auto it = careers_.find(id);
        if (it != careers_.end())
            it->second["status"] = json::Value("missing");
        if (current_ == id)
            current_.clear();
    }

    int64_t clock() const { return clock_; }
    size_t data_files() const { return data_.size(); }

private:
    static Saves *&self()
    {
        static Saves *s = nullptr;
        return s;
    }

    static int copy(const std::string &text, char *out, size_t cap, size_t *len)
    {
        if (len)
            *len = text.size();
        if (!out || cap < text.size() + 1)
            return SWOFTY_TOO_SMALL;
        text.copy(out, text.size());
        out[text.size()] = 0;
        return SWOFTY_ASSET_OK;
    }

    static std::string list(bool only_missing)
    {
        json::Value arr = json::Value::array();
        for (auto &[id, c] : self()->careers_)
            if (!only_missing || c["status"].as_string() == "missing")
                arr.push_back(c);
        return json::dump(json::Value::object({{"version", json::Value(int64_t(1))}, {"careers", arr}}));
    }

    static int available() { return self() ? 1 : 0; }
    static int index(char *out, size_t cap, size_t *len) { return copy(list(false), out, cap, len); }
    static int missing(char *out, size_t cap, size_t *len) { return copy(list(true), out, cap, len); }

    static int current(char *out, size_t cap, size_t *len)
    {
        if (self()->current_.empty())
            return SWOFTY_ASSET_LATE;
        return copy(self()->current_, out, cap, len);
    }

    static int describe(SwoftyText request)
    {
        Saves *s = self();
        if (s->current_.empty())
            return SWOFTY_ASSET_LATE;
        json::Value v = json::parse(std::string_view(request.data, request.size));
        if (!v.is_object())
            return SWOFTY_ASSET_INVALID;
        json::Value &c = s->careers_[s->current_];
        for (const char *key : {"driver", "team", "label", "season", "round"})
            if (v.contains(key))
                c[key] = v[key];
        return SWOFTY_ASSET_OK;
    }

    static int need(const char *addon, const char *range, const char *why)
    {
        Saves *s = self();
        if (!addon || !*addon)
            return SWOFTY_ASSET_INVALID;
        if (s->current_.empty())
            return SWOFTY_ASSET_LATE;
        json::Value &list = s->careers_[s->current_]["addons"];
        json::Value entry = json::Value::object({{"id", json::Value(addon)}});
        if (range && *range)
            entry["need"] = json::Value(range);
        if (why && *why)
            entry["why"] = json::Value(why);
        if (auto *a = list.if_array())
            for (json::Value &e : *a)
                if (e["id"].as_string() == addon) {
                    e = entry;
                    return SWOFTY_ASSET_OK;
                }
        list.push_back(entry);
        return SWOFTY_ASSET_OK;
    }

    static std::string key(const char *career, const char *addon) { return std::string(career) + "|" + addon; }

    static int data_get(const char *career, const char *addon, char *out, size_t cap, size_t *len, uint32_t *version)
    {
        auto it = self()->data_.find(key(career, addon));
        if (it == self()->data_.end())
            return SWOFTY_ASSET_INVALID;
        if (version)
            *version = it->second.first;
        return copy(it->second.second, out, cap, len);
    }

    static int data_set(const char *career, const char *addon, uint32_t version, SwoftyText text)
    {
        Saves *s = self();
        if (!s->careers_.count(career))
            return SWOFTY_REFUSED;
        json::Error e;
        json::Value v = json::parse(std::string_view(text.data, text.size), &e);
        if (!e.ok())
            return SWOFTY_ASSET_INVALID;
        s->data_[key(career, addon)] = {version, json::dump(v)};
        return SWOFTY_ASSET_OK;
    }

    static int data_remove(const char *career, const char *addon)
    {
        self()->data_.erase(key(career, addon));
        return SWOFTY_ASSET_OK;
    }

    SwoftyCareerSaves table_{};
    std::map<std::string, json::Value> careers_;
    std::map<std::string, std::pair<uint32_t, std::string>> data_;
    std::string current_;
    int64_t clock_ = 1000;
};

}
