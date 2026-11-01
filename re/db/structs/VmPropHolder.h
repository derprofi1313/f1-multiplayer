// struct: VmPropHolder
// subsystem: vm
// size: 0x20 L2 decomp:work/exe/decomp/141697b90_item_base_populate.c;doc:research/exe.md#3.2 nine consecutive holders 0x20 apart in HomeItem
struct VmPropHolder {
    /* 0x000 */ void **vtable;          // L1 doc:research/exe.md#3.2
    /* 0x008 */ void *node;             // L3 decomp:work/exe/decomp/141697b90_item_base_populate.c;doc:research/exe.md#3.2;trace:T000/run1:57 HomeItem+0x20 = button_node.node
    /* 0x010 */ void *link_prev;        // L1 doc:research/exe.md#3.2
    /* 0x018 */ void *link_next;        // L1 doc:research/exe.md#3.2
};
