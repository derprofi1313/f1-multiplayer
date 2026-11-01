// struct: WeatherState
// subsystem: session
// size: 0x10 L2 decomp:re/decomp/14461f400.c;xref:0x14461f7d3 (TOD update indexes states with shl 4)
// notes: one weather state of a TimeOfDaySystem dataset (WeatherDataset.states). The state holds `type`
// until `end` (fraction of the dataset length), then blends to the next state over `transition`. The
// exact-timing path of Tod_Build_WeatherScenario 0x14461bea0 writes end = (sum + hold) / len and
// transition = trans / len. Raised by ce-weather (career-editor weather.cpp).
struct WeatherState {
    /* 0x000 */ uint32_t type;          // L2 decomp:re/decomp/14461bea0.c;xref:0x14461c8d0 engine weather type 0 clear, 1 light_cloud, 2 overcast, 3 light_rain, 4 heavy_rain, 5 stormy (names 0x145bf6358); indexes WeatherTypeRange
    /* 0x004 */ float intensity;        // L2 decomp:re/decomp/14461bea0.c;xref:0x14461c8b3 drawn by the builder from WeatherTypeRange[type] min..max with the session seed
    /* 0x008 */ float end;              // L2 decomp:re/decomp/14461f400.c;xref:0x14461f7d7,0x14461c9a0 fraction where the hold ends and the transition to the next state starts
    /* 0x00c */ float transition;       // L2 decomp:re/decomp/14461f400.c;xref:0x14461f7ed,0x14461c9b5 transition length as a fraction of the dataset length
};
