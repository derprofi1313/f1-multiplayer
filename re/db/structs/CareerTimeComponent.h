// struct: CareerTimeComponent
// subsystem: career
// size: ?
// notes: CareerTimeComponent (vtable 0x1456bf530). Owned by ce-econ (restored by ce-rnd after an accidental overwrite at 17:35; evidence re-checked by ce-econ: 0x14113d451 mov esi,[rcx+0x2b0], 0x14113d457 add rcx,0x290 then call 0x14111dd30, 0x1412435fd/0x14123a8bb mov reg,[reg+0x2b0]). Field current_minutes keeps the name season.cpp uses. The career day everywhere is current_minutes / 1440 ((minutes >> 5) / 45 in the R&D code).
struct CareerTimeComponent {
    /* 0x290 */ uint8_t season_dates;               // L2 decomp:re/decomp/14113d120.c;xref:0x14113d457,0x14113d45e start of the season date block (date table at +0x298..+0x2a0, 0x10 stride) passed to the career-length helper and the calendar helpers 0x14111dbc0/0x1410f5a80
    /* 0x2b0 */ uint32_t current_minutes;           // L2 decomp:re/decomp/1412431a0.c;decomp:re/decomp/14123a860.c;xref:0x1412435fd,0x14123a8bb,0x14113d451 career clock in minutes; read through UpgradesComponent.career_time by the research start and the R&D day update
};
