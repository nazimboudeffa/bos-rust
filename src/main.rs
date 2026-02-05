// ===================================================================
// CONFIGURATION DU SYSTÈME
// ===================================================================

// #![no_std] : Désactive la bibliothèque standard Rust
// Nécessaire car nous n'avons pas d'OS sous-jacent pour fournir
// les fonctionnalités standards (allocation mémoire, threads, etc.)
#![no_std]

// #![no_main] : Désactive le point d'entrée standard Rust (fn main)
// Nous définissons notre propre point d'entrée _start() qui sera
// appelé directement par le bootloader
#![no_main]

// Active l'ABI x86-interrupt pour gérer les interruptions matérielles
// Cette feature instable permet de créer des handlers d'interruption
// avec la convention d'appel x86-interrupt
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use core::arch::asm; // Pour l'assembleur inline
use core::ptr::{addr_of, addr_of_mut}; // Pour obtenir l'adresse d'un static mut de façon sûre

// Déclarer le module shell
mod shell;
use shell::Shell;

// ===================================================================
// PANIC HANDLER
// ===================================================================
// En mode no_std, nous devons définir nous-mêmes le comportement
// en cas de panic. Ici, on boucle infiniment.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/* =========================================================
   VGA TEXT MODE (0xb8000)
   
   Le mode texte VGA permet d'afficher du texte à l'écran.
   La mémoire vidéo est mappée à l'adresse physique 0xb8000.
   
   Format : [caractère][couleur][caractère][couleur]...
   - 1 octet pour le caractère ASCII
   - 1 octet pour les attributs de couleur (4 bits fond, 4 bits texte)
   
   Résolution : 80 colonnes × 25 lignes = 2000 caractères
   Taille totale : 2000 × 2 octets = 4000 octets
========================================================= */

// Adresse mémoire du buffer VGA en mode texte
const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;

// Dimensions de l'écran en mode texte VGA
const VGA_WIDTH: usize = 80;  // Nombre de colonnes
const VGA_HEIGHT: usize = 25; // Nombre de lignes

// Taille totale du buffer VGA (2 octets par caractère : 1 pour le char, 1 pour la couleur)
const VGA_SIZE: usize = VGA_WIDTH * VGA_HEIGHT * 2;

// Position actuelle du curseur (en octets, donc multiple de 2)
// Déclarée publique pour être accessible depuis d'autres modules (comme shell.rs)
pub static mut VGA_CURSOR: usize = 0;

/// Affiche une chaîne de caractères à l'écran en utilisant le mode texte VGA
/// 
/// Gère automatiquement :
/// - Le retour à la ligne (\n)
/// - Le dépassement de l'écran (wrap au début)
pub fn vga_print(s: &str) {
    unsafe {
        for byte in s.bytes() {
            // Vérifier si on dépasse la taille de l'écran
            if VGA_CURSOR >= VGA_SIZE {
                VGA_CURSOR = 0; // Retour au début de l'écran
            }
            
            // Gérer le retour à la ligne
            if byte == b'\n' {
                // Calculer la position du début de la ligne suivante
                // Division entière pour obtenir le numéro de ligne actuel,
                // puis +1 pour passer à la ligne suivante
                VGA_CURSOR = ((VGA_CURSOR / (VGA_WIDTH * 2)) + 1) * (VGA_WIDTH * 2);
                if VGA_CURSOR >= VGA_SIZE {
                    VGA_CURSOR = 0;
                }
            } else {
                // Écrire le caractère à la position actuelle
                *VGA_BUFFER.add(VGA_CURSOR) = byte;
                // Écrire l'attribut de couleur : 0x0f = blanc sur fond noir
                // Format : 0x[fond][texte] où 0=noir, f=blanc
                *VGA_BUFFER.add(VGA_CURSOR + 1) = 0x0f;
                // Avancer de 2 octets (caractère + couleur)
                VGA_CURSOR += 2;
            }
        }
        // Mettre à jour le curseur matériel pour qu'il clignote à la bonne position
        update_hardware_cursor();
    }
}

/// Affiche un seul caractère à l'écran
/// Utilisé par le handler du clavier pour afficher les touches pressées en temps réel
pub fn vga_print_char(c: char) {
    unsafe {
        let byte = c as u8;
        if VGA_CURSOR >= VGA_SIZE {
            VGA_CURSOR = 0;
        }
        
        if byte == b'\n' {
            VGA_CURSOR = ((VGA_CURSOR / (VGA_WIDTH * 2)) + 1) * (VGA_WIDTH * 2);
            if VGA_CURSOR >= VGA_SIZE {
                VGA_CURSOR = 0;
            }
        } else {
            *VGA_BUFFER.add(VGA_CURSOR) = byte;
            *VGA_BUFFER.add(VGA_CURSOR + 1) = 0x0f;
            VGA_CURSOR += 2;
        }
        // Mettre à jour le curseur matériel pour qu'il clignote à la bonne position
        update_hardware_cursor();
    }
}

/// Efface le dernier caractère affiché (backspace)
/// 
/// Recule le curseur et affiche un espace pour effacer visuellement le caractère
pub fn vga_backspace() {
    unsafe {
        if VGA_CURSOR >= 2 {
            // Reculer de 2 octets (caractère + couleur)
            VGA_CURSOR -= 2;
            // Effacer en écrivant un espace
            *VGA_BUFFER.add(VGA_CURSOR) = b' ';
            *VGA_BUFFER.add(VGA_CURSOR + 1) = 0x0f;
            // Mettre à jour le curseur matériel
            update_hardware_cursor();
        }
    }
}

/* =========================================================
   IDT - INTERRUPT DESCRIPTOR TABLE
   
   L'IDT est une table qui associe chaque numéro d'interruption
   (0-255) à une fonction qui sera appelée quand cette
   interruption se produit.
   
   Structure : 256 entrées de 16 octets chacune
   Adresse : configurée via l'instruction assembleur LIDT
   
   Interruptions importantes :
   - 0-31  : Exceptions CPU (division par zéro, page fault, etc.)
   - 32-47 : IRQs matérielles (après remapping du PIC)
   - 33    : IRQ 1 = Interruption clavier
========================================================= */

// Structure d'une entrée dans l'IDT (16 octets)
// #[repr(C, packed)] garantit que le compilateur n'ajoute pas de padding
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct IdtEntry {
    offset_low: u16,    // Bits 0-15 de l'adresse du handler
    selector: u16,      // Sélecteur de segment de code (GDT)
    zero: u8,           // Toujours 0 (réservé)
    type_attr: u8,      // Type et attributs (présent, DPL, type de gate)
    offset_mid: u16,    // Bits 16-31 de l'adresse du handler
    offset_high: u32,   // Bits 32-63 de l'adresse du handler (mode 64-bit)
    reserved: u32,      // Réservé (doit être 0)
}

impl IdtEntry {
    /// Crée une entrée IDT vide (initialisée à zéro)
    const fn new() -> IdtEntry {
        IdtEntry {
            offset_low: 0,
            selector: 0,
            zero: 0,
            type_attr: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }

    /// Configure cette entrée IDT pour pointer vers un handler d'interruption
    /// 
    /// L'adresse 64-bit du handler est divisée en 3 parties (low, mid, high)
    /// car l'architecture x86-64 utilise ce format pour la compatibilité
    fn set_handler(&mut self, handler: unsafe extern "x86-interrupt" fn(InterruptStackFrame)) {
        let addr = handler as u64;
        
        // Diviser l'adresse 64-bit en 3 parties
        self.offset_low = addr as u16;              // Bits 0-15
        self.offset_mid = (addr >> 16) as u16;      // Bits 16-31
        self.offset_high = (addr >> 32) as u32;     // Bits 32-63
        
        // Sélecteur de segment de code (défini par le bootloader)
        // 0x08 = entrée 1 de la GDT (Global Descriptor Table) = segment de code kernel
        self.selector = 0x08;
        
        self.zero = 0;
        
        // Attributs de l'entrée : 0x8E
        // 0x80 = Present (l'entrée est valide)
        // 0x0E = Type = 64-bit interrupt gate, DPL=0 (niveau privilège 0)
        self.type_attr = 0x8E;
        
        self.reserved = 0;
    }
}

// Structure pour décrire l'IDT au CPU (utilisée par l'instruction LIDT)
#[repr(C, packed)]
struct IdtDescriptor {
    size: u16,      // Taille de l'IDT - 1 (en octets)
    offset: u64,    // Adresse mémoire de l'IDT
}

// Table globale contenant les 256 entrées d'interruption
// Initialisée avec des entrées vides
static mut IDT: [IdtEntry; 256] = [IdtEntry::new(); 256];

/// Initialise l'IDT et la charge dans le CPU
fn init_idt() {
    unsafe {
        // Configurer l'entrée 33 (IRQ 1 après remapping) pour le clavier
        // IRQ 1 correspond à l'interruption matérielle du clavier PS/2
        IDT[33].set_handler(keyboard_interrupt_handler);
        
        // Créer le descripteur qui pointe vers notre IDT
        let idt_desc = IdtDescriptor {
            // Taille en octets - 1 (256 entrées × 16 octets = 4096)
            size: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
            // Utiliser addr_of! pour obtenir l'adresse sans créer de référence
            offset: addr_of!(IDT) as u64,
        };
        
        // Charger l'IDT dans le CPU avec l'instruction LIDT
        // À partir de maintenant, le CPU utilisera cette table pour
        // gérer les interruptions
        asm!(
            "lidt [{}]",
            in(reg) &idt_desc,
            options(nostack, preserves_flags)
        );
    }
}

/* =========================================================
   PIC - PROGRAMMABLE INTERRUPT CONTROLLER
   
   Le PIC (8259) gère les interruptions matérielles (IRQs).
   Sur PC, il y a 2 PICs en cascade : PIC1 (maître) et PIC2 (esclave)
   
   PIC1 gère IRQ 0-7  : Timer, Clavier, Cascade, COM2, COM1, etc.
   PIC2 gère IRQ 8-15 : RTC, Souris, Disques, etc.
   
   Par défaut, les IRQs sont mappées sur INT 8-15, ce qui entre
   en conflit avec les exceptions CPU. On les remappera sur INT 32-47.
   
   Après remapping :
   - IRQ 0 (timer)   → INT 32
   - IRQ 1 (clavier) → INT 33  ← Nous utilisons celle-ci !
   - IRQ 2-7         → INT 34-39
   - IRQ 8-15        → INT 40-47
========================================================= */

// Ports I/O pour communiquer avec les PICs
const PIC1_COMMAND: u16 = 0x20;  // Port de commande du PIC maître
const PIC1_DATA: u16 = 0x21;     // Port de données du PIC maître
const PIC2_COMMAND: u16 = 0xA0;  // Port de commande du PIC esclave
const PIC2_DATA: u16 = 0xA1;     // Port de données du PIC esclave

/// Écrit un octet sur un port I/O (OUT instruction)
/// 
/// Les ports I/O permettent de communiquer avec les périphériques matériels.
/// Exemples : 0x60 = clavier, 0x20/0xA0 = PICs, 0x3F8 = COM1, etc.
unsafe fn outb(port: u16, value: u8) {
    asm!(
        "out dx, al",           // Instruction assembleur OUT
        in("dx") port,          // Port dans le registre DX
        in("al") value,         // Valeur dans le registre AL
        options(nostack, preserves_flags)
    );
}

/// Lit un octet depuis un port I/O (IN instruction)
/// 
/// Utilisé pour lire l'état des périphériques matériels.
/// Par exemple : lire le scancode du clavier depuis le port 0x60
unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    asm!(
        "in al, dx",            // Instruction assembleur IN
        out("al") value,        // Lire le résultat depuis AL
        in("dx") port,          // Port dans le registre DX
        options(nostack, preserves_flags)
    );
    value
}

/// Met à jour la position du curseur matériel VGA (le curseur clignotant)
/// 
/// Le curseur matériel est contrôlé via les ports VGA 0x3D4 (commande) et 0x3D5 (données).
/// Il attend une position en nombre de caractères (pas d'octets), d'où la division par 2.
pub fn update_hardware_cursor() {
    unsafe {
        // Convertir la position en octets vers position en caractères
        let pos = (VGA_CURSOR / 2) as u16;
        
        // Port de commande VGA : sélectionner le registre "Cursor Location Low"
        outb(0x3D4, 0x0F);
        // Port de données VGA : envoyer les 8 bits de poids faible
        outb(0x3D5, (pos & 0xFF) as u8);
        
        // Port de commande VGA : sélectionner le registre "Cursor Location High"
        outb(0x3D4, 0x0E);
        // Port de données VGA : envoyer les 8 bits de poids fort
        outb(0x3D5, ((pos >> 8) & 0xFF) as u8);
    }
}

/// Initialise et configure les PICs (remapping des IRQs)
/// 
/// Cette fonction envoie une séquence d'Initialization Command Words (ICW)
/// pour configurer les deux PICs en mode cascade.
fn init_pic() {
    unsafe {
        // ===== ICW1 : Commencer l'initialisation =====
        // 0x11 = 00010001 :
        //   - Bit 4 = 1 : Mode d'initialisation
        //   - Bit 0 = 1 : ICW4 sera envoyé
        outb(PIC1_COMMAND, 0x11);
        outb(PIC2_COMMAND, 0x11);
        
        // ===== ICW2 : Remapper les vecteurs d'interruption =====
        // Par défaut, IRQ 0-7 sont mappées sur INT 8-15 (conflit avec CPU exceptions)
        // On les remapppe sur INT 32-47 pour éviter les conflits
        outb(PIC1_DATA, 0x20);  // PIC1 : IRQ 0-7  → INT 32-39 (0x20 = 32)
        outb(PIC2_DATA, 0x28);  // PIC2 : IRQ 8-15 → INT 40-47 (0x28 = 40)
        
        // ===== ICW3 : Configurer le mode cascade =====
        // Le PIC2 est connecté au PIC1 via IRQ2
        outb(PIC1_DATA, 0x04);  // 0x04 = 00000100 : IRQ2 a un PIC esclave
        outb(PIC2_DATA, 0x02);  // 0x02 : Le PIC2 est l'esclave sur IRQ2
        
        // ===== ICW4 : Mode de fonctionnement =====
        // 0x01 = Mode 8086/88 (vs mode MCS-80/85)
        outb(PIC1_DATA, 0x01);
        outb(PIC2_DATA, 0x01);
        
        // ===== Masquer/démasquer les IRQs =====
        // Chaque bit contrôle une IRQ (0=activée, 1=masquée)
        // 0xFD = 11111101 : Toutes les IRQs masquées sauf l'IRQ 1 (clavier)
        //   Bit 0 = 1 : IRQ 0 (timer) masqué
        //   Bit 1 = 0 : IRQ 1 (clavier) ACTIF ← C'est ce qu'on veut !
        //   Bit 2-7 = 1 : IRQ 2-7 masqués
        outb(PIC1_DATA, 0xFD);
        // 0xFF : Masquer toutes les IRQs du PIC2 (on n'en a pas besoin pour l'instant)
        outb(PIC2_DATA, 0xFF);
    }
}

/* =========================================================
   KEYBOARD HANDLER
   
   Le clavier PS/2 communique via le port 0x60 (données).
   
   Quand une touche est pressée :
   1. Le contrôleur clavier envoie un "scancode" (code de la touche)
   2. Le PIC déclenche l'IRQ 1
   3. Le CPU appelle notre handler (INT 33 après remapping)
   4. On lit le scancode du port 0x60
   5. On convertit le scancode en caractère
   6. On affiche le caractère
   7. On envoie EOI (End Of Interrupt) au PIC
   
   Scancodes :
   - "Make code" : envoyé quand la touche est pressée (bit 7 = 0)
   - "Break code" : envoyé quand la touche est relâchée (bit 7 = 1)
   
   On ignore les break codes car on veut afficher seulement quand
   la touche est pressée, pas relâchée.
========================================================= */

// Port I/O pour lire les données du clavier
const KEYBOARD_DATA_PORT: u16 = 0x60;

// Table de conversion des scancodes (Scan Code Set 1, layout AZERTY français)
// Index = scancode, Valeur = caractère ASCII correspondant
// '\0' = touche sans caractère (Shift, Ctrl, etc.)
// Note : Les caractères accentués (é, è, ç, à, ù) sont remplacés par leur équivalent non-accenté
//        car ils ne sont pas dans ASCII standard. Pour un support complet, il faudrait utiliser UTF-8.
static SCANCODE_TABLE: [char; 58] = [
    '\0', '\0', '&', 'e', '"', '\'', '(', '-', 'e', '_', 'c', 'a', ')', '=', '\x08', // 0-14 : Ligne chiffres AZERTY
    '\t', 'a', 'z', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '^', '$', '\n', // 15-28 : AZERTY première ligne
    '\0', 'q', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', 'm', 'u', '~', // 29-42 : AZERTY deuxième ligne (ù→u)
    '\0', '<', 'w', 'x', 'c', 'v', 'b', 'n', ',', ';', ':', '!', '\0', '*', // 43-55 : AZERTY troisième ligne
    '\0', ' ', // 56-57 : Alt, Espace
];

// Structure représentant l'état de la pile quand une interruption se produit
// Le CPU sauvegarde automatiquement ces informations sur la pile
#[repr(C)]
struct InterruptStackFrame {
    instruction_pointer: u64,  // Adresse de l'instruction interrompue (RIP)
    code_segment: u64,         // Segment de code (CS)
    cpu_flags: u64,            // Flags du CPU (RFLAGS)
    stack_pointer: u64,        // Pointeur de pile (RSP)
    stack_segment: u64,        // Segment de pile (SS)
}

// Instance globale du shell (mutable pour gérer l'état)
static mut SHELL: Shell = Shell::new();

/// Handler d'interruption pour le clavier (IRQ 1 = INT 33)
/// 
/// Cette fonction est appelée automatiquement par le CPU chaque fois
/// qu'une touche est pressée ou relâchée sur le clavier.
/// 
/// Convention d'appel x86-interrupt :
/// - Le CPU sauvegarde automatiquement l'état
/// - Le CPU désactive les interruptions (CLI)
/// - À la fin, le CPU restaure l'état et réactive les interruptions (IRET)
unsafe extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // Lire le scancode depuis le port 0x60
    // Ce port contient le code de la touche qui vient d'être pressée/relâchée
    let scancode = inb(KEYBOARD_DATA_PORT);
    
    // Vérifier si c'est un "make code" (touche pressée) et non un "break code" (touche relâchée)
    // Les break codes ont le bit 7 à 1 (valeur >= 0x80)
    if scancode & 0x80 == 0 {
        // Convertir le scancode en caractère si possible
        if (scancode as usize) < SCANCODE_TABLE.len() {
            let c = SCANCODE_TABLE[scancode as usize];
            // Ignorer les touches spéciales (Shift, Ctrl, etc.) qui sont marquées '\0'
            if c != '\0' {
                // Passer le caractère au shell pour traitement
                // Utiliser addr_of_mut! pour accéder au static sans créer de référence directe
                (*addr_of_mut!(SHELL)).handle_char(c);
            }
        }
    }
    
    // ===== IMPORTANT : Envoyer EOI (End Of Interrupt) au PIC =====
    // On doit signaler au PIC que l'interruption a été traitée.
    // Sans cela, le PIC ne déclenchera plus d'interruptions !
    // 0x20 sur le port de commande du PIC1 = commande EOI
    outb(PIC1_COMMAND, 0x20);
}

// ===================================================================
// POINT D'ENTRÉE DU KERNEL
// ===================================================================
// Cette fonction est appelée par le bootloader après le chargement du kernel.
// #[no_mangle] empêche Rust de renommer la fonction (nécessaire pour le linker)
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Afficher le message de démarrage
    vga_print("=== BOOT DEBUT ===\n");
    
    // ===== ÉTAPE 1 : Initialiser l'IDT =====
    // Configure la table des interruptions et la charge dans le CPU
    vga_print("Initialisation IDT...\n");
    init_idt();
    vga_print("IDT OK\n");

    // ===== ÉTAPE 2 : Initialiser le PIC =====
    // Remapping des IRQs pour éviter les conflits avec les exceptions CPU
    vga_print("Initialisation PIC...\n");
    init_pic();
    vga_print("PIC OK\n");
    
    // ===== ÉTAPE 3 : Activer les interruptions matérielles =====
    // Par défaut, les interruptions sont désactivées au démarrage (flag IF=0)
    // L'instruction STI (Set Interrupt Flag) les réactive
    vga_print("Activation des interruptions...\n");
    unsafe {
        asm!("sti", options(nostack, preserves_flags));
    }
    vga_print("=== BOOT FIN ===\n");

    // ===== ÉTAPE 4 : Initialiser le shell =====
    unsafe {
        // Utiliser addr_of_mut! pour accéder au static sans créer de référence directe
        (*addr_of_mut!(SHELL)).init();
    }
    
    // ===== BOUCLE PRINCIPALE =====
    // Le kernel entre dans une boucle infinie.
    // L'instruction HLT met le CPU en veille jusqu'à la prochaine interruption,
    // ce qui économise de l'énergie.
    // 
    // Déroulement :
    // 1. HLT → CPU en veille
    // 2. Touche pressée → IRQ 1 → Handler appelé → Caractère affiché
    // 3. Handler termine (EOI envoyé) → Retour à HLT
    // 4. Répéter...
    loop {
        unsafe {
            asm!("hlt", options(nostack, preserves_flags));
        }
    }
}