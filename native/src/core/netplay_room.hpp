#pragma once
#include <string>

#include "netplay_room_logic.hpp"

// Host or join a LAN room with no front-end screens (netplay_room.cpp).
//
// Entry point for the 1.6 direct start (its set_up_target() for the room target), poll style like its other
// targets: call it on the game's flow thread every tick until it returns done or failed.
//
//     Setup runtime::netplay_room::set_up(Role role, const char *host_ip, std::string &why);
//
// The first call checks the whole plan (SWOFTY_NETPLAY armed, the LAN systems reached, every object the
// plan reads readable, no request pending, the address parses) and returns failed with the reason in why
// and the game untouched when something is missing; otherwise it starts the run and returns pending. Each
// further call steps it (netplay_tick steps it too). done means the session is established: the session
// role (*(*(ns+0x418)+8)) is 1 for a host or 2 for a client, or the game entered StateOnlineLobby. Nothing
// here needs the main menu: the objects come from the private LAN item, which the probe in
// netplay_drive.cpp builds from the tile builder's registry and context, or from provide_context below
// when the caller has that pair earlier. host_ip is the host's address for Role::join and ignored for
// Role::host. A host takes its Car Category from set_category (stock F1 2025 by default);
// SWOFTY_ROOM_HOST=screens makes a host stop after the LAN mode switch and report done, for the Car Category
// screen to take over.
//
// set_up_from_home is the same run started from the main menu, for netplay_drive's join command.

namespace runtime::netplay_room {

enum class Phase { idle, running, done, failed };

struct Status {
    Phase phase = Phase::idle;
    std::string detail;
};

enum class Setup { pending, done, failed };

Setup set_up(Role role, const char *host_ip, std::string &why);

// The room target's request (role and host address from the launch JSON), and the poll function for
// direct_start to register: room_poll(why) is set_up with that request, and its Setup has the same three
// values as runtime::direct_start::Setup, so the adapter registered with direct_start_register_room is
// `return Setup(room_poll(why));` once #686 is in.
void set_room_request(Role role, const std::string &host_ip);
Setup room_poll(std::string &why);
bool room_requested();

bool set_up_from_home(Role role, const char *host_ip, std::string &why);
bool provide_context(void *registry, void *context);
void set_category(const Category &category);
Status status();
std::string last_error();
enum class CoverSignal { waiting, shown, failed };
CoverSignal cover_signal();
void tick();

}
