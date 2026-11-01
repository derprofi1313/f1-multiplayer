// struct: DriverStatistic
// subsystem: career
// size: 0x140 L2 decomp:re/decomp/14115ac30.c (statistics loop step 0x140)
// notes: one statistic of a driver (save keys of DriverStatistics_Serialise_Stat 0x14115ac30). Raised by ce-drivers (the lead authorised --force).
struct DriverStatistic {
    /* 0x010 */ uint16_t info_stat_id;           // L2 decomp:re/decomp/14115ac30.c;decomp:re/decomp/14a523160.c;xref:0x14115b010,0x14a523290 save key infoStatId (lea r13,[r15+0x10]; getter cmp word [rdi-0x90]) (V0615)
    /* 0x018 */ EvolutionCurve evolution_graph;  // L2 decomp:re/decomp/14115ac30.c;decomp:re/decomp/14a523160.c;xref:0x14115b053,0x14a523368 save key evolutionGraph (lea rcx,[r15+0x18]; getter lea rcx,[rdi-0x88]) (V0616)
    /* 0x0a0 */ EvolutionCurve patched_evolution_graph; // L2 decomp:re/decomp/14115ac30.c;decomp:re/decomp/14a523160.c;xref:0x14115b074,0x14a52327d save key patchedEvolutionGraph (the getter steps rdi = stat+0xa0)
    /* 0x128 */ uint8_t has_patched_graph;       // L2 decomp:re/decomp/14115ac30.c;decomp:re/decomp/14a523160.c;xref:0x14115ad8c,0x14a52334b patchedEvolutionGraph engaged (optional flag)
    /* 0x130 */ float temporary_boost_value;     // L2 decomp:re/decomp/14115ac30.c;decomp:re/decomp/141140ee0.c;xref:0x14115b0bf,0x1411412c8 save key temporaryBoostValue
    /* 0x134 */ uint8_t is_dynamic_stat;         // L2 decomp:re/decomp/14115ac30.c;decomp:re/decomp/141140ee0.c;xref:0x14115b132,0x1411413af save key isDynamicStat; dynamic -> base 0
    /* 0x138 */ float dynamic_level;             // L2 decomp:re/decomp/14115ac30.c;decomp:re/decomp/141140ee0.c;xref:0x14115b1a1,0x141141054 save key dynamicLevel; added by the getter
};
