# Changelog

Tous les changements notables de ce projet seront documentés dans ce fichier.

Le format est basé sur [Keep a Changelog](https://keepachangelog.com/fr/1.0.0/).

## [0.0.2] - 2026-02-05

### Ajouté
- Shell interactif de base avec buffer de commande
- Module `shell.rs` séparé pour une meilleure organisation du code
- Commandes disponibles : `help`, `clear`, `echo`, `about`, `uptime`
- Support du clavier AZERTY français (scancodes)
- Curseur matériel VGA qui suit la position d'écriture
- Fonction `vga_backspace()` pour effacer les caractères
- Commentaires détaillés en français dans tout le code
- Documentation README en français

### Modifié
- Conversion des scancodes de QWERTY vers AZERTY
- Handler d'interruption clavier intégré au shell
- Utilisation de `addr_of!` et `addr_of_mut!` pour éviter les warnings Rust 2024

### Corrigé
- Warnings de compilation sur les références aux statics mutables
- Curseur clignotant qui était bloqué en haut à gauche (maintenant il suit le texte)

## [0.0.2] - 2026-02-04

### Ajouté
- Configuration initiale du projet avec bootloader 0.9
- Mode texte VGA 80×25 avec affichage de base
- IDT (Interrupt Descriptor Table) pour gérer les interruptions
- Configuration du PIC (Programmable Interrupt Controller)
- Remapping des IRQs (32-47) pour éviter les conflits CPU
- Driver clavier PS/2 basique
- Gestion des interruptions matérielles (IRQ 1 pour le clavier)
- Table de conversion des scancodes vers caractères ASCII
- Structure `InterruptStackFrame` pour les handlers x86-interrupt
- Fonctions d'entrée/sortie I/O (`inb`, `outb`)
- Message de démarrage du kernel
- Boucle principale avec instruction HLT pour économie d'énergie

### Technique
- Architecture x86_64 bare metal
- Compilé avec Rust nightly
- Target custom : x86_64-bos.json
- Linker script personnalisé (linker.ld)
- Sans bibliothèque standard (`#![no_std]`)
- Point d'entrée personnalisé (`_start`)
