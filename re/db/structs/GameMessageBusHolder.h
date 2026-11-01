// struct: GameMessageBusHolder
// subsystem: game
// size: ?
// notes: *g_Game_MessageBus. Raised by ce-econ.
struct GameMessageBusHolder {
    /* 0x008 */ NeMessageBus *bus;                   // L2 decomp:re/decomp/140c68310.c;decomp:re/decomp/140c64aa0.c;xref:0x140c68483 the bus every component posts to and subscribes on
};
