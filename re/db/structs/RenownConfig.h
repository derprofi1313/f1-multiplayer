// struct: RenownConfig
// subsystem: career
// size: ?
// notes: renown tier configuration shared by the per-team renown objects. Raised by ce-econ (career lane owner career-re is stale; the lead authorised --force).
struct RenownConfig {
    /* 0x008 */ uint32_t * tiers_begin;              // L2 decomp:re/decomp/140f98cb0.c;decomp:re/decomp/1414c5f80.c;xref:0x140f98d03,0x140f98d45,0x1414c60c8 tier thresholds (renown points)
    /* 0x010 */ uint32_t * tiers_end;                // L2 decomp:re/decomp/140f98cb0.c;xref:0x140f98d49 end of the thresholds
};
