#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use core::panic::PanicInfo;

mod vga_buffer;

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
	println!("0000  000  000  000000   00000  0000  000  000   00000   00000  000000   ");    
	println!("0000  0000 000  00  000  00000  0000  0000 000  000 000  00000  00  000  ");    
	println!(" 00   00000000  000000    000    00   00000000  000       000   000000   ");    
	println!(" 00   00000000   000000   000    00   00000000  000       000    000000  ");    
	println!("0000  000 0000  000 000   000   0000  000 0000  000 000   000   000 000  00  00");    
	println!("0000  000  000   000000   000   0000  000  000   00000    000    000000  00  00");    
	println!("");    
	println!("");    
	println!("Welcome{} to {}", "!", "metallic");    

    loop {}
}

// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
