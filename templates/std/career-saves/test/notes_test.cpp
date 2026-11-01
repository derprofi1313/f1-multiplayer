// SPDX-License-Identifier: MIT
#include <cstdio>

#include <swofty/fake_saves.hpp>

#include "../native/src/notes.hpp"

static int failures = 0;

static void check(bool ok, const char *what)
{
    std::printf("%s  %s\n", ok ? "ok  " : "FAIL", what);
    failures += !ok;
}

int main()
{
    namespace saves = swofty::career::saves;
    {
        notes::on_loaded();
        check(!saves::available(), "without the core's table nothing is written and nothing crashes");
    }

    swofty::fake::Saves fake;
    fake.install();
    const std::string career = "driver_career_notes_2012:7731";

    notes::on_loaded();
    check(saves::data(notes::kAddon).get(career) == std::nullopt, "no career loaded: no data written");

    fake.load(career, "driver_career", "f1_career_notes_2012");
    notes::on_loaded();
    notes::on_loaded();
    check(notes::read(career).loads == 2, "each load adds one to the career's counter");
    auto entry = saves::find(career);
    check(entry && entry->addons.size() == 1 && entry->addons[0].id == notes::kAddon && entry->addons[0].range == "^1.0.0",
          "the index lists the career as needing this addon");

    saves::describe({.round = 7});
    notes::on_saving();
    check(notes::read(career).last_round == 7 && notes::read(career).loads == 2, "career.saving stores the index's round beside the counter");
    auto stored = saves::data(notes::kAddon).get(career);
    check(stored && stored->version == notes::kVersion, "the data is stored at this addon's version");

    saves::data(notes::kAddon).set(career, swofty::json::parse(R"({"loads":5})"), 1);
    check(notes::read(career).loads == 5 && notes::read(career).last_round == 0, "a version 1 file reads with a last round of 0");

    fake.load("driver_career_notes_2012:8800");
    notes::on_loaded();
    check(notes::read("driver_career_notes_2012:8800").loads == 1 && notes::read(career).loads == 5, "careers keep separate data");

    check(notes::on_missing().empty(), "nothing is missing");
    fake.drop(career);
    check(notes::on_missing() == career, "a dropped career is named by career.missing's handler");
    return failures ? 1 : 0;
}
