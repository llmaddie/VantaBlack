Si tu veux chercher des papiers théoriques ou des projets mathématiques sur la façon dont on détecte une rupture de cohérence dans le CPU, le terme exact est **Microarchitectural Fingerprinting** ou **Hypervisor Fingerprinting**.

- **Document à chercher :** _"VMware hypervisor fingerprinting"_ (par Pedro Mendes da Silva).
    
- **Le concept sous-jacent :** Comment un code invité (Guest) peut déduire les propriétés physiques du CPU et voir si l'environnement a été "altéré" par une couche intermédiaire, en analysant la structure des caches L1/L2 et les temps de réponse.
    

### La conclusion sur la "Faisabilité"

Si tu lis la documentation technique d'Intel (le _Intel SDM, Volume 3C_), tu verras qu'Intel fournit des outils comme le **TSC Offset** et le **TSC Scaling** directement dans le VMCS (Virtual Machine Control Structure).

L'hyperviseur peut dire au processeur : _"Chaque fois que le Guest appelle RDTSC, retire-lui automatiquement X cycles"_. C'est faisable pour **une** instruction. Mais là où le papier de Korkin (arXiv:1506.04131) a raison, c'est que si l'outil de détection fait une suite d'instructions (`CPUID` -> `INVD` -> `CPUID`), le temps matériel d'exécution de cette suite varie selon la façon dont le processeur physique gère son _pipeline_ et ses _caches_. Un rootkit qui applique bêtement un offset fixe va détruire la distribution statistique normale de ces instructions.

Pour bypasser ça, ton rootkit au Ring -1 devrait intercepter le `RDTSC`, regarder quel est le pattern d'instructions qui vient d'être exécuté, et générer un calcul mathématique (une régression ou une injection de bruit pseudo-aléatoire) pour recréer une courbe de Gauss parfaite. C'est théoriquement modélisable, mais en pratique, l'overhead de calcul nécessaire pour faire cette simulation logicielle te fera perdre encore plus de cycles, te trahissant à coup sûr.