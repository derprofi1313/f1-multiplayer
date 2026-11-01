// struct: WeatherSessionSettings
// subsystem: session
// size: ?
// notes: the session weather settings setupSession reads (rdx): filled from the WeatherAndTimeOfDay
// component by 0x140bde510 (weekend hub) or by the scripted-weather handler 0x141352160; the per-dataset
// slot blocks at +0xb74 + s*0x44 are not listed here. Raised by ce-weather.
struct WeatherSessionSettings {
    /* 0x004 */ uint32_t session_type;  // L2 decomp:re/decomp/1438b7350.c;xref:0x1438b7679 game session type (0x12 time trial takes the single-dataset path)
    /* 0xb14 */ uint32_t mode;          // L2 decomp:re/decomp/1438b7350.c;xref:0x1438b7f60,0x1438b7f81 0 = custom slot blocks, else realistic generator mode
    /* 0xb18 */ uint8_t exact_timing;   // L2 decomp:re/decomp/1438b7350.c;xref:0x1438b7524,0x1438b7697 copied to TimeOfDaySystem.exact_timing; set by the scripted-weather handler
};
