use crate::{Flags, Registers, State,};

/// Binary ADD operation with 2 possible encodings
/// The number between the () indicates the amount of bits of that field or its value
/// * Register mode:    |OP_Code (0001)|DR (3)|SR1 (3)|0|00|SR2 (3)|
/// * Immediate mode:   |OP_Code (0001)|DR (3)|SR1 (3)|1| IMMR5 (5)|<br>
///   When finished update flags
pub(crate) fn add(instruction: u16, state: &mut State) {
    // Shift right so that the 3 dr bits are in the less significant position, do an binary and operation with 3 ones (0x7) to take their value
    let destination_register = Registers::from((instruction >> 9) & 0x7);
    let source_register_1 = Registers::from((instruction >> 6) & 0x7);
    let mode = (instruction >> 5) & 0x1;
    if mode == 1 {
        let value_to_add = sign_extend((instruction) & 0x1F, 5);
        state.registers[destination_register] =
            u16::wrapping_add(state.registers[source_register_1], value_to_add);
    } else {
        let source_register_2 = Registers::from(instruction & 0x7);
        state.registers[destination_register] = u16::wrapping_add(
            state.registers[source_register_1],
            state.registers[source_register_2],
        );
    }
    update_flags(destination_register, &mut state.registers);
}
/// Load the data from a memory location into the destination register
/// The number between the () indicates the amount of bits of that field or its value
/// * Instruction: | OP_Code (1010)| DR (3)| PCOffset9 (9)|<br>
///   When finished update flags
pub(crate) fn load_indirect(instruction: u16, state: &mut State) {
    let destination_register = Registers::from((instruction >> 9) & 0x7); // Take the 3 DR bits
    let pc_offset = sign_extend(instruction & 0x1FF, 9); // Take the 9 PCOffset bits and sign_extend them
    let memory_index = u16::wrapping_add(state.registers[Registers::Rpc], pc_offset) as usize;
    state.registers[destination_register] = state.memory[state.memory[memory_index] as usize];
    update_flags(destination_register, &mut state.registers);
}
/// Binary AND operation with two possible encodings
/// The number between the () indicates the amount of bits of that field or its value
/// * Register mode:    |OP_Code (0101)|DR (3)|SR1 (3)|0|00|SR2 (3)|
/// * Immediate mode:   |OP_Code (0101)|DR (3)|SR1 (3)|1| IMMR5 (5)|<br>
///   When finished update flags
pub(crate) fn and(instruction: u16, state: &mut State) {
    let destination_register = Registers::from((instruction >> 9) & 0x7);
    let source_register_1 = Registers::from((instruction >> 6) & 0x7);
    let mode = (instruction >> 5) & 0x1;
    if mode == 1 {
        let value_to_and = sign_extend((instruction) & 0x1F, 5);
        state.registers[destination_register] = state.registers[source_register_1] & value_to_and;
    } else {
        let source_register_2 = Registers::from(instruction & 0x7);
        state.registers[destination_register] =
            state.registers[source_register_1] & state.registers[source_register_2];
    }
    update_flags(destination_register, &mut state.registers);
}

/// Conditional branch operator
/// * Instruction: |OP_Code (0000)| n (1)| z (1)| p (1)| PCOffset9 (9)|<br>
///   The three bits after the OP_Code set which flags will be tested
/// * n = 1 => The Neg flag is tested
/// * z = 1 => The Zro flag is tested
/// * p = 1 => The Pos flag is tested
///
/// If the flag tested is has the value 1, then the sign extended PCOffset9 is added to the Program counter<br>
/// Only one of the flags will have the value 1 at each moment, so if multiple flags are tested only one needs to be in 1 for the branch to occure
pub(crate) fn conditional_branch(instruction: u16, state: &mut State) {
    let negative_indicator = (instruction >> 11) & 1;
    let zero_indicator = (instruction >> 10) & 1;
    let positive_indicator = (instruction >> 9) & 1;
    let current_flags = state.registers[Registers::Rcond];
    if ((negative_indicator & current_flags >> 2) == 1)
        || ((zero_indicator & current_flags >> 1) == 1)
        || ((positive_indicator & current_flags) == 1)
    {
        let pc_offset = sign_extend(instruction & 0x1FF, 9);
        state.registers[Registers::Rpc] =
            u16::wrapping_add(state.registers[Registers::Rpc], pc_offset)
    }
}

/// Set the program counter to the value of the base register
/// * Instruction: |OP_Code (1100)|000| BaseR (3)|000000|
pub(crate) fn jump(instruction: u16, state: &mut State) {
    let base_register = Registers::from((instruction >> 6) & 0x7);
    state.registers[Registers::Rpc] = state.registers[base_register];
}

/// Save the value of the program counter in register 7 and increment the program counter
/// by an singn extended offset or set its value to the one in the base register
/// * Immediate mode (JSR):    |OP_Code (0100)|1 (Mode)|PCOffset (11)|
/// * Register mode (JSRR):    |OP_Code (0100)|0 (Mode)|00|BaseR (3)|000000|
pub(crate) fn jump_to_subrutine(instruction: u16, state: &mut State) {
    state.registers[Registers::Rr7] = state.registers[Registers::Rpc];
    let mode = (instruction >> 11) & 1;
    if mode == 1 {
        let offset = sign_extend(instruction & 0x7FF, 11);
        state.registers[Registers::Rpc] =
            u16::wrapping_add(state.registers[Registers::Rpc], offset);
    } else {
        let base_register = Registers::from((instruction >> 6) & 0x7);
        state.registers[Registers::Rpc] = state.registers[base_register];
    }
}

/// Read the value from the memory location at progam counter + sign extended offset and write it in the destination registry
/// * Instruction: |OP_Code (0010)|DR (3)|PCOffset (9)|<br>
///   When finished update flags
pub(crate) fn load(instruction: u16, state: &mut State) {
    let sign_extended_offset = sign_extend(instruction & 0x1FF, 9);
    let destination_register = Registers::from((instruction >> 9) & 0x7);
    let memory_index =
        u16::wrapping_add(state.registers[Registers::Rpc], sign_extended_offset) as usize;
    state.registers[destination_register] = state.memory[memory_index];
    update_flags(destination_register, &mut state.registers);
}

/// Load a value from memory to the destination register. <br>
/// The memory direction is given by the value inside the base register and the sign extended offset
/// * Instruction: |OP_Code (0110)|DR (3)|BaseR (3)|Offset (6)|<br>
///   When finished update flags
pub(crate) fn load_register(instruction: u16, state: &mut State) {
    let sign_extended_offset = sign_extend(instruction & 0x3F, 6);
    let base_register = Registers::from(instruction >> 6 & 0x7);
    let destination_register = Registers::from(instruction >> 9 & 0x7);
    let memory_index =
        u16::wrapping_add(state.registers[base_register], sign_extended_offset) as usize;
    state.registers[destination_register] = state.memory[memory_index];
    update_flags(destination_register, &mut state.registers);
}

/// Load a memory address into a register
/// * Instruction: |OP_Code (1110)|DR (3)|Offset (9)|<br>
///   When finished update flags
pub(crate) fn load_effective_address(instruction: u16, state: &mut State) {
    let sign_extended_offset = sign_extend(instruction & 0x1FF, 9);
    let destination_register = Registers::from((instruction >> 9) & 0x7);
    let address = u16::wrapping_add(state.registers[Registers::Rpc], sign_extended_offset);
    state.registers[destination_register] = address;
    update_flags(destination_register, &mut state.registers);
}

/// Calculate the bitwise complement of the source registry and save it in the destination registry
/// * Instruction: |OP_Code (1001)|DR (3)|SR (3)|1|11111|<br>
///   When finished update flags
pub(crate) fn not(instruction: u16,state: &mut State){
    let source_registry = Registers::from((instruction>>6) & 0x7);
    let destination_registry = Registers::from((instruction>>9) & 0x7);
    state.registers[destination_registry] = !(state.registers[source_registry]);
    update_flags(destination_registry, &mut state.registers);
}   

/// Store the contents of a source register into a specific location in memory
/// * Instruction: |OP_Code (0011)|SR (3)|PCOffset (9)|<br>
pub(crate) fn store(instruction: u16,state: &mut State){
    let sign_extended_offset = sign_extend(instruction & 0x1FF, 9);
    let source_register = Registers::from((instruction >> 9) & 0x7);
    let memory_address = u16::wrapping_add(state.registers[Registers::Rpc], sign_extended_offset) as usize;
    state.memory[memory_address] = state.registers[source_register];
}

/// The instruction receives the address where the memory address for storing the value is located, it reads it and saves the register value in it
/// * Instruction: |OP_Code (1011)|SR (3)|PCOffset (9)|<br>
pub(crate) fn store_indirect(instruction: u16,state: &mut State){
    let sign_extended_offset = sign_extend(instruction & 0x1FF, 9);
    let source_register = Registers::from((instruction >> 9) & 0x7);
    let memory_address = u16::wrapping_add(state.registers[Registers::Rpc], sign_extended_offset) as usize;
    state.memory[state.memory[memory_address] as usize] = state.registers[source_register];
}
/// Store the register in memory, the address is calculated using the base register's content and a sign extended offset
/// * Instruction: |OP_Code (0111)|SR (3)|BaseR (3)|Offset (6)|<br>
pub(crate) fn store_register(instruction: u16,state: &mut State){
    let sign_extended_offset = sign_extend(instruction & 0x3F, 6);
    let base_register = Registers::from(instruction >> 6 & 0x7);
    let source_register = Registers::from(instruction >> 9 & 0x7);
    let memory_address = u16::wrapping_add(state.registers[base_register], sign_extended_offset) as usize;
    state.memory[memory_address] = state.registers[source_register];
}

fn update_flags(register: Registers, registers: &mut [u16; 10]) {
    if registers[register] == 0 {
        registers[Registers::Rcond] = Flags::Zro as u16;
    }
    // If the left-most bit is a 1 then the number is negative
    else if registers[register] >> 15 == 1 {
        registers[Registers::Rcond] = Flags::Neg as u16;
    } else {
        registers[Registers::Rcond] = Flags::Pos as u16;
    }
}

fn sign_extend(value: u16, bit_count: u16) -> u16 {
    let mut x = value;
    // If the value is negative add 1s to complete the 16 bytes, if not do nothing
    if (value >> (bit_count - 1)) == 1 {
        x |= 0xFFFF << bit_count;
    }
    x
}
