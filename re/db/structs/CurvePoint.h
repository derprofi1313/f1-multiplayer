// struct: CurvePoint
// subsystem: career
// size: 0x8 L2 decomp:re/decomp/1445075d0.c (points indexed *8; count = (end-begin)>>3)
// notes: one EvolutionCurve point; y is normalised 0..1 and scaled by min_y..max_y. Raised by ce-drivers (the lead authorised --force).
struct CurvePoint {
    /* 0x000 */ float x;               // L2 decomp:re/decomp/1445075d0.c;decomp:re/decomp/141157100.c;xref:0x144507692 binary search key (Career_Eval_Curve), parsed before the ':'
    /* 0x004 */ float y;               // L2 decomp:re/decomp/1445075d0.c;decomp:re/decomp/141157100.c;xref:0x144507714 Catmull-Rom control value, parsed after the ':'
};
