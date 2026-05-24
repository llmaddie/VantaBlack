
# 1. Beginning 
First, i made my research about how LKM works, how Hypervisor Work VT-X i.e..... 


16/05/
At 12:03am, I wrote a basic LKM that can be compiled with a Makefile and run on a Debian VM build with Rust for linux  (every test should be done on this VM, nothing on my Host). I configured the VM and wrote this code:
````rs
#![doc = "vantablack kernel module"]

use kernel::prelude::*;

module! {
    type: RustMinimal,
    name: "vantablack",
    authors: ["maddie"],
    description: "module kernel minimal on rust",
    license: "GPL",
}

struct RustMinimal;

impl kernel::Module for RustMinimal {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust minimal module load\n");
        Ok(RustMinimal)
    }
}

impl Drop for RustMinimal {
    fn drop(&mut self) {
        pr_info!("Rust module unload\n");
    }
}
````


finally after ~1h of setup 
I CAN COMPILE AND RUN MY MODULE ON THE KERNEL 
proof : 
![[Pasted image 20260516164548.png]]

![[Pasted image 20260516164557.png]]

after that  
on 5/17/2026
# 2. CPUID Implementation + MSR check

i dev a cpuid file that can execute cpuid command on the victim.
so i wrote a basic code :
```rs
use std::arch::asm;

  

#[derive(Debug, Clone, Copy)]

pub struct CpuidResult {

pub eax: u32,

pub ebx: u32,

pub ecx: u32,

pub edx: u32,

}

  

#[inline]

pub fn cpuid(eax_in: u32, ecx_in: u32) -> CpuidResult {

let mut eax = eax_in;

let mut ecx = ecx_in;

let ebx: u32;

let edx: u32;

  

unsafe {

asm!(

"push rbx",

"cpuid",

"mov {ebx_out:e}, ebx",

"pop rbx",

ebx_out = lateout(reg) ebx,

inout("eax") eax,

inout("ecx") ecx,

lateout("edx") edx,

);

}

  

CpuidResult {

eax,

ebx,

ecx,

edx,

}

}
```
and call that on my lib.rs to execute on the victim 
![[Pasted image 20260517163033.png]]
i inject my .ko and it return the CPUID value of the victim machine 


now i just add a condition if vmx_bit == cpu.ecx >> 5  alors VT-X est disponible
![[Pasted image 20260517183819.png]]

next step was to turn cpuid value in hex and add a MSR check ( the biggest part was done)
after that we turn to chapter 2 VMXON & VMCS allocation ! 


![[Pasted image 20260517191835.png]]
we can see the value of the register was turn into a hex and a checkout for the MSR was passed the code look like that : 
```rs
#[inline]

unsafe fn read_msr(msr: u32) -> u64 {

let low: u32;

let high: u32;

  

unsafe { // wrap the inline assembly to his own unsafe

core::arch::asm!(

"rdmsr",

in("ecx") msr,

out("eax") low,

out("edx") high,

);

}

  

((high as u64) << 32) | (low as u64)

}
```
it was used to declare read_msr and now the condition to pr_alert! : 
```rs
let msr_value = unsafe { read_msr(0x3A) };

if (msr_value & 1) == 0 {

pr_alert!("vantablack: MSR lock bit is 0, VMX is disabled in BIOS\n");

return Err(kernel::error::code::ENODEV);

}

if (msr_value & (1 << 2)) == 0 {

pr_alert!("vantablack: VMX outside SMX is disabled\n");

return Err(kernel::error::code::ENODEV);

}

pr_alert!("vantablack: MSR check passed. BIOS allow VMX operation\n");
```

Now we can officially jump to the second chapter ! 

#  3. VMXON & VMCS allocation 


for introduce what is **VMXON** and **VMCS**  **VMXON** is the Intel VT-x structure used to enable VMX operation (hardware virtualization mode) on a CPU core.
Before launching a hypervisor, the LKM must allocate a **4KB aligned physical memory region** for the VMXON region, initialize it with the CPU VMX revision ID (from `IA32_VMX_BASIC` MSR), then execute the `VMXON` instruction with its physical address.4

**VMCS (Virtual Machine Control Structure)** is the core data structure used by the hypervisor to manage a virtual machine state.  

It stores:

- guest CPU state,
- host state,
- VM-exit handlers,
- execution controls.

Like VMXON, the VMCS requires:

- a dedicated **4KB aligned page**,
- physical contiguous memory,
- initialization with the VMX revision ID,
- activation through `VMPTRLD`.

## 3.1 Theoretical Background

### What is VMXON?

**VMXON** is the Intel VT-x structure used to enable VMX operation (hardware virtualization mode) on a CPU core. Before launching a hypervisor, the LKM must:

1. Allocate a **4KB aligned physical memory region** for the VMXON region
    
2. Initialize it with the CPU VMX revision ID (from `IA32_VMX_BASIC` MSR)
    
3. Execute the `VMXON` instruction with its physical address
    

### What is VMCS?

**VMCS (Virtual Machine Control Structure)** is the core data structure used by the hypervisor to manage a virtual machine state. It stores:

- Guest CPU state
    
- Host state
    
- VM-exit handlers
    
- Execution controls
    

Like VMXON, the VMCS requires:

- A dedicated **4KB aligned page**
    
- Physical contiguous memory
    
- Initialization with the VMX revision ID
    
- Activation through `VMPTRLD`
    

## 3.2 Implementation

### 3.2.1 FFI Bindings to Kernel Memory Allocation

To allocate memory from the Linux kernel, we bind the C functions through FFI:
```rs
extern "C" {
    pub fn __get_free_pages(gfp_mask: u32, order: u32) -> *mut u8;
    pub fn free_pages(addr: *mut u8, order: u32);
    pub fn virt_to_phys(virt: *mut u8) -> u64;
    pub fn printk(fmt: *const u8, ...);
}

pub const GFP_KERNEL: u32 = 0xCC0;
pub const __GFP_ZERO: u32 = 0x8000;
```
### 3.2.2 Memory Abstraction Layer
```rs
pub unsafe fn alloc_zeroed_page() -> *mut u8 {
    __get_free_pages(GFP_KERNEL | __GFP_ZERO, 0)
}

pub unsafe fn free_page(addr: *mut u8) {
    free_pages(addr, 0);
}

pub unsafe fn virt_to_phys(virt: *mut u8) -> u64 {
    ffi::virt_to_phys(virt)
}
```

### 3.2.3 VMXON Region Structure

The VMXON region is the first structure required to enter VMX operation. It must be initialized with the VMX revision ID extracted from the `IA32_VMX_BASIC` MSR (0x480).
```rs
pub struct VmxonRegion {
    va: *mut u8,    // Virtual address (kernel use)
    pa: u64,        // Physical address (CPU use)
}

impl VmxonRegion {
    pub unsafe fn new() -> Result<Self, VmxError> {
        // Allocate 4KB zeroed page from kernel
        let va = alloc_zeroed_page();
        if va.is_null() {
            return Err(VmxError::AllocFailed);
        }

        // Convert virtual to physical address
        let pa = virt_to_phys(va);

        // Verify 4KB alignment (bits 11:0 must be zero)
        if (pa & 0xFFF) != 0 {
            free_page(va);
            return Err(VmxError::NotAligned);
        }

        // Write VMCS revision ID to first 4 bytes
        // This is MANDATORY before executing VMXON
        let vmx_basic = rdmsr(0x480);
        let revision_id = vmx_basic as u32;
        core::ptr::write_volatile(va as *mut u32, revision_id);

        pr_alert!(
            "vantablack: VMXON region allocated at PA={:#x}, revision ID={:#x}\n",
            pa, revision_id
        );

        Ok(Self { va, pa })
    }

    pub unsafe fn activate(&self) -> Result<(), VmxError> {
        vmxon(self.pa)
    }

    pub unsafe fn deactivate(&self) {
        vmxoff();
    }

    pub fn physical_address(&self) -> u64 {
        self.pa
    }
}

impl Drop for VmxonRegion {
    fn drop(&mut self) {
        unsafe { free_page(self.va); }
    }
}
```
### 3.2.4 VMCS Region Structure

The VMCS region controls the virtual machine execution. It follows the same allocation pattern but is activated with `VMPTRLD` instead of `VMXON`.
```rs
pub struct VmcsRegion {
    va: *mut u8,
    pa: u64,
}

impl VmcsRegion {
    pub unsafe fn new() -> Result<Self, VmxError> {
        let va = alloc_zeroed_page();
        if va.is_null() {
            return Err(VmxError::AllocFailed);
        }

        let pa = virt_to_phys(va);
        if (pa & 0xFFF) != 0 {
            free_page(va);
            return Err(VmxError::NotAligned);
        }

        // Same revision ID as VMXON
        let vmx_basic = rdmsr(0x480);
        core::ptr::write_volatile(va as *mut u32, vmx_basic as u32);

        pr_alert!("vantablack: VMCS region allocated at PA={:#x}\n", pa);

        Ok(Self { va, pa })
    }

    pub unsafe fn clear(&self) -> Result<(), VmxError> {
        vmclear(self.pa)
    }

    pub unsafe fn load(&self) -> Result<(), VmxError> {
        vmptrld(self.pa)
    }

    pub unsafe fn read(&self, field: u64) -> Result<u64, VmxError> {
        vmread(field)
    }

    pub unsafe fn write(&self, field: u64, value: u64) -> Result<(), VmxError> {
        vmwrite(field, value)
    }
}

impl Drop for VmcsRegion {
    fn drop(&mut self) {
        unsafe { free_page(self.va); }
    }
}
```
### 3.2.5 Complete VMX Initialization Sequence
```rs

pub unsafe fn init_vmx() -> Result<(), VmxError> {
    // Step 1: Enable VMX in CR4
    let mut cr4 = read_cr4();
    cr4 |= 1 << 13;  // CR4.VMXE
    write_cr4(cr4);

    // Step 2: Allocate and initialize VMXON region
    let vmxon = VmxonRegion::new()?;

    // Step 3: Enter VMX operation
    vmxon.activate()?;
    pr_alert!("vantablack: VMXON successful\n");

    // Step 4: Allocate VMCS
    let vmcs = VmcsRegion::new()?;

    // Step 5: Clear and load VMCS
    vmcs.clear()?;
    vmcs.load()?;
    pr_alert!("vantablack: VMCS loaded\n");

    // VMX is now active and ready for configuration
    Ok(())
}
```

## 3.3 Key Technical Considerations

### Memory Alignment

Both VMXON and VMCS require **4KB alignment** (not just 4-byte alignment). The physical address must have bits [11:0] equal to zero. This is verified with `(pa & 0xFFF) != 0`.

### Volatile Writes

The VMX revision ID must be written using `core::ptr::write_volatile()` because the CPU reads the VMXON region directly from physical memory without cache coherency. A regular write might be cached and invisible to the VMX hardware.

### Error Handling

VMX instructions set RFLAGS.CF (Carry Flag) or RFLAGS.ZF (Zero Flag) on error:

- **CF=1**: VMfailInvalid - VMCS/VMXON pointer is invalid
    
- **ZF=1**: VMfailValid - Error code available in VM-instruction error field
    

These flags are checked after each VMX instruction to ensure correct operation.
# Chapter 4: Project Architecture & Cross-Platform Design

My project is cross-platform so i must handle VMX/SVM ( AMD & INTEL) virtualization module 
and for a windows distribution or linux distribution so i abstract  my project on 3 directory : platform / hypervisor core 
