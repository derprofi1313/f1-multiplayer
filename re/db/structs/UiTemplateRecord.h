// struct: UiTemplateRecord
// subsystem: vm
// size: ?
// notes: one registered template in the global template store (*(mgr+0x8168), record array +0x50/+0x58, name map +0x28). Written by Vm_Load_UidfTemplates, read by every instantiating caller (Vm_UiTabs_AddTabTemplate passes &rec->node_ref and rec->pkg to Vm_Instantiate_Template_Thunk). Pool-allocated; at least 0x28 bytes. No C++ class name recovered.
struct UiTemplateRecord {
    /* 0x000 */ NeStr *name;            // L2 decomp:re/decomp/144177470.c;kb:subsystems/vm.md#templates template name from the node props (fnv1a32 key 0x8d39bde6); the store map key
    /* 0x008 */ void *resid;            // L2 decomp:re/decomp/144177470.c;decomp:re/decomp/14449e140.c;kb:subsystems/vm.md#templates points at the UIDF tracker's resource id copy (tracker+8), passed as arg 2 by Vm_Load_UidfResource
    /* 0x010 */ void *pkg;              // L2 decomp:re/decomp/144177470.c;decomp:re/decomp/144266e30.c;kb:subsystems/vm.md#templates base of the .ui !pkg buffer; runtime-observed 2026-10-23 equal to the pkg buffer of the instance root (re/tools/ui_trace)
    /* 0x018 */ uint32_t node_ref;      // L2 decomp:re/decomp/144177470.c;decomp:re/decomp/144266e30.c;kb:subsystems/vm.md#templates !pkg reference of the template's node array (CHUNK tag 0x80 | offset; runtime values 0x80000060, 0x800001f0); &node_ref is the node_off argument of Vm_Instantiate_Template_Thunk
    /* 0x020 */ void *ctx;              // L1 decomp:re/decomp/144177470.c arg 4 of Vm_Load_UidfTemplates; role not read
};
