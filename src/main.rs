#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

//test
![feature(custom_test_frameworks)]
#![test_runner(metallic::test_runner)]
#![reexport_test_harness_main = "test_main"]

use metallic::println;
use core::panic::PanicInfo;

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
	println!("Welcome{} to {}", "!", "metallic");    


	#[cfg(test)]
	test_main();


    loop {}
}


/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    metallic::test_panic_handler(info)
}