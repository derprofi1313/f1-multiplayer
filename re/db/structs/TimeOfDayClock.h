// struct: TimeOfDayClock
// subsystem: session
// size: 0x288 L2 decomp:re/decomp/14461f400.c;xref:0x14461f739 (imul 0x288 by the active dataset index)
// notes: per-dataset time-of-day clock at TimeOfDaySystem+0x2c8 (10 of them, one per WeatherDataset). The
// TOD update computes weather progress = (time_a + time_b) * time_scale / dataset_length and clamps it to
// 0..1. Raised by ce-weather.
struct TimeOfDayClock {
    /* 0x068 */ double time_a;          // L2 decomp:re/decomp/14461f400.c;xref:0x14461f740 first term of the elapsed weather time
    /* 0x070 */ double time_b;          // L2 decomp:re/decomp/14461f400.c;xref:0x14461f747 second term of the elapsed weather time
    /* 0x080 */ float time_scale;       // L2 decomp:re/decomp/14461f400.c;xref:0x14461f74e multiplier on the elapsed time
    /* 0x084 */ float length;           // L2 decomp:re/decomp/14461bea0.c;xref:0x14461c0b1,0x155755609 max(dataset length, 0.01) copied by the builder and the build thunk body
};
