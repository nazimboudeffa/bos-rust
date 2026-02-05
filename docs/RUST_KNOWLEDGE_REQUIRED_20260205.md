# Connaissances Rust nécessaires pour développer BOS

*Guide d'apprentissage - 5 février 2026*

Ce document liste l'ensemble des connaissances Rust nécessaires pour comprendre, maintenir et étendre le projet BOS (Basic Operating System).

---

## 📚 **Niveau 1 : Fondamentaux Rust (INDISPENSABLE)**

### **1.1 - Types et variables**
```rust
// Utilisé partout dans BOS
let x: u8 = 42;                    // Types entiers (u8, u16, u32, u64, usize)
let y: usize = 0x100000;           // Hexadécimal pour adresses mémoire
const VGA_WIDTH: usize = 80;       // Constantes
static mut CURSOR: usize = 0;      // Variables statiques mutables
```

**Pourquoi c'est important :**
- Les types entiers sont utilisés pour les adresses mémoire, les ports I/O, les scancodes
- Les constantes définissent les paramètres hardware (taille écran, taille heap, etc.)
- Les variables statiques mutables stockent l'état global (curseur, shell, allocateur)

### **1.2 - Fonctions**
```rust
// Types de retour, paramètres
fn vga_print(s: &str) { ... }                    // Fonction simple
fn create_file(name: &str) -> Result<(), &str>   // Avec gestion d'erreur
unsafe fn outb(port: u16, value: u8) { ... }     // Fonction unsafe
```

**Concepts clés :**
- Paramètres par référence (`&str`, `&[u8]`)
- Types de retour (unit `()`, `Result`, `Option`)
- Fonctions `unsafe` pour opérations dangereuses

### **1.3 - Structures et méthodes**
```rust
// Utilisé pour File, Shell, IDT, etc.
struct Shell {
    buffer: [u8; 256],
    position: usize,
}

impl Shell {
    const fn new() -> Shell { ... }      // Fonction constructeur
    fn handle_char(&mut self, c: char)   // Méthode mutante
}
```

**Dans BOS :**
- `Shell` : Gère le buffer de commandes
- `IdtEntry` : Entrée dans la table d'interruptions
- `File` : Structure fichier (après implémentation FS)
- `Allocator` : Gestionnaire heap (à implémenter)

### **1.4 - Ownership et Borrowing**
```rust
// Crucial pour comprendre les erreurs du compilateur
fn take_ownership(s: String) { ... }     // Prend ownership
fn borrow_ref(s: &str) { ... }           // Emprunte (immutable)
fn borrow_mut(s: &mut String) { ... }    // Emprunte (mutable)
```

**Règles fondamentales :**
1. Chaque valeur a un propriétaire unique
2. On peut avoir plusieurs références immutables OU une référence mutable
3. Les références doivent toujours être valides

**Impact sur BOS :**
- Évite les use-after-free
- Garantit la sécurité mémoire (même en `unsafe`)
- Force à penser à la durée de vie des données

### **1.5 - Pattern matching**
```rust
// Utilisé dans le shell pour dispatcher les commandes
match command {
    "help" => self.cmd_help(),
    "clear" => self.cmd_clear(),
    "echo" => self.cmd_echo(args),
    _ => { /* commande inconnue */ }
}
```

**Utilisations dans BOS :**
- Dispatch des commandes shell
- Gestion des scancodes clavier
- Traitement des erreurs avec `Result`
- Parsing de structures de données

---

## ⚙️ **Niveau 2 : Rust systèmes bas niveau (ESSENTIEL pour OS)**

### **2.1 - Programmation unsafe**
```rust
// TRÈS utilisé dans BOS (accès matériel direct)
unsafe {
    // Déréférencer des pointeurs raw
    *VGA_BUFFER.add(cursor) = byte;
    
    // Accéder à static mut
    VGA_CURSOR += 2;
    
    // Appeler des fonctions unsafe
    outb(0x20, 0x20);
    
    // Assembleur inline
    asm!("sti", options(nostack));
}
```

**Pourquoi `unsafe` est nécessaire :**
- Accès direct à la mémoire hardware (VGA, clavier)
- Manipulation de pointeurs raw
- Communication avec ports I/O
- Assembleur inline pour instructions CPU

**Responsabilités dans `unsafe` :**
- Vous garantissez manuellement la sécurité
- Pas de vérification du compilateur
- Bugs possibles : corruption mémoire, race conditions

### **2.2 - Pointeurs raw**
```rust
// Manipulation directe de la mémoire
const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;

unsafe {
    *VGA_BUFFER.add(0) = b'A';           // Écriture
    let value = *VGA_BUFFER.add(100);    // Lecture
}
```

**Types de pointeurs :**
- `*const T` : Pointeur immutable
- `*mut T` : Pointeur mutable
- Différence avec références : pas de vérifications du borrow checker

**Opérations courantes :**
```rust
let ptr = 0x100000 as *mut u8;         // Cast adresse → pointeur
let offset_ptr = ptr.add(10);          // Arithmétique de pointeurs
unsafe { *ptr = 42; }                  // Déréférencement
let addr = ptr as usize;               // Pointeur → adresse
```

### **2.3 - Tableaux et slices**
```rust
// Structures de données de taille fixe
let buffer: [u8; 256] = [0; 256];         // Tableau fixe
let slice: &[u8] = &buffer[0..10];        // Slice (vue)
let scancode_table: [char; 58] = [...]    // Table de conversion
```

**Différences importantes :**
- **Tableau** : Taille fixe connue à la compilation, sur la pile
- **Slice** : Vue sur un tableau, taille dynamique, sur pile ou heap
- **Vec** : Tableau dynamique sur heap (nécessite allocateur)

**Dans BOS :**
```rust
static SCANCODE_TABLE: [char; 58] = [...];  // Table de conversion clavier
let buffer: [u8; 256] = [0; 256];           // Buffer shell
```

### **2.4 - Représentation mémoire (#[repr])**
```rust
// Contrôle du layout en mémoire (critique pour hardware)
#[repr(C)]          // Layout compatible C
#[repr(C, packed)]  // Sans padding (IDT entries)
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    zero: u8,
    type_attr: u8,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}
```

**Options de #[repr] :**
- `#[repr(C)]` : Layout compatible avec C (interopérabilité)
- `#[repr(packed)]` : Pas de padding entre champs (économie mémoire)
- `#[repr(align(N))]` : Force l'alignement à N bytes

**Pourquoi c'est crucial :**
- Le hardware attend un format précis (IDT, GDT, page tables)
- Pas de padding = taille exacte requise
- Ordre des champs = ordre en mémoire

### **2.5 - static mut et synchronisation**
```rust
// Variables globales mutables (dangereuses mais nécessaires)
static mut SHELL: Shell = Shell::new();

unsafe {
    SHELL.handle_char('a');  // Accès direct
}
```

**Problèmes avec static mut :**
- Potentiellement unsafe (race conditions)
- Pas de vérification du borrow checker
- Nécessite `unsafe` pour accéder

**Alternatives plus sûres (après allocateur) :**
```rust
use core::sync::atomic::{AtomicUsize, Ordering};
static COUNTER: AtomicUsize = AtomicUsize::new(0);
COUNTER.fetch_add(1, Ordering::SeqCst);  // Thread-safe
```

---

## 🔧 **Niveau 3 : Features Rust avancées (NÉCESSAIRE)**

### **3.1 - Attributes (#[...])**
```rust
#![no_std]                    // Désactive std
#![no_main]                   // Pas de fn main()
#![feature(abi_x86_interrupt)] // Features unstable

#[no_mangle]                  // Empêche le name mangling
#[panic_handler]              // Handler de panic custom
#[repr(C, packed)]            // Layout mémoire
```

**Attributes au niveau crate (`#![...]`) :**
- `#![no_std]` : Pas de bibliothèque standard (seulement `core`)
- `#![no_main]` : Point d'entrée custom (`_start` au lieu de `main`)
- `#![feature(...)]` : Active des features Rust instables

**Attributes au niveau item (`#[...]`) :**
- `#[no_mangle]` : Garde le nom de fonction tel quel (pour linker)
- `#[panic_handler]` : Définit le comportement en cas de panic
- `#[inline]` : Suggère l'inlining de la fonction
- `#[derive(...)]` : Génère automatiquement implémentation de traits

### **3.2 - Inline assembly**
```rust
// Interaction directe avec le CPU
unsafe {
    asm!(
        "out dx, al",          // Instruction x86
        in("dx") port,         // Entrées
        in("al") value,        // Registres
        options(nostack, preserves_flags)
    );
}
```

**Syntaxe assembleur inline :**
```rust
asm!(
    "instruction",             // Code assembleur
    in(reg) variable,          // Registre d'entrée
    out(reg) variable,         // Registre de sortie
    options(...)               // Options
);
```

**Instructions courantes dans BOS :**
- `sti` : Set Interrupt flag (active interruptions)
- `cli` : Clear Interrupt flag (désactive interruptions)
- `hlt` : Halt (met CPU en veille)
- `out dx, al` : Écrit sur port I/O
- `in al, dx` : Lit depuis port I/O
- `lidt [addr]` : Charge l'IDT

### **3.3 - Const functions**
```rust
// Fonctions évaluées à la compilation
const fn new() -> Shell {
    Shell {
        buffer: [0; 256],
        position: 0,
    }
}

// Permet d'initialiser des statics
static mut SHELL: Shell = Shell::new();
```

**Restrictions des const fn :**
- Pas d'allocation heap
- Pas de boucles (dans anciennes versions)
- Pas de pointeurs raw (sauf cas limités)
- Évaluation à la compilation seulement

**Utilité :**
- Initialisation de variables statiques
- Calculs à la compilation (optimisation)
- Tableaux de taille constante

### **3.4 - Macros**
```rust
// core::fmt pour formattage (write!, format!)
use core::fmt::{self, Write};

// Custom macros (optionnel mais utile)
macro_rules! print {
    ($($arg:tt)*) => {
        vga_print(&format!($($arg)*));
    };
}
```

**Macros utiles en no_std :**
```rust
println!("x = {}", x);        // Nécessite std (pas disponible)
write!(buffer, "x = {}", x);  // Disponible avec core::fmt
format!("x = {}", x);         // Nécessite allocateur
```

**Créer ses propres macros :**
```rust
macro_rules! debug {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        vga_print(&format!("[DEBUG] {}\n", format_args!($($arg)*)));
    };
}
```

### **3.5 - Traits de base**
```rust
// Copy, Clone pour les types simples
#[derive(Clone, Copy)]
struct IdtEntry { ... }

// Debug pour affichage
#[derive(Debug)]
struct File { ... }
```

**Traits importants :**
- `Copy` : Type copiable par memcpy (entiers, pointeurs)
- `Clone` : Type clonabable explicitement (`.clone()`)
- `Debug` : Formattage debug (`{:?}`)
- `Default` : Valeur par défaut (`Default::default()`)

**Implémentation manuelle :**
```rust
impl Clone for MyStruct {
    fn clone(&self) -> Self {
        // Implémentation custom
    }
}
```

---

## 🚀 **Niveau 4 : Pour aller plus loin (UTILE)**

### **4.1 - Allocator API (pour heap)**
```rust
// Quand vous implémenterez l'allocateur
use core::alloc::{GlobalAlloc, Layout};

#[global_allocator]
static ALLOCATOR: MyAllocator = MyAllocator;

unsafe impl GlobalAlloc for MyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Votre implémentation
        // 1. Trouver un bloc libre de taille >= layout.size()
        // 2. Aligner selon layout.align()
        // 3. Retourner le pointeur ou null
    }
    
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Votre implémentation
        // 1. Marquer le bloc comme libre
        // 2. Fusionner avec blocs adjacents si possible
    }
}
```

**Concepts clés :**
- `Layout` : Décrit la taille et l'alignement requis
- `GlobalAlloc` : Trait pour allocateur global
- `#[global_allocator]` : Désigne l'allocateur à utiliser

**Une fois implémenté, vous pouvez utiliser :**
```rust
extern crate alloc;
let v = alloc::vec![1, 2, 3];
let s = alloc::string::String::from("hello");
let b = alloc::boxed::Box::new(42);
```

### **4.2 - Collections (après allocateur)**
```rust
// Disponibles après avoir implémenté GlobalAlloc
extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;

let files = Vec::new();
let name = String::from("test.txt");
let cache = BTreeMap::new();
```

**Collections disponibles dans `alloc` :**
- `Vec<T>` : Tableau dynamique
- `String` : Chaîne UTF-8 dynamique
- `Box<T>` : Pointeur unique vers heap
- `Rc<T>` : Pointeur partagé (reference counted)
- `BTreeMap<K, V>` : Map ordonnée
- `BTreeSet<T>` : Set ordonné
- `LinkedList<T>` : Liste chaînée

**Note :** Pas de `HashMap` (nécessite un hasher avec randomness)

### **4.3 - Gestion d'erreurs avancée**
```rust
// Result et Option pour robustesse
fn read_file(name: &str) -> Result<&[u8], FileError> {
    if !file_exists(name) {
        return Err(FileError::NotFound);
    }
    // ...
    Ok(data)
}

// Custom error types
#[derive(Debug)]
enum FileError {
    NotFound,
    PermissionDenied,
    IOError,
    InvalidName,
}
```

**Pattern matching sur Result :**
```rust
match read_file("test.txt") {
    Ok(data) => process(data),
    Err(FileError::NotFound) => create_file("test.txt"),
    Err(e) => panic!("Error: {:?}", e),
}
```

**Opérateur `?` (propagation d'erreur) :**
```rust
fn operation() -> Result<(), FileError> {
    let data = read_file("test.txt")?;  // Retourne early si Err
    write_file("out.txt", data)?;
    Ok(())
}
```

### **4.4 - Lifetimes**
```rust
// Parfois nécessaire pour références complexes
fn get_file<'a>(fs: &'a FileSystem, name: &str) -> Option<&'a File> {
    fs.files.iter().find(|f| f.name == name)
}
```

**Concepts de lifetimes :**
- `'a` : Nom de lifetime (durée de vie)
- La référence retournée vit aussi longtemps que `fs`
- Évite les dangling pointers

**Lifetime elision (implicite) :**
```rust
// Ces deux signatures sont équivalentes :
fn first(s: &str) -> &str
fn first<'a>(s: &'a str) -> &'a str
```

**Cas complexes :**
```rust
// Plusieurs lifetimes
fn longest<'a, 'b>(x: &'a str, y: &'b str) -> &'a str
    where 'b: 'a  // 'b vit au moins aussi longtemps que 'a
{
    if x.len() > y.len() { x } else { y }
}
```

### **4.5 - Iterators**
```rust
// Pour parcourir efficacement
for byte in data.iter() {
    process(byte);
}

let names: Vec<_> = files.iter()
    .filter(|f| f.size > 0)
    .map(|f| &f.name)
    .collect();
```

**Méthodes d'iterator courantes :**
- `.iter()` : Itérateur sur références `&T`
- `.iter_mut()` : Itérateur sur références mutables `&mut T`
- `.into_iter()` : Itérateur qui consomme (prend ownership)
- `.map(f)` : Transforme chaque élément
- `.filter(p)` : Garde éléments qui respectent prédicat
- `.fold(init, f)` : Réduit en accumulant
- `.collect()` : Collecte en collection

**Créer ses propres iterators :**
```rust
struct FileIterator<'a> {
    files: &'a [File],
    index: usize,
}

impl<'a> Iterator for FileIterator<'a> {
    type Item = &'a File;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.files.len() {
            let file = &self.files[self.index];
            self.index += 1;
            Some(file)
        } else {
            None
        }
    }
}
```

---

## 📖 **Connaissances spécifiques OS**

### **5.1 - Architecture x86-64**

**Registres généraux :**
```
RAX, RBX, RCX, RDX  // Registres 64-bit généraux
RSI, RDI            // Source/Destination index
RBP, RSP            // Base/Stack pointer
R8-R15              // Registres supplémentaires x64
```

**Segments et descripteurs :**
- GDT (Global Descriptor Table) : Définit les segments mémoire
- IDT (Interrupt Descriptor Table) : Table des interruptions
- LDT (Local Descriptor Table) : Rarement utilisé

**Ports I/O :**
```rust
0x60, 0x64  // Clavier PS/2
0x3D4, 0x3D5  // VGA cursor
0x20, 0x21  // PIC maître
0xA0, 0xA1  // PIC esclave
0x40-0x43   // PIT (Programmable Interval Timer)
0x70, 0x71  // RTC (Real-Time Clock)
```

**Interruptions :**
```
INT 0-31    // Exceptions CPU
INT 32-47   // IRQs matérielles (après remapping)
INT 0x80    // Syscall Linux (convention)
```

**Niveaux de privilège (rings) :**
```
Ring 0  // Kernel (accès complet)
Ring 1  // Rarement utilisé
Ring 2  // Rarement utilisé
Ring 3  // User mode (accès restreint)
```

### **5.2 - Bits et manipulation**
```rust
// Très utilisé pour flags et registres
let flags = 0b10101010;
if flags & 0x80 != 0 { ... }      // Test bit 7
let value = (addr >> 16) as u16;   // Shift right 16 bits
let masked = value & 0xFF;         // Garde 8 bits bas
let combined = (hi << 8) | lo;     // Combine deux bytes
```

**Opérations bitwise courantes :**
```rust
// AND : masquer des bits
let lower_byte = value & 0xFF;

// OR : définir des bits
let flags = flags | 0x80;  // Set bit 7

// XOR : inverser des bits
let inverted = value ^ 0xFF;

// Shifts
let doubled = x << 1;   // Multiplication par 2
let halved = x >> 1;    // Division par 2

// Rotation (pas en Rust standard, nécessite intrinsics)
let rotated = x.rotate_left(4);
```

**Bit fields (struct avec bits) :**
```rust
// Pas directement supporté, mais peut simuler :
const PRESENT: u8 = 1 << 7;
const DPL_RING3: u8 = 3 << 5;
const GATE_TYPE_INT: u8 = 0x0E;

let type_attr = PRESENT | DPL_RING3 | GATE_TYPE_INT;
```

### **5.3 - Conversion de types**
```rust
// Casts pour adresses et hardware
let addr = 0xB8000 as *mut u8;           // usize → pointeur
let port = 0x60 as u16;                  // i32 → u16
let byte = c as u8;                      // char → u8
let num = byte as usize;                 // u8 → usize

// Transmute (très dangereux, éviter)
let float_bits: u32 = unsafe { core::mem::transmute(3.14f32) };
```

**Conversions sûres :**
```rust
// TryFrom/TryInto pour conversions faillibles
use core::convert::TryInto;
let x: u32 = 1000;
let y: u8 = x.try_into().unwrap_or(255);  // Saturate si trop grand

// From/Into pour conversions infaillibles
let x: u32 = 42u8.into();
```

---

## 🎯 **Synthèse par priorité**

### **ABSOLUMENT NÉCESSAIRE (pour comprendre le code actuel) :**
1. ✅ **Bases Rust** : variables, fonctions, struct, impl
2. ✅ **Ownership et borrowing** : comprendre les emprunts
3. ✅ **Pattern matching** : `match`, `if let`
4. ✅ **`unsafe` blocks** : quand et pourquoi
5. ✅ **Pointeurs raw** : `*mut`, `*const`, déréférencement
6. ✅ **`static mut`** : variables globales
7. ✅ **`#[repr]` et attributes** : contrôle layout mémoire

### **TRÈS UTILE (pour étendre BOS) :**
1. ⭐ **Inline assembly** : instructions CPU directes
2. ⭐ **Const functions** : initialisation static
3. ⭐ **Traits de base** : Copy, Clone, Debug
4. ⭐ **Result et Option** : gestion d'erreurs
5. ⭐ **Slices et références** : manipulation de données

### **NÉCESSAIRE PLUS TARD (pour allocateur et FS) :**
1. 🔮 **GlobalAlloc trait** : implémenter allocateur
2. 🔮 **Collections** : Vec, String, HashMap
3. 🔮 **Lifetimes complexes** : références multiples
4. 🔮 **Error handling avancé** : types d'erreur custom
5. 🔮 **Iterators** : parcours efficaces

---

## 📚 **Ressources d'apprentissage recommandées**

### **Pour Rust général :**

#### 1. **The Rust Book** (gratuit en ligne)
- https://doc.rust-lang.org/book/
- **Chapitres essentiels :**
  - Ch. 1-10 : Fondamentaux (ownership, structs, enums, modules)
  - Ch. 15 : Smart pointers (Box, Rc, RefCell)
  - Ch. 16 : Concurrence (Send, Sync)
  - Ch. 19 : Unsafe Rust (pointeurs raw, traits unsafe)

#### 2. **Rust By Example**
- https://doc.rust-lang.org/rust-by-example/
- Exemples pratiques de chaque concept
- Format très accessible

#### 3. **Rustlings**
- https://github.com/rust-lang/rustlings
- Exercices interactifs
- Apprendre en pratiquant

#### 4. **The Rustonomicon**
- https://doc.rust-lang.org/nomicon/
- Guide du Rust unsafe
- **Indispensable pour OS dev**

### **Pour Rust OS dev :**

#### 1. **Writing an OS in Rust** (Philipp Oppermann)
- https://os.phil-opp.com/
- Tutoriel complet OS en Rust
- Explications détaillées de chaque étape
- **Ressource #1 pour OS en Rust**

#### 2. **OSDev Wiki**
- https://wiki.osdev.org/
- Concepts hardware et OS génériques
- Pas spécifique à Rust mais très complet
- Sections importantes :
  - IDT, GDT, Paging
  - Drivers (VGA, Keyboard, ATA)
  - File systems (FAT, ext2)

#### 3. **Intel/AMD Manuals**
- Intel 64 and IA-32 Architectures Software Developer's Manual
- Documentation x86-64 officielle
- Très technique mais référence absolue

#### 4. **Exemples de projets :**
- **Redox OS** : OS complet en Rust
- **Tock** : OS embarqué en Rust
- **Blog OS** : Exemples du tutorial Philipp Oppermann

---

## 🔍 **Exemple de progression d'apprentissage**

### **Semaine 1-2 : Bases Rust**
**Objectif :** Comprendre ce code
```rust
fn main() {
    let mut buffer = [0u8; 256];
    buffer[0] = b'A';
    
    let s = String::from("Hello");
    print_text(&s);
}

fn print_text(text: &str) {
    for c in text.chars() {
        // ...
    }
}
```

**Exercices :**
- Rust Book chapitres 1-8
- Rustlings variables, fonctions, structs
- Comprendre ownership avec des dessins

### **Semaine 3-4 : Ownership avancé**
**Objectif :** Comprendre les erreurs du compilateur
```rust
struct Shell {
    buffer: Vec<u8>,
}

impl Shell {
    fn new() -> Self {
        Shell { buffer: Vec::new() }
    }
    
    fn process(&mut self, input: &str) {
        self.buffer.extend_from_slice(input.as_bytes());
    }
}
```

**Exercices :**
- Rust Book chapitres 9-11
- Rustlings move_semantics, lifetimes
- Corriger des erreurs de borrow checker intentionnelles

### **Semaine 5-6 : Unsafe et pointeurs**
**Objectif :** Manipuler la mémoire directement
```rust
unsafe {
    let ptr = 0xB8000 as *mut u8;
    *ptr.add(0) = b'H';
    *ptr.add(2) = b'i';
}
```

**Exercices :**
- Rust Book chapitre 19.1 (Unsafe)
- Rustonomicon introduction
- Écrire une fonction qui manipule un buffer raw

### **Semaine 7-8 : Code OS réel**
**Objectif :** Comprendre et modifier BOS
```rust
// Lire et tracer l'exécution :
_start() → init_idt() → init_pic() → loop { hlt }
```

**Exercices :**
- Lire main.rs ligne par ligne
- Ajouter une nouvelle commande shell
- Modifier la table de scancode (QWERTY au lieu d'AZERTY)
- Changer les couleurs VGA

### **Semaine 9-12 : Premier gros projet**
**Objectif :** Implémenter quelque chose de nouveau

**Projets suggérés :**
1. **Commande `color`** : Changer la couleur du texte
2. **Timer simple** : Afficher un compteur qui s'incrémente
3. **Buffer d'historique** : Naviguer commandes précédentes avec flèches
4. **Calculs** : Commande `calc` pour additions/soustractions

---

## ✅ **Checklist de compétences pour BOS**

Vous êtes prêt à coder sur BOS si vous pouvez :

**Fondamentaux :**
- [ ] Expliquer ownership, borrowing, lifetimes
- [ ] Écrire des fonctions avec `&self` et `&mut self`
- [ ] Utiliser `match` et `if let`
- [ ] Différencier `String` et `&str`

**Bas niveau :**
- [ ] Comprendre quand utiliser `unsafe`
- [ ] Manipuler des pointeurs raw (`*mut`, `*const`)
- [ ] Lire et écrire en assembleur inline basique
- [ ] Utiliser `static mut` (et comprendre les risques)

**Hardware :**
- [ ] Déclarer des structs avec `#[repr(C)]`
- [ ] Convertir entre types (casts)
- [ ] Manipuler des bits (shift, AND, OR, XOR)
- [ ] Comprendre les adresses mémoire (hexadécimal)

**Debugging :**
- [ ] Lire et comprendre les erreurs du compilateur
- [ ] Utiliser `cargo check` et `cargo clippy`
- [ ] Tracer l'exécution d'un programme mentalement

---

## 💡 **Conseils pratiques**

### **1. Commencez petit**
```rust
// Ne commencez pas par l'allocateur !
// Commencez par ajouter une commande shell simple
bos> reverse "hello"
olleh
```

**Premiers projets suggérés :**
- Nouvelle commande `date` (affiche date fictive)
- Commande `repeat N <cmd>` (répète une commande)
- Mode caps lock (inverse majuscules/minuscules)

### **2. Lisez le code existant**
```rust
// Ouvrez main.rs et shell.rs
// Tracez le flux d'exécution :
// _start() → init_idt() → loop { hlt }
// IRQ 1 → keyboard_handler → shell.handle_char()
```

**Méthode de lecture :**
1. Commencez par `_start()`
2. Suivez chaque appel de fonction
3. Dessinez un diagramme du flux
4. Notez les parties que vous ne comprenez pas
5. Recherchez ces concepts

### **3. Expérimentez**
```rust
// Modifiez la couleur VGA (0x0f → 0x0a)
*VGA_BUFFER.add(cursor + 1) = 0x0a;  // Vert au lieu de blanc

// Essayez différentes couleurs :
// 0x0f = Blanc sur noir
// 0x0a = Vert clair sur noir
// 0x0c = Rouge clair sur noir
// 0x1f = Blanc sur bleu
// 0x4e = Jaune sur rouge
```

### **4. Utilisez le compilateur comme professeur**
```rust
// Les erreurs Rust sont très pédagogiques
error[E0506]: cannot assign to `x` because it is borrowed
  --> src/main.rs:10:5
   |
9  |     let y = &x;
   |             -- borrow of `x` occurs here
10 |     x = 5;
   |     ^^^^^ assignment to borrowed `x` occurs here
```

**Comment lire une erreur :**
1. Code d'erreur (E0506) → chercher dans documentation
2. Message principal → ce qui ne va pas
3. Notes → pourquoi ça ne va pas
4. Suggestions → comment corriger

### **5. Testez dans QEMU régulièrement**
```powershell
# Compilez et testez souvent
cargo bootimage
qemu-system-x86_64 -drive format=raw,file=target\x86_64-bos\debug\bootimage-bos.bin
```

**Cycle de développement :**
1. Faites un petit changement
2. Compilez (`cargo bootimage`)
3. Testez dans QEMU
4. Si ça marche → committez
5. Si ça casse → revertez et recommencez

### **6. Documentez ce que vous apprenez**
```rust
// Écrivez des commentaires pour vous-même
// Même si ça semble évident maintenant, vous oublierez !

/// Cette fonction convertit un scancode en caractère.
/// 
/// # Arguments
/// * `scancode` - Code de la touche (0-127)
/// 
/// # Returns
/// Le caractère correspondant ou '\0' si touche spéciale
fn scancode_to_char(scancode: u8) -> char {
    // ...
}
```

---

## 🎓 **Temps d'apprentissage estimé**

| Niveau | Temps | Objectif | Capacités |
|--------|-------|----------|-----------|
| **Débutant Rust** | 2-4 semaines | Comprendre le code BOS existant | Lire et tracer l'exécution |
| **Intermédiaire** | 2-3 mois | Ajouter commandes et fonctionnalités simples | Modifier comportement existant |
| **Avancé** | 6-12 mois | Implémenter allocateur, FS, processus | Architecturer de gros composants |
| **Expert** | 1-2 ans | Optimiser, sécuriser, étendre | OS production-ready |

**Note :** Ces durées sont pour quelqu'un qui :
- Consacre 1-2h par jour
- A des bases en programmation (C, C++, Python, etc.)
- Pratique activement (pas juste lire)

---

## 🚀 **Plan d'apprentissage sur 12 semaines**

### **Semaines 1-2 : Rust fondamental**
- [ ] Rust Book chapitres 1-8
- [ ] Rustlings sections : variables, functions, if, primitive_types
- [ ] Projet : Programme "Hello World" en Rust standard

### **Semaines 3-4 : Ownership et erreurs**
- [ ] Rust Book chapitres 9-11
- [ ] Rustlings sections : move_semantics, structs, enums
- [ ] Projet : Parser de commandes simple (sans OS)

### **Semaines 5-6 : Unsafe Rust**
- [ ] Rust Book chapitre 19.1
- [ ] Rustonomicon : Introduction, Data representation
- [ ] Projet : Manipuler un buffer avec pointeurs raw

### **Semaines 7-8 : Comprendre BOS**
- [ ] Lire main.rs et shell.rs
- [ ] Tutorial "Writing an OS in Rust" parties 1-4
- [ ] Projet : Ajouter 3 nouvelles commandes au shell

### **Semaines 9-10 : Interruptions et hardware**
- [ ] OSDev Wiki : IDT, PIC, Keyboard
- [ ] Tutorial OS in Rust parties 5-7
- [ ] Projet : Ajouter support souris PS/2

### **Semaines 11-12 : Premier gros projet**
- [ ] Choisir : Timer PIT OU Allocateur bump OU VGA colors
- [ ] Implémenter avec tests
- [ ] Documenter et commiter

**Après 12 semaines, vous serez capable de :**
- Comprendre tout le code BOS actuel
- Ajouter des fonctionnalités simples/moyennes
- Commencer l'allocateur mémoire
- Débugger efficacement

---

## 📝 **Exercices pratiques pour BOS**

### **Exercice 1 : Nouvelle commande (débutant)**
```rust
// Ajoutez une commande "uptime" qui affiche un compteur fictif
bos> uptime
Uptime: 123 seconds
```

**Étapes :**
1. Ajouter le case dans `execute_command()`
2. Implémenter `cmd_uptime()`
3. Utiliser une variable static pour compter

### **Exercice 2 : Couleurs VGA (débutant)**
```rust
// Commande pour changer la couleur du texte
bos> color green
bos> echo "Je suis vert"
Je suis vert  // ← En vert !
```

**Étapes :**
1. Variable static pour couleur actuelle
2. Parser l'argument (green, red, blue, etc.)
3. Modifier le byte de couleur dans vga_print

### **Exercice 3 : Calculs (intermédiaire)**
```rust
// Calculatrice simple
bos> calc 5 + 3
8
bos> calc 10 * 2
20
```

**Étapes :**
1. Parser l'expression (nombre op nombre)
2. Convertir strings en nombres
3. Effectuer l'opération
4. Afficher le résultat

### **Exercice 4 : Historique (avancé)**
```rust
// Naviguer dans l'historique avec flèches haut/bas
bos> echo "hello"
hello
bos> clear
bos> [flèche haut] // ← Affiche "clear"
bos> [flèche haut] // ← Affiche "echo "hello""
```

**Étapes :**
1. Buffer circulaire pour historique
2. Détecter scancodes flèches (0x48 haut, 0x50 bas)
3. Afficher commande précédente
4. Gérer l'index dans l'historique

### **Exercice 5 : Timer PIT (avancé)**
```rust
// Interruption timer toutes les secondes
=== BOOT FIN ===
Uptime: 1s
Uptime: 2s
Uptime: 3s
```

**Étapes :**
1. Configurer le PIT (port 0x40-0x43)
2. Activer IRQ 0 dans le PIC
3. Créer handler pour IRQ 0
4. Incrémenter compteur et afficher

---

## 🔗 **Ressources complémentaires**

### **Documentation officielle Rust :**
- https://doc.rust-lang.org/std/ (bibliothèque standard)
- https://doc.rust-lang.org/core/ (core, no_std)
- https://doc.rust-lang.org/alloc/ (alloc, après allocateur)
- https://doc.rust-lang.org/reference/ (référence langage)

### **Communautés :**
- r/rust (Reddit)
- r/osdev (Reddit)
- Users.rust-lang.org (forum officiel)
- Discord Rust (serveur communautaire)

### **Outils utiles :**
- `cargo clippy` : Linter avancé
- `cargo fmt` : Formatteur de code
- `cargo expand` : Voir macros expansées
- `cargo asm` : Voir l'assembleur généré

---

## ✨ **Mot de la fin**

Le développement d'OS en Rust est un excellent moyen d'apprendre :
- Le Rust en profondeur (unsafe, no_std, embedded)
- L'architecture des ordinateurs (CPU, mémoire, périphériques)
- Les concepts OS (processus, mémoire, fichiers)

**C'est difficile, mais extrêmement gratifiant !**

Prenez votre temps, expérimentez, cassez des choses, et surtout : amusez-vous ! 🚀

---

*Document créé le 5 février 2026*
*Projet : BOS (Basic Operating System)*
*Auteur : Nazim Boudeffa*
