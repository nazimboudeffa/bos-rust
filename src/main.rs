#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let vga = 0xb8000 as *mut u8;

    unsafe {
        *vga = b'B';
        *vga.add(1) = 0x0f;
        *vga.add(2) = b'O';
        *vga.add(3) = 0x0f;
        *vga.add(4) = b'S';
        *vga.add(5) = 0x0f;
    }

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}