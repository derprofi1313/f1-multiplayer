// struct: UpgradeRecord
// subsystem: career
// size: 0x80 L2 decomp:re/decomp/143539bc0.c (0x80 stride of the output vector in Career_Load_UpgradeRows)
// notes: one parsed R&D upgrade node, built per DB row by Career_Load_UpgradeRows (0x143539bc0). +0x08 is an SSO string builder (vtable 0x145613548) holding upgrade_name. enum bytes at +0x70..+0x72. See subsystems/career.md#load-upgrade-rows.
struct UpgradeRecord {
    /* 0x000 */ void **vtable;              // L2 decomp:re/decomp/143539bc0.c;kb:subsystems/career.md#load-upgrade-rows SSO string-builder vtable (0x145613550 then 0x145613548)
    /* 0x008 */ uint64_t name_cap;          // L1 decomp:re/decomp/143539bc0.c SSO capacity (0x40)
    /* 0x010 */ char *name_ptr;             // L1 decomp:re/decomp/143539bc0.c -> inline buffer at +0x18 or heap
    /* 0x018 */ char name_buf[64];          // L1 decomp:re/decomp/143539bc0.c inline upgrade_name chars
    /* 0x058 */ uint32_t name_djb2;         // L2 decomp:re/decomp/143539bc0.c;kb:subsystems/career.md#load-upgrade-rows djb2 of upgrade_name (seed 0x1505)
    /* 0x05c */ int32_t cost;               // L2 decomp:re/decomp/143539bc0.c;kb:subsystems/career.md#load-upgrade-rows resource-point cost (field "cost")
    /* 0x060 */ int32_t cost_to_develop;    // L2 decomp:re/decomp/143539bc0.c;kb:subsystems/career.md#load-upgrade-rows field "cost_to_develop"
    /* 0x064 */ float weighting;            // L2 decomp:re/decomp/143539bc0.c;kb:subsystems/career.md#load-upgrade-rows field "weighting" (AI weighted-pick weight)
    /* 0x068 */ int32_t num_days_to_research;// L2 decomp:re/decomp/143539bc0.c;kb:subsystems/career.md#load-upgrade-rows fitting_times.num_days_to_research
    /* 0x06c */ int32_t num_days_to_develop; // L2 decomp:re/decomp/143539bc0.c;kb:subsystems/career.md#load-upgrade-rows fitting_times.num_days_to_develop
    /* 0x070 */ uint8_t upgrade_type;       // L2 decomp:re/decomp/143539bc0.c;kb:subsystems/career.md#load-upgrade-rows enum 0..18 (k_Career_UpgradeTypeNameTable)
    /* 0x071 */ uint8_t upgrade_size;       // L2 decomp:re/decomp/143539bc0.c;kb:subsystems/career.md#load-upgrade-rows 0 Minor / 1 Major / 2 Ultimate
    /* 0x072 */ uint8_t upgrade_department; // L2 decomp:re/decomp/143539bc0.c;kb:subsystems/career.md#load-upgrade-rows enum A: 0 def,1 Chassis,2 Durability,3 Engine,4 Fake,5 Invalid
    /* 0x073 */ uint8_t unk_0x73;           // L1 decomp:re/decomp/143539bc0.c packed with the enum dword (uStack_73d)
    /* 0x074 */ int32_t unlock_at_facility_level; // L2 decomp:re/decomp/143539bc0.c;kb:subsystems/career.md#load-upgrade-rows field "unlock_at_facility_level" (default 1)
    /* 0x078 */ int32_t supplier_delivery_date_index; // L2 decomp:re/decomp/143539bc0.c;kb:subsystems/career.md#load-upgrade-rows field "supplier_delivery_date_index" (default 1)
    /* 0x07c */ uint8_t flag_param3;        // L1 decomp:re/decomp/143539bc0.c caller-supplied flag copied into +0x7c
};
