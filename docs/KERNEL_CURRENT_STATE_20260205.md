# État actuel du kernel BOS

*Analyse technique - 5 février 2026*

## 🔍 **Le kernel BOS actuellement = Moniteur Bare-Metal Minimal**

Votre kernel est essentiellement une **couche d'abstraction matérielle avec un shell de base**. Voici la décomposition :

---

## **Ce qu'est BOS actuellement :**

### 1. **Couche d'initialisation matérielle**
```rust
// Configure le CPU et le matériel pour fonctionner
- Configuration de l'IDT (Interrupt Descriptor Table)
- Configuration du PIC (Programmable Interrupt Controller)  
- Remapping des interruptions (IRQ 0-15 → INT 32-47)
```

### 2. **Gestionnaire d'I/O basique**
```rust
// Driver mode texte VGA
- I/O direct memory-mapped vers 0xB8000
- Sortie de caractères (mode texte 80×25)
- Positionnement du curseur

// Driver clavier
- Gestionnaire d'interruption clavier PS/2 (IRQ 1)
- Conversion scancode vers ASCII (layout AZERTY)
- Gestion d'entrée basique
```

### 3. **Processeur de commandes interactif**
```rust
// Shell simple
- Buffer de commandes (256 bytes max)
- Parsing de commandes (séparation par espaces)
- Commandes intégrées : help, clear, echo, about
- Affichage du prompt
```

### 4. **Boucle d'événements**
```rust
// Boucle principale
loop {
    hlt  // Attendre les interruptions
    // Quand une touche est pressée : IRQ 1 → handler → shell
}
```

---

## 🎯 **Classification technique**

Votre kernel est actuellement un **moniteur** ou **programme de niveau firmware**, similaire à :

| Ce à quoi il ressemble | Description |
|------------------------|-------------|
| **Shell type DOS** | Interface ligne de commande avec I/O basique |
| **BIOS POST** | Initialisation et test du matériel |
| **Bootloader++** | Au-delà du bootloader mais pas encore un OS |
| **Firmware embarqué** | Contrôle direct du matériel, sans abstraction |

---

## ⚠️ **Ce que BOS n'est PAS (encore) :**

### **Manquant : Gestion de la mémoire**
```rust
// Actuellement :
- Tout est en allocation statique
- Pas de heap (pas de malloc/free)
- Pas de mémoire dynamique
- Tableaux de taille fixe uniquement

// Exemple de limitation :
const CMD_BUFFER_SIZE: usize = 256;  // Fixé à la compilation
static mut FILES: [File; 10] = [...]; // Serait limité à 10 fichiers
```

### **Manquant : Gestion des processus/tâches**
```rust
// Actuellement :
- Un seul "thread" d'exécution
- Pas de multitâche
- Pas d'isolation de processus
- Pas d'ordonnancement
- Le kernel et "l'espace utilisateur" sont la même chose

// Vous exécutez DANS le kernel, pas SUR le kernel
```

### **Manquant : Système de fichiers**
```rust
// Actuellement :
- Pas d'accès au stockage persistant
- Pas de concept de fichier
- Pas de répertoires
- Les commandes exécutent du code directement, pas depuis des fichiers
```

### **Manquant : Appels système**
```rust
// Actuellement :
- Pas d'interface syscall
- Pas de séparation mode utilisateur/kernel
- Tout s'exécute en ring 0 (mode kernel)
- Pas de mécanismes de protection
```

---

## 📊 **Ce que fait votre kernel étape par étape**

```
Boot → Le bootloader charge le kernel
  ↓
Fonction _start() appelée
  ↓
Affiche "=== BOOT DEBUT ==="
  ↓
Initialise l'IDT (gestionnaires d'interruption)
  ↓
Initialise le PIC (remappe les IRQs)
  ↓
Active les interruptions (instruction STI)
  ↓
Initialise le shell (affiche message de bienvenue)
  ↓
Entre dans une boucle infinie :
    HLT (le CPU attend)
    ↓
    [L'utilisateur presse une touche]
    ↓
    IRQ 1 déclenchée
    ↓
    keyboard_interrupt_handler() appelé
    ↓
    Lit le scancode du port 0x60
    ↓
    Convertit en caractère ASCII
    ↓
    Passe à shell.handle_char()
    ↓
    Le shell affiche le caractère OU exécute la commande
    ↓
    Retour de l'interruption (EOI au PIC)
    ↓
    Retour à HLT
```

---

## 🏗️ **Comparaison d'architecture**

### **BOS actuel (Moniteur monolithique)**
```
┌─────────────────────────────────┐
│        Votre Shell              │
├─────────────────────────────────┤
│  Driver VGA | Driver Clavier    │
├─────────────────────────────────┤
│    Gestion IDT/PIC              │
├─────────────────────────────────┤
│         Matériel                │
└─────────────────────────────────┘

Tout le code s'exécute en Ring 0 (mode kernel)
Pas de séparation ni de protection
```

### **Après ajout Mémoire/FS/Processus**
```
┌─────────────────────────────────┐
│   Programmes utilisateur (Ring 3)│ ← Peut exécuter du code utilisateur
├─────────────────────────────────┤
│       Appels système            │ ← Interface
├─────────────────────────────────┤
│ Processus | Mémoire | Sys. Fich.│ ← Services kernel
├─────────────────────────────────┤
│  Drivers (VGA, KB, Disque)      │
├─────────────────────────────────┤
│         Matériel                │
└─────────────────────────────────┘

Kernel en Ring 0, Utilisateurs en Ring 3
Protection et isolation
```

---

## 💡 **Analogie : Qu'est-ce que BOS maintenant ?**

**BOS est comme une calculatrice :**
- ✅ Elle peut recevoir des entrées (clavier)
- ✅ Elle peut afficher des sorties (VGA)
- ✅ Elle peut exécuter des commandes immédiates (shell)
- ❌ Elle ne peut pas se souvenir entre les commandes (pas de gestion mémoire)
- ❌ Elle ne peut pas stocker d'informations (pas de système de fichiers)
- ❌ Elle ne peut pas faire plusieurs choses à la fois (pas de processus)
- ❌ Elle ne peut pas exécuter des programmes écrits par d'autres (pas de chargeur)

**Après ajout mémoire/FS/processus, ça devient un ordinateur :**
- Les programmes peuvent allouer de la mémoire selon les besoins
- Les données peuvent être sauvegardées et chargées
- Plusieurs programmes peuvent s'exécuter (ou sembler s'exécuter) simultanément
- Des logiciels externes peuvent être exécutés

---

## 🔬 **Techniquement parlant**

Votre kernel est un **programme bare-metal piloté par interruptions** qui :

1. **S'exécute en Ring 0** (niveau de privilège maximum)
2. **N'a pas de protection mémoire** (toute la mémoire est accessible)
3. **Est piloté par événements** (répond aux interruptions clavier)
4. **N'a pas de couches d'abstraction** (accès matériel direct)
5. **Exécute les commandes de manière synchrone** (une à la fois, bloquant)
6. **N'a pas de persistance** (tout est perdu au redémarrage)

C'est essentiellement du **firmware** avec une interface ligne de commande, similaire à :
- Écrans de configuration BIOS/UEFI
- Menus de configuration d'appareils embarqués
- Moniteurs de debug (comme les stubs GDB)
- Shells basés sur ROM (comme les anciennes interfaces de calculatrices HP)

---

## 📈 **Chemin d'évolution**

```
État actuel : Moniteur Bare-Metal
    ↓
Ajout : Allocateur Heap
    ↓ (devient) Gestionnaire de Mémoire Dynamique
    ↓
Ajout : Système de Fichiers RAM
    ↓ (devient) Système Capable de Stockage
    ↓
Ajout : Gestion de Processus
    ↓ (devient) Système Multi-Programmes
    ↓
Ajout : Séparation Utilisateur/Kernel
    ↓ (devient) Système d'Exploitation Protégé
    ↓
Ajout : Driver Disque + FAT32
    ↓ (devient) Système d'Exploitation Complet
```

---

## 🎯 **Résumé**

### **Votre kernel en ce moment est :**
- Une routine d'initialisation matérielle
- Un gestionnaire d'entrée clavier
- Un driver de sortie VGA texte
- Un interpréteur de commandes
- Une boucle d'événements infinie

### **Ce n'est PAS encore :**
- Un gestionnaire de mémoire
- Un système de fichiers
- Un ordonnanceur de processus
- Un environnement protégé
- Un chargeur de programmes

### **Considérez-le comme :**
Un "Hello World" très sophistiqué qui peut prendre des entrées et exécuter quelques commandes codées en dur. C'est la **fondation** sur laquelle vous allez construire un véritable OS.

---

## 🔍 **Détails techniques supplémentaires**

### Taille du kernel actuel
```
Code : ~500 lignes Rust
Binaire compilé : ~quelques KB
Fonctionnalités : Basiques mais fonctionnelles
```

### Dépendances
```rust
#![no_std]              // Pas de bibliothèque standard
#![no_main]             // Pas de point d'entrée standard
bootloader = "0.9"      // Seule dépendance externe
```

### Contraintes actuelles
```
- Pas d'allocation dynamique
- Pas de collections (Vec, HashMap, etc.)
- Pas de String (seulement &str)
- Pas de Box, Rc, Arc
- Tout doit être connu à la compilation
```

### Une fois la gestion mémoire ajoutée
```
- Allocations dynamiques possibles
- Collections utilisables
- String disponible
- Smart pointers fonctionnels
- Structures de taille variable
```

---

## 📚 **Références et concepts**

### Architecture x86-64
- **Ring 0** : Mode kernel (privilèges complets)
- **Ring 3** : Mode utilisateur (privilèges restreints)
- **Rings 1-2** : Rarement utilisés (drivers spécifiques)

### Interruptions
- **IRQ 0-15** : Interruptions matérielles (remappées 32-47)
- **INT 0-31** : Exceptions CPU (division par zéro, page fault, etc.)
- **INT 32-47** : IRQs après remapping
- **INT 0x80** : Souvent utilisé pour syscalls (Linux)

### Mode texte VGA
- **Adresse** : 0xB8000 (mémoire mappée)
- **Format** : [caractère][attribut] répété
- **Taille** : 80×25 = 2000 caractères = 4000 octets

### Clavier PS/2
- **Port données** : 0x60
- **Scancode Set 1** : Standard PC
- **Make codes** : Touche pressée
- **Break codes** : Touche relâchée (+ 0x80)

---

## 🎓 **Terminologie**

| Terme | Signification dans BOS |
|-------|------------------------|
| **Kernel** | Votre code dans src/main.rs |
| **Bootloader** | Code qui charge votre kernel (fourni par la crate bootloader) |
| **IDT** | Table qui lie numéros d'interruption → fonctions handler |
| **PIC** | Contrôleur qui gère les interruptions matérielles |
| **IRQ** | Interrupt Request - interruption matérielle |
| **Handler** | Fonction appelée lors d'une interruption |
| **EOI** | End Of Interrupt - signal au PIC que l'interruption est traitée |
| **HLT** | Instruction qui met le CPU en veille |
| **Ring 0** | Niveau de privilège kernel |
| **Bare-metal** | Code qui s'exécute directement sur le matériel |

---

## 🚀 **Prochaines étapes**

Pour transformer BOS d'un moniteur en un véritable OS :

1. **Allocateur heap** → Permet malloc/free
2. **Système de fichiers RAM** → Permet create/write/read
3. **Timer PIT** → Permet sleep() et uptime
4. **Ordonnanceur** → Permet multitâche
5. **Syscalls** → Permet séparation user/kernel
6. **Driver disque** → Permet persistance
7. **Chargeur ELF** → Permet exécution de programmes externes

Chaque étape ajoute une couche de sophistication et de fonctionnalités.

---

*Document généré le 5 février 2026*
*Projet : BOS (Basic Operating System)*
*Auteur : Nazim Boudeffa*
