// struct: DriverMovesFinalisedMessage
// subsystem: career
// size: 0x298 L2 decomp:re/decomp/140d766b0.c (count at stack +0x290 from the message start, posted after the payload copy)
// notes: MessageDriverMovesFinalised, MessageDriverMovesFinalised_upgrades and MessageDriverMovesFinalised_recognition share this layout; the payload (changes + count) is what Career_Build_LineupChanges writes. Raised by ce-drivers (the lead authorised --force).
struct DriverMovesFinalisedMessage {
    /* 0x000 */ void **vtable;            // L2 decomp:re/decomp/140d766b0.c;xref:0x140d78522,0x140d7858e,0x140d785b8 one of the three finalised vtables
    /* 0x008 */ uint64_t reserved;        // L2 decomp:re/decomp/140d766b0.c;xref:0x140d785af written 0
    /* 0x010 */ LineupChange changes[40]; // L2 decomp:re/decomp/14123b3f0.c;decomp:re/decomp/142457a90.c;xref:0x14123b430 the changed drivers
    /* 0x290 */ uint64_t count;           // L2 decomp:re/decomp/14123b3f0.c;decomp:re/decomp/142457a90.c;xref:0x14123b420,0x142457aad number of changes (builder +0x280)
};
