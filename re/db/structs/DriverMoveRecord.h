// struct: DriverMoveRecord
// subsystem: career
// size: 0x18 L2 decomp:re/decomp/140d8f1f0.c (append step 0x18, (end-begin)>>3 * -0x5555555555555555)
// notes: one driverMoveData entry (save keys driverMoveDriverStatId / NewTeamStatId / OldTeamStatId / Reason / weekendIndex / careerDayIndex, deserialiser 0x140d848f0); written by Career_Record_DriverMove (verify V0280). Raised by ce-drivers (the lead authorised --force).
struct DriverMoveRecord {
    /* 0x000 */ uint32_t driver;        // L2 decomp:re/decomp/140d8f1f0.c;xref:0x140d8f33a driver key (u8 kind, u16 stat id at +2)
    /* 0x004 */ uint32_t old_team;      // L2 decomp:re/decomp/140d8f1f0.c;xref:0x140d8f33c team key the driver left
    /* 0x008 */ uint32_t new_team;      // L2 decomp:re/decomp/140d8f1f0.c;xref:0x140d8f33f team key the driver joined
    /* 0x00c */ uint8_t reason;         // L2 decomp:re/decomp/140d8f1f0.c;xref:0x140d8f342 driverMoveReason (0 signing, 2 reserve backfill, 3 benched)
    /* 0x010 */ uint32_t weekend_index; // L2 decomp:re/decomp/140d8f1f0.c;xref:0x140d8f346 vtable +0x108 of [self+0x270] (call 0x140d8f21e)
    /* 0x014 */ uint16_t day_index;     // L2 decomp:re/decomp/140d8f1f0.c;xref:0x140d8f34a clock_minutes / 1440 (0x140d8f23e)
};
