// struct: FlowState
// subsystem: fe_flow
// size: ?
// notes: base of every front-end flow state (StateTeamHQ, StateTeamHQUpdate, StateSession, ...), as FeFlow_Set_StateStatus(state, status) reads it for the FLOW log line '[machine] Transitioning "<name>" <old> -> <new>'. The name buffer holds k_Gdb_StringXorKey-encoded text until the game decodes it in place (the setter decodes when djb2(buffer) differs from name_hash). Raised by ce-platform for core's flow events (lead-authorised; fe_flow owner ui-re is stale).
struct FlowState {
    /* 0x020 */ uint64_t name_size;                  // L2 decomp:re/decomp/144423180.c;xref:0x14442329a byte count of the name buffer including the terminator; the setter's in-place decode loop runs over it
    /* 0x028 */ char *name;                          // L2 decomp:re/decomp/144423180.c;xref:0x14442323a,0x1444232e6,0x144423366 name buffer (XOR-encoded until decoded in place), passed to the FLOW log line
    /* 0x0b0 */ uint32_t name_hash;                  // L2 decomp:re/decomp/144423180.c;xref:0x14442327c,0x14442328e djb2 (seed 0x1505, *33 + c) of the plain name; the setter compares djb2(name) against it to decide whether to decode
    /* 0x0b8 */ int32_t status;                      // L2 decomp:re/decomp/144423180.c;xref:0x144423222,0x1444233c4 current status (0 None, 1 Entering, 2 Updating, 3 Exiting); still the old status at the setter's entry
};
