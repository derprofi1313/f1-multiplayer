// struct: GameManager
// subsystem: game
// size: ?
// notes: the game manager singleton (*g_Game_Manager). Only the fields the career editor reads. Raised by ce-econ.
struct GameManager {
    /* 0x0c0 */ GameModeContext *mode_context;       // L2 decomp:re/decomp/140249730.c;decomp:re/decomp/1401b0880.c;xref:0x140249756,0x1401b0a2c current game-mode context (session code and the Team HQ player chain read it, then +0x88)
};
