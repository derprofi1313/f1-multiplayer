// struct: CareerDriverContractRenewalComponent
// subsystem: career
// size: ?
// notes: game::championship::CareerDriverContractRenewalComponent (vtable 0x145690a30, both modes). driverMoveData is the game's move log written by Career_Record_DriverMove. Raised by ce-drivers (the lead authorised --force).
struct CareerDriverContractRenewalComponent {
    /* 0x3a1 */ uint8_t disable_driver_moves;        // L2 decomp:re/decomp/140d848f0.c;xref:0x140d88459 save key disableDriverMoves
    /* 0x3a8 */ void *driver_moves_allocator;        // L2 decomp:re/decomp/140d8f1f0.c;xref:0x140d8f29b,0x140d8f313 driverMoveData allocator
    /* 0x3b0 */ DriverMoveRecord *driver_moves_begin; // L2 decomp:re/decomp/140d8f1f0.c;decomp:re/decomp/140d848f0.c;xref:0x140d8f24c,0x140d88886 save key driverMoveData, 0x18 stride
    /* 0x3b8 */ DriverMoveRecord *driver_moves_end;   // L2 decomp:re/decomp/140d8f1f0.c;decomp:re/decomp/140d848f0.c;xref:0x140d8f22e,0x140d8887c end of driverMoveData
    /* 0x3c0 */ uint64_t driver_moves_capacity;      // L2 decomp:re/decomp/140d8f1f0.c;xref:0x140d8f265,0x140d8f331 capacity of driverMoveData
};
