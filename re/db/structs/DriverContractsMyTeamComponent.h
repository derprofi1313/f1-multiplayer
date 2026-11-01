// struct: DriverContractsMyTeamComponent
// subsystem: career
// size: ?
// notes: game::championship::DriverContractsMyTeamComponent (vtable 0x145692330, My Team). Four contract/move lists, each {allocator, begin, end, cap} of ContractRecord, passed by the serialiser 0x140da41d0 to Career_Serialise_DriverContractList at +0x790/+0x7b0/+0x7d0/+0x7f0 (which reads list+8/list+0x10). Raised by ce-drivers; car_numbers raised by ce-swap-number (the lead authorised --force).
struct DriverContractsMyTeamComponent {
    /* 0x2e8 */ CarNumberComponent *car_numbers;              // L2 decomp:re/decomp/140d9fc00.c;decomp:re/decomp/140db70a0.c;xref:0x140d9ffe5,0x140db740d the CarNumberComponent: vtable slot +0x78 (0x140d9fc00) stores the entity component registry get with its descriptor 0x14605ade0 here (0x140d9ffe5); Career_Assign_EntrantTeamSlot passes it to Career_DriverNumbers_CheckExisting for a row whose +0x83 is 0xff after its entrant write (read 0x140db740d, call 0x140db7418)
    /* 0x798 */ ContractRecord *pending_contracts_begin;    // L2 decomp:re/decomp/140da5610.c;xref:0x140da5245,0x140da571e,0x140db7a4c,0x140d9fae3 pendingDriverContracts (list +0x790; 0x140db7a10 walks it, 0x70 step) (V0660)
    /* 0x7a0 */ ContractRecord *pending_contracts_end;      // L2 decomp:re/decomp/140da5610.c;xref:0x140da5245,0x140da5722,0x140db7a58 end of pendingDriverContracts (V0661)
    /* 0x7b8 */ ContractRecord *decided_contracts_begin;    // L2 decomp:re/decomp/140da5610.c;xref:0x140da5265,0x140da571e,0x140db7c35 decidedDriverContracts (list +0x7b0) (V0662)
    /* 0x7c0 */ ContractRecord *decided_contracts_end;      // L2 decomp:re/decomp/140da5610.c;xref:0x140da5265,0x140da5722,0x140db7c3c end of decidedDriverContracts (V0663)
    /* 0x7d8 */ ContractRecord *pending_moves_begin;        // L2 decomp:re/decomp/140da5610.c;decomp:re/decomp/140db18e0.c;xref:0x140da5225,0x140db1abb,0x140db3b82 pendingDriverMoves (list +0x7d0), walked by Career_Process_PendingDriverMoves
    /* 0x7e0 */ ContractRecord *pending_moves_end;          // L2 decomp:re/decomp/140db18e0.c;xref:0x140db1ac3,0x140db3b8a end of pendingDriverMoves
    /* 0x7f8 */ ContractRecord *last_season_signings_begin; // L2 decomp:re/decomp/140da5610.c;xref:0x140da5285,0x140da571e,0x140db26ad lastSeasonsSignings (list +0x7f0) (V0666)
    /* 0x800 */ ContractRecord *last_season_signings_end;   // L2 decomp:re/decomp/140da5610.c;xref:0x140da5285,0x140da5722,0x140db26b5 end of lastSeasonsSignings (V0667)
};
