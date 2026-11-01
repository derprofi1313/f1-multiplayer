// struct: BankSubAccount
// subsystem: career
// size: 0x1b0 L2 decomp:re/decomp/140bf51c0.c (sub-account vector stride 0x1b0)
// notes: a bank account's sub-account; category 1 is team cash. Raised by ce-econ (career lane owner career-re is stale; the lead authorised --force).
struct BankSubAccount {
    /* 0x00c */ uint8_t category;                    // L2 decomp:re/decomp/140c635c0.c;xref:0x140c636e0,0x140c63790 matched against BankTransactionMessage.category
    /* 0x010 */ int32_t balance;                     // L2 decomp:re/decomp/140c66c30.c;decomp:re/decomp/140bf51c0.c;xref:0x140c66c45,0x140c66c5e,0x140bf5370 saturating int32 balance; the team balance is the sum over sub-accounts
    /* 0x014 */ int32_t minimum;                     // L2 decomp:re/decomp/140c67b10.c;xref:0x140c67d4f,0x140c67dcc balance floor the affordability check compares against
    /* 0x040 */ BankTransaction * history_begin;     // L2 decomp:re/decomp/140c66c30.c;xref:0x140c66c68,0x140c66cc7 transaction history
    /* 0x048 */ BankTransaction * history_end;       // L2 decomp:re/decomp/140c66c30.c;xref:0x140c66c61,0x140c66ccb end of the history
};
