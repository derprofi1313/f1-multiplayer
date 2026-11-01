// struct: WeatherAndTimeOfDayComponent
// subsystem: session
// size: ?
// notes: cxx=WeatherAndTimeOfDayComponent (career/GP component; deserialiser 0x140be23c0 reads rnd_seed,
// w_type, w_quick, w_*_active, w_*_slots). Raised by ce-weather.
struct WeatherAndTimeOfDayComponent {
    /* 0x2a0 */ GameWeatherSystem *weather_system; // L2 decomp:re/decomp/140bde3e0.c;xref:0x140bde4d4,0x140bde4e9 the game weather system the seed goes to
    /* 0x2c4 */ uint32_t random_seed;   // L2 decomp:re/decomp/140bde3e0.c;xref:0x140bde4db,0x140bde4ba rnd_seed (saved); passed to updateRandomSeed by setWeatherSystemRandomSeed
};
