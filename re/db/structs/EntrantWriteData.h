// struct: EntrantWriteData
// subsystem: entrants
// size: 0x2c L2 decomp:re/decomp/148618ee0.c (the writer reads dwords +0x00..+0x18 and +0x28)
// notes: the data block Career_Set_EntrantData_Thunk copies into an EntrantRecord (+0x50.. and +0x78). Career_Sign_DriverToTeam and Career_Process_PendingDriverMoves build it on the stack. Raised by ce-drivers (the lead authorised --force).
struct EntrantWriteData {
    /* 0x000 */ uint32_t driver;                // L2 decomp:re/decomp/148618ee0.c;xref:0x148619035 -> EntrantRecord +0x50
    /* 0x004 */ uint32_t driver_customisation;  // L2 decomp:re/decomp/148618ee0.c;xref:0x14861903b -> +0x54
    /* 0x008 */ uint32_t team_stat_id;          // L2 decomp:re/decomp/148618ee0.c;xref:0x148619042 -> +0x58
    /* 0x00c */ uint32_t vehicle_ref;           // L2 decomp:re/decomp/148618ee0.c;xref:0x148619049 -> +0x5c
    /* 0x010 */ uint32_t driver_reg_value;      // L2 decomp:re/decomp/148618ee0.c;xref:0x148619050 -> +0x60
    /* 0x014 */ uint32_t vehicle_customisation; // L2 decomp:re/decomp/148618ee0.c;xref:0x148619057 -> +0x64
    /* 0x018 */ uint32_t badge_stat_id;         // L2 decomp:re/decomp/148618ee0.c;xref:0x14861905e -> +0x68
    /* 0x028 */ uint32_t seat_value;            // L2 decomp:re/decomp/148618ee0.c;xref:0x148619070 -> +0x78
};
