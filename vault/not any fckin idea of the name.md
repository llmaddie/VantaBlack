
i ask to @serexp cause idk 

is a lightweight, research-focused Type-2 hypervisor implemented as a Linux Kernel Module (LKM) in Rust. Leveraging Intel VT-x hardware virtualization, it dynamically virtualizes an already running host OS ("Blue-Pill") and implements Extended Page Tables (EPT) manipulation to achieve transparent memory stealth capabilities ("Shadow Walker").

---

### Étape 1 : Le Check de compatibilité (Le "Hello World" du Ring -1)

Avant de coder, tu dois parler au CPU. Ton premier mini-objectif est d'écrire un petit driver qui vérifie si le flag **VMX** (Virtual Machine Extensions) est disponible et activé dans le BIOS/UEFI.

- **La ressource :** Le "Intel SDM (System Programming Guide), Volume 3C". C'est ta nouvelle Bible.
    
- **L'objectif :** Exécuter l'instruction `CPUID` et vérifier le bit 5 du registre `ECX`.
    

### Étape 2 : Le passage en mode VMX

C'est là que ça devient sérieux. Tu dois allouer des régions de mémoire physique pour ce qu'on appelle la **VMXON region** et la **VMCS** (Virtual Machine Control Structure).

- **La VMCS :** C'est le cerveau de ton hyperviseur. C'est une structure qui contient l'état complet du CPU hôte et du CPU invité.
    
- **Le défi :** Gérer l'alignement mémoire (souvent 4KB) et les adresses physiques (PA) vs virtuelles (VA).
    

### Étape 3 : La gestion de la mémoire (EPT)

C'est l'étape la plus gratifiante pour la cybersécurité. Tu vas implémenter les **Extended Page Tables**.

- **Le concept :** Tu crées une deuxième couche de traduction d'adresses. L'OS invité croit accéder à la RAM physique, mais en réalité, ton hyperviseur intercepte l'accès et peut lui mentir.
    
- **Application sécu :** Tu peux rendre une zone mémoire "invisible" ou mettre des droits "lecture seule" sur des parties critiques du kernel pour détecter des hooks de rootkits.
    

### Étape 4 : Les VM-Exits (L'interception)

Ton hyperviseur est maintenant un fantôme sous l'OS. À chaque fois que l'invité fait une action sensible (écrire dans un registre de contrôle, exécuter une instruction spécifique), le CPU "sort" de la VM et donne la main à ton code.

- **À faire :** Écrire un _Handler_ de VM-Exit qui analyse pourquoi on est sorti, fait le job, et relance la VM.