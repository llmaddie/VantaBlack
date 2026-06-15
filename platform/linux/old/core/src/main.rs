    #![no_main]
    #![allow(missing_docs, unreachable_pub, dead_code)]

    use core::arch::asm;
    use core::sync::atomic::{AtomicPtr, Ordering};
    use kernel::alloc::allocator::Kmalloc;
    use kernel::prelude::Box;
    use kernel::alloc::flags;
    // ============================================
    // FFI LINUX
    // ============================================
// ffi.rs 
    // ============================================
    // PRINTK
    // ============================================
    #[inline]
    pub unsafe fn pr_alert(msg: &str) {
        let mut buf = [0u8; 256];
        let len = core::cmp::min(msg.len(), 255);
        buf[..len].copy_from_slice(&msg.as_bytes()[..len]);
        buf[len] = 0;
        unsafe {
            linux_ffi::vb_printk(buf.as_ptr());
        }
    }

    // ============================================
    // CPUID
    // ============================================

    // cpuid.rs 
    // ============================================
    // MSR
    // ============================================
// instruction.rs
    // ============================================
    // CR4
    // ============================================
// cpu.rs 

    // ============================================
    // VMXON REGION
    // ============================================
// VMX.rs 

    // ============================================
    // GLOBAL STORAGE
    // ============================================

    static VMXON_REGION: AtomicPtr<VmxonRegion> =
        AtomicPtr::new(core::ptr::null_mut());

    // ============================================
    // MODULE INIT
    // ============================================

    #[no_mangle]
    pub extern "C" fn vantablack_init() -> i32 {
        unsafe {
            pr_alert("vantablack_init()\n");

            // ====================================
            // CPUID CHECK
            // ====================================

            let cpu = cpuid(1, 0);

            let vmx_supported = ((cpu.ecx >> 5) & 1) != 0;

            if !vmx_supported {
                pr_alert("VMX unsupported\n");
                return -1;
            }

            pr_alert("VMX supported\n");

            // ====================================
            // IA32_FEATURE_CONTROL
            // ====================================

            let feature_control = read_msr(0x3A);

            let lock_bit = (feature_control & 1) != 0;

            let vmx_enabled = (feature_control & (1 << 2)) != 0;

            if !lock_bit || !vmx_enabled {
                pr_alert("VMX disabled in BIOS\n");
                return -1;
            }

            pr_alert("FEATURE_CONTROL ok\n");

            // ====================================
            // ENABLE CR4.VMXE
            // ====================================

            let mut cr4 = read_cr4();
            cr4 |= 1 << 13;   
            write_cr4(cr4);

            pr_alert("CR4.VMXE enabled\n");

            // ====================================
            // ALLOC VMXON REGION
            // ====================================

            let vmxon = match VmxonRegion::new() {
                Some(v) => v,

                None => {
                    pr_alert("VMXON allocation failed\n");
                    return -1;
                }
            };

            pr_alert("VMXON region allocated\n");

            // ====================================
            // VMXON
            // ====================================

            if vmxon.activate().is_err() {
                pr_alert("VMXON failed\n");
                    linux_ffi::vb_free_pages(vmxon.va(), 0);

                return -1;
            }

            pr_alert("VMXON success\n");

            // ====================================
            // STORE GLOBALLY
            // ====================================

        let boxed = Box::into_raw(
            Box::<VmxonRegion, Kmalloc>::new(
                vmxon,
            flags::GFP_KERNEL,
            )
        .unwrap(),
        );
            VMXON_REGION.store(
                boxed,
                Ordering::SeqCst,
            );

            pr_alert("vantablack initialized\n");
        }

        0
    }

    /// ============================================
    // MODULE EXIT
    // ============================================

    #[no_mangle]
    pub extern "C" fn vantablack_exit() {
        unsafe {
            pr_alert("vantablack_exit()\n");

            let ptr = VMXON_REGION.load(Ordering::SeqCst);

            if ptr.is_null() {
                pr_alert("No VMXON region\n");
                return;
            }

            // ====================================
            // VMXOFF 
            // ====================================

            (*ptr).deactivate();
            pr_alert("VMXOFF done\n");

            // ====================================
            // DISABLE CR4.VMXE
            // ====================================

            let mut cr4 = read_cr4();
            cr4 &= !(1 << 13);     
            write_cr4(cr4);
            pr_alert("CR4.VMXE disabled\n");

            // ====================================
            // FREE PAGES
            // ====================================

            linux_ffi::vb_free_pages(
                (*ptr).va(),
                0,
            );
            pr_alert("Pages freed\n");

            // ====================================
            // FREE BOX
            // ====================================

            let _ = Box::<VmxonRegion, Kmalloc>::from_raw(ptr);
            VMXON_REGION.store(
                core::ptr::null_mut(),
                Ordering::SeqCst,
            );

            pr_alert("vantablack unloaded\n");
        }
    }