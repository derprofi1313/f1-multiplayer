// struct: EntrantRecord
// subsystem: entrants
// size: 0x88 L2 decomp:re/decomp/142452e60.c (Sess_Parse_Entrant builds one; session vectors step 0x88)
// notes: one entrants.entrant_lists row -> one 0x88 record, assembled by Sess_Parse_Entrant
// (0x142452e60). The record is the grid participant's identity: driver/team/vehicle/badge stat ids
// (each resolved by label against its entrants.* table) plus the two customisation packings and the
// car number / team position. In the Weekend/session participant vector the element carries a
// polymorphic header (vtable &UNK_145612e20 at +0x00, sub-object vtable &UNK_145612ed0 at +0x70, set
// by Sess_ParticipantVec_PushBack / the parser); +0x08..+0x40 is an embedded NeStr name block (the
// entrant_reference label). Unresolved label -> stat id 0x3fff (sentinel) in the high word.
struct EntrantRecord {
    /* 0x000 */ void **vtable;          // L2 decomp:re/decomp/142452e60.c;xref:0x1424536be &UNK_145612e20
    /* 0x008 */ void *name_block[8];    // L1 decomp:re/decomp/142452e60.c;xref:0x1424536be entrant_reference NeStr/name block (+0x08..+0x47), built by FUN_1441207d0
    /* 0x048 */ uint32_t ordinal_flags; // L2 decomp:re/decomp/142452e60.c;xref:0x1424536c8 low 24 bits = per-entrant running index (parser post-increments *param_2); high byte (+0x4b) is a flags byte set after parse, bit 0x10 excludes from AI count (doc:research/grid24-cap.md#3)
    /* 0x050 */ uint8_t id_lo;          // L2 decomp:re/decomp/142452e60.c;decomp:re/decomp/148618ee0.c;decomp:re/decomp/141140ee0.c;xref:0x148619037,0x141141267 low (kind) byte of the driver key {u8 kind, u16 stat id at +0x52}; 0 at parse; written with the whole key by Career_Set_EntrantData_Thunk (0x148619037); matched by Career_Get_DriverStatValue (0x141141267) and Sess_TyreAllocation_SendNetMessage (V0670)
    /* 0x052 */ uint16_t id_hi;         // L2 decomp:re/decomp/142452e60.c;xref:0x142453703 driver stat id (parser writes dword@0x50 = driver_stat_id<<16; entrants.drivers label lookup; sentinel 0x3fff)
    /* 0x054 */ uint32_t driver_customisation; // L2 decomp:re/decomp/142452e60.c;xref:0x142453707 packed driver_customisation (Ent_Pack_DriverCustomisation 0x142453880; 0x3fff0000 if empty)
    /* 0x058 */ uint32_t team_stat_id;  // L2 decomp:re/decomp/142452e60.c;xref:0x142453710,0x1424530d9,0x142453204,0x14245354a team stat id in high word +0x5a (entrants.teams label lookup); 0x3fff0000 when the entrant has no team label, 0 when a label yields no stat_id (V0673)
    /* 0x05c */ uint32_t vehicle_ref;   // L2 decomp:re/decomp/142452e60.c;xref:0x142453717 packed vehicle ref (vehicle stat id in high word; Ent_Pack_VehicleRef 0x1425069b0; 0x3fff0000 if empty)
    /* 0x060 */ uint32_t driver_reg_value; // L2 decomp:re/decomp/142452e60.c;xref:0x14245371e value from the matched driver-registry node (+0x4b0), keyed on driver/team/vehicle ids
    /* 0x064 */ uint32_t vehicle_customisation; // L2 decomp:re/decomp/142452e60.c;xref:0x142453721 packed vehicle_customisation (livery preset; Ent_Pack_VehicleCustomisation 0x1424fc250; 0x3fff0000 if empty)
    /* 0x068 */ uint32_t badge_stat_id; // L2 decomp:re/decomp/142452e60.c;xref:0x142453724 badge_customisation stat id in high word (entrants.badges label lookup; sentinel 0x3fff)
    /* 0x070 */ void **subobj_vtable;   // L2 decomp:re/decomp/142452e60.c;xref:0x142453735 &UNK_145612ed0
    /* 0x078 */ uint32_t const_ffff0012; // L2 decomp:re/decomp/142452e60.c;decomp:re/decomp/148618ee0.c;xref:0x142453727,0x148619073 0xffff0012 written by the parser; Career_Set_EntrantData_Thunk copies EntrantWriteData::seat_value here, and Career_Sign_DriverToTeam takes it from the replaced seat (raised by ce-drivers)
    /* 0x080 */ uint16_t team_position; // L2 decomp:re/decomp/142452e60.c;xref:0x142453739 entrant team_position field, packed with a flag bit (value ^ 0x100)
    /* 0x082 */ uint8_t unk_0x82;       // L1 decomp:re/decomp/142452e60.c;xref:0x142453740 constant 0xff
    /* 0x083 */ uint8_t car_number;     // L2 decomp:re/decomp/142452e60.c;decomp:re/decomp/148618ee0.c;xref:0x14245374b,0x14861909a entrant car_number field (racing number); Career_Set_EntrantData_Thunk writes its flag argument here and the game's signing passes the incoming driver's own value, so the number stays with the driver
};
