// struct: GdbStringRecord
// subsystem: gamedb
// size: ?
// notes: a string-typed game database record (the value the database map holds for a string key; Gdb_Lookup_Key returns a slot pointing at it). The text is stored XOR-encoded: plain[i] = k_Gdb_StringXorKey[(i-1)&3] ^ enc[i] for i < size-1, plain[size-1] = 0. The game decodes inline at every read into the record's own buffer (text_buffer, capacity 1 = empty) and uses the NeStr at +0x38. Raised by ce-platform for core's db string reads (lead-authorised; gamedb has no owner).
struct GdbStringRecord {
    /* 0x010 */ uint64_t enc_size;                   // L2 decomp:re/decomp/14020ccd0.c;decomp:re/decomp/14114bb60.c;xref:0x14020cdbf encoded byte count including the terminator; 0 = no text
    /* 0x018 */ uint8_t *enc;                        // L2 decomp:re/decomp/14020ccd0.c;decomp:re/decomp/14114bb60.c;xref:0x14020cdb2 encoded text
    /* 0x040 */ uint64_t text_capacity;              // L2 decomp:re/decomp/14020ccd0.c;xref:0x14020cda7 capacity of the decode buffer; 1 = empty, the game then uses the shared empty string
    /* 0x048 */ char *text_buffer;                   // L2 decomp:re/decomp/14020ccd0.c;xref:0x14020ce05 the game's in-place decode target; core never writes it and keeps its own decoded copy
};
