// struct: NeMessageBus
// subsystem: core
// size: ?
// notes: the engine message bus (GameMessageBusHolder +8). Ne_Post_BusMessage dispatches synchronously to the subscribers of msg->vtbl[1](); subscribe 0x143987810, unsubscribe 0x143988350. Raised by ce-econ.
struct NeMessageBus {
    /* 0x068 */ uint8_t active;                      // L2 decomp:re/decomp/1439876c0.c;decomp:re/decomp/140c64aa0.c;xref:0x1439876cb,0x140c64ac2 posting and subscribing do nothing while 0
};
