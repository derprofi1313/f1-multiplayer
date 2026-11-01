// struct: DriverMoveConfirmedMessage
// subsystem: career
// size: 0x28 L2 decomp:re/decomp/140d766b0.c (stack message: vtable, 0, then the 0x18-byte move)
// notes: MessageDriverMoveConfirmed as Career_Record_DriverMove and the Driver Career signing 0x140d766b0 build and post it (Ne_Post_BusMessage). Subscribed by the Upgrades component (handler 0x14352e370, which reassigns per-driver upgrade state) and telemetry. Raised by ce-drivers (the lead authorised --force).
struct DriverMoveConfirmedMessage {
    /* 0x000 */ void **vtable;           // L2 decomp:re/decomp/140d766b0.c;decomp:re/decomp/140d8f1f0.c;xref:0x140d76f65,0x140d76f6c,0x140d8f385 vt_Career_MessageDriverMoveConfirmed
    /* 0x008 */ uint64_t reserved;       // L2 decomp:re/decomp/140d766b0.c;xref:0x140d76f5c written 0
    /* 0x010 */ DriverMoveRecord move;   // L2 decomp:re/decomp/140d766b0.c;decomp:re/decomp/140d8f1f0.c;xref:0x140d76f71,0x140d76f78,0x140d76f7f the move: driver, old team, new team, reason, weekend, day
};
