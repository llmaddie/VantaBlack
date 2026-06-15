#include <ntddk.h>
#include <intrin.h> // Contient les fonctions magiques pour CPUID et MSR

// Fonction de déchargement
VOID VantablackUnload(PDRIVER_OBJECT DriverObject) {
    UNREFERENCED_PARAMETER(DriverObject);
    KdPrint(("Vantablack: Driver unloaded. Bye Ring 0!\n"));
}

// Point d'entrée du driver Windows
extern "C" NTSTATUS DriverEntry(PDRIVER_OBJECT DriverObject, PUNICODE_STRING RegistryPath) {
    UNREFERENCED_PARAMETER(RegistryPath);
    DriverObject->DriverUnload = VantablackUnload;

    KdPrint(("Vantablack: Hello from Windows Kernel in C++!\n"));

    // Check VMX direct avec les fonctions intrinsèques du processeur
    int cpuInfo[4] = { 0 };
    __cpuid(cpuInfo, 1); // Exécute CPUID avec EAX = 1

    // Le bit 5 de ECX est stocké dans cpuInfo[2]
    if ((cpuInfo[2] & (1 << 5)) != 0) {
        KdPrint(("Vantablack: Intel VT-x is supported!\n"));
    } else {
        KdPrint(("Vantablack: Intel VT-x NOT supported.\n"));
    }

    return STATUS_SUCCESS;
}
