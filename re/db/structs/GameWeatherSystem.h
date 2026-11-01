// struct: GameWeatherSystem
// subsystem: session
// size: ?
// notes: cxx=GameWeatherSystem (logs "GameWeatherSystem::updateRandomSeed", "setupSession"). Reached as
// *(WeatherAndTimeOfDayComponent+0x2a0). Raised by ce-weather.
struct GameWeatherSystem {
    /* 0x1d98 */ TimeOfDaySystem *tod;  // L2 decomp:re/decomp/1438b7350.c;xref:0x1438b74a3,0x1438c81b8 the time-of-day system setupSession configures
    /* 0x1da8 */ TimeOfDayOwner *tod_owner; // L2 decomp:re/decomp/1438b7350.c;xref:0x1438b7703,0x1438b8300 first argument of the dataset build thunks
    /* 0x1f02 */ uint8_t weekend_valid; // L2 decomp:re/decomp/140bde3e0.c;xref:0x140bde4f0,0x1438b764e set by setWeatherSystemRandomSeed (current weekend exists); read by setupSession's first-weekend rule
    /* 0x1f04 */ uint32_t season_index; // L2 decomp:re/decomp/140bde3e0.c;xref:0x140bde4f7,0x1438b765d value of component+0x270 vtable +0xf8 (season, L1 meaning); first-weekend rule needs 0
    /* 0x1f08 */ uint32_t weekend_index; // L2 decomp:re/decomp/140bde3e0.c;xref:0x140bde4fd,0x1438b7667 current weekend index written by setWeatherSystemRandomSeed; first-weekend rule needs 0
    /* 0x1f10 */ uint32_t base_seed;    // L2 decomp:re/decomp/1438c80e0.c;xref:0x1438c8161,0x1438c81d5 base weather seed (= forced_seed when that is non-zero)
    /* 0x1f14 */ uint32_t forced_seed;  // L2 decomp:re/decomp/1438c80e0.c;xref:0x1438c8157 forced seed override honoured by updateRandomSeed
    /* 0x1f18 */ uint32_t weekend;      // L2 decomp:re/decomp/1438c80e0.c;xref:0x1438c80ec,0x1438c81c4 weekend passed to updateRandomSeed (TOD seed = base + weekend*10)
    /* 0x2f3a */ uint8_t dataset_dynamic[8]; // L2 decomp:re/decomp/1438b7350.c;xref:0x1438b7729,0x1438b7fb4 1 = setupSession built the dataset with the realistic generator, 0 = custom/exact
};
