# BOS (Basic Operating System)

Système d'exploitation basique écrit en Rust et en franglais.

Je voulais juste prononcer "bos" au lieu de "dos", donc utilisez le B pour ce que vous voulez : beta, bagel, binouze (sans alcool pour moi), ... lol

## Prérequis

Installer Rust, ensuite :

1. **Installer Rust nightly et les composants requis :**
	```sh
	rustup default nightly
	rustup component add llvm-tools-preview rust-src
	cargo install bootimage
	```

2. **Ajouter la cible pour bare metal x86_64 :**
	```sh
	rustup target add x86_64-unknown-none
	```

## Compilation de l'image bootable

1. **Compiler le kernel et créer l'image bootable :**
	```sh
	cargo bootimage
	```
	Cela créera une image disque bootable à l'emplacement :
	```
	target/x86_64-bos/debug/bootimage-bos.bin
	```

## Exécution

Vous pouvez exécuter l'image dans [QEMU](https://www.qemu.org/) avec :
```sh
& 'C:\Program Files\qemu\qemu-system-x86_64.exe' -drive format=raw,file=.\target\x86_64-bos\debug\bootimage-bos.bin
```

Ou plus simplement :
```sh
qemu-system-x86_64 -drive format=raw,file=target\x86_64-bos\debug\bootimage-bos.bin
```

## Fonctionnalités

- ✅ Mode texte VGA 80×25 (affichage à l'écran)
- ✅ Gestion des interruptions (IDT - Interrupt Descriptor Table)
- ✅ Configuration du PIC (Programmable Interrupt Controller)
- ✅ Driver clavier PS/2 avec layout AZERTY français
- ✅ Shell interactif de base avec commandes
- ✅ Backspace fonctionnel

### Commandes du shell

- `help` - Affiche la liste des commandes disponibles
- `clear` - Efface l'écran
- `echo <message>` - Affiche un message
- `about` - Informations sur BOS

## Structure du projet

```
src/
├── main.rs   - Point d'entrée, gestion VGA, IDT, PIC, interruptions
└── shell.rs  - Module shell avec parser de commandes
```

## Notes techniques

- Le projet utilise `bootloader = "0.9"` pour une compatibilité maximale avec les kernels minimaux
- La configuration de compilation est configurée pour une cible personnalisée et utilise `build-std` pour compiler les bibliothèques core en bare metal
- Code entièrement commenté en français pour faciliter la compréhension
- Pour plus de détails sur la création d'un OS en Rust : [https://os.phil-opp.com/minimal-rust-kernel/](https://os.phil-opp.com/minimal-rust-kernel/)

## Auteur

Nazim Boudeffa
