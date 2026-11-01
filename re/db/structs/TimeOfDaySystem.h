// struct: TimeOfDaySystem
// subsystem: session
// size: ?
// notes: cxx=TimeOfDaySystem (logs "TimeOfDaySystem::update", "WEATHER_LOGIC"). Reached as
// *(GameWeatherSystem+0x1d98) and as *(TimeOfDayOwner+0x2d0). Holds the per-session weather datasets,
// their clocks and lengths, the per-type intensity ranges and the build flags. Raised by ce-weather.
struct TimeOfDaySystem {
    /* 0x2c8 */ TimeOfDayClock clocks[10]; // L2 decomp:re/decomp/14461f400.c;xref:0x14461f5be,0x14461f739 one clock per dataset (0x2c8 + d*0x288)
    /* 0x2654 */ uint8_t scripted_scenario; // L2 decomp:re/decomp/155254f20.c;xref:0x1552550e7 with override_active set and random_override clear, the build body runs the scripted-scenario build 0x14461b3d0 (scenario from +0x2648) instead of the builder
    /* 0x26c4 */ uint8_t active_dataset; // L2 decomp:re/decomp/14461f400.c;xref:0x14461f705,0x1438b7841 dataset the update plays (0..7 sessions, 8 photo mode, 9 override)
    /* 0x26d8 */ WeatherDataset datasets[10]; // L2 decomp:re/decomp/14461f400.c;xref:0x14461f72e,0x14461c0c8 the ten weather datasets (0x26d8 + d*0xe00)
    /* 0xb33c */ uint32_t seed;         // L2 decomp:re/decomp/14461bea0.c;xref:0x14461c827 weather seed (base + weekend*10 from updateRandomSeed); the builder seeds per dataset with seed + d*k
    /* 0xb350 */ float dataset_length[10]; // L2 decomp:re/decomp/14461f400.c;xref:0x14461f764,0x1557555f4 dataset length in weather seconds, set by the build thunk body
    /* 0xb378 */ WeatherTypeRange type_ranges[6]; // L2 decomp:re/decomp/14461bea0.c;xref:0x14461c0fc,0x1438b74c1 per weather type weight and intensity range
    /* 0xb426 */ uint8_t exact_timing;  // L2 decomp:re/decomp/14461bea0.c;xref:0x14461c4e6,0x1438b752b copied from settings+0xb18; the builder then takes hold/transition seconds as given
    /* 0xb429 */ uint8_t ready_a;       // L2 decomp:re/decomp/155254f20.c;xref:0x155254f53,0x14461f45b the build and the update do nothing while clear
    /* 0xb42a */ uint8_t ready_b;       // L2 decomp:re/decomp/155254f20.c;xref:0x155254f5c,0x14461f46b the build and the update do nothing while clear
    /* 0xb42d */ uint8_t random_override; // L2 decomp:re/decomp/155254f20.c;xref:0x155254fed with override_active set, the build body makes a random 1..6-state dataset with null hold/trans instead of the given states
    /* 0xb432 */ uint8_t busy;          // L2 decomp:re/decomp/155254f20.c;xref:0x155254f33,0x14461bf0d the build does nothing while set
    /* 0xb436 */ uint8_t override_active; // L2 decomp:re/decomp/155254f20.c;xref:0x155254fe0 gate for the override paths: diverts only together with random_override or scripted_scenario (V0313); with both clear the normal build at 0x155255140 runs
};
