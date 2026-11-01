// struct: BankAccount
// subsystem: career
// size: 0x30 L2 decomp:re/decomp/140bf51c0.c (account vector stride 0x30)
// notes: one CareerBankComponent account (team accounts at bank+0x298, driver accounts at bank+0x2b8). Raised by ce-econ (career lane owner career-re is stale; the lead authorised --force).
struct BankAccount {
    /* 0x010 */ BankSubAccount * subs_begin;         // L2 decomp:re/decomp/140bf51c0.c;xref:0x140bf5213 sub-accounts
    /* 0x018 */ BankSubAccount * subs_end;           // L2 decomp:re/decomp/140bf51c0.c;xref:0x140bf5217 end of the sub-accounts
    /* 0x028 */ uint32_t key;                        // L2 decomp:re/decomp/140bf51c0.c;xref:0x140bf51fd u8 kind | u16 stat id << 16
};
