src/
├── main.rs              # Entry point LKM + feature detection
├── lib.rs               # Re-exports
├── arch/                # Architecture-specific code
│   ├── mod.rs           # Detection AMD vs Intel
│   ├── intel/
│   │   ├── mod.rs       # Re-exports Intel
│   │   ├── vmx.rs       # VMXON/VMCS regions + setup
│   │   ├── vmexit.rs    # VM exit handlers
│   │   ├── ept.rs       # Extended Page Tables
│   │   └── instructions.rs  # VMX instructions brutes
│   └── amd/
│       ├── mod.rs       # Re-exports AMD
│       ├── svm.rs       # VMRUN/VMSAVE regions
│       ├── vmcb.rs      # Virtual Machine Control Block
│       ├── npt.rs       # Nested Page Tables
│       └── instructions.rs  # SVM instructions brutes
├── platform/            # OS-specific code
│   ├── mod.rs           # Abstraction layer
│   ├── linux/
│   │   ├── mod.rs       # Linux kernel module glue
│   │   ├── ffi.rs       # Bindings C kernel
│   │   ├── memory.rs    # alloc_pages, virt_to_phys
│   │   └── cpu.rs       # Per-CPU operations, IPI
│   └── windows/
│       ├── mod.rs       # Future Windows driver glue
│       ├── memory.rs    # ExAllocatePool, MmGetPhysicalAddress
│       └── cpu.rs       # KeIpiGenericCall, etc.
├── hypervisor/          # Core hypervisor logic (cross-platform)
│   ├── mod.rs           # Main hypervisor struct
│   ├── guest.rs         # Guest state management
│   ├── memory/          # Memory virtualization
│   │   ├── mod.rs
│   │   ├── shadow_walker.rs  # Shadow Walker implementation
│   │   └── page_hooks.rs     # Page-level hooks
│   └── blue_pill.rs     # Blue Pill detection/anti-detection
├── utils/
│   ├── mod.rs           # Helpers
│   ├── locks.rs         # Spinlocks
│   ├── log.rs           # printk/DbgPrint abstraction
│   └── cpu_features.rs  # CPUID wrappers
└── error.rs             # Error types centralisés


- **Compiler sur Linux** maintenant (`make`)
    
- **Ajouter Windows** plus tard (implémenter `platform/windows/`)
    
- **Ajouter AMD SVM** (implémenter `arch/amd/`)
    
- **Blue-piller** n'importe quel OS sans détection
    
- **Cloaker** ta mémoire avec Shadow Walker