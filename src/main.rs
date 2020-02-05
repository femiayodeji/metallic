#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(const_string_new)]
#![test_runner(metallic::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, string::String, string::ToString};
use metallic::{gdt, hlt_loop, print, println};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use lazy_static::lazy_static;
use pic8259_simple::ChainedPics;
use spin;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

entry_point!(kernel_main);

const PIC_1_OFFSET: u8 = 32;
const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

//command prompt
const MARGIN: &str = "  ";
static mut COUNT: u8 = 0;
static mut COMMAND_LINE: String = String::new();

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use metallic::allocator;
    use metallic::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    println!("{}Welcome{} to {}",MARGIN, ",", "metallic");    
    metallic::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    // allocate a number on the heap
    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);

    // terminal start
    init_idt();
    unsafe { PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
    //terminal end

    //alloc sample

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    unsafe {
        print!(" -> Line {}: ", COUNT);
    }
    hlt_loop();
}
//start interrupt methods
fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    // print!(".");
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    use pc_keyboard::{layouts, DecodedKey, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1));
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => {
                    unsafe {
                        if character == '\n' {
                            print!("\n");
                            if COMMAND_LINE == "exit" {
                                COMMAND_LINE = String::new();
                                println!("{}end",MARGIN);
                                metallic::exit_qemu(metallic::QemuExitCode::Failed);
                            }
                            else if COMMAND_LINE == "help" || COMMAND_LINE == "about" {
                                COMMAND_LINE = String::new();
                                metallic::about();
                            }
                            else if COMMAND_LINE == "info" {
                                COMMAND_LINE = String::new();
                                metallic::info();
                            }
                            else{
                                let cmd = COMMAND_LINE.to_string();
                                metallic::commander(cmd);
                                COMMAND_LINE = String::new();                                
                            }
                            COUNT = COUNT + 1;
                            let line = COUNT;                        
                            print!("{} -> Line {}: ", character, line);
                        }
                        else{
                            COMMAND_LINE.push(character);
                            print!("{}", character);
                        }
                    }
                },
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

//end interrupt methods

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    metallic::test_panic_handler(info)
}