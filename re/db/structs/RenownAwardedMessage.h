// struct: RenownAwardedMessage
// subsystem: career
// size: ?
// notes: MessageRAwarded (vt_Career_RenownAwardedMessage), posted by the MyTeamRenownReward grant 0x140fc5470; handled per team by 0x140f98cb0. Raised by ce-econ (career lane owner career-re is stale; the lead authorised --force).
struct RenownAwardedMessage {
    /* 0x000 */ void ** vtable;                      // L2 decomp:re/decomp/140fc5470.c;xref:0x140fc54d3 vt_Career_RenownAwardedMessage
    /* 0x008 */ void * ref;                          // L2 decomp:re/decomp/140fc5470.c;xref:0x140fc54c7 0 in the poster
    /* 0x010 */ uint32_t team;                       // L2 decomp:re/decomp/140f98cb0.c;xref:0x140f98cec team key (u8 kind, u16 stat id at +0x12)
    /* 0x014 */ float amount;                        // L2 decomp:re/decomp/140f98cb0.c;xref:0x140f98d3a,0x140fc54e0 renown added to TeamRenownData.points
};
