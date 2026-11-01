// struct: BankTransaction
// subsystem: career
// size: 0x14 L2 decomp:re/decomp/140c66c30.c (history vector stride 0x14, /0x14 count)
// notes: one record of a bank sub-account's history (copy of BankTransactionMessage +0x14..+0x28). Raised by ce-econ (career lane owner career-re is stale; the lead authorised --force).
struct BankTransaction {
    /* 0x000 */ uint8_t type;                        // L2 decomp:re/decomp/140c679d0.c;xref:0x140c67a83,0x140c66a4d transaction type (log names it)
    /* 0x004 */ int32_t amount;                      // L2 decomp:re/decomp/140c66c30.c;xref:0x140c66c41,0x140c67d44 amount added to the balance
    /* 0x008 */ uint32_t created;                    // L2 decomp:re/decomp/140c635c0.c;decomp:re/decomp/140c66a00.c;xref:0x140c635ed,0x140c66a8b career clock (minutes) when the bank handled the transaction message; a deferred record keeps it when it lands (time holds the due time)
    /* 0x00c */ uint32_t time;                       // L2 decomp:re/decomp/140c679d0.c;xref:0x140c67aa2 due time in minutes; the log prints it as day = time/1440
    /* 0x010 */ uint8_t flow;                        // L2 decomp:re/decomp/140c679d0.c;xref:0x140c67a92 flow (eImmediate .. eCustomDay)
};
