// SPDX-License-Identifier: MIT
#include <string>

#include <swofty/fake_core.hpp>

#include "../native/src/track.cpp"
#include "hockenheim.hpp"

using namespace hockenheim_test;

static const swofty::fake::Record *recorded(const swofty::fake::Core &core, std::string_view op, std::string_view id)
{
    for (const auto &r : core.records())
        if (r.op == op && r.id == id)
            return &r;
    return nullptr;
}

int main()
{
    swofty::fake::Core probe{"mod.hockenheim"};
    if (!probe.rules()) {
        probe.run_register();
        const auto *track = recorded(probe, "add", "mod.hockenheim/hockenheim");
        check(track && track->registry == "track", "adds the track mod.hockenheim/hockenheim");
        check(track && track->value["base"].as_string() == "game/austria", "cloned from game/austria");
        auto layers = probe.layers("track.data");
        check(layers.size() == 5, "five layers on track.data");
        std::printf("skip  effective values: no registry engine linked\n");
        return failures ? 1 : 0;
    }

    auto &st = fresh_store({"mod.hockenheim"});
    core_blocks(st);
    swofty::fake::Core core{"mod.hockenheim"};
    core.run_register();
    st.end_register();
    for (auto &r : st.refusals())
        check(false, "refused: " + r.registry + " " + r.id + ": " + r.message);
    check_hockenheim(st, "code");
    return failures ? 1 : 0;
}
