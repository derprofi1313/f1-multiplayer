#include "box.hpp"

#include "../applier.hpp"
#include "../scheduler.hpp"

#include <swofty/mixin.hpp>

#include <set>

// track.per_package (Applies::content): at each box press the box's list is built against the effective base and
// its per_package entries along its variant chain, logged as the [tracks] box line, written to catalog.json's
// "tracks" object and kept as the active box for the guard and the fuel writes (data_session.cpp: a Grand Prix
// box's fuel at the press, the session's track again at each session load). The next box press, leaving the box
// and a career unload put the box's fuel writes back in reverse order. A career's own activation lists nothing:
// careers keep their own calendars. With no track base (no track content), no list is built.

namespace swofty_core::apply::tracks {

namespace {

constexpr const char *STOCK = "stock box ";

class PerPackage : public Applier {
public:
    const char *registry() const override { return "track.per_package"; }
    Applies applies() const override { return Applies::content; }

    void activate(Store &store, const Context &ctx, Report &report) override
    {
        if (!base().ready || !ctx.career.empty())
            return;
        if (base().tracks.empty()) {
            if (!empty_logged_) {
                empty_logged_ = true;
                swofty::log("[tracks] no track base (no track content): box track lists are not built");
            }
            return;
        }
        std::string package = ctx.content.empty() ? std::string() : ctx.content.front(), list;
        std::vector<std::string> chain = ctx.content;
        if (package.empty()) {
            std::string label = scheduler().active();
            if (label.rfind(STOCK, 0) != 0)
                return;
            list = label.substr(std::string(STOCK).size());
            chain = {"game/" + list};
        }
        Mode mode = mode_named(ctx.mode);
        Active a;
        a.set = true;
        a.mode = mode;
        a.box = box_info(store, package, list);
        a.package = a.box.package;
        a.entries = box_entries(store, chain);
        List l = box_list(store, a.box, mode, a.entries);
        swofty::log("%s", list_line(a.package, mode, l).c_str());
        for (auto &line : a.entries.lines)
            swofty::log("%s", line.c_str());
        if (!l.no_facts.empty() && logged_.insert(a.package + "|" + mode_name(mode)).second) {
            std::string what;
            for (size_t i = 0; i < l.no_facts.size() && i < 6; ++i)
                what += (i ? ", " : "") + l.no_facts[i];
            if (l.no_facts.size() > 6)
                what += ", " + std::to_string(l.no_facts.size() - 6) + " more";
            swofty::log("[tracks] box %s: no facts (%s); those rules pass", a.package.c_str(), what.c_str());
        }
        write_catalog(a.package, mode, l);
        report.applied += int(a.entries.by_track.size());
        bool own = !package.empty();
        int stat = own ? setup_mode_stat(a.box.setup_mode) : -1;
        set_active(a);
        if (own)
            write_box_fuel_now(a, stat);
    }

    void deactivate(Store &, Report &report) override
    {
        report.put_back += put_back_box_fuel("the box was left");
        clear_active();
    }

private:
    std::set<std::string> logged_;
    bool empty_logged_ = false;
};

}

void register_per_package()
{
    make<PerPackage>();
}

}
