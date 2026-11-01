// struct: ContractRecord
// subsystem: career
// size: 0x70 L2 decomp:re/decomp/140da5610.c (list stride: (end-begin)>>4 * 0x6db6db6db6db6db7)
// notes: driver contract / move record as written by Career_Serialise_DriverContractList (keys and offsets agreed in verify V0281). Raised by ce-drivers (the lead authorised --force).
struct ContractRecord {
    /* 0x028 */ uint8_t clause_stat;           // L2 decomp:re/decomp/140da5610.c;xref:0x140da5c89,0x140da5d1c,0x140db79b2 performanceClauses stat (5 = no clause; the ctor sets 5) (V0640)
    /* 0x02c */ int32_t clause_target;         // L2 decomp:re/decomp/140da5610.c;xref:0x140da5dbf performanceClauses target
    /* 0x034 */ int32_t salary;                // L2 decomp:re/decomp/140da5610.c;xref:0x140da57f6 save key salary
    /* 0x038 */ int32_t signing_bonus;         // L2 decomp:re/decomp/140da5610.c;xref:0x140da5865 save key signingBonus
    /* 0x03c */ int32_t team_perf_money;       // L2 decomp:re/decomp/140da5610.c;xref:0x140da58d4 save key teamPerfMoney
    /* 0x040 */ int32_t team_perf_race_value;  // L2 decomp:re/decomp/140da5610.c;xref:0x140da5943,0x140db79ba save key teamPerfRaceValue (V0645)
    /* 0x044 */ float success_chance;          // L2 decomp:re/decomp/140da5610.c;xref:0x140da59b2 save key successChance
    /* 0x048 */ int32_t negotiation_bonus;     // L2 decomp:re/decomp/140da5610.c;xref:0x140da5a21 save key negotiationBonus
    /* 0x04c */ uint8_t is_counteroffer;       // L2 decomp:re/decomp/140da5610.c;xref:0x140da5a90 save key isCounteroffer
    /* 0x050 */ uint32_t driver;               // L2 decomp:re/decomp/140da5610.c;xref:0x140da5ad2,0x140da5ad8,0x140db79d4,0x140db7a98 save key driverStatId (u8 kind, u16 stat id at +0x52; ctor 0x3fff0000) (V0649)
    /* 0x054 */ uint32_t old_team;             // L2 decomp:re/decomp/140da5610.c;xref:0x140da5bca save key oldTeamStatId (stat id in the high word)
    /* 0x058 */ uint32_t new_team;             // L2 decomp:re/decomp/140da5610.c;xref:0x140da5b4e save key newTeamStatId (stat id in the high word)
    /* 0x05c */ uint8_t driver_move_reason;    // L2 decomp:re/decomp/140da5610.c;xref:0x140da5e8a save key driverMoveReason
    /* 0x060 */ uint32_t weekend_index;        // L2 decomp:re/decomp/140da5610.c;xref:0x140da5f2e save key weekendIndex
    /* 0x064 */ uint16_t career_day_index;     // L2 decomp:re/decomp/140da5610.c;xref:0x140da5f9f save key careerDayIndex
    /* 0x068 */ uint16_t day_contract_sent;    // L2 decomp:re/decomp/140da5610.c;xref:0x140da6010 save key dayContractSent
    /* 0x06a */ uint16_t day_contract_decided; // L2 decomp:re/decomp/140da5610.c;xref:0x140da607d save key dayContractDecided
    /* 0x06c */ uint8_t accepted;              // L2 decomp:re/decomp/140da5610.c;xref:0x140da60f6 save key accepted
    /* 0x06d */ uint8_t acknowledged;          // L2 decomp:re/decomp/140da5610.c;xref:0x140da6165 save key acknowledged
};
