// ===================================================================
// SHELL DE BASE POUR BOS
// ===================================================================
// 
// Ce module implémente un shell simple avec :
// - Un buffer de commande pour stocker les caractères tapés
// - L'affichage d'un prompt
// - L'exécution de commandes de base
// - Gestion du backspace

use crate::{vga_print, vga_print_char, vga_backspace, update_hardware_cursor};

/// Taille maximale d'une commande
const CMD_BUFFER_SIZE: usize = 256;

/// Structure représentant le shell
pub struct Shell {
    /// Buffer contenant la commande en cours de saisie
    cmd_buffer: [u8; CMD_BUFFER_SIZE],
    /// Position actuelle dans le buffer (nombre de caractères)
    cmd_position: usize,
}

impl Shell {
    /// Crée une nouvelle instance du shell
    pub const fn new() -> Shell {
        Shell {
            cmd_buffer: [0; CMD_BUFFER_SIZE],
            cmd_position: 0,
        }
    }

    /// Initialise le shell et affiche le message de bienvenue
    pub fn init(&mut self) {
        vga_print("\n");
        vga_print("========================================\n");
        vga_print("  Bienvenue dans BOS Shell\n");
        vga_print("========================================\n");
        vga_print("Tapez 'help' pour la liste des commandes\n");
        vga_print("\n");
        self.print_prompt();
    }

    /// Affiche le prompt du shell
    fn print_prompt(&self) {
        vga_print("bos> ");
    }

    /// Traite un caractère reçu du clavier
    /// 
    /// Retourne true si le caractère a été traité, false sinon
    pub fn handle_char(&mut self, c: char) {
        match c {
            // Touche Entrée : exécuter la commande
            '\n' => {
                vga_print_char('\n');
                self.execute_command();
                self.clear_buffer();
                self.print_prompt();
            }
            
            // Backspace : effacer le dernier caractère
            '\x08' => {
                if self.cmd_position > 0 {
                    self.cmd_position -= 1;
                    self.cmd_buffer[self.cmd_position] = 0;
                    vga_backspace();
                }
            }
            
            // Caractères normaux : ajouter au buffer
            _ => {
                if self.cmd_position < CMD_BUFFER_SIZE - 1 {
                    self.cmd_buffer[self.cmd_position] = c as u8;
                    self.cmd_position += 1;
                    vga_print_char(c);
                }
            }
        }
    }

    /// Efface le buffer de commande
    fn clear_buffer(&mut self) {
        for i in 0..CMD_BUFFER_SIZE {
            self.cmd_buffer[i] = 0;
        }
        self.cmd_position = 0;
    }

    /// Obtient la commande actuelle sous forme de &str
    fn get_command(&self) -> &str {
        // Convertir le buffer en &str
        let valid_bytes = &self.cmd_buffer[..self.cmd_position];
        core::str::from_utf8(valid_bytes).unwrap_or("")
    }

    /// Exécute la commande contenue dans le buffer
    fn execute_command(&mut self) {
        let cmd = self.get_command().trim();

        if cmd.is_empty() {
            return;
        }

        // Parser la commande (séparer la commande des arguments)
        let parts: (&str, &str) = match cmd.find(' ') {
            Some(pos) => (&cmd[..pos], &cmd[pos+1..]),
            None => (cmd, ""),
        };

        let command = parts.0;
        let args = parts.1;

        // Dispatcher vers la bonne commande
        match command {
            "help" => self.cmd_help(),
            "clear" => self.cmd_clear(),
            "echo" => self.cmd_echo(args),
            "about" => self.cmd_about(),
            "uptime" => self.cmd_uptime(),
            "" => {},
            _ => {
                vga_print("Commande inconnue: ");
                vga_print(command);
                vga_print("\nTapez 'help' pour voir les commandes disponibles.\n");
            }
        }
    }

    /// Commande: help - Affiche la liste des commandes
    fn cmd_help(&self) {
        vga_print("Commandes disponibles:\n");
        vga_print("  help   - Affiche cette aide\n");
        vga_print("  clear  - Efface l'ecran\n");
        vga_print("  echo   - Affiche un message\n");
        vga_print("  about  - Informations sur BOS\n");
        vga_print("  uptime - (Pas encore implemente)\n");
    }

    /// Commande: clear - Efface l'écran
    fn cmd_clear(&self) {
        unsafe {
            // Effacer tout l'écran
            let vga_buffer = 0xb8000 as *mut u8;
            for i in 0..(80 * 25 * 2) {
                if i % 2 == 0 {
                    *vga_buffer.add(i) = b' ';
                } else {
                    *vga_buffer.add(i) = 0x0f;
                }
            }
            // Réinitialiser le curseur
            crate::VGA_CURSOR = 0;
            // Mettre à jour le curseur matériel
            update_hardware_cursor();
        }
    }

    /// Commande: echo - Affiche un message
    fn cmd_echo(&self, args: &str) {
        if args.is_empty() {
            vga_print("\n");
        } else {
            vga_print(args);
            vga_print("\n");
        }
    }

    /// Commande: about - Affiche des informations sur BOS
    fn cmd_about(&self) {
        vga_print("BOS - Basic Operating System\n");
        vga_print("Version: 0.1\n");
        vga_print("Auteur: Nazim Boudeffa\n");
        vga_print("Ecrit en Rust (bare metal)\n");
        vga_print("\nCaracteristiques:\n");
        vga_print("  - Mode texte VGA 80x25\n");
        vga_print("  - Gestion des interruptions (IDT)\n");
        vga_print("  - Driver clavier PS/2\n");
        vga_print("  - Shell de base\n");
    }

    /// Commande: uptime - Affiche le temps depuis le démarrage
    fn cmd_uptime(&self) {
        vga_print("Uptime: (non implemente)\n");
        vga_print("TODO: Integrer un timer PIT pour compter le temps\n");
    }
}
