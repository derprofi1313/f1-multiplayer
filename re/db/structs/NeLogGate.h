// struct: NeLogGate
// subsystem: core
// size: ?
// notes: g_Ne_LogGate; channel hash set probed inline before Ne_Log_Thunk calls (open addressing: start H%373, step 13 + H%13 mod 373, 0 = empty); filled by 0x14304cee0, cleared by 0x1406fd500 / 0x1431a4b10
struct NeLogGate {
    /* 0x000 */ uint32_t channel_hashes[373]; // L2 decomp:re/decomp/140af0460.c;xref:0x140af048a,0x140af04a0 lea rcx,[gate]; mov edx,[rcx+rax*4] with rax = H%373, + (13 + H%13) mod 0x175
    /* 0x5d4 */ uint8_t enabled;        // L2 decomp:re/decomp/140af0460.c;xref:0x140af047c,0x140af073c cmp byte [gate+0x5d4],1 before the probe; every call site does this first
};
