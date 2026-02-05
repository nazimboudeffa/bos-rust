# Discussion - Roadmap BOS

*Date : 5 février 2026*

## Question 1 : Faire tourner freeDOOM sur BOS

### État actuel de BOS
- ✅ Mode texte VGA 80×25
- ✅ Gestion des interruptions (IDT)
- ✅ Driver clavier PS/2 (AZERTY)
- ✅ Shell de base

### Ce qui manque pour freeDOOM

#### 🎨 1. Mode graphique VGA/VESA (CRITIQUE)
- Passer du mode texte au mode graphique (320×200 ou 640×480 en 256 couleurs)
- Implémenter un framebuffer pixel par pixel
- Driver VESA pour résolutions supérieures

#### 💾 2. Gestionnaire de mémoire (CRITIQUE)
- Allocateur heap (GlobalAlloc trait en Rust)
- Gestion de la pagination (MMU)
- Au moins 4-8 MB de RAM disponible

#### 📁 3. Système de fichiers (CRITIQUE)
- Support FAT32 ou ext2 pour lire les fichiers
- Driver IDE/SATA ou ATA pour accéder au disque
- Lecture des fichiers WAD de DOOM (contiennent niveaux, sprites, sons)

#### 🎮 4. Périphériques supplémentaires
- **Timer PIT** : Pour le timing du jeu, FPS
- **Souris PS/2** : Pour les contrôles
- **Son** : PC Speaker basique ou Sound Blaster 16

#### 🔧 5. Chargeur d'exécutable
- Parser et charger un binaire ELF ou PE
- Ou porter le code source de freeDOOM directement dans le kernel

#### 📊 6. Fonctionnalités système
- Math: sin, cos, sqrt (pour le rendu 3D)
- Memcpy/memset optimisés
- Support des nombres flottants

### Estimation du travail
1. **Mode graphique VGA** : 2-4 semaines
2. **Allocateur mémoire + pagination** : 3-6 semaines
3. **Système de fichiers** : 4-8 semaines
4. **Timer + interruptions avancées** : 1-2 semaines
5. **Driver souris** : 1 semaine
6. **Port de freeDOOM** : 2-4 semaines
7. **Son** (optionnel) : 2-4 semaines

**Total : Plusieurs mois (voire années)**

### Approches alternatives (plus réalistes)
- **Option 1** : Mini-jeux simples (Snake en mode texte, Tetris en VGA)
- **Option 2** : Port d'un moteur plus simple (Wolfenstein 3D)
- **Option 3** : Étapes progressives (mode graphique → allocateur → jeu simple)

---

## Question 2 : Faire tourner Python sur BOS

### Ce qui manque pour Python

#### 💾 1. Gestionnaire de mémoire (CRITIQUE)
- Allocateur heap (GlobalAlloc trait)
- malloc/calloc/realloc/free
- Gestion de la pagination mémoire
- Minimum 8-16 MB de RAM disponible

#### 📚 2. Bibliothèque C standard (libc) - CRITIQUE
Options :
- **newlib** : libc légère pour embedded
- **musl** : libc minimale
- Implémenter sa propre libc minimale

Fonctions nécessaires :
- malloc, free, realloc, calloc
- strcpy, strcmp, strlen, memcpy, memset
- printf, sprintf, scanf
- fopen, fread, fwrite, fclose

#### 📁 3. Système de fichiers (CRITIQUE)
- Driver disque (IDE/SATA/ATA)
- Système de fichiers (FAT32 ou ext2)
- VFS (Virtual File System)
- Pour lire les scripts .py et modules Python

#### 🔧 4. Support POSIX minimal
Appels système nécessaires :
- open(), close(), read(), write()
- stat(), lstat() (pour os.path)
- getcwd(), chdir() (pour os module)
- getenv(), setenv() (variables d'environnement)

#### 🐍 5. Interpréteur Python

**Option A : MicroPython (RECOMMANDÉ)**
- Conçu pour systèmes embarqués
- Beaucoup plus léger (~200 KB)
- Moins de dépendances
- Sous-ensemble de Python 3

**Option B : CPython**
- Interpréteur Python officiel complet
- Très lourd (plusieurs MB)
- Nombreuses dépendances système
- Support complet de Python 3.x

#### ⏱️ 6. Timer système
- Timer PIT pour sleep(), time.time()
- RTC (Real-Time Clock) pour datetime

#### 🔤 7. Support Unicode/Encodage
- UTF-8 pour les strings Python
- Table de conversion de caractères

### Estimation du travail (MicroPython)
1. **Allocateur heap + pagination** : 4-6 semaines
2. **Libc minimale** : 3-5 semaines
3. **Système de fichiers** : 4-8 semaines
4. **Driver disque** : 2-4 semaines
5. **Port MicroPython** : 2-4 semaines
6. **Timer système** : 1 semaine
7. **Support I/O et debugging** : 2-3 semaines

**Total : 4-7 mois**

### Approches alternatives
- **Option 1** : Interpréteur minimaliste custom (2-4 semaines)
- **Option 2** : Lua au lieu de Python (plus léger et adapté)
- **Option 3** : Étapes progressives (bases système → interpréteur simple → MicroPython)

---

## Question 3 : Quel système de fichiers implémenter ?

### 🥇 Recommandation : Implémentation progressive

#### Phase 1 : RAM FS (2-3 semaines) - COMMENCER ICI

**Avantages :**
- ✅ Pas besoin de driver disque
- ✅ Simple à implémenter (structures en mémoire)
- ✅ Rapide pour tester et debugger
- ✅ Parfait pour apprendre les concepts de FS
- ✅ Base solide pour un FS persistant plus tard

**Structure suggérée :**
```rust
struct RamFile {
    name: [u8; 64],        // Nom du fichier
    size: usize,           // Taille en octets
    data: *mut u8,         // Pointeur vers les données
    is_directory: bool,    // Fichier ou dossier
}
```

**Complexité : ⭐⭐☆☆☆**

#### Phase 2 : FAT32 (6-10 semaines) - ENSUITE

**Avantages :**
- ✅ Compatible avec Windows/Linux/Mac
- ✅ Peut lire des clés USB formatées en FAT32
- ✅ Bien documenté (specs Microsoft disponibles)
- ✅ Pas de journal (plus simple que ext3/4)
- ✅ Largement utilisé dans l'embedded

**Inconvénients :**
- ⚠️ Limites : fichiers max 4GB, pas de permissions Unix
- ⚠️ Fragmentation possible
- ⚠️ Nécessite un driver IDE/SATA

**Complexité : ⭐⭐⭐☆☆**

### 🔍 Comparaison détaillée

| Option | Difficulté | Temps | Avantages | Inconvénients |
|--------|-----------|-------|-----------|---------------|
| **FAT12/16** | ⭐⭐☆☆☆ | 4-6 sem | Plus simple que FAT32 | Limité aux petits disques |
| **FAT32** | ⭐⭐⭐☆☆ | 6-10 sem | Standard moderne, compatible | Limites (4GB/fichier) |
| **ext2** | ⭐⭐⭐⭐☆ | 8-12 sem | Permissions Unix, liens | Plus complexe |
| **Custom FS** | ⭐⭐☆☆☆ | 3-5 sem | Adapté aux besoins | Pas de compatibilité |
| **tmpfs/RAM** | ⭐☆☆☆☆ | 2-3 sem | Très simple, rapide | Volatile (perdu au reboot) |

### 📊 Progression recommandée

1. **RAM FS basique** (maintenant) - 2-3 semaines
2. **Driver disque IDE/ATA** (après RAM FS) - 2-3 semaines
3. **FAT32 lecture seule** (ensuite) - 3-4 semaines
4. **FAT32 écriture** (plus tard) - 3-4 semaines

**Total : 10-14 semaines**

### 💡 Pourquoi FAT32 plutôt qu'ext2 ?

1. **Simplicité** : Pas de journal, pas de permissions complexes
2. **Compatibilité** : Préparation de disques sous Windows/Linux possible
3. **Documentation** : Specs officielles Microsoft + nombreux tutoriels
4. **Testabilité** : Facile de créer des images FAT32 pour tester
5. **Cas d'usage** : Parfait pour un OS éducatif

---

## Question 4 : Commencer par gestion mémoire ou système de fichiers ?

### 🎯 Réponse : Gestion mémoire D'ABORD (obligatoire)

### Pourquoi la mémoire en premier ?

#### 1. Le système de fichiers a BESOIN d'allocation
```rust
// Un système de fichiers simple nécessite :
let file = alloc(size_of::<File>());      // Allouer une structure
let buffer = alloc(512);                   // Buffer de secteur
let path = alloc(name.len());              // Copier un nom
```

#### 2. Sans allocateur : limitations importantes
```rust
// Seulement des tableaux statiques de taille fixe
static mut FILES: [File; 10] = [...];     // Max 10 fichiers !
static mut BUFFER: [u8; 512] = [0; 512];  // 1 seul buffer !
```

#### 3. Avec allocateur : flexibilité totale
```rust
// Taille dynamique adaptée aux besoins
let files = Vec::new();                    // Illimité
let buffer = vec![0u8; sector_size];       // Taille variable
let cache = HashMap::new();                // Structures complexes
```

### 📋 Ordre d'implémentation recommandé

#### Phase 1 : Gestion mémoire (4-6 semaines)

**1.1 - Allocateur basique (semaine 1-2)**
```rust
// Bump allocator simple
// Permet d'allouer, mais pas de libérer
```

**1.2 - Allocateur avec libération (semaine 3-4)**
```rust
// Linked list allocator ou buddy allocator
// Permet alloc() ET free()
```

**1.3 - Tests et commandes shell (semaine 5-6)**
```rust
// Commandes pour valider l'allocateur
bos> alloc 1024
bos> free 0x100000
bos> meminfo
bos> memtest
```

#### Phase 2 : Système de fichiers RAM (2-3 semaines)

**2.1 - Structures de base (semaine 1)**
```rust
struct File {
    name: String,      // ← Utilise l'allocateur !
    data: Vec<u8>,     // ← Utilise l'allocateur !
    size: usize,
}
```

**2.2 - API du FS (semaine 2)**
```rust
fn create_file(name: &str) -> Result<FileHandle>
fn write_file(handle: FileHandle, data: &[u8])
fn read_file(handle: FileHandle) -> &[u8]
fn delete_file(name: &str)
```

**2.3 - Commandes shell (semaine 3)**
```rust
bos> create test.txt
bos> write test.txt "Hello"
bos> read test.txt
bos> ls
bos> rm test.txt
```

### ⚠️ Ce qui se passe si on inverse l'ordre

**Sans allocateur :**
```rust
// Problèmes :
// ❌ Max 10 fichiers seulement
// ❌ Chaque fichier max 1024 octets
// ❌ Gaspillage de mémoire (10 × 1024 = 10KB même vide)
// ❌ Impossible à étendre
// ❌ Pas de buffers dynamiques
```

**Avec allocateur :**
```rust
// Avantages :
// ✅ Nombre illimité de fichiers (limité par RAM)
// ✅ Taille de fichier flexible
// ✅ Pas de gaspillage mémoire
// ✅ Structures complexes possibles
// ✅ Buffers adaptés aux besoins
```

---

## Question 5 : Que peut-on tester lors du développement de la gestion mémoire ?

### 🧪 Tests de gestion mémoire

#### Niveau 1 : Tests de base (essentiels)

**1.1 - Allocation simple**
```rust
bos> alloc 64
Allocated: 0x100000 (64 bytes)

// Validation :
// ✓ Le pointeur retourné n'est pas null
// ✓ L'adresse est dans la heap
// ✓ L'adresse est alignée (multiple de 8 ou 16)
```

**1.2 - Allocation multiple**
```rust
bos> alloc 32
0x100000
bos> alloc 64
0x100020
bos> alloc 128
0x100060

// Validation :
// ✓ Chaque allocation retourne une adresse différente
// ✓ Les blocs ne se chevauchent pas
// ✓ L'ordre croissant des adresses est respecté
```

**1.3 - Écriture et lecture**
```rust
bos> alloc 16
0x100000
bos> write 0x100000 "Hello World"
bos> read 0x100000 16
Hello World

// Validation :
// ✓ Pas de page fault à l'écriture
// ✓ Les données écrites sont correctement relues
// ✓ Pas de corruption de données
```

**1.4 - Alignement mémoire**
```rust
bos> alloc 1
0x100000   // Aligné sur 16 bytes
bos> alloc 1
0x100010   // Encore aligné

// Validation :
// ✓ Toutes les adresses sont alignées
// ✓ Respecte les contraintes hardware
```

#### Niveau 2 : Tests de libération

**2.1 - Free basique**
```rust
bos> alloc 64
0x100000
bos> free 0x100000
OK
```

**2.2 - Réutilisation après free**
```rust
bos> alloc 64
0x100000
bos> free 0x100000
bos> alloc 64
0x100000   // ← Même adresse réutilisée !
```

**2.3 - Free multiple**
```rust
bos> alloc 32
bos> alloc 32
bos> alloc 32
bos> free 0x100020   // Libère le milieu
bos> free 0x100000
bos> free 0x100040
```

**2.4 - Coalescence de blocs**
```rust
bos> alloc 32
0x100000
bos> alloc 32
0x100020
bos> free 0x100000
bos> free 0x100020
bos> meminfo
Free blocks: 1 (64 bytes)  // ← Fusion en 1 bloc
```

#### Niveau 3 : Tests de robustesse

**3.1 - Double free (détection d'erreur)**
```rust
bos> free 0x100000
OK
bos> free 0x100000
ERROR: Double free detected
```

**3.2 - Free d'adresse invalide**
```rust
bos> free 0x999999
ERROR: Invalid address
```

**3.3 - Out of memory**
```rust
bos> alloc 1000000
bos> alloc 1000000
bos> alloc 1000000
ERROR: Out of memory
```

**3.4 - Use after free**
```rust
bos> alloc 64
bos> write 0x100000 "test"
bos> free 0x100000
bos> read 0x100000 16
ERROR: Use after free detected
```

#### Niveau 4 : Tests de performance

**4.1 - Benchmark d'allocation**
```rust
bos> benchmark alloc 1000
Time: 125ms (8000 allocs/sec)
```

**4.2 - Fragmentation**
```rust
bos> alloc 32    // A
bos> alloc 32    // B
bos> alloc 32    // C
bos> free B
bos> alloc 64    // Ne peut pas utiliser B
bos> meminfo
Fragmentation: 32 bytes wasted (3.2%)
```

**4.3 - Stress test**
```rust
bos> stress 10000
Allocating and freeing 10000 random blocks...
Success: 10000/10000
Time: 2.5s
```

#### Niveau 5 : Tests d'intégrité

**5.1 - Heap corruption check**
```rust
bos> heapcheck
Scanning heap...
✓ Free list intact
✓ All blocks have valid headers
✓ No overlapping blocks
Heap: OK
```

**5.2 - Guard pages**
```rust
bos> alloc 16
bos> write 0x100010 "OVERFLOW"
ERROR: Heap corruption detected
```

**5.3 - Memory leak detection**
```rust
bos> meminfo
Used: 0 bytes, Free: 1048576 bytes
bos> alloc 100
bos> alloc 200
bos> meminfo
Used: 300 bytes
Leaked: 0 bytes
```

#### Niveau 6 : Tests visuels

**6.1 - Dump de la heap**
```rust
bos> heapdump
0x100000: [USED] 64 bytes
0x100040: [FREE] 128 bytes
0x1000C0: [USED] 32 bytes
```

**6.2 - Graphique ASCII**
```rust
bos> heapmap
[##########----------........................] 25% used
```

### Commandes de test essentielles à implémenter

```rust
// Commandes essentielles
alloc <size>           // Alloue N bytes
free <addr>            // Libère un bloc
meminfo                // Stats générales

// Commandes de test
write <addr> <data>    // Écrit des données
read <addr> <size>     // Lit des données
heapcheck              // Vérifie l'intégrité

// Commandes avancées
heapdump               // Affiche tous les blocs
heapmap                // Carte visuelle
stress <n>             // Test de stress
benchmark <n>          // Test de performance
```

### Tests automatisés suggérés

```rust
bos> selftest
Running memory allocator tests...
[1/10] Basic allocation................ PASS
[2/10] Multiple allocations............ PASS
[3/10] Free and reuse.................. PASS
[4/10] Alignment check................. PASS
[5/10] Out of memory handling.......... PASS
[6/10] Invalid free detection.......... PASS
[7/10] Stress test (1000 ops).......... PASS
[8/10] Fragmentation test.............. PASS
[9/10] Heap integrity check............ PASS
[10/10] Memory leak detection.......... PASS

All tests passed! ✓
```

---

## Question 6 : Pourra-t-on créer des fichiers après ?

### ✅ Oui, absolument !

C'est la progression logique :
```
Gestion mémoire (allocateur heap)
    ↓
Système de fichiers (RAM FS)
    ↓
Commandes de fichiers
```

### 📁 Ce que vous pourrez faire

#### Commandes de création et écriture
```bash
bos> create document.txt
Created: document.txt

bos> write document.txt "Bonjour depuis BOS!"
Written: 20 bytes

bos> read document.txt
Bonjour depuis BOS!

bos> ls
document.txt (20 bytes)

bos> create notes.txt
bos> append notes.txt "Ligne 1"
bos> append notes.txt "Ligne 2"
bos> read notes.txt
Ligne 1
Ligne 2

bos> rm document.txt
bos> ls
notes.txt (14 bytes)
```

#### Structures rendues possibles

```rust
// Avec allocateur :
struct File {
    name: String,           // ← Alloué dynamiquement
    data: Vec<u8>,          // ← Taille dynamique
    size: usize,
    created: u64,
}

struct FileSystem {
    files: Vec<File>,       // ← Liste dynamique
    current_dir: String,    // ← Alloué dynamiquement
}
```

### 🎯 Progression complète

| Phase | Durée | Résultat |
|-------|-------|----------|
| **Phase 1** : Gestion mémoire | 4-6 sem | alloc, free, meminfo |
| **Phase 2** : RAM FS | 2-3 sem | create, write, read, ls, rm ← Vous créez des fichiers ! |
| **Phase 3** : Améliorations FS | 2-4 sem | mkdir, cd, pwd, cp, mv |
| **Phase 4** : Persistance disque | 6-10 sem | FAT32, sauvegarde réelle (optionnel) |

### 💡 Commandes fichiers disponibles

#### Création et écriture
```bash
create <fichier>              # Créer un fichier vide
write <fichier> <contenu>     # Écrire (écrase)
append <fichier> <contenu>    # Ajouter à la fin
```

#### Lecture
```bash
read <fichier>                # Afficher tout
cat <fichier>                 # Alias de read
head <fichier> 10             # 10 premières lignes
tail <fichier> 10             # 10 dernières lignes
```

#### Gestion
```bash
ls                            # Lister
rm <fichier>                  # Supprimer
mv <src> <dest>              # Renommer/déplacer
cp <src> <dest>              # Copier
stat <fichier>               # Infos détaillées
```

#### Répertoires
```bash
mkdir <dossier>              # Créer
cd <dossier>                 # Changer
pwd                          # Afficher courant
tree                         # Arborescence
```

### ⚡ Timeline réaliste

| Semaine | Tâche | Résultat |
|---------|-------|----------|
| 1-2 | Bump allocator | `alloc`, `meminfo` |
| 3-4 | Linked-list allocator | `free`, réutilisation |
| 5-6 | Tests et debug | Allocateur stable |
| 7 | Structures File/FS | Définitions de base |
| 8 | API create/write/read | Opérations basiques |
| 9 | Commandes shell | `create`, `write`, `read`, `ls`, `rm` |
| 10-11 | Répertoires | `mkdir`, `cd`, `pwd` |
| 12+ | Extensions | `cp`, `mv`, `find` |

---

## 🚀 Plan d'action recommandé

### Priorité 1 : Gestion mémoire (MAINTENANT)
1. Implémenter bump allocator (2 semaines)
2. Implémenter linked-list allocator (2 semaines)
3. Ajouter commandes de test (2 semaines)

### Priorité 2 : Système de fichiers RAM (ENSUITE)
1. Structures File et FileSystem (1 semaine)
2. API de base (create, write, read, delete) (1 semaine)
3. Commandes shell (1 semaine)

### Priorité 3 : Améliorations (PLUS TARD)
1. Répertoires (mkdir, cd, pwd)
2. Opérations avancées (cp, mv, find)
3. Driver disque et FAT32 (optionnel)

---

## 📝 Conclusions

### Points clés de la discussion

1. **freeDOOM** nécessite beaucoup trop de composants (plusieurs mois de travail)
2. **Python** nécessite également énormément de travail (4-7 mois minimum pour MicroPython)
3. **Système de fichiers** : Commencer par RAM FS, puis FAT32 si besoin de persistance
4. **Ordre impératif** : Gestion mémoire AVANT système de fichiers
5. **Tests mémoire** : Nombreux tests possibles à tous les niveaux
6. **Création de fichiers** : Sera possible naturellement après l'allocateur mémoire

### Recommandation finale

**Commencer immédiatement par :**
1. Allocateur mémoire (bump puis linked-list)
2. Tests et commandes shell pour valider
3. Système de fichiers RAM simple
4. Commandes de manipulation de fichiers

**Cette approche progressive permet :**
- D'apprendre les concepts fondamentaux
- De tester à chaque étape
- De construire des bases solides
- D'avoir un OS fonctionnel rapidement

---

*Fin de la discussion - Prêt pour l'implémentation !*
