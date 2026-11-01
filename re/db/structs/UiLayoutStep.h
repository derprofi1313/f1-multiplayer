// struct: UiLayoutStep
// subsystem: vm
// size: 0x30 L2 decomp:re/decomp/1441ffb90.c;kb:subsystems/vm.md#layout-attach vector stride 0x30 in the attach behaviour (+0x20 begin, +0x28 end, +0x30 capacity)
// notes: one layout-path step of a node, built by Vm_ElementBase_Init from slot 4 path[0] (one per step) plus one for the slot 5 component, stored in the attach behaviour returned by Vm_GetOrCreate_AttachBehaviour (vt_Vm_AttachBehaviour). Not resolved at load; resolution happens at attach time (not mapped). Candidate class string NeUIElementAttacher is not linked to this code (L0).
struct UiLayoutStep {
    /* 0x000 */ uint8_t kind;           // L2 decomp:re/decomp/1441ffb90.c;kb:subsystems/vm.md#layout-attach 1 = componentId step (isComponentId set), 0 = nodeId step
    /* 0x004 */ uint32_t node_id;       // L2 decomp:re/decomp/1441ffb90.c;kb:subsystems/vm.md#layout-attach nodeId (fnv1a32 key 0x181e12b8) = djb2(scene element leaf name); required when kind 0
    /* 0x008 */ uint64_t component_id;  // L2 decomp:re/decomp/1441ffb90.c;kb:subsystems/vm.md#layout-attach componentId (key 0x14018abd) = md5-64 of the scene path; required when kind 1, optional otherwise
};
