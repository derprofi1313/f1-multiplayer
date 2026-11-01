// struct: DriverCareerContractOwner
// subsystem: career
// size: ?
// notes: the object at DriverCareerContractsManagerComponent::contract_owner (class not named yet); its contract list is searched by the Driver Career signing to release an outgoing driver. Raised by ce-drivers (the lead authorised --force).
struct DriverCareerContractOwner {
    /* 0x298 */ void *contracts_begin;  // L2 decomp:re/decomp/140d766b0.c;xref:0x140d76884,0x140d77740 16-byte slots {?, DriverContract *}
    /* 0x2a0 */ void *contracts_end;    // L2 decomp:re/decomp/140d766b0.c;xref:0x140d7688b,0x140d77747 end of the slots
};
