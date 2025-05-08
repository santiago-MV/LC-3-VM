use core::panic;
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

impl From<u16> for Registers {
    fn from(value: u16) -> Self {
        match value {
            0 => Registers::Rr0,
            1 => Registers::Rr1,
            2 => Registers::Rr2,
            3 => Registers::Rr3,
            4 => Registers::Rr4,
            5 => Registers::Rr5,
            6 => Registers::Rr6,
            7 => Registers::Rr7,
            _ => todo!(),
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

impl From<u16> for Operations {
    fn from(value: u16) -> Self {
        match value {
            0 => Operations::Br,
            1 => Operations::Add,
            2 => Operations::Ld,
            3 => Operations::St,
            4 => Operations::Jsr,
            5 => Operations::And,
            6 => Operations::Ldr,
            7 => Operations::Str,
            8 => Operations::Rti,
            9 => Operations::Not,
            10 => Operations::Ldi,
            11 => Operations::Sti,
            12 => Operations::Jmp,
            13 => Operations::Res,
            14 => Operations::Lea,
            15 => Operations::Trap,
            _ => todo!(),
        }
    }
}

struct State {
    memory: [u16; MEM_MAX],
    registers: [u16; Registers::Rcount as usize],
    running: bool,
}

fn main() {
    let mut termio = Termios::from_fd(io::stdin().as_raw_fd()).unwrap();
    let _ = ctrlc::set_handler(move || {
        restore_input_buffering(&mut termio);
        std::process::exit(0);
    });
    //ctrlc::set_handler(move || {restore_input_buffering(&mut termio); return;}).unwrap();
    disable_input_buffering(&mut termio);
    // Initialize empty memory and array with registers
    let mut state = State {
        memory: [0_u16; MEM_MAX],
        registers: [0_u16; Registers::Rcount as usize],
        running: true,
    };
    // Read file
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("No files were entered");
    }
    let paths = &args[1..].to_vec();
    for p in paths {
        file_management::read_file_to_memory(p, &mut state.memory);
    }
    state.registers[Registers::Rpc] = PC_START;
    state.registers[Registers::Rcond] = Flags::Zro as u16;
    while state.running {
        // Get next instruction from memory, increment the PC by one and get the OP_CODE
        let instruction = memory_read(state.registers[Registers::Rpc] as usize, &mut state);
        state.registers[Registers::Rpc] += 1;
        let op_code = instruction >> 12;

        match Operations::from(op_code) {
            Operations::Br => conditional_branch(instruction, &mut state),
            Operations::Add => add(instruction, &mut state),
            Operations::Ld => load(instruction, &mut state),
            Operations::St => store(instruction, &mut state),
            Operations::Jsr => jump_to_subrutine(instruction, &mut state),
            Operations::And => and(instruction, &mut state),
            Operations::Ldr => load_register(instruction, &mut state),
            Operations::Str => store_register(instruction, &mut state),
            Operations::Rti => todo!(),
            Operations::Not => not(instruction, &mut state),
            Operations::Ldi => load_indirect(instruction, &mut state),
            Operations::Sti => store_indirect(instruction, &mut state),
            Operations::Jmp => jump(instruction, &mut state),
            Operations::Res => todo!(),
            Operations::Lea => load_effective_address(instruction, &mut state),
            Operations::Trap => trap(instruction, &mut state),
        }
    }
    restore_input_buffering(&mut termio);
}

fn disable_input_buffering(termio: &mut Termios) {
    tcgetattr(io::stdin().as_raw_fd(), termio).unwrap();
    let new_tio = termio;
    new_tio.c_lflag &= !ICANON & !ECHO;
    tcsetattr(io::stdin().as_raw_fd(), TCSANOW, new_tio).unwrap();
}

fn restore_input_buffering(termio: &mut Termios) {
    tcsetattr(io::stdin().as_raw_fd(), TCSANOW, termio).unwrap();
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
