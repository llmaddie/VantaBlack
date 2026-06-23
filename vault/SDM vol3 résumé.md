### 1.2 La VMCS (Virtual-Machine Control Structure)

C'est LE cœur du contrôle. La VMCS contient 6 zones :

- **Guest-state area** : État sauvegardé du guest lors des VM exits
    
- **Host-state area** : État chargé lors des VM exits
    
- **VM-execution control fields** : Contrôles du comportement en non-root
    
- **VM-exit control fields** : Contrôles des VM exits
    
- **VM-entry control fields** : Contrôles des VM entries
    
- **VM-exit information fields** : Informations sur la dernière VM exit (read-only)
    

---

## CHAPITRE 2 : LE TSC (TIME-STAMP COUNTER) - FONDAMENTAUX

### 2.1 Registres et instructions

Table

|Élément|Description|
|:--|:--|
|`IA32_TIME_STAMP_COUNTER` (MSR 10H)|MSR 64 bits contenant le TSC|
|`RDTSC`|Lit TSC dans EDX:EAX|
|`RDTSCP`|Lit TSC + IA32_TSC_AUX dans ECX (atomique, serializing)|
|`IA32_TSC_AUX` (MSR C0000103H)|Registre auxiliaire lu par RDTSCP|
|`IA32_TSC_ADJUST` (MSR 3BH)|Ajustement local par CPU|

### 2.2 Invariant TSC

**CRUCIAL** : Le bit **Invariant TSC** (CPUID.80000007H:EDX[8]) indique que le TSC est invariant à la fréquence du cœur. C'est-à-dire qu'il ne change pas avec P-states, C-states, ou Turbo Boost. Si ce bit est 0, le TSC varie avec la fréquence CPU → **détection facile du blue-pilling possible**.

> **Pour ton hyperviseur** : Tu dois émuler un Invariant TSC même si le hardware ne l'a pas, ou synchroniser parfaitement les fréquences.

---

## CHAPITRE 3 : TSC OFFSET & TSC SCALING - LES ARMES ANTI-DÉTECTION

### 3.1 TSC Offsetting (Bit 3 du Primary Processor-Based VM-Execution Controls)

**Contrôle :** `Use TSC offsetting` (bit 3)

Quand ce bit est **1**, les exécutions de :

- `RDTSC`
    
- `RDTSCP`
    
- `RDMSR` lisant `IA32_TIME_STAMP_COUNTER`
    

Retournent : **`TSC_guest = TSC_host + TSC_offset`**

Le **TSC offset** est un champ 64 bits signé dans la VMCS. Il permet de :

- **Cacher le temps passé dans l'hyperviseur** (VM exit handler)
    
- **Désynchroniser artificiellement** le guest du host
    
- **Émuler un TSC "natif"** qui ne montre pas les pauses VM exits
    

**Formule :**

plain

```plain
Valeur retournée au guest = (TSC_physique × TSC_multiplier / 2^48) + TSC_offset
```

### 3.2 TSC Scaling (Bit 25 du Secondary Processor-Based VM-Execution Controls)

**Contrôle :** `Use TSC scaling` (bit 25) — **NÉCESSITE que TSC offsetting soit activé**

**TSC multiplier** est un champ 64 bits dans la VMCS (fractional, 48 bits de fraction).

**Formule complète :**

plain

```plain
TSC_guest = (TSC_physique × TSC_multiplier / 2^48) + TSC_offset
```

**Bits du TSC multiplier :**

- Bits 63:48 → **Fraction** (permet des ratios < 1)
    
- Bits 47:0 → **Integer part**
    

**Exemples de ratios :**

- Multiplier = 0x0001000000000000 → Ratio 1.0 (pas de scaling)
    
- Multiplier = 0x0000800000000000 → Ratio 0.5 (TSC ralenti 2×)
    
- Multiplier = 0x0002000000000000 → Ratio 2.0 (TSC accéléré 2×)
    

> **Pour ton anti-détection** : Le TSC scaling permet d'émuler une fréquence CPU différente, masquant les variations de fréquence réelles dues au Turbo Boost ou au thermal throttling.

---

## CHAPITRE 4 : VM-EXECUTION CONTROLS DÉTAILLÉS POUR LE TIMING

### 4.1 Primary Processor-Based VM-Execution Controls (Bits critiques)

Table

|Bit|Nom|Utilité Anti-Détection|
|:--|:--|:--|
|3|**Use TSC offsetting**|⭐ OBLIGATOIRE - Masque le temps VM exit|
|12|**RDTSC exiting**|Si 1, RDTSC cause VM exit (lent, mais contrôle total)|
|31|**Activate secondary controls**|Nécessaire pour activer TSC scaling|

**Stratégie blue-pill :**

- **Bit 3 = 1** (TSC offsetting actif)
    
- **Bit 12 = 0** (pas de VM exit sur RDTSC → plus rapide, moins détectable)
    
- **Bit 31 = 1** (secondary controls activés)
    

### 4.2 Secondary Processor-Based VM-Execution Controls

Table

|Bit|Nom|Utilité|
|:--|:--|:--|
|3|**Enable RDTSCP**|Si 0, RDTSCP cause #UD (à activer pour compatibilité guest)|
|25|**Use TSC scaling**|⭐ Permet le ratio TSC (émulation fréquence)|
|31|**Activate tertiary controls**|Pour contrôles avancés|

### 4.3 Tertiary Processor-Based VM-Execution Controls (64 bits)

Contient des bits avancés pour la virtualisation des interruptions utilisateur et autres features récentes.

---

## CHAPITRE 5 : COMPORTEMENT DES INSTRUCTIONS TIMING EN VMX NON-ROOT

### 5.1 RDTSC avec TSC offsetting actif

plain

```plain
IF (CR4.TSD = 0) OR (CPL = 0) OR (CR0.PE = 0)
    IF "Use TSC offsetting" = 1
        EDX:EAX ← (TSC_physique × TSC_multiplier / 2^48) + TSC_offset
    ELSE
        EDX:EAX ← TSC_physique
```

### 5.2 RDTSCP avec TSC offsetting + TSC scaling

plain

```plain
IF (CR4.TSD = 0) OR (CPL = 0)
    IF "Use TSC offsetting" = 1
        EDX:EAX ← (TSC_physique × TSC_multiplier / 2^48) + TSC_offset
    ELSE
        EDX:EAX ← TSC_physique
    ECX ← IA32_TSC_AUX
```

### 5.3 RDMSR sur IA32_TIME_STAMP_COUNTER (MSR 10H)

Même comportement que RDTSC si TSC offsetting est actif.

---

## CHAPITRE 6 : VMX-PREEMPTION TIMER - L'ARMURE INVISIBLE

### 6.1 Principe

Le **VMX-preemption timer** est un compteur 32 bits qui décompte à une **fréquence fixe** (taux du TSC) en VMX non-root operation. Quand il atteint 0 → **VM exit** (preemption).

**Contrôle :** Bit 6 du Pin-Based VM-Execution Controls (`Activate VMX-preemption timer`)

**Champ VMCS :** `VMX-preemption timer value` (32 bits)

### 6.2 Comportement

- Décompte uniquement en **VMX non-root operation**
    
- **Ne décompte PAS pendant les VM exits** (temps hyperviseur caché)
    
- La valeur initiale est rechargée depuis le champ VMCS à chaque VM entry
    
- Après VM exit, la valeur restante est sauvegardée dans le VMCS
    

### 6.3 Anti-détection

Le guest ne peut PAS lire ce timer directement. C'est un mécanisme **transparent** pour forcer des VM exits périodiques sans que le guest ne s'en aperçoive (grâce au TSC offsetting qui masque le temps passé dans l'hyperviseur).

---

## CHAPITRE 7 : LES INSTRUCTIONS QUI CAUSENT VM EXIT (OU PAS)

### 7.1 Instructions timing critiques

Table

|Instruction|VM exit par défaut ?|Contrôle|
|:--|:--|:--|
|`RDTSC`|Non|Bit 12 primary controls|
|`RDTSCP`|Non|Bit 12 primary controls (si Enable RDTSCP = 1)|
|`RDMSR` (TSC)|Non|MSR bitmaps|
|`WRMSR` (TSC)|Non|MSR bitmaps|
|`CPUID`|**OUI**|Bit 7 secondary controls (si désiré)|
|`INVD`|**OUI**|Toujours|
|`WBINVD`|Configurable|Bit 6 secondary controls|

### 7.2 Stratégie blue-pill optimale

Pour minimiser la détection par timing :

1. **TSC offsetting ACTIF** (bit 3 = 1)
    
2. **TSC scaling ACTIF** (bit 25 secondary = 1) si émulation fréquence
    
3. **RDTSC exiting INACTIF** (bit 12 = 0) → pas de VM exit, hardware gère le offset
    
4. **MSR bitmaps ACTIFS** (bit 28 = 1) → filtrer RDMSR/WRMSR sur MSR 10H
    
5. **CPUID exiting ACTIF** (bit 7 secondary = 1) → émuler CPUID pour cacher hyperviseur
    

---

## CHAPITRE 8 : MSR BITMAPS - CONTRÔLE FIN DES ACCÈS MSR

### 8.1 Structure

Quand `Use MSR bitmaps` (bit 28) = 1, deux bitmaps de 4KB chacune :

- **Read bitmap** (adresse dans VMCS) : contrôle RDMSR
    
- **Write bitmap** (adresse dans VMCS) : contrôle WRMSR
    

Chaque bit correspond à un MSR. Bit = 0 → accès direct, Bit = 1 → VM exit.

### 8.2 MSR 10H (IA32_TIME_STAMP_COUNTER)

**Position dans bitmap :** Offset calculé par `(MSR & 0x1FFF) >> 3`, bit `(MSR & 7)`

**Stratégie :**

- Bit = 0 → RDMSR retourne TSC avec offset/scaling appliqué (hardware)
    
- Bit = 1 → VM exit, hyperviseur émule manuellement (plus lent, plus contrôle)
    

> **Recommandation** : Laisser le hardware gérer (bit = 0) avec TSC offsetting actif. C'est **atomique** et **indétectable** par timing.

---

## CHAPITRE 9 : ÉMULATION DU TSC AUX (IA32_TSC_AUX)

### 9.1 Le problème

`RDTSCP` retourne aussi `IA32_TSC_AUX` dans ECX. Ce registre contient généralement l'**APIC ID** du CPU logique. Si l'hyperviseur migre le vCPU entre cœurs physiques sans mettre à jour TSC_AUX, le guest détecte la migration.

### 9.2 Solution

1. **Lire IA32_TSC_AUX** du host au démarrage
    
2. **Émuler une valeur fixe** dans le guest (par exemple, vCPU ID)
    
3. **Intercepter WRMSR sur IA32_TSC_AUX** (MSR C0000103H) si le guest essaie de le modifier
    
4. **Synchroniser** avec la valeur attendue par le guest
    

---

## CHAPITRE 10 : DÉTECTIONS PAR TIMING ET CONTRE-MESURES

### 10.1 Attaques de détection classiques

Table

|Attaque|Principe|Contre-mesure|
|:--|:--|:--|
|**RDTSC delta**|Mesure le temps d'une instruction connue|TSC offsetting + scaling|
|**RDTSCP + CPUID**|CPUID force VM exit, mesure le gap|TSC offsetting masque le temps VM exit|
|**RDTSC + RDMSR(TSC)**|Compare les deux valeurs|Assurer cohérence offset/scaling|
|**Multi-core TSC skew**|Compare TSC entre cœurs|TSC offsetting par vCPU, synchronisation|
|**TSC frequency check**|Vérifie si TSC match CPU frequency|TSC scaling pour émuler fréquence attendue|
|**Invariant TSC check**|Vérifie CPUID.80000007H:EDX[8]|Émuler CPUID si nécessaire|

### 10.2 La technique "RDTSCP + CPUID" (détection VMX classique)

asm

```asm
; Code de détection dans le guest
RDTSCP          ; Read TSC + processor ID
CPUID           ; Force serializing + potentiel VM exit
RDTSCP          ; Read TSC again
SUB             ; Delta = temps CPUID
```

**Sans TSC offsetting :** Le delta révèle le temps passé dans le VM exit handler (10-1000+ cycles).

**Avec TSC offsetting :** Le TSC lu après CPUID inclut déjà l'offset qui compense le temps VM exit. Le delta est "normal".

> **⚠️ Piège :** Si `RDTSC exiting` = 0 mais `CPUID exiting` = 1, le TSC offsetting compense automatiquement le temps VM exit pour RDTSC. MAIS si le guest fait `RDTSC` (sans P) avant et après `CPUID`, il faut que le TSC offset soit mis à jour **avant** le RDTSC suivant.

### 10.3 TSC Deadline Mode (APIC Timer)

Le **TSC-deadline mode** de l'APIC timer (bit 24 de IA32_APIC_BASE) utilise le TSC comme référence pour les interruptions timer.

**Problème :** Si le guest utilise TSC-deadline, il compare le TSC lu avec la deadline programmée. Incohérence = détection.

**Solution :**

- Virtualiser l'APIC timer (voir APIC virtualization, bit 21 primary controls)
    
- Émuler le TSC-deadline avec le TSC offset/scaling appliqué
    
- Ou utiliser **virtual-interrupt delivery** (bit 9 secondary) pour injecter les timers
    

---

## CHAPITRE 11 : VM-EXIT INFORMATION FIELDS (POUR LE DEBUG)

### 11.1 Champs critiques pour le timing

Table

|Champ VMCS|Encoding|Description|
|:--|:--|:--|
|Exit reason|4402H|Pourquoi la VM exit s'est produite|
|Exit qualification|6400H|Info supplémentaire (dépend du reason)|
|Guest linear address|640AH|Adresse linéaire si pertinent|
|Guest physical address|240CH|Adresse physique si EPT violation|
|VM-exit instruction length|440CH|Longueur de l'instruction fautive|
|VM-exit instruction info|440EH|Détails sur l'instruction|

### 11.2 Exit reasons liés au timing

Table

|Reason|Valeur|Cause|
|:--|:--|:--|
|`RDTSC exiting`|10|RDTSC/RDTSCP avec bit 12 = 1|
|`RDMSR`|31|RDMSR sur MSR bitmapé|
|`WRMSR`|32|WRMSR sur MSR bitmapé|
|`EPT violation`|48|Violation EPT (peut être timing si accès MMIO timer)|
|`VMX-preemption timer`|52|Expiration du preemption timer|
|`Control-register accesses`|28|MOV to/from CR (CR4.TSD par exemple)|

---

## CHAPITRE 12 : EPT (EXTENDED PAGE TABLE) ET TIMING

### 12.1 Principe

L'EPT (bit 1 secondary controls) traduit les **Guest Physical Addresses (GPA)** en **Host Physical Addresses (HPA)**.

### 12.2 EPT et timing

**Problème :** Si le guest accède à une page MMIO (ex: HPET, LAPIC timer), l'EPT peut configurer des **EPT violations** qui causent VM exits.

**Solution :**

- Mapper les pages timer guest vers des pages "shadow" émulées
    
- Utiliser **EPT misconfigurations** (reason 49) pour piéger les accès
    
- Ou utiliser **APIC virtualization** (bits 21, 8, 9) pour éviter les VM exits sur accès APIC
    

---

## CHAPITRE 13 : APIC VIRTUALIZATION ET TIMING

### 13.1 Contrôles disponibles

Table

|Bit|Contrôle|Fonction|
|:--|:--|:--|
|21 primary|Use TPR shadow|Active l'ombre du TPR (Task Priority Register)|
|8 secondary|APIC-register virtualization|Virtualise les registres APIC|
|9 secondary|Virtual-interrupt delivery|Livraison d'interruptions virtuelles|
|4 secondary|Virtualize x2APIC mode|Pour mode x2APIC|

### 13.2 Virtual-APIC Page

Page 4KB pointée par le VMCS. Contient l'état virtuel des registres APIC :

- **Offset 080H** : Virtual APIC timer current count
    
- **Offset 0E0H** : Virtual timer divide configuration
    
- **Offset 320H** : Virtual timer LVT entry
    
- **Offset 380H** : Virtual timer initial count
    

> **Pour ton hyperviseur** : Virtualiser complètement le timer APIC permet d'émuler un timing parfait sans dépendre du TSC host, évitant toute fuite de timing.

---

## CHAPITRE 14 : VPID (VIRTUAL PROCESSOR IDENTIFIER)

### 14.1 Fonction

Le VPID (bit 5 secondary controls) tague les TLB entries avec un identifiant de vCPU.

**Impact timing :** Sans VPID, chaque VM entry/exit nécessite un **VM flush** des TLB (INVVPID ou INVEPT), ce qui est **mesurable en temps**. Avec VPID, le hardware garde les traductions taguées → pas de flush → pas de latence mesurable.

> **OBLIGATOIRE** pour l'anti-détection timing.

---

## CHAPITRE 15 : CAPABILITY MSRS - DÉCOUVERTE DES FEATURES

### 15.1 MSRs à consulter au démarrage

Table

|MSR|Adresse|Contenu|
|:--|:--|:--|
|`IA32_VMX_BASIC`|480H|Version VMCS, taille, memory type|
|`IA32_VMX_PINBASED_CTLS`|481H|Capabilités pin-based controls|
|`IA32_VMX_PROCBASED_CTLS`|482H|Capabilités primary processor-based|
|`IA32_VMX_EXIT_CTLS`|483H|Capabilités VM-exit controls|
|`IA32_VMX_ENTRY_CTLS`|484H|Capabilités VM-entry controls|
|`IA32_VMX_MISC`|485H|Misc info (timer rate, etc.)|
|`IA32_VMX_CR0_FIXED0/1`|486H/487H|Bits fixes de CR0|
|`IA32_VMX_CR4_FIXED0/1`|488H/489H|Bits fixes de CR4|
|`IA32_VMX_PROCBASED_CTLS2`|48BH|Capabilités secondary controls|
|`IA32_VMX_EPT_VPID_CAP`|48CH|Capabilités EPT et VPID|
|`IA32_VMX_TRUE_PINBASED_CTLS`|48DH|Vraies capabilités pin-based|
|`IA32_VMX_TRUE_PROCBASED_CTLS`|48EH|Vraies capabilités primary|
|`IA32_VMX_TRUE_EXIT_CTLS`|48FH|Vraies capabilités VM-exit|
|`IA32_VMX_TRUE_ENTRY_CTLS`|490H|Vraies capabilités VM-entry|
|`IA32_VMX_VMFUNC`|491H|Fonctions VM supportées|
|`IA32_VMX_PROCBASED_CTLS3`|492H|Capabilités tertiary controls|

### 15.2 Bits à vérifier pour TSC

Dans `IA32_VMX_PROCBASED_CTLS2` (48BH) :

- Bit 25 : **Use TSC scaling** supporté ?
    

Dans `IA32_VMX_MISC` (485H) :

- Bits 4:0 : **VMX-preemption timer rate** (TSC ticks par tick timer)
    
    - Valeur N signifie : timer décompte toutes les 2^N ticks TSC
        

---

## CHAPITRE 16 : SYNTHÈSE PRATIQUE - CONFIGURATION VMCS POUR ANTI-DÉTECTION

### 16.1 Configuration minimale recommandée

c

```c
// Pin-Based VM-Execution Controls
// Bit 6 = 1 (Activate VMX-preemption timer) - OPTIONNEL
// Bit 0 = 1 (External-interrupt exiting) - selon besoin

// Primary Processor-Based VM-Execution Controls
// Bit 3 = 1  (Use TSC offsetting)        ⭐ CRITIQUE
// Bit 7 = 1  (HLT exiting)               - pour idle detection
// Bit 12 = 0 (RDTSC exiting)             ⭐ CRITIQUE (0 = pas de VM exit)
// Bit 15 = 0 (CR3-load exiting)          - avec EPT
// Bit 16 = 0 (CR3-store exiting)         - avec EPT
// Bit 19 = 1 (CR8-load exiting)          - pour TPR shadow
// Bit 20 = 1 (CR8-store exiting)         - pour TPR shadow
// Bit 21 = 1 (Use TPR shadow)            - APIC virt
// Bit 25 = 1 (Use I/O bitmaps)           - pour piéger ports timer
// Bit 28 = 1 (Use MSR bitmaps)            ⭐ CRITIQUE
// Bit 31 = 1 (Activate secondary controls) ⭐ CRITIQUE

// Secondary Processor-Based VM-Execution Controls
// Bit 0 = 1  (Virtualize APIC accesses)  - si pas EPT full
// Bit 1 = 1  (Enable EPT)                  ⭐ CRITIQUE
// Bit 3 = 1  (Enable RDTSCP)               - compatibilité
// Bit 5 = 1  (Enable VPID)                 ⭐ CRITIQUE
// Bit 8 = 1  (APIC-register virtualization) - timer virt
// Bit 9 = 1  (Virtual-interrupt delivery)   - perf
// Bit 25 = 1 (Use TSC scaling)             ⭐ SI SUPPORTÉ

// VM-Exit Controls
// Bit 15 = 1 (Acknowledge interrupt on exit) - pour PIC/APIC

// VM-Entry Controls
// Bit 2 = 1 (Load debug controls) - si debug
```

### 16.2 Calcul du TSC Offset

c

```c
// À chaque VM exit, mettre à jour le TSC offset pour compenser
// le temps passé dans l'hyperviseur

uint64_t tsc_enter_vmx;    // TSC au moment du VM exit
uint64_t tsc_exit_vmx;     // TSC au moment du VM entry
uint64_t tsc_offset;       // Champ VMCS TSC offset

// Au début du VM exit handler :
rdtsc(tsc_enter_vmx);

// ... traitement VM exit ...

// Avant VM entry :
rdtsc(tsc_exit_vmx);
uint64_t time_in_hypervisor = tsc_exit_vmx - tsc_enter_vmx;

// Ajouter au TSC offset existant
tsc_offset += time_in_hypervisor;
vmwrite(VMCS_TSC_OFFSET, tsc_offset);
```

### 16.3 Calcul du TSC Multiplier (Scaling)

c

```c
// Si tu veux émuler une fréquence CPU différente
// Ex: Host à 3.6GHz, Guest doit voir 2.4GHz

double host_freq = 3600000000.0;  // Hz
double guest_freq = 2400000000.0;  // Hz
double ratio = guest_freq / host_freq;  // 0.666...

// Conversion en format VMCS TSC multiplier
// Format: 16 bits integer, 48 bits fraction
uint64_t multiplier = (uint64_t)(ratio * (double)(1ULL << 48));

vmwrite(VMCS_TSC_MULTIPLIER, multiplier);
```

---

## CHAPITRE 17 : PIEÈGES ET EDGE CASES

### 17.1 TSC_DEADLINE et VM exits

Quand le guest programme un **TSC deadline** (mode deadline de l'APIC timer), il écrit un TSC absolu dans le registre IA32_TSC_DEADLINE (MSR 6E0H).

**Problème :** Si le TSC offset change entre la programmation et l'expiration, la deadline ne correspond plus.

**Solution :**

- Intercepter WRMSR sur IA32_TSC_DEADLINE (MSR bitmap bit = 1)
    
- Convertir la deadline guest → host : `deadline_host = (deadline_guest - tsc_offset) / ratio`
    
- Programmer le vrai timer avec la valeur host
    
- À l'expiration, injecter interruption virtuelle
    

### 17.2 RDTSCP et CPU migration

Si le vCPU migre entre deux cœurs physiques avec des TSC différents (non-synchronisés) :

**Problème :** Le guest voit un saut de TSC.

**Solution :**

- S'assurer que tous les cœurs ont le TSC synchronisé (WRMSR IA32_TIME_STAMP_COUNTER au boot)
    
- Ou utiliser TSC offsetting pour compenser la différence par cœur
    
- Ou éviter la migration (pin vCPU à pCPU)
    

### 17.3 IA32_TSC_ADJUST

Ce MSR permet à chaque cœur d'avoir un ajustement local du TSC. Si le guest le lit :

**Solution :**

- Intercepter RDMSR/WRMSR sur IA32_TSC_ADJUST (MSR 3BH)
    
- Émuler une valeur cohérente avec le TSC offset
    

### 17.4 CPUID leaf 15H/16H (TSC frequency)

CPUID leaf 15H et 16H retournent la fréquence TSC et CPU. Si le guest les utilise pour calibrer :

**Solution :**

- Intercepter CPUID (bit 7 secondary = 1, ou always cause VM exit)
    
- Émuler les valeurs correspondant au TSC scaling configuré
    

---

## CHAPITRE 18 : CHECKLIST IMPLEMENTATION

### 18.1 Phase 1 : VMX Basic Setup

- [ ] Vérifier CPUID.1:ECX[5] (VMX support)
    
- [ ] Activer CR4.VMXE
    
- [ ] Allouer VMXON region (4KB aligné)
    
- [ ] Exécuter VMXON
    
- [ ] Allouer VMCS region (4KB aligné)
    
- [ ] VMPTRLD le VMCS
    

### 18.2 Phase 2 : VMCS Configuration (Timing)

- [ ] Lire IA32_VMX_PROCBASED_CTLS2 → vérifier bit 25 (TSC scaling)
    
- [ ] Configurer Primary Controls : bit 3 (TSC offsetting) = 1, bit 31 (secondary) = 1
    
- [ ] Configurer Secondary Controls : bit 25 (TSC scaling) = 1 si dispo, bit 5 (VPID) = 1, bit 1 (EPT) = 1
    
- [ ] Configurer MSR bitmaps : bit 0 pour MSR 10H (TSC) = 0 (pas de VM exit)
    
- [ ] Allouer Virtual-APIC page si APIC virtualization
    
- [ ] Configurer EPTP (Extended-Page-Table Pointer)
    

### 18.3 Phase 3 : TSC Offset Management

- [ ] Lire TSC host au démarrage
    
- [ ] Initialiser TSC offset = 0
    
- [ ] À chaque VM exit : mesurer temps hyperviseur, ajouter à TSC offset
    
- [ ] Si TSC scaling : calculer multiplier selon fréquence émulée
    

### 18.4 Phase 4 : Anti-Détection Avancée

- [ ] Intercepter CPUID pour cacher VMX (leaf 1, leaf 7, leaf 0x80000001)
    
- [ ] Émuler IA32_TSC_AUX cohérent
    
- [ ] Virtualiser APIC timer (TSC deadline mode)
    
- [ ] Gérer IA32_TSC_ADJUST si lu par guest
    
- [ ] Vérifier cohérence RDTSC vs RDMSR(TSC) vs RDTSCP
    

---

## CHAPITRE 19 : ENCODINGS VMCS (POUR VMREAD/VMWRITE)

### 19.1 Champs TSC clés

Table

|Champ|Encoding|Width|Type|
|:--|:--|:--|:--|
|TSC offset|4008H|64 bits|VM-execution control|
|TSC multiplier|400AH|64 bits|VM-execution control|
|VMX-preemption timer value|482EH|32 bits|VM-execution control|
|Guest IA32_TSC_AUX|2803H|64 bits|Guest state|
|Host IA32_TSC_AUX|6C03H|64 bits|Host state|
|Guest IA32_TSC_ADJUST|2808H|64 bits|Guest state|
|Host IA32_TSC_ADJUST|6C08H|64 bits|Host state|
|Guest IA32_TSC_DEADLINE|282EH|64 bits|Guest state|

---

## CONCLUSION : LA STRATÉGIE BLUE-PILL ULTIME

Pour un hyperviseur type 1.5 indétectable par timing :

1. **TSC offsetting** : Masque le temps passé dans l'hyperviseur (VM exits invisibles)
    
2. **TSC scaling** : Émule une fréquence CPU cohérente indépendante du host
    
3. **Pas de VM exit sur RDTSC/RDTSCP** : Hardware applique offset/scaling atomiquement
    
4. **VPID activé** : Évite les flushes TLB mesurables
    
5. **EPT activé** : Gestion mémoire sans VM exits coûteux
    
6. **APIC virtualization** : Timer interrupts sans VM exits
    
7. **MSR bitmaps** : Filtrage fin des accès timer (TSC, TSC_ADJUST, TSC_DEADLINE)
    
8. **CPUID interception** : Cacher la présence VMX, émuler fréquences cohérentes
    

Le **TSC offset** est ton bouclier principal. Le **TSC scaling** est ton arme d'émulation. Ensemble, ils permettent de créer une **réalité temporelle parfaitement cohérente** pour le guest, quelle que soit la charge du host ou les optimisations d'énergie.

---

Tu veux que j'approfondisse un chapitre spécifique, ou que je génère du code C/ASM pour la configuration VMCS ?

High demand. Switched to K2.6 Instant for speed. Upgrade to use K2.6 Thinking.