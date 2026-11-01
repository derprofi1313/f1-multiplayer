// struct: EvolutionCurve
// subsystem: career
// size: 0x88 L2 decomp:re/decomp/1411337e0.c (copy ctor copies 0x88 bytes; patched curve at +0xa0 follows the curve at +0x18)
// notes: NE curve (vtable 0x145be6460) used for driver evolution graphs; built by Career_Parse_CurveString (0x141157100), evaluated by Career_Eval_Curve (0x1445075d0), serialised by 0x144507ab0. Raised by ce-drivers (the lead authorised --force).
struct EvolutionCurve {
    /* 0x000 */ void **vtable;          // L2 decomp:re/decomp/1411337e0.c;xref:0x141133866 &UNK_145be6460 (copy ctor lea)
    /* 0x008 */ float inv_x_range;      // L2 decomp:re/decomp/141157100.c;xref:0x141157886 1/(maxx-minx)
    /* 0x00c */ float min_x;            // L2 decomp:re/decomp/141157100.c;decomp:re/decomp/144507ab0.c;xref:0x141157181,0x144507c2e key minx:
    /* 0x010 */ float y_step;           // L2 decomp:re/decomp/141157100.c;xref:0x14115789c (maxy-miny)/255
    /* 0x014 */ float end_value;        // L2 decomp:re/decomp/141157100.c;xref:0x1411578c4 y of point n-2 scaled to miny..maxy
    /* 0x018 */ float max_x;            // L2 decomp:re/decomp/141157100.c;decomp:re/decomp/144507ab0.c;xref:0x1411571cc,0x144507e2e,0x14115786b key maxx: (parser store, serialiser read) (V0628)
    /* 0x01c */ float min_y;            // L2 decomp:re/decomp/141157100.c;decomp:re/decomp/144507ab0.c;xref:0x141157217,0x144507fce key miny:
    /* 0x020 */ float max_y;            // L2 decomp:re/decomp/141157100.c;decomp:re/decomp/144507ab0.c;xref:0x141157262,0x144508170 key maxy:
    /* 0x024 */ uint8_t lut[65];        // L2 decomp:re/decomp/141157100.c;xref:0x14115785d clamp(int(0.5 + Career_Eval_Curve(c, i/64) * 255))
    /* 0x068 */ void *allocator;        // L2 decomp:re/decomp/144507ab0.c;decomp:re/decomp/141157100.c;xref:0x144507aeb points allocator
    /* 0x070 */ CurvePoint *points_begin;    // L2 decomp:re/decomp/1445075d0.c;decomp:re/decomp/144507ab0.c;xref:0x144507613,0x144507b1c,0x14450830e points {f32 x, f32 y}, y normalised 0..1; written by the parser regrow 0x141157753, read by the dtor 0x14450871a (V0633)
    /* 0x078 */ CurvePoint *points_end;      // L2 decomp:re/decomp/1445075d0.c;decomp:re/decomp/144507ab0.c;xref:0x144507617,0x144507b18,0x144508312 end of the points; written 0x141157757, dtor 0x14450871e (V0634)
    /* 0x080 */ uint64_t points_capacity; // L2 decomp:re/decomp/141157100.c;xref:0x1411576cf,0x14115775b capacity in points
};
