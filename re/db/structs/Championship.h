// struct: Championship
// subsystem: career
// size: 0x5b0 L2 decomp:re/decomp/1407d2820.c;xref:0x1407d3cba (the entity's factory allocates 0x5b0 and runs the ctor 0x140ae01d0)
// notes: game::championship::Championship, the career's championship (CareerEntity.current_championship); holds the seasons. Raised by ce-season with the lead's authorisation (career lane owned by career-re).
struct Championship {
    /* 0x000 */ void **vtable;                       // L2 decomp:re/decomp/140ae22f0.c;decomp:re/decomp/140ae01d0.c;xref:0x140ae230f vt_Career_Championship
    /* 0x208 */ void **list_vtable;                  // L2 decomp:re/decomp/140ae22f0.c;decomp:re/decomp/140ae78f0.c;xref:0x140ae2319 secondary component-list vtable 0x14566ab10; Career_Rebuild_ChampionshipStandings runs with this = &list_vtable
    /* 0x538 */ uint32_t completion;                 // L2 decomp:re/decomp/140ae9650.c;decomp:re/decomp/140ae9b10.c;xref:0x140ae96d8,0x140ae9f7b saved as completion
    /* 0x548 */ ChampionshipSeasonEntry *seasons_begin; // L2 decomp:re/decomp/140ae9650.c;decomp:re/decomp/140ae9b10.c;xref:0x140ae985c,0x140ae9e5f saved as seasons[] (the Season names)
    /* 0x550 */ ChampionshipSeasonEntry *seasons_end;   // L2 decomp:re/decomp/140ae9650.c;decomp:re/decomp/140ae9b10.c;xref:0x140ae9855,0x140ae9e66 end of the season entries
    /* 0x560 */ uint32_t season_index;               // L2 decomp:re/decomp/140ae9b10.c;xref:0x140ae9e42,0x140aea229 current season index, set from the saved season_index when below the season count
};
