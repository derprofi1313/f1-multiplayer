// struct: DriverContractChangedMessage
// subsystem: career
// size: 0x78 L2 decomp:re/decomp/140d766b0.c (stack message at rsp+0x70: vtable, 0, driver, the record before, the record after, kind at rsp+0xe0)
// notes: MessageDriverContractChanged as Career_Set_DriverContractState (0x140d9de30), the Driver Career signing's add-contract step (0x140d77c20..0x140d77cbb), Career_Sign_DriverToTeam, Career_Sign_DriverContract and 0x140d8b770 build and post it (Ne_Post_BusMessage, dispatched synchronously on the pointer). Raised by ce-numbers (the lead authorised --force).
struct DriverContractChangedMessage {
    /* 0x000 */ void **vtable;          // L2 decomp:re/decomp/140d766b0.c;decomp:re/decomp/140d9de30.c;xref:0x140d77c30,0x140d9de68 vt_Career_MessageDriverContractChanged
    /* 0x008 */ uint64_t reserved;      // L2 decomp:re/decomp/140d766b0.c;xref:0x140d77c27,0x140d9de5f written 0
    /* 0x010 */ uint32_t driver;        // L2 decomp:re/decomp/140d766b0.c;decomp:re/decomp/141143e80.c;xref:0x140d77c3c,0x140d9de74 driver key (DriverContract +0x50); DriverStatisticsComponent's handler matches its driver list on it
    /* 0x014 */ uint8_t before[0x30];   // L2 decomp:re/decomp/140d766b0.c;xref:0x140d77c66,0x140d77c6c,0x140d77c72 the contract record with its active byte (+0x2c): as it was before the signing's copy; Career_Set_DriverContractState takes it after storing the day, reason and value, so there it is the ended record
    /* 0x044 */ uint8_t after[0x2c];    // L2 decomp:re/decomp/140d766b0.c;decomp:re/decomp/140d9de30.c;xref:0x140d77c95,0x140d77c9b,0x140d77ca9,0x140d77c7f the record after: the offer copied in, or an empty record (team 0x3fff0000, days 0xffff, kind and state 0xc) when Career_Set_DriverContractState ends a contract
    /* 0x070 */ uint8_t kind;           // L2 decomp:re/decomp/140d766b0.c;decomp:re/decomp/140d9de30.c;decomp:re/decomp/141143e80.c;xref:0x140d77cad,0x141143e84 the contract reason (enum of DriverContract::kind): the new record's added reason from the signing, the removed reason from Career_Set_DriverContractState, 0 from 0x140d79ff0 and 0x140d8b770; DriverStatisticsComponent and the emails handler act only below 2
};
