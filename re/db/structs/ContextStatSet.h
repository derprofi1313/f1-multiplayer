// struct: ContextStatSet
// subsystem: career
// size: 0x38 L2 decomp:re/decomp/141134c30.c (contextAwareStatistics loop step 0x38)
// notes: a context-aware statistic set; the getter uses it when the stat context matches both masks, otherwise the driver's default set. Raised by ce-drivers (the lead authorised --force).
struct ContextStatSet {
    /* 0x010 */ DriverStatistic *stats_begin; // L2 decomp:re/decomp/14a523160.c;xref:0x14a523270 statistics vector, 0x140 stride
    /* 0x018 */ DriverStatistic *stats_end;   // L2 decomp:re/decomp/14a523160.c;xref:0x14a523274 end of the statistics
    /* 0x028 */ uint32_t mask_a;              // L2 decomp:re/decomp/14a523160.c;xref:0x14a523247 bit ctx[0] must be set when ctx[1] != 0
    /* 0x02c */ uint32_t mask_b;              // L2 decomp:re/decomp/14a523160.c;xref:0x14a52325a bit ctx[2] must be set when ctx[3] != 0
};
