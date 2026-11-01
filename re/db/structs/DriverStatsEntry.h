// struct: DriverStatsEntry
// subsystem: career
// size: 0x50 L2 decomp:re/decomp/141134c30.c (driverData stride: (end-begin)>>4 * -0x3333333333333333, element step 0x14 dwords)
// notes: one driverData element of DriverStatisticsComponent: the DriverStatId key plus the default statistic set and the context-aware sets. Raised by ce-drivers (the lead authorised --force).
struct DriverStatsEntry {
    /* 0x000 */ uint8_t kind;              // L2 decomp:re/decomp/14a523160.c;decomp:re/decomp/141134c30.c;xref:0x14a5231d4 DriverStatId kind byte (save packs kind<<14 | stat id)
    /* 0x002 */ uint16_t stat_id;          // L2 decomp:re/decomp/14a523160.c;decomp:re/decomp/141134c30.c;xref:0x14a5231cd DriverStatId stat id
    /* 0x018 */ DriverStatistic *stats_begin;   // L2 decomp:re/decomp/14a523160.c;xref:0x14a523270 default set (entry+8) statistics vector, 0x140 stride
    /* 0x020 */ DriverStatistic *stats_end;     // L2 decomp:re/decomp/14a523160.c;xref:0x14a523274 end of the default set statistics
    /* 0x038 */ ContextStatSet *context_sets_begin; // L2 decomp:re/decomp/14a523160.c;decomp:re/decomp/141134c30.c;xref:0x14a5231e6 contextAwareStatistics, 0x38 stride
    /* 0x040 */ ContextStatSet *context_sets_end;   // L2 decomp:re/decomp/14a523160.c;xref:0x14a5231ea end of contextAwareStatistics
};
