### 1. Le Blue-Pilling (La Pilule Bleue)

Le nom vient de _Matrix_. L'idée est de "virtualiser" un système qui est déjà en train de tourner, à la volée, sans redémarrer.

- **Le concept :** Tu lances ton driver sur ton PC. Ce driver active les fonctions de virtualisation du CPU (Intel VT-x) et "glisse" l'OS actuel (ton Windows ou Arch) à l'intérieur d'une machine virtuelle.
    
- **Le résultat :** L'OS continue de fonctionner normalement, il ne voit rien. Mais en réalité, il n'est plus le maître du matériel. C'est ton hyperviseur (le "VMM") qui contrôle tout.
    
- **Pourquoi c'est puissant :** Comme tu es en **Ring -1** (un niveau de privilège plus profond que le Kernel qui est en Ring 0), tu es invisible pour l'OS. Si l'OS demande au CPU "Est-ce qu'un hyperviseur tourne ?", ton code intercepte la question et répond "Non, non, promis".
    

### 2. Le Shadow Walker (La Mémoire Fantôme)

C'est une technique spécifique de gestion de la mémoire (souvent utilisée via l'**EPT** - Extended Page Tables) pour cacher du code malveillant ou des modifications du kernel.

- **Le problème du hacker :** Si tu modifies le kernel pour cacher un processus, les outils de protection (comme PatchGuard sur Windows) vont scanner la mémoire, voir la modification, et faire un Blue Screen (BSOD).
    
- **La solution Shadow Walker :** Grâce à l'hyperviseur, tu crées une "double vue" de la mémoire :
    
    1. **Vue "Exécution" :** Quand le processeur veut _exécuter_ le code, l'hyperviseur le dirige vers la page mémoire où se trouve ton malware/rootkit.
        
    2. **Vue "Lecture/Ecriture" :** Quand un antivirus ou le système veut _lire_ cette même zone mémoire pour la scanner, l'hyperviseur intercepte l'accès et le redirige vers une page mémoire "propre" qui contient le code original non modifié.
        
- **Le résultat :** L'antivirus scanne du vide ou du code légitime, alors que le processeur est en train d'exécuter ton code caché. C'est une désynchronisation totale entre ce qui est lu et ce qui est exécuté.
    

---