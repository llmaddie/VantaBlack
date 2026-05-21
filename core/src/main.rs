// ============================================
// VANTABLACK HYPERVISOR - Cross-Platform Type-2 Hypervisor
// Supports: Intel VMX (VT-x) | AMD SVM (AMD-V)
// Platforms: Linux LKM | Windows Driver (future)
// Features: Blue Pilling | Shadow Walker | EPT/NPT Hooking
// ============================================

#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![feature(asm_const)]
#![feature(naked_functions)]

extern crate alloc;

use core::panic::PanicInfo;

// Platform abstraction
#[cfg(target_os = "linux")]
use crate::platform::linux as platform;
#[cfg(target_os = "windows")]
use crate::platform::windows as platform;

// Architecture detection
use crate::arch::{HypervisorArch, ArchType};
use crate::hypervisor::Hypervisor;
use crate::utils::log::LogLevel;

mod arch;
mod platform;
mod hypervisor;
mod utils;
mod error;

use error::{HypervisorError, Result};

// ============================================
// LINUX KERNEL MODULE GLUE
// ============================================

#[cfg(target_os = "linux")]
mod linux_exports {
    use super::*;
    use platform::ffi::{c_int, c_char, THIS_MODULE};
    
    #[no_mangle]
    pub static mut __this_module: platform::ffi::module = unsafe { core::mem::zeroed() };
    
    #[link_section = ".modinfo"]
    #[used]
    static MODINFO: [u8; 256] = *b"license=GPL\0author=Vantablack\0description=Cross-platform Type-2 Hypervisor\0";
    
    /// Module init - called on insmod
    #[no_mangle]
    pub extern "C" fn init_module() -> c_int {
        match vantablack_init() {
            Ok(_) => 0,
            Err(e) => {
                log::error!("Vantablack init failed: {:?}", e);
                -1
            }
        }
    }
    
    /// Module exit - called on rmmod
    #[no_mangle]
    pub extern "C" fn cleanup_module() {
        if let Err(e) = vantablack_exit() {
            log::error!("Vantablack exit failed: {:?}", e);
        }
    }
}

// ============================================
// WINDOWS DRIVER GLUE (Future implementation)
// ============================================

#[cfg(target_os = "windows")]
mod windows_exports {
    use super::*;
    use platform::ffi::{NTSTATUS, PDRIVER_OBJECT, PUNICODE_STRING};
    
    #[no_mangle]
    pub extern "system" fn DriverEntry(
        _driver_object: PDRIVER_OBJECT,
        _registry_path: PUNICODE_STRING,
    ) -> NTSTATUS {
        match vantablack_init() {
            Ok(_) => 0, // STATUS_SUCCESS
            Err(_) => 0xC0000001u32 as NTSTATUS, // STATUS_UNSUCCESSFUL
        }
    }
}

// ============================================
// CORE INITIALIZATION
// ============================================

/// Global hypervisor instance (per-CPU array in reality)
static mut HYPERVISOR: Option<Hypervisor> = None;

/// Main initialization sequence
fn vantablack_init() -> Result<()> {
    log::info!("Vantablack Hypervisor v0.1.0");
    log::info!("Cross-platform: Intel VMX / AMD SVM");
    
    // 1. Detect CPU architecture
    let arch_type = arch::detect_architecture()?;
    log::info!("Detected architecture: {:?}", arch_type);
    
    // 2. Check CPU features (VMX/SVM support)
    if !arch::check_cpu_support(arch_type) {
        return Err(HypervisorError::CpuNotSupported);
    }
    
    // 3. Initialize per-CPU data structures
    let cpu_count = platform::cpu::get_cpu_count();
    log::info!("CPU count: {}", cpu_count);
    
    // 4. Allocate and initialize hypervisor
    let hv = Hypervisor::new(arch_type, cpu_count)?;
    
    // 5. Install hypervisor on all CPUs (IPI)
    platform::cpu::smp_call_function(|cpu_id| {
        log::info!("Installing on CPU {}", cpu_id);
        hv.install_on_cpu(cpu_id)
    })?;
    
    // 6. Blue pill check - are we already virtualized?
    if hypervisor::blue_pill::detect_nested_virtualization() {
        log::warn!("Nested virtualization detected - entering stealth mode");
        hypervisor::blue_pill::enable_stealth_mode();
    }
    
    // Store global instance
    unsafe {
        HYPERVISOR = Some(hv);
    }
    
    log::info!("Vantablack installed successfully");
    Ok(())
}

/// Cleanup and uninstall
fn vantablack_exit() -> Result<()> {
    log::info!("Vantablack shutting down...");
    
    unsafe {
        if let Some(ref hv) = HYPERVISOR {
            // Uninstall from all CPUs
            platform::cpu::smp_call_function(|cpu_id| {
                hv.uninstall_from_cpu(cpu_id)
            })?;
            
            // Cleanup memory
            hv.destroy()?;
        }
        HYPERVISOR = None;
    }
    
    log::info!("Vantablack removed");
    Ok(())
}

// ============================================
// PANIC HANDLER
// ============================================

#[cfg(target_os = "linux")]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("PANIC: {}", info);
    unsafe { core::intrinsics::abort() }
}

#[cfg(target_os = "windows")]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("PANIC: {}", info);
    loop {}
}