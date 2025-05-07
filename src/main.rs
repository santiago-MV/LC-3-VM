use std::env;
use std::ops::{Index, IndexMut};
pub mod file_management;
pub mod operations;
use operations::*;
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
}

fn main() -> Result<(), String> {
    // Initialize empty memory and array with registers
    let mut state = State {
        memory: [0_u16; MEM_MAX],
        registers: [0_u16; Registers::Rcount as usize],
    };
    // Read file
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err("No files were entered".to_string());
    }
    let paths = &args[1..].to_vec();
    for p in paths {
        file_management::read_file_to_memory(p, &mut state.memory);
    }

    state.registers[Registers::Rpc] = PC_START;
    state.registers[Registers::Rcond] = Flags::Zro as u16;
    loop {
        // Get next instruction from memory, increment the PC by one and get the OP_CODE
        let instruction = state.memory[state.registers[Registers::Rpc] as usize];
        state.registers[Registers::Rpc] += 1;
        let op_code = instruction >> 12;
        match Operations::from(op_code) {
            Operations::Br => conditional_branch(instruction, &mut state),
            Operations::Add => add(instruction, &mut state),
            Operations::Ld => load(instruction,&mut state),
            Operations::St => todo!(), //store(instruction),
            Operations::Jsr => jump_to_subrutine(instruction, &mut state),
            Operations::And => and(instruction, &mut state),
            Operations::Ldr => todo!(), 
            Operations::Str => todo!(), //store_register(instruction),
            Operations::Rti => todo!(),
            Operations::Not => todo!(), //not(instruction),
            Operations::Ldi => load_indirect(instruction, &mut state),
            Operations::Sti => todo!(), //store_indirect(instruction),
            Operations::Jmp => jump(instruction, &mut state),
            Operations::Res => todo!(),
            Operations::Lea => todo!(), //load_effective_address(instruction),
            Operations::Trap => todo!(), //execute_trap(instruction),
        }
    }
}
