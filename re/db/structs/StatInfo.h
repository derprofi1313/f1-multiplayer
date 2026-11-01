// struct: StatInfo
// subsystem: career
// size: ?
// notes: a driver_statistics_info row as loaded by 0x14114bb60 (stat_id, weighting_factor, default_baseline_value); reached through DriverStatisticsComponent::stat_infos_begin. Raised by ce-drivers (the lead authorised --force).
struct StatInfo {
    /* 0x008 */ uint16_t stat_id;       // L2 decomp:re/decomp/14113d120.c;decomp:re/decomp/14114bb60.c;xref:0x14113d3f3,0x14113d297 infoStatId matched against DriverStatistic::info_stat_id
    /* 0x130 */ float weight;           // L2 decomp:re/decomp/14113d120.c;decomp:re/decomp/14114bb60.c;xref:0x14113d415,0x14113d2e0 weighting_factor (normalised) used by the overall rating
    /* 0x134 */ float default_value;    // L2 decomp:re/decomp/14a523160.c;decomp:re/decomp/14114bb60.c;xref:0x14a5232ea default_baseline_value, the getter's fallback
};
