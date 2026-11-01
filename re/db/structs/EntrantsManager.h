// struct: EntrantsManager
// subsystem: session
// size: ?
// notes: mgr = *(this+0x278) in Sess_Setup_ChampionshipEntrants
struct EntrantsManager {
    /* 0x2b0 */ GameSetupMode *game_setup_mode; // L2 decomp:re/decomp/14079f300.c;decomp:re/decomp/142f45960.c;decomp:re/decomp/1424b2c90.c;xref:0x148596e7f,0x148596e72,0x142f45d91,0x1424b2d20,0x1424b3023,0x1424b322f the current game setup mode: set by 0x14079f300 (thunk to 0x148596c20, logs 'trying to set invalid game setup mode %s' with the mode name at +0xd8 -> +0x10), which then resolves setup_mode (+0xe10) and selectable_list (+0xe18) from it; read by Sess_Setup_VehicleComponents through [[module+0x80]+0x88]+0x260; read by the DRS system through [[drs+0xb0]+0x88]+0x260 (0x142bebf3f, 0x142becd45, 0x142bef8f7, 0x142befa63)
    /* 0x2e0 */ EntrantRecord *session_begin; // L2 decomp:re/decomp/14245fb70.c;doc:research/grid24-cap.md#2 selector output vector
    /* 0x2e8 */ EntrantRecord *session_end; // L2 decomp:re/decomp/14245fb70.c;doc:research/grid24-cap.md#2 cleared to begin before selection
    /* 0xe10 */ EntrantSetupMode *setup_mode; // L2 decomp:re/decomp/140ba3ad0.c;doc:research/grid24-cap.md#2
    /* 0xe18 */ void *selectable_list;  // L2 decomp:re/decomp/140ba3ad0.c;doc:research/grid24-cap.md#2 first selectable list; records at +0x10..+0x18
};
