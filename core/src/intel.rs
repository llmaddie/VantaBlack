// VMX VMCS Implmentation 

struct VmxRegion {
    virtual_address: *mut u32,
    physical_address: u64,
}

impl VmxRegion {
    unsafe fn new() -> Result<Self, int> {
        // allocate 4096oct for memory kernel
        // on C it was : __get_free_pages(GFP_KERNEL, 0) -> order 0 = 1 page (4Kb)
        // here we use external "C" to the kernel 
        let va = unsafe { alloc_zeroed_page() };
        if va.is_null() {
            return Err(-1);
        }
        // convert va to physical address for the CPU
        // equal to virt_to_phys(va) on C kernel
        let pa = unsafe { virt_to_phys(va as u64) };

        // check to 4KB
        if (pa & 0xFFF) != 0 {
            pr_alert!("vantablack: allocated page is not 4KB aligned\n");
            // free thee page to bypass leak
            return Err(-2);
        }
        Ok(Self {
            virtual_address: va as *mut u32,
            physical_address: pa,

        })
    }
}

pub fn read_msr{
// read the MSR VMX_BASIC
let vmx_basic = unsafe { read_msr(0x480) };
let revision_id = vmx_basic as u32; // we keep 32bits
// write ID
unsafe {
    *(vmx_region.virtual_address) = revision_id;
}
pr_alert!("vantablack: VMX revision ID {:#x} written to VMXON region\n", revision_id);
}