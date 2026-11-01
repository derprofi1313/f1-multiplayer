#include <swofty/career.hpp>
#include <swofty/events.hpp>
#include <swofty/mixin.hpp>

SWOFTY_SETTING(bool, verbose, false);
SWOFTY_SETTING(int, greeting_count, 1);
SWOFTY_SETTING(std::string_view, greeting, "hello");

SWOFTY_ON_CHANGE(verbose, bool on)
{
    swofty::log("[sample] verbose logging %s", on ? "on" : "off");
}

SWOFTY_ON_CHANGE(greeting, std::string_view text)
{
    if (verbose)
        swofty::log("[sample] greeting is now \"%.*s\"", int(text.size()), text.data());
}

void on_career_loaded(const SwoftyEvent *ev, void *)
{
    auto *s = static_cast<const SwoftyCareerState *>(ev->data);
    swofty::log("[sample] career loaded: %s %s", swofty::career::name(swofty::career::mode(*s)), s->key);
}

SWOFTY_ON_LOAD
{
    if (swofty::career::available())
        swofty::events::subscribe(SWOFTY_EVENT_CAREER_LOADED, on_career_loaded);
    std::string_view text = greeting;
    for (int i = 0; i < greeting_count; i++)
        swofty::log("[sample] %.*s from the sample mod", int(text.size()), text.data());
    uintptr_t hit = 0;
    int n = swofty::api->scan(".xtls", "48 89 5C 24 ?? 57 48 83 EC 20", &hit);
    if (verbose)
        swofty::log("[sample] example signature: %d hit(s), first at %p", n, reinterpret_cast<void *>(hit));
}

SWOFTY_ADDON()
