#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::{panic::PanicInfo};
use alloc::{string::String,vec::Vec};

pub mod allocator;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod serial;
pub mod vga_buffer;

const MARGIN: &str = "  ";

pub fn init() {
    gdt::init();
}

pub fn commander(command_string: String){
    let mut strings = command_string.split(' ');
    let command = strings.next();
    if command == Some("echo") {
        let val = command_string.replace("echo","");
        println!("{}{}",MARGIN, val);        
    }
    else if command == Some("calc") {
        let val = command_string.replace("calc","");
        calc(val);
    }
    else if command == Some("compute") {
        let val = command_string.replace("compute","");
        compute(val);
    }
    else {
        println!("{} Oops! unrecognise command syntax",MARGIN);        
    }
}

pub fn calc(expression: String){
    print!("{} = ", expression);
    let postfix_expression = to_postfix(expression);
    compute(postfix_expression);
}

pub fn to_postfix(expression: String) -> String{
    let mut postfix: String = String::new();
    let mut op_stack: Vec<char> = Vec::new();
    let mut operator = ' ';
    for tk in expression.split_whitespace() {
        let token = tk.chars().next().unwrap();
        match tk.parse::<f64>() {
            Ok(n) => {
                postfix.push(' ');
                postfix.push(token);
            },
            _ => {
                if op_stack.len() == 0 {
                    op_stack.push(token);
                } 
                else if precedence(token) > precedence(op_stack[op_stack.len()-1]) || op_stack[op_stack.len()-1] == '(' {
                    op_stack.push(token);
                }
                else{
                    loop {
                        if precedence(token) >= precedence(op_stack[op_stack.len()-1]){
                            break;
                        }
                        operator = op_stack.pop().unwrap();
                        postfix.push(' ');
                        postfix.push(operator);
                    }
                }
                if token == ')' {
                    loop {
                        if op_stack[op_stack.len()-1] == '('{
                            operator = op_stack.pop().unwrap();
                            break;
                        }
                        operator = op_stack.pop().unwrap();
                        postfix.push(' ');
                        postfix.push(operator);
                    }
                }
            }
        }
    }
    //empty op_stack
    loop {
        if op_stack.len() < 1{
            break;
        }
        operator = op_stack.pop().unwrap();
        postfix.push(' ');
        postfix.push(operator);
    }
    return postfix;
}

pub fn precedence(c: char) -> i8{ 
    let mut p = -1;
    match c {
      '+' => {p = 1;},
      '-' => {p = 1;},
      '*' => {p = 2;},
      '/' => {p = 2;},
      '^' => {p = 3;},
      _ => {p=0;}
    }
    return p; 
} 

pub fn compute(expression: String){
let mut stk: Vec<f64> = Vec::new();
    let mut err = false;
    for tk in expression.split_whitespace() {
      if let Ok(x) = tk.parse() {
        stk.push(x);
      } else {
        err = stk.len()<2;
        if err { break; }
        let y = stk.pop().unwrap();
        let x = stk.pop().unwrap();
        match tk {
          "+" => stk.push(x+y),
          "-" => stk.push(x-y),
          "*" => stk.push(x*y),
          "/" => stk.push(x/y),
          _ => {err = true; break;}
        }
      }
    }
    if !err && stk.len()==1 {
        println!(" {} ", stk[0]);
    }
    else if err || stk.len()>1 {
        println!("error");
    }
}

pub fn command_list(){
    println!("{}These shell commands are defined internally.  Type `help' to see this list.",MARGIN);
    println!("exit");
    println!("help");
    println!("info");
    println!("echo -value");
    println!("calc -INFIX expression");
    println!("compute -POSTFIX expression");
}

pub fn info(){
    println!("{}Crux, version 0.1.6 - genesis (x86_64-unknown-none) developed by instincts.",MARGIN);
}

pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn preview(){
    println!("0000  000  000  000000   00000  0000  000  000   00000   00000  000000   ");    
    println!("0000  0000 000  00  000  00000  0000  0000 000  000 000  00000  00  000  ");    
    println!(" 00   00000000  000000    000    00   00000000  000       000   000000   ");    
    println!(" 00   00000000   000000   000    00   00000000  000       000    000000  ");    
    println!("0000  000 0000  000 000   000   0000  000 0000  000 000   000   000 000  00  00");    
    println!("0000  000  000   000000   000   0000  000  000   00000    000    000000  00  00");    

    println!("");    
    println!(" -----------------");    
    println!("     -----------------");    
    println!("");    
    println!("       00000  000000    000  000 000  000          ");    
    println!("      000 000 000  000  000  000 000  000          ");    
    println!("     000      00000000  000  000  000000           ");    
    println!("     000      0000000   000  000  000000           ");    
    println!("      000 000 000  000  00000000 000  000  000 000 ");    
    println!("       00000  000  000   000000  000  000  000 000 ");    
    println!("");    
    println!("                                  ----------------");    
    println!("                                      ----------------");    
    println!("");    
}
#[cfg(test)]
use bootloader::{entry_point, BootInfo};

#[cfg(test)]
entry_point!(test_kernel_main);

/// Entry point for `cargo xtest`
#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}