use memory_management::memory_read;
use operations::*;
use std::io::{Read, stdin};
use std::ops::{Index, IndexMut};
use std::time::Duration;
use std::{env, io};
use termios::*;
pub mod file_management;
mod operations;
mod tests;
use std::os::fd::AsRawFd;
use timeout_readwrite::TimeoutReadExt;
mod memory_management;

static MEM_MAX: usize = 1 << 16;
static PC_START: u16 = 0x3000;
/// Traps are predefined routines, each trap in the enum represents a routine
pub enum Traps {
    Getc = 0x20,
    Out = 0x21,
    Puts = 0x22,
    In = 0x23,
    Putsp = 0x24,
    Halt = 0x25,
}

impl Traps {
    fn from(value: u16) -> Result<Traps, Errors> {
        match value {
            0x20 => Ok(Traps::Getc),
            0x21 => Ok(Traps::Out),
            0x22 => Ok(Traps::Puts),
            0x23 => Ok(Traps::In),
            0x24 => Ok(Traps::Putsp),
            0x25 => Ok(Traps::Halt),
            badcode => Err(Errors::BadTrapCode(badcode.to_string())),
        }
    }
}
pub enum Errors {
    BadRegisterReference(String),
    BadOpCode(String),
    BadFile(String),
    DisableInputBuffering,
    RestoreInputBuffering,
    BadTrapCode(String),
    Trap(Traps),
    FewArguments,
    BadTermios,
}
#[derive(Clone, Copy)]
enum Registers {
    Rr0,
    Rr1,
    Rr2,
    Rr3,
    Rr4,
    Rr5,
    Rr6,
    Rr7,
    Rpc,
    Rcond,
    Rcount,
}

impl Registers {
    fn from(value: u16) -> Result<Self, Errors> {
        match value {
            0 => Ok(Registers::Rr0),
            1 => Ok(Registers::Rr1),
            2 => Ok(Registers::Rr2),
            3 => Ok(Registers::Rr3),
            4 => Ok(Registers::Rr4),
            5 => Ok(Registers::Rr5),
            6 => Ok(Registers::Rr6),
            7 => Ok(Registers::Rr7),
            register => Err(Errors::BadRegisterReference(format!(
                "Register {register} doesn't exist!"
            ))),
        }
    }
}

impl<T> Index<Registers> for [T; Registers::Rcount as usize] {
    type Output = T;
    fn index(&self, index: Registers) -> &Self::Output {
        &self[index as usize]
    }
}

impl<T> IndexMut<Registers> for [T; Registers::Rcount as usize] {
    fn index_mut(&mut self, index: Registers) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

enum Flags {
    Pos = 1 << 0,
    Zro = 1 << 1,
    Neg = 1 << 2,
}

enum Operations {
    Br,   // Branch
    Add,  // Add
    Ld,   // Load
    St,   // Store
    Jsr,  // Jump register
    And,  // And
    Ldr,  // Load register
    Str,  // Store register
    Rti,  // unused
    Not,  // Not
    Ldi,  // Load indirect
    Sti,  // Store indirect
    Jmp,  // Jump
    Res,  // unused
    Lea,  // Load effective address
    Trap, // Execute trap
}

impl Operations {
    fn from(value: u16) -> Result<Operations, Errors> {
        match value {
            0 => Ok(Operations::Br),
            1 => Ok(Operations::Add),
            2 => Ok(Operations::Ld),
            3 => Ok(Operations::St),
            4 => Ok(Operations::Jsr),
            5 => Ok(Operations::And),
            6 => Ok(Operations::Ldr),
            7 => Ok(Operations::Str),
            8 => Ok(Operations::Rti),
            9 => Ok(Operations::Not),
            10 => Ok(Operations::Ldi),
            11 => Ok(Operations::Sti),
            12 => Ok(Operations::Jmp),
            13 => Ok(Operations::Res),
            14 => Ok(Operations::Lea),
            15 => Ok(Operations::Trap),
            op_code => Err(Errors::BadOpCode(format!(
                "Operation Code {:#x} doesn't exist!",
                op_code
            ))),
        }
    }
}

struct State {
    memory: [u16; MEM_MAX],
    registers: [u16; Registers::Rcount as usize],
    running: bool,
}

fn disable_input_buffering(termio: &mut Termios) -> Result<(), Errors> {
    let new_tio = termio;
    new_tio.c_lflag &= !ICANON & !ECHO;
    match tcsetattr(io::stdin().as_raw_fd(), TCSANOW, new_tio) {
        Ok(_) => Ok(()),
        Err(_) => Err(Errors::DisableInputBuffering),
    }
}

fn restore_input_buffering(termio: &mut Termios) -> Result<(), Errors> {
    match tcsetattr(io::stdin().as_raw_fd(), TCSANOW, termio) {
        Ok(_) => Ok(()),
        Err(_) => Err(Errors::RestoreInputBuffering),
    }
}

fn run_loop(state: &mut State) {
    while state.running {
        // Get next instruction from memory, increment the PC by one and get the OP_CODE
        let instruction = memory_read(state.registers[Registers::Rpc] as usize, state);
        state.registers[Registers::Rpc] += 1;
        let op_code = instruction >> 12;
        let operation_code = error_handler(Operations::from(op_code));
        match operation_code {
            Operations::Br => conditional_branch(instruction, state),
            Operations::Add => error_handler(add(instruction, state)),
            Operations::Ld => error_handler(load(instruction, state)),
            Operations::St => error_handler(store(instruction, state)),
            Operations::Jsr => error_handler(jump_to_subrutine(instruction, state)),
            Operations::And => error_handler(and(instruction, state)),
            Operations::Ldr => error_handler(load_register(instruction, state)),
            Operations::Str => error_handler(store_register(instruction, state)),
            Operations::Rti => {
                print!(
                    "Error: Invalid OPCode:  RTI = {:#x} is not defined",
                    Operations::Rti as u16
                );
                std::process::exit(1);
            }
            Operations::Not => error_handler(not(instruction, state)),
            Operations::Ldi => error_handler(load_indirect(instruction, state)),
            Operations::Sti => error_handler(store_indirect(instruction, state)),
            Operations::Jmp => error_handler(jump(instruction, state)),
            Operations::Res => {
                print!(
                    "Error: Invalid OPCode:  RES = {:#x} is not defined",
                    Operations::Res as u16
                );
                std::process::exit(1);
            }
            Operations::Lea => error_handler(load_effective_address(instruction, state)),
            Operations::Trap => error_handler(trap(instruction, state)),
        }
    }
}

pub fn check_key() -> Result<u16, &'static str> {
    let mut buffer = [0; 1];
    match stdin()
        .with_timeout(Duration::new(0, 0))
        .read_exact(&mut buffer)
    {
        Ok(_) => Ok(buffer[0] as u16),
        Err(_) => Err("Failed to read the value"),
    }
}

fn error_handler<T>(result: Result<T, Errors>) -> T {
    match result {
        Ok(x) => return x,
        Err(Errors::BadFile(s)) => print!("Error {}", s),
        Err(Errors::DisableInputBuffering) => print!("Error couldn't disable input buffering"),
        Err(Errors::RestoreInputBuffering) => print!("Error couldn't restore input buffering"),
        Err(Errors::BadRegisterReference(s)) => print!("Error bad register reference: {}", s),
        Err(Errors::BadOpCode(s)) => print!("Error bad op code: {}", s),
        Err(Errors::Trap(t)) => print!("Error failed to excecute trap {}", trap_to_string(t)),
        Err(Errors::BadTrapCode(s)) => print!("Error bad trap code {}", s),
        Err(Errors::FewArguments) => print!("Error no arguments were provided!"),
        Err(Errors::BadTermios) => print!("Error failed to load termios!"),
    };
    std::process::exit(0);
}

fn trap_to_string(t: Traps) -> String {
    match t {
        Traps::Getc => String::from("Get character"),
        Traps::Out => String::from("Output a character"),
        Traps::Puts => String::from("Output a null terminating string"),
        Traps::In => String::from("Prompt text and input a character"),
        Traps::Putsp => String::from("Output a string"),
        Traps::Halt => String::from("Halt"),
    }
}

fn main() {
    let mut termio = match Termios::from_fd(io::stdin().as_raw_fd()) {
        Ok(x) => x,
        Err(_) => error_handler(Err(Errors::BadTermios)),
    };
    let _ = ctrlc::set_handler(move || {
        error_handler(restore_input_buffering(&mut termio));
        std::process::exit(0);
    });
    error_handler(disable_input_buffering(&mut termio));
    // Initialize empty memory and array with registers
    let mut state = State {
        memory: [0_u16; MEM_MAX],
        registers: [0_u16; Registers::Rcount as usize],
        running: true,
    };
    // Read file
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        error_handler::<()>(Result::Err(Errors::FewArguments));
    }
    let paths = &args[1..].to_vec();
    for p in paths {
        error_handler(file_management::read_file_to_memory(p, &mut state.memory));
    }
    // Update the Program counter and flags, and run the program
    state.registers[Registers::Rpc] = PC_START;
    state.registers[Registers::Rcond] = Flags::Zro as u16;
    run_loop(&mut state);
    error_handler(restore_input_buffering(&mut termio));
}

#[cfg(test)]
mod test {

    use crate::{memory_management::memory_write, *};

    #[test]
    fn integration_test() {
        let mut state = State {
            memory: [0_u16; MEM_MAX],
            registers: [0_u16; Registers::Rcount as usize],
            running: true,
        };
        memory_write(50, 25689, &mut state);
        memory_write(25689, 25, &mut state);
        memory_write(56, 777, &mut state);
        memory_write(9, 50, &mut state);
        state.registers[Registers::Rpc] = 10;
        memory_write(10, 0xAA27, &mut state); // Load indirect 25 to R5
        memory_write(11, 0x27FD, &mut state); // Load 50 to R3
        memory_write(12, 0x12C5, &mut state); // Add R3 + R5 into R1
        memory_write(13, 0x56E0, &mut state); // Clear R3 by doing R3 AND 0x0
        memory_write(14, 0x0405, &mut state); // Branch to 20 if flag Z = 1
        memory_write(20, 0x96FF, &mut state); // Negate R3
        memory_write(21, 0xC140, &mut state); // Jump to the value at R5 PC = 25
        memory_write(25, 0x635F, &mut state); // Load register R1 with R5 + 40
        memory_write(26, 0x4048, &mut state); // Jump to the value at register 1, R7 = 27, PC = 777
        memory_write(777, 0xB34C, &mut state); // Save at memory address 0 the value from register 1
        memory_write(778, 0x3E03, &mut state); // Save R7 into 782
        memory_write(779, 0x7A40, &mut state); // Save R5 into 777
        memory_write(780, 0xF025, &mut state); // Halt
        run_loop(&mut state);
        assert_eq!(memory_read(0, &mut state), 777);
        assert_eq!(memory_read(782, &mut state), 27);
        assert_eq!(memory_read(777, &mut state), 25);
        assert_eq!(state.registers[Registers::Rr7], 27);
    }
}
