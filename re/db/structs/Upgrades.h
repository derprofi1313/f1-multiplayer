// struct: Upgrades
// subsystem: career
// size: 0x1b10 L2 decomp:re/decomp/14123ca40.c;imm:0x1b10 (Career_Add_TeamUpgrades allocates it, mov edx,0x1b10 at 0x14123cad4)
// notes: one team's R&D model (one per team in UpgradesComponent.upgrades_begin..end). The car lane's CarPerfVehicle describes the same object as Car_Fit_UpgradesToRating's write-back target with a different field reading. Raised by ce-rnd (the lead authorised creating career-lane structs); see subsystems/rnd_ui.md#instances.
struct Upgrades {
    /* 0x0008 */ uint8_t mode;                      // L2 decomp:re/decomp/14352f300.c;decomp:re/decomp/143530980.c;xref:0x14352f324,0x1435309de 0 one shared tree with an owner cache, 1 per team, 2 per driver (availability refresh only in 1/2)
    /* 0x000c */ uint32_t team_key;                 // L2 decomp:re/decomp/1412431a0.c;xref:0x141243260,0x1412432f0 packed team key (u8 kind | u16 stat id << 16) the requests match against the entrant's team
    /* 0x00e0 */ UpgradeTree trees[2];              // L2 decomp:re/decomp/14352e1c0.c;decomp:re/decomp/14352cac0.c;xref:0x14352e1da,0x14352caff up to two trees (the second only in per-driver mode 2)
    /* 0x1960 */ uint64_t tree_count;               // L2 decomp:re/decomp/14352e1c0.c;xref:0x14352e1ce number of trees in use
    /* 0x1aa8 */ uint32_t owner_cache;              // L2 decomp:re/decomp/14352e620.c;decomp:re/decomp/14352e1c0.c;xref:0x14352e6fd,0x14352e25c owner the state bytes were last evaluated for (mode 0, and set in mode 2 too at 0x1435188ff); 0x3fff0000 = invalid
};
