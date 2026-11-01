// struct: UiSignal
// subsystem: vm
// size: 0x18 L2 decomp:re/decomp/1442c17e0.c;kb:subsystems/vm.md#signals allocated 0x18 bytes per signal by Vm_Read_SignalList
// notes: one signal of a command (slot 2) or event (slot 3) entry. The class string NeUISignal exists in the exe but is not linked to this allocation (L0).
struct UiSignal {
    /* 0x000 */ void **link;            // L2 decomp:re/decomp/144251800.c;decomp:re/decomp/1442c17e0.c;kb:subsystems/vm.md#signals link handle filled by Vm_Link_NodeReference (or left pending); Vm_Dispatch_EventSignals takes target = **link and skips a null handle or target
    /* 0x008 */ uint32_t to_pin;        // L2 decomp:re/decomp/1442c17e0.c;kb:subsystems/vm.md#signals djb2 of the target pin name (key 0x40d4ab10), required
    /* 0x00c */ uint32_t to_object;     // L2 decomp:re/decomp/1442c17e0.c;decomp:re/decomp/144349300.c;kb:subsystems/vm.md#signals djb2 of the target node's authored path (key 0x0c3e545e), required; Vm_Remap_InstanceSignals rewrites it to the runtime id when the target is inside the template
    /* 0x010 */ void *attributes;       // L1 decomp:re/decomp/1442c17e0.c zeroed at allocation; attributes (key 0xe1ffdb94) are read after it, where they are stored is not confirmed
};
