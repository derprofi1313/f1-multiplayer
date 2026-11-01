// struct: NeStr
// subsystem: core
// size: ?
// notes: refcounted interned UI string; a null NeStr* means "" (hash 0x1505); chars are NUL-terminated
struct NeStr {
    /* 0x000 */ uint32_t hash;          // L2 decomp:work/exe/decomp/141bb3c20_fe_home_dispatch.c;doc:research/exe.md#1 djb2 of the chars; focus_changed copies it to vm+0x3f0
    /* 0x004 */ uint16_t refcount;      // L2 decomp:work/exe/decomp/1441974e0_nestr_from_cstr.c;doc:research/exe.md#1 LOCK-decremented on release, set to -1 when it hits 0
    /* 0x006 */ uint8_t len_flags[3];   // L2 decomp:work/exe/decomp/1441974e0_nestr_from_cstr.c;doc:research/exe.md#1 read as u32 at +6: bits 0-19 length, bits 20-21 flags (byte +9 untouched)
    /* 0x009 */ char chars[1];          // L2 decomp:work/exe/decomp/1441974e0_nestr_from_cstr.c;doc:research/exe.md#1 compared against the C string
};
