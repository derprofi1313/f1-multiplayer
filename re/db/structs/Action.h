// struct: Action
// subsystem: fe_tiles
// size: 0x10 L2 decomp:work/exe/decomp/14173da50_item_a_getaction.c
// notes: type 1 = fire route str; 3 = store/URL; 4 = string commands (reboot, end_of_trial, ...)
struct Action {
    /* 0x000 */ int32_t type;           // L2 decomp:work/exe/decomp/14173da50_item_a_getaction.c;doc:research/exe.md#3.3
    /* 0x004 */ int32_t pad;            // L1 doc:research/exe.md#3.5
    /* 0x008 */ NeStr *str;             // L2 decomp:work/exe/decomp/14173da50_item_a_getaction.c;doc:research/exe.md#3.3 route name for type 1
};
