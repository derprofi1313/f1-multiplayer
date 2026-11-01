// struct: ResourcePointsComponent
// subsystem: career
// size: ?
// notes: ResourcePointsComponent (vtable 0x1456cc040). +0x4a8 {entity, mode} is the wallet view Career_Get_RpWallet takes; the two wallet stores are embedded objects whose address the adjust functions take. Raised by ce-econ (career lane owner career-re is stale; the lead authorised --force).
struct ResourcePointsComponent {
    /* 0x4a8 */ CareerEntity * wallet_entity;        // L2 decomp:re/decomp/141229e00.c;decomp:re/decomp/1435086c0.c;xref:0x141229edc entity whose entrants map a driver key to its team key
    /* 0x4b0 */ uint8_t wallet_mode;                 // L2 decomp:re/decomp/141229e00.c;decomp:re/decomp/14122a070.c;decomp:re/decomp/1435086c0.c;xref:0x141229e9f,0x14122a074,0x1435086cc 0 = wallet per driver, 1 = wallet per team
    /* 0x4b8 */ void * driver_wallets;               // L2 decomp:re/decomp/141229e00.c;decomp:re/decomp/1434fefe0.c;xref:0x141229ebc first qword of the embedded per-driver wallet store (its entity); &driver_wallets goes to Career_Adjust_RpDriverWallet
    /* 0x528 */ void * team_wallets;                 // L2 decomp:re/decomp/141229e00.c;decomp:re/decomp/14122a070.c;xref:0x141229ec2,0x14122a08c first qword of the embedded per-team wallet store; &team_wallets goes to Career_Adjust_RpTeamWallet
};
