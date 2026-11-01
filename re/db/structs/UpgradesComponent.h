// struct: UpgradesComponent
// subsystem: career
// size: ?
// notes: the career's R&D component (vtable 0x1456cc8b0, "UpgradesComponent"); found through ce::comp::get(Upgrades). Holds one Upgrades model per team and its CareerTime sibling; the purchase request handler (0x1412414c0), the research/develop requests (0x1412431a0, 0x141242530) and its day update (vtable slot +0x128, 0x14123a860) all read this layout. Raised by ce-rnd (the lead authorised creating career-lane structs); also described in addons/career-editor/protocol/rnd.md.
struct UpgradesComponent {
    /* 0x298 */ Upgrades **upgrades_begin;          // L2 decomp:re/decomp/1412431a0.c;decomp:re/decomp/14123a860.c;xref:0x141243220,0x14124347c per-team Upgrades models (vector of pointers), searched by Upgrades.team_key
    /* 0x2a0 */ Upgrades **upgrades_end;            // L2 decomp:re/decomp/1412431a0.c;xref:0x141243227,0x141243483 end of the Upgrades pointer vector
    /* 0x2c0 */ CareerTimeComponent *career_time;   // L2 decomp:re/decomp/1412431a0.c;decomp:re/decomp/141242bd0.c;xref:0x1412435f6,0x14124367c the CareerTime sibling; day = career_time->current_minutes / 1440 ((x >> 5) / 45 in the code), the day stamped on every state entry
};
