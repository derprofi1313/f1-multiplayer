// struct: UpgradeTree
// subsystem: career
// size: 0xc40 L2 decomp:re/decomp/14352e1c0.c;imm:0xc40 (Career_ResetAll_Upgrades steps trees by 0xc40 at 0x14352e1e1)
// notes: one upgrade tree inside an Upgrades model (Upgrades.trees[k], k < Upgrades.tree_count): the instances by value plus an inline pointer array (+0x30) the queries use. Raised by ce-rnd (the lead authorised creating career-lane structs).
struct UpgradeTree {
    /* 0x010 */ UpgradeInstance *instances_begin;   // L2 decomp:re/decomp/14352e1c0.c;xref:0x14352e20c instance vector (0xe0 each)
    /* 0x018 */ UpgradeInstance *instances_end;     // L2 decomp:re/decomp/14352e1c0.c;xref:0x14352e210 end of the instance vector
};
