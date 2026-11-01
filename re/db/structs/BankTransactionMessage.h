// struct: BankTransactionMessage
// subsystem: career
// size: ?
// notes: BankAccountTransactionMessage (vt_Career_BankTransactionMessage), built on the stack by the game's posters (0x140d6b750, 0x1411243c0, 0x140f898c0) and handled by the bank (0x140c68310 resolves the day, 0x140c67b10 validates, 0x140c635c0 applies). +0x14..+0x28 is copied into a BankTransaction record. Raised by ce-econ (career lane owner career-re is stale; the lead authorised --force).
struct BankTransactionMessage {
    /* 0x000 */ void ** vtable;                      // L2 decomp:re/decomp/140d6b750.c;xref:0x140d6b78b vt_Career_BankTransactionMessage
    /* 0x008 */ void * ref;                          // L2 decomp:re/decomp/140d6b750.c;decomp:re/decomp/140c68310.c;decomp:re/decomp/140ce2fc0.c;xref:0x140d6b77f pointer-size slot, 0 in every poster; the handler copies it, nothing reads it
    /* 0x010 */ uint16_t account;                    // L2 decomp:re/decomp/140c67b10.c;xref:0x140c67b33 bank account id: kind << 14 | team stat id
    /* 0x014 */ uint8_t type;                        // L2 decomp:re/decomp/140c67b10.c;xref:0x140c67b42 eBankTransactionType (names 0x14568d010)
    /* 0x018 */ int32_t amount;                      // L2 decomp:re/decomp/140c67b10.c;xref:0x140c67b47 signed amount
    /* 0x01c */ uint32_t created;                    // L2 decomp:re/decomp/140c635c0.c;xref:0x140c635ed set by the apply to the current career clock (minutes)
    /* 0x020 */ uint32_t time;                       // L2 decomp:re/decomp/140c67b10.c;xref:0x140c67b4b due time in career minutes (resolved by 0x140c68310 from flow)
    /* 0x024 */ uint8_t flow;                        // L2 decomp:re/decomp/140c635c0.c;xref:0x140c6363e,0x140c67b4e 0 eImmediate, 1 eNextDay, 2 eNextMonday, 3 eCustomDay
    /* 0x025 */ uint8_t check_funds;                 // L2 decomp:re/decomp/140c67b10.c;xref:0x140c67b1e 1 = affordability check (and the handler posts the validity message)
    /* 0x028 */ uint8_t category;                    // L2 decomp:re/decomp/140c67b10.c;decomp:re/decomp/140c635c0.c;xref:0x140c67b2e,0x140c635f0 sub-account category (1 team cash; 2 uses the driver accounts)
};
