// struct: ChampionshipHolder
// subsystem: game
// size: ?
// notes: GameModeContext +0x88. Raised by ce-econ.
struct ChampionshipHolder {
    /* 0x260 */ CareerEntity *entity;                // L2 decomp:re/decomp/140249730.c;decomp:re/decomp/1401b0880.c;decomp:re/decomp/1414c7450.c;xref:0x14024976b,0x1401b0a3f,0x1414c7478 the current championship entity (CareerEntity in a career); 0x1401b0880 and 0x1414c7450 null-check it, 0x140249730 dereferences it unchecked
};
