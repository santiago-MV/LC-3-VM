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
use thiserror::Error;
use timeout_readwrite::TimeoutReadExt;

static MEM_MAX: usize = 1 << 16;
static PC_START: u16 = 0x3000;

// Special registers that are in memory
pub enum MemoryMappedRegisters {
    Kbsr = 0xFE00, // Keyboard Status Register, identifies when a key is pressed
    Kbdr = 0xFE02, // Keyboard Data Register, identifies what key was pressed
}

/// Traps are predefined routines, each trap in the enum represents a routine
#[derive(Debug)]
pub enum Traps {
    Getc = 0x20,
    Out = 0x21,
    Puts = 0x22,
    In = 0x23,
    Putsp = 0x24,
    Halt = 0x25,
}

impl TryFrom<u16> for Traps {
    type Error = Errors;
    fn try_from(value: u16) -> Result<Traps, Self::Error> {
        match value {
            0x20 => Ok(Traps::Getc),
            0x21 => Ok(Traps::Out),
            0x22 => Ok(Traps::Puts),
            0x23 => Ok(Traps::In),
            0x24 => Ok(Traps::Putsp),
            0x25 => Ok(Traps::Halt),
            badcode => Err(Errors::BadTrapCode(badcode)),
        }
    }
}
#[derive(Error, Debug)]
pub enum Errors {
    #[error("Bad register: `{0} does not exist!`")]
    BadRegisterReference(u16),
    #[error("Bad operation code: `{0}` does not exist!")]
    BadOpCode(u16),
    #[error("Bad file: {0}")]
    BadFile(#[from] std::io::Error),
    #[error("Couldn't disable input buffering")]
    DisableInputBuffering,
    #[error("Couldn't restore input buffering")]
    RestoreInputBuffering,
    #[error("Bad trap code: `{0}`")]
    BadTrapCode(u16),
    #[error("Bad trap `{0:?}`")]
    Trap(Traps),
    #[error("Not enough arguments")]
    FewArguments,
    #[error("Couldn't initialize termios")]
    BadTermios,
    #[error("Bad image size")]
    BadImageSize,
}
#[derive(Clone, Copy)]
enum Registers {
    R0,      // Register 0
    R1,      // Register 1
    R2,      // Register 2
    R3,      // Register 3
    R4,      // Register 4
    R5,      // Register 5
    R6,      // Register 6
    R7,      // Register 7
    Pc,      // Program Counter
    Flags,   // Flags
    InstRet, // Amount of registers
}

impl TryFrom<u16> for Registers {
    type Error = Errors;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Registers::R0),
            1 => Ok(Registers::R1),
            2 => Ok(Registers::R2),
            3 => Ok(Registers::R3),
            4 => Ok(Registers::R4),
            5 => Ok(Registers::R5),
            6 => Ok(Registers::R6),
            7 => Ok(Registers::R7),
            register => Err(Errors::BadRegisterReference(register)),
        }
    }
}

impl<T> Index<MemoryMappedRegisters> for [T; MEM_MAX] {
    type Output = T;
    fn index(&self, index: MemoryMappedRegisters) -> &Self::Output {
        &self[index as usize]
    }
}

impl<T> IndexMut<MemoryMappedRegisters> for [T; MEM_MAX] {
    fn index_mut(&mut self, index: MemoryMappedRegisters) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

impl<T> Index<Registers> for [T; Registers::InstRet as usize] {
    type Output = T;
    fn index(&self, index: Registers) -> &Self::Output {
        &self[index as usize]
    }
}

impl<T> IndexMut<Registers> for [T; Registers::InstRet as usize] {
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

impl TryFrom<u16> for Operations {
    type Error = Errors;
    fn try_from(value: u16) -> Result<Operations, Self::Error> {
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
            op_code => Err(Errors::BadOpCode(op_code)),
        }
    }
}

struct State {
    memory: [u16; MEM_MAX],
    registers: [u16; Registers::InstRet as usize],
    running: bool,
}

impl State {
    pub fn default() -> State {
        let mut state = State {
            memory: [0_u16; MEM_MAX],
            registers: [0_u16; Registers::InstRet as usize],
            running: true,
        };
        state.register_write(Registers::Pc, PC_START);
        state.register_write(Registers::Flags, Flags::Zro as u16);
        state
    }

    pub fn memory_write(&mut self, address: usize, value: u16) {
        self.memory[address] = value;
    }

    pub fn memory_read(&mut self, address: usize) -> u16 {
        if address == MemoryMappedRegisters::Kbsr as usize {
            match check_key() {
                Ok(rv) => {
                    self.memory[MemoryMappedRegisters::Kbsr] = 1 << 15;
                    self.memory[MemoryMappedRegisters::Kbdr] = rv
                }
                Err(_) => self.memory[MemoryMappedRegisters::Kbsr] = 0,
            };
        }
        self.memory[address]
    }

    pub fn register_read(&self, address: Registers) -> u16 {
        self.registers[address]
    }

    pub fn register_write(&mut self, address: Registers, value: u16) {
        self.registers[address] = value;
    }

    pub fn increment_pc(&mut self) {
        self.registers[Registers::Pc] += 1;
    }
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

fn run_loop(state: &mut State) -> Result<(), Errors> {
    while state.running {
        // Get next instruction from memory, increment the PC by one and get the OP_CODE
        let memory_address = state.register_read(Registers::Pc) as usize;
        let instruction = state.memory_read(memory_address);
        state.increment_pc();
        run_step(instruction, state)?;
    }
    Ok(())
}

fn run_step(instruction: u16, state: &mut State) -> Result<(), Errors> {
    let op_code = instruction >> 12;
    let operation_code = Operations::try_from(op_code).unwrap(); // Since op_code is an u16 that was right shifted 12 bits, its maximum value is 15 (1111) that will always map in the try_from, so it will never fail, that's why the unwrap is used
    match operation_code {
        Operations::Br => conditional_branch(instruction, state),
        Operations::Add => add(instruction, state)?,
        Operations::Ld => load(instruction, state)?,
        Operations::St => store(instruction, state)?,
        Operations::Jsr => jump_to_subrutine(instruction, state)?,
        Operations::And => and(instruction, state)?,
        Operations::Ldr => load_register(instruction, state)?,
        Operations::Str => store_register(instruction, state)?,
        Operations::Rti => {
            print!(
                "Error: Invalid OPCode:  RTI = {:#x} is not defined",
                Operations::Rti as u16
            );
            std::process::exit(1);
        }
        Operations::Not => not(instruction, state)?,
        Operations::Ldi => load_indirect(instruction, state)?,
        Operations::Sti => store_indirect(instruction, state)?,
        Operations::Jmp => jump(instruction, state)?,
        Operations::Res => {
            print!(
                "Error: Invalid OPCode:  RES = {:#x} is not defined",
                Operations::Res as u16
            );
            std::process::exit(1);
        }
        Operations::Lea => load_effective_address(instruction, state)?,
        Operations::Trap => trap(instruction, state)?,
    }
    Ok(())
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
// This function is responsible of printing the error message and return whether it failed or not
fn error_handler<T>(result: Result<T, Errors>) -> bool {
    match result {
        Ok(_) => true,
        Err(e) => {
            print!("{}", e);
            false
        }
    }
}

fn main() {
    let mut termio = match Termios::from_fd(io::stdin().as_raw_fd()) {
        Ok(x) => x,
        Err(_) => {
            let _ = error_handler::<()>(Err(Errors::BadTermios));
            std::process::exit(0);
        }
    };
    let _ = ctrlc::set_handler(move || {
        let _ = error_handler(restore_input_buffering(&mut termio));
        std::process::exit(0);
    });
    if !error_handler(disable_input_buffering(&mut termio)) {
        std::process::exit(0);
    };
    // Initialize default state
    let mut state = State::default();
    // Read file
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        let _ = error_handler::<()>(Result::Err(Errors::FewArguments));
        std::process::exit(0);
    }
    let paths = &args[1..].to_vec();
    for p in paths {
        if !error_handler(file_management::read_file_to_memory(p, &mut state)) {
            std::process::exit(0);
        };
    }
    // Run the program
    let er1 = error_handler(run_loop(&mut state));
    let er2 = error_handler(restore_input_buffering(&mut termio));
    // Exit if either the run_loop or the restore_input_buffering failed
    if !(er1 && er2) {
        std::process::exit(0);
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn loop_test() {
        let mut state = State {
            memory: [0_u16; MEM_MAX],
            registers: [0_u16; Registers::InstRet as usize],
            running: true,
        };
        state.memory_write(50, 25689);
        state.memory_write(25689, 25);
        state.memory_write(56, 777);
        state.memory_write(9, 50);
        state.register_write(Registers::Pc, 10);
        state.memory_write(10, 0xAA27); // Load indirect 25 to R5
        state.memory_write(11, 0x27FD); // Load 50 to R3
        state.memory_write(12, 0x12C5); // Add R3 + R5 into R1
        state.memory_write(13, 0x56E0); // Clear R3 by doing R3 AND 0x0
        state.memory_write(14, 0x0405); // Branch to 20 if flag Z = 1
        state.memory_write(20, 0x96FF); // Negate R3
        state.memory_write(21, 0xC140); // Jump to the value at R5 PC = 25
        state.memory_write(25, 0x635F); // Load register R1 with R5 + 40
        state.memory_write(26, 0x4048); // Jump to the value at register 1, R7 = 27, PC = 777
        state.memory_write(777, 0xB34C); // Save at memory address 0 the value from register 1
        state.memory_write(778, 0x3E03); // Save R7 into 782
        state.memory_write(779, 0x7A40); // Save R5 into 777
        state.memory_write(780, 0xF025); // Halt
        let _ = run_loop(&mut state);
        assert_eq!(state.memory_read(0), 777);
        assert_eq!(state.memory_read(782), 27);
        assert_eq!(state.memory_read(777), 25);
        assert_eq!(state.register_read(Registers::R7), 27);
    }
}
