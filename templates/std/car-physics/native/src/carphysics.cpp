// SPDX-License-Identifier: MIT
#include <swofty/events.hpp>
#include <swofty/registry.hpp>
#include <swofty/systems/cars.hpp>

using namespace swofty;
using cars::live::Cars;
using cars::live::Layer;

static Layer lift, ballast;

void make_layers()
{
    if (!lift.ok())
        lift = Layer::scale(Cars::all(), "aero.lift_scale", 0.95f, 0);
    if (!ballast.ok())
        ballast = Layer::add(Cars::player(), "chassis.ballast_kg", 12.0f, 0);
}

void add_wing_pieces(const cars::events::PartsRegister &e)
{
    e.add(cars::Part{.node = "frontwing_fl_03", .detach_at = 0.9f, .debris = true});
}

SWOFTY_REGISTER
{
    events::on<cars::events::Built>().after([](const cars::events::Built &, const events::Context &) { make_layers(); });
    events::on<cars::events::PartsRegister>().after(
        [](const cars::events::PartsRegister &e, const events::Context &) { add_wing_pieces(e); });
}

SWOFTY_ADDON()
