// struct: SponsorLoyaltyXpMessage
// subsystem: career
// size: ?
// notes: PrimarySponsorLoyaltyXPGainedMessage (vt_Career_SponsorLoyaltyXpMessage), posted by the loyalty reward 0x140fc4cf0, handled by 0x14110cf40. Raised by ce-econ (career lane owner career-re is stale; the lead authorised --force).
struct SponsorLoyaltyXpMessage {
    /* 0x000 */ void ** vtable;                      // L2 decomp:re/decomp/140fc4cf0.c;xref:0x140fc4d57 vt_Career_SponsorLoyaltyXpMessage
    /* 0x008 */ void * ref;                          // L2 decomp:re/decomp/140fc4cf0.c;xref:0x140fc4d4b 0 in the poster
    /* 0x010 */ uint32_t team;                       // L2 decomp:re/decomp/14110cf40.c;xref:0x14110cf49,0x140fc4d60 team key
    /* 0x014 */ int32_t amount;                      // L2 decomp:re/decomp/14110cf40.c;xref:0x14110cf4c,0x140fc4d64 loyalty XP before the perk multiplier
    /* 0x018 */ uint8_t reason;                      // L2 decomp:re/decomp/14110cf40.c;decomp:re/decomp/141106870.c;xref:0x14110cf44,0x140fc4d68 reason enum (0 CompletedRaceWeekendPrimary .. 5 ActivityOutcome)
};
