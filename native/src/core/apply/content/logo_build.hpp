#pragma once

#include "erp_file.hpp"
#include "logo_array.hpp"
#include "uidb.hpp"

#include <functional>
#include <map>
#include <memory>
#include <optional>
#include <set>
#include <string>
#include <vector>

// The archives a set of team logos changes. Each logo is a slice for one stock texture array (an encoded image, or a
// copy of the slice a stock key already has) under a key in overrides.uidb. The result names, for every archive in
// the game's ui_package folder that carries a changed array and for databases.erp, the fragments to replace, and the
// archives whose contents it read: databases.erp and every archive that carries a wanted array.

namespace swofty_core::apply::content::logo_build {

inline constexpr const char *UI_FOLDER = "2025_asset_groups/ui_package/";
inline constexpr const char *DATABASES = "2025_asset_groups/ui_package/databases.erp";
inline constexpr const char *OVERRIDES = "eaid://ui_package/databases/overrides.uidb";
inline constexpr const char *ARRAY_URI = "eaid://ui_package/art/texture_arrays/overrides.uidb/overrides.";
inline constexpr const char *TEXTURE_PATH = "ui_package/art/textures/swofty/team_logos/";

struct Want {
    std::string array, key, donor;
    std::string rgba;
    uint32_t width = 0, height = 0;
    bool fallback = false;
};

struct Env {
    std::function<std::vector<std::string>()> list;
    std::function<std::shared_ptr<erpfile::Source>(const std::string &rel)> open;
    std::function<std::vector<std::string>(const std::string &rel)> overlays;
};

struct Result {
    std::map<std::string, erpfile::Changes> changes;
    std::set<std::string> read;
    std::vector<std::string> lines;
    int built = 0, refused = 0;
};

struct Refusals {
    std::map<std::string, std::pair<int, std::string>> by;

    void add(const std::string &reason, const std::string &example, int n = 1)
    {
        auto &r = by[reason];
        if (!r.first)
            r.second = example;
        r.first += n;
    }

    void flush(std::vector<std::string> &lines) const
    {
        for (auto &[reason, r] : by)
            lines.push_back("team.logo: " + std::to_string(r.first) + " slot(s) refused: " + reason + " (first " + r.second + ")");
    }
};

inline std::string list_path(const std::string &array)
{
    std::string path = "overrides.Contents.", rest = array;
    size_t dot;
    while ((dot = rest.find('.')) != std::string::npos) {
        path += rest.substr(0, dot) + ".Contents.";
        rest = rest.substr(dot + 1);
    }
    return path + rest + ".Contents";
}

inline bool build(const Env &env, const std::vector<Want> &wants, Result &out)
{
    using erpfile::Archive;
    struct Open {
        std::shared_ptr<erpfile::Source> src;
        Archive arc;
    };
    std::map<std::string, Open> opened;
    std::string why;
    Refusals refusals;
    struct Flush {
        Refusals &r;
        Result &out;
        ~Flush() { r.flush(out.lines); }
    } flush{refusals, out};
    auto refuse = [&](const std::string &reason, const std::string &example, int n = 1) {
        out.refused += n;
        refusals.add(reason, example, n);
    };
    auto get = [&](const std::string &rel) -> Open * {
        auto it = opened.find(rel);
        if (it != opened.end())
            return &it->second;
        auto src = env.open(rel);
        Open o;
        if (!src || !erpfile::open(*src, o.arc, why))
            return nullptr;
        o.src = src;
        return &opened.emplace(rel, std::move(o)).first->second;
    };
    auto layers_of = [&](const std::string &rel) {
        std::vector<std::string> layers{rel};
        if (env.overlays)
            for (auto &o : env.overlays(rel))
                if (std::find(layers.begin(), layers.end(), o) == layers.end())
                    layers.push_back(o);
        return layers;
    };
    struct Found {
        Open *open = nullptr;
        const erpfile::Entry *entry = nullptr;
    };
    auto top = [&](const std::vector<std::string> &layers, const std::string &name) {
        Found f;
        for (auto &l : layers) {
            Open *o = get(l);
            const erpfile::Entry *e = o ? o->arc.find(name) : nullptr;
            if (e)
                f = {o, e};
        }
        return f;
    };
    auto carrying = [&](const std::vector<std::string> &layers, const std::string &name) {
        std::vector<std::string> out_layers;
        for (auto &l : layers) {
            Open *o = get(l);
            if (o && o->arc.find(name))
                out_layers.push_back(l);
        }
        return out_layers;
    };
    std::vector<std::string> db_layers = layers_of(DATABASES);
    Open *db = get(DATABASES);
    if (db)
        out.read.insert(DATABASES);
    Found uf = db ? top(db_layers, OVERRIDES) : Found{};
    const erpfile::Entry *ue = uf.entry;
    std::string uidb_bytes;
    if (!ue || ue->frags.empty() || !erpfile::fragment(*uf.open->src, uf.open->arc, ue->frags[0], uidb_bytes, why)) {
        refuse("overrides.uidb is not in " + std::string(DATABASES) + (why.empty() ? "" : " (" + why + ")"),
               wants.empty() ? "" : wants[0].array + "/" + wants[0].key, int(wants.size()));
        return false;
    }
    uidb::Pkg pkg;
    if (!pkg.read(uidb_bytes)) {
        refuse("overrides.uidb does not read: " + pkg.why, wants.empty() ? "" : wants[0].array + "/" + wants[0].key, int(wants.size()));
        return false;
    }
    std::vector<std::string> files = env.list();
    std::map<std::string, std::vector<const Want *>> by_array;
    for (auto &w : wants)
        by_array[w.array].push_back(&w);
    bool uidb_changed = false;
    std::set<std::string> viewed;
    for (auto &[array, list] : by_array) {
        uidb::Node *node = pkg.find(list_path(array));
        if (!node || node->kind != uidb::Node::array) {
            refuse("overrides.uidb has no list " + array, array + "/" + list[0]->key, int(list.size()));
            continue;
        }
        std::vector<std::string> names;
        for (auto &e : node->list)
            names.push_back(uidb::Pkg::text_of(e, "Name"));
        std::string image = ARRAY_URI + array + ".texarray.image", view = ARRAY_URI + array + ".texarray";
        std::vector<std::string> carriers;
        logo_array::Stock stock;
        bool have = false, mismatch = false;
        std::map<std::string, std::string> repaired;
        std::map<std::string, std::vector<std::string>> image_in, view_in;
        for (auto &rel : files) {
            if (!get(rel))
                continue;
            std::vector<std::string> layers = layers_of(rel);
            Found fi = top(layers, image), fv = top(layers, view);
            if (!fi.entry && !fv.entry)
                continue;
            for (auto &l : carrying(layers, image))
                image_in[rel].push_back(l), out.read.insert(l);
            for (auto &l : carrying(layers, view))
                view_in[rel].push_back(l), out.read.insert(l);
            out.read.insert(rel);
            const erpfile::Entry *ei = fi.entry, *ev = fv.entry;
            logo_array::Stock got;
            if (!ei || !ev || ei->frags.size() != 2 || ev->frags.size() != 1 ||
                !erpfile::fragment(*fi.open->src, fi.open->arc, ei->frags[0], got.head, why) ||
                !erpfile::fragment(*fi.open->src, fi.open->arc, ei->frags[1], got.data, why) ||
                !erpfile::fragment(*fv.open->src, fv.open->arc, ev->frags[0], got.view, why)) {
                out.lines.push_back("team.logo: " + rel + " carries " + array + " in a layout this build does not read");
                mismatch = true;
                continue;
            }
            uint32_t said = 0;
            if (logo_array::repair_view(got, said)) {
                repaired[rel] = got.view;
                if (viewed.insert(array).second)
                    out.lines.push_back("team.logo: " + array + " view said " + std::to_string(said) + " slices for " +
                                        std::to_string(logo_array::le32(got.head, logo_array::HEAD_COUNT)) + "; fixed");
            }
            if (have && (got.head != stock.head || got.data != stock.data || got.view != stock.view)) {
                out.lines.push_back("team.logo: " + rel + " carries a different " + array + " than " + carriers[0]);
                mismatch = true;
                continue;
            }
            stock = std::move(got);
            have = true;
            carriers.push_back(rel);
        }
        for (auto &[rel, bytes] : repaired)
            for (auto &l : view_in[rel])
                out.changes[l][view] = {bytes};
        stock.names = names;
        logo_array::Shape shape;
        bool sound = !mismatch && have && logo_array::shape_of_images(stock, shape, why);
        if (sound && names.size() > shape.count) {
            size_t dangling = names.size() - shape.count;
            std::string blank = bc7::encode_logo(reinterpret_cast<const uint8_t *>(std::string(shape.slice_bytes() * 4, '\0').data()),
                                                 int(shape.width), int(shape.height));
            out.lines.push_back("team.logo: " + array + " lists " + std::to_string(names.size()) + " keys but its image holds " +
                                std::to_string(shape.count) + " slice(s): another addon replaced the whole array, so " +
                                std::to_string(dangling) + " key(s) from " + names[shape.count] + " on have no image of their own; padded with blank slices so new logos land on their own keys");
            logo_array::pad(stock, shape, blank, uint32_t(names.size()));
            shape.count = uint32_t(names.size());
        }
        if (sound)
            sound = logo_array::shape_of(stock, shape, why);
        if (!sound) {
            std::string reason = mismatch ? "the carriers of " + array + " do not agree" : !have ? array + " is in no archive of the ui_package folder" : array + ": " + why;
            refuse(reason, array + "/" + list[0]->key, int(list.size()));
            continue;
        }
        std::vector<logo_array::Slice> add;
        std::set<std::string> seen(names.begin(), names.end());
        std::vector<const Want *> used;
        for (auto *w : list) {
            if (!seen.insert(w->key).second) {
                out.lines.push_back("team.logo: " + array + " lists " + w->key + " already; its slice is kept");
                continue;
            }
            logo_array::Slice s{w->key, {}};
            if (!w->donor.empty()) {
                auto at = std::find(names.begin(), names.end(), w->donor);
                if (at == names.end()) {
                    out.lines.push_back("team.logo: " + array + " has no stock key " + w->donor + " for " + w->key +
                                        (w->fallback ? "; the donor fallback is left out" : ""));
                    if (!w->fallback)
                        refuse(array + " has no stock key to copy", w->donor + " for " + w->key);
                    seen.erase(w->key);
                    continue;
                }
                s.bc7 = stock.data.substr(size_t(at - names.begin()) * shape.slice_bytes(), shape.slice_bytes());
            } else {
                bool scaled = false;
                if (!logo_array::slice_from_rgba(shape, reinterpret_cast<const uint8_t *>(w->rgba.data()), w->width, w->height, s.bc7,
                                                 why, &scaled)) {
                    refuse(array + ": " + why, w->key);
                    seen.erase(w->key);
                    continue;
                }
                if (scaled)
                    out.lines.push_back("team.logo: " + w->key + " scaled " + std::to_string(w->width) + "x" + std::to_string(w->height) +
                                        " -> " + std::to_string(shape.width) + "x" + std::to_string(shape.height) + " for " + array);
            }
            add.push_back(std::move(s));
            used.push_back(w);
        }
        if (add.empty())
            continue;
        logo_array::Stock grown;
        if (!logo_array::grow(stock, add, grown, why)) {
            refuse(array + ": " + why, array + "/" + add[0].key, int(add.size()));
            continue;
        }
        for (auto &rel : carriers) {
            for (auto &l : image_in[rel])
                out.changes[l][image] = {grown.head, grown.data};
            for (auto &l : view_in[rel])
                out.changes[l][view] = {grown.view};
        }
        for (auto &s : add) {
            std::string e;
            if (!pkg.append_entry(*node, s.key, std::string(TEXTURE_PATH) + s.key + ".tif", e)) {
                refuse(array + ": " + e, s.key);
            }
        }
        uidb_changed = true;
        out.built += int(add.size());
        out.lines.push_back("team.logo: " + array + " grows by " + std::to_string(add.size()) + " slice(s) in " +
                            std::to_string(carriers.size()) + " archive(s)");
    }
    if (uidb_changed) {
        std::string written = pkg.write();
        for (auto &l : carrying(db_layers, OVERRIDES)) {
            out.changes[l][OVERRIDES] = {written};
            out.read.insert(l);
        }
    }
    return true;
}

}
