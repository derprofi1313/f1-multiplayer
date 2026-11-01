// struct: NeSharedCtrl
// subsystem: core
// size: ?
// notes: control block of the engine's shared/weak pointers. Raised by ce-econ.
struct NeSharedCtrl {
    /* 0x000 */ void **vtable;                       // L2 decomp:re/decomp/140fc4cf0.c;xref:0x140fc4d85,0x140fc4d93 slot 0 destroys the object, slot 1 frees the block
    /* 0x008 */ int32_t strong;                      // L2 decomp:re/decomp/140fc4cf0.c;xref:0x140fc4d02,0x140fc4d1c strong count (lock cmpxchg increment to upgrade a weak ref)
    /* 0x00c */ int32_t weak;                        // L2 decomp:re/decomp/140fc4cf0.c;xref:0x140fc4d8d,0x140fc4db1 weak count (lock dec; at 0 call vtable slot 1)
};
