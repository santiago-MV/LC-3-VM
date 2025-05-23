use crate::{Errors, Flags, Registers, State, Traps};
use std::{
    char,
    io::{Read, Write, stdin, stdout},
};

const NULL_WORD: u16 = 0x0;

/// Binary ADD operation with 2 possible encodings
/// The number between the () indicates the amount of bits of that field or its value
/// * Register mode:    |OP_Code (0001)|DR (3)|SR1 (3)|0|00|SR2 (3)|
/// * Immediate mode:   |OP_Code (0001)|DR (3)|SR1 (3)|1| IMMR5 (5)|<br>
///   When finished update flags
pub(crate) fn add(instruction: u16, state: &mut State) -> Result<(), Errors> {
    // Shift right so that the 3 dr bits are in the less significant position, do an binary and operation with 3 ones (0x7) to take their value
    let destination_register = Registers::try_from((instruction >> 9) & 0x7).unwrap(); // Can't break because its maximum value is 8 (111)
    let source_register_1 = Registers::try_from((instruction >> 6) & 0x7).unwrap(); // Can't break because its maximum value is 8 (111)
    let mode = (instruction >> 5) & 0x1;
    if mode == 1 {
        let value_to_add = sign_extend((instruction) & 0x1F, 5);
        state.register_write(
            destination_register,
            u16::wrapping_add(state.register_read(source_register_1), value_to_add),
        );
    } else {
        let source_register_2 = Registers::try_from(instruction & 0x7).unwrap(); // Can't break because its maximum value is 8 (111)
        state.register_write(
            destination_register,
            u16::wrapping_add(
                state.register_read(source_register_1),
                state.register_read(source_register_2),
            ),
        );
    }
    update_flags(destination_register, &mut state.registers);
    Ok(())
}
/// Load the data from a memory location into the destination register
/// The number between the () indicates the amount of bits of that field or its value
/// * Instruction: | OP_Code (1010)| DR (3)| PCOffset9 (9)|<br>
///   When finished update flags
pub(crate) fn load_indirect(instruction: u16, state: &mut State) -> Result<(), Errors> {
    let destination_register = Registers::try_from((instruction >> 9) & 0x7).unwrap(); // Take the 3 DR bits, Can't break because its maximum value is 8 (111)
    let pc_offset = sign_extend(instruction & 0x1FF, 9); // Take the 9 PCOffset bits and sign_extend them
    let memory_index = u16::wrapping_add(state.register_read(Registers::Pc), pc_offset) as usize;
    let actual_index = state.memory_read(memory_index as usize) as usize;
    let value = state.memory_read(actual_index);
    state.register_write(destination_register, value);
    update_flags(destination_register, &mut state.registers);
    Ok(())
}
/// Binary AND operation with two possible encodings
/// The number between the () indicates the amount of bits of that field or its value
/// * Register mode:    |OP_Code (0101)|DR (3)|SR1 (3)|0|00|SR2 (3)|
/// * Immediate mode:   |OP_Code (0101)|DR (3)|SR1 (3)|1| IMMR5 (5)|<br>
///   When finished update flags
pub(crate) fn and(instruction: u16, state: &mut State) -> Result<(), Errors> {
    let destination_register = Registers::try_from((instruction >> 9) & 0x7).unwrap(); // Can't break because its maximum value is 8 (111)
    let source_register_1 = Registers::try_from((instruction >> 6) & 0x7).unwrap(); // Can't break because its maximum value is 8 (111)
    let mode = (instruction >> 5) & 0x1;
    let value_to_and = if mode == 1 {
        sign_extend((instruction) & 0x1F, 5)
    } else {
        let source_register_2 = Registers::try_from(instruction & 0x7).unwrap(); // Can't break because its maximum value is 8 (111)
        state.register_read(source_register_2)
    };
    let value = state.register_read(source_register_1) & value_to_and;
    state.register_write(destination_register, value);
    update_flags(destination_register, &mut state.registers);
    Ok(())
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
    let current_flags = state.register_read(Registers::Flags);
    let is_negative = (negative_indicator & current_flags >> 2) == 1;
    let is_zero = (zero_indicator & current_flags >> 1) == 1;
    let is_positive = (positive_indicator & current_flags) == 1;
    if is_negative || is_zero || is_positive {
        let pc_offset = sign_extend(instruction & 0x1FF, 9);
        state.register_write(
            Registers::Pc,
            u16::wrapping_add(state.register_read(Registers::Pc), pc_offset),
        );
    }
}

/// Set the program counter to the value of the base register
/// * Instruction: |OP_Code (1100)|000| BaseR (3)|000000|
pub(crate) fn jump(instruction: u16, state: &mut State) -> Result<(), Errors> {
    let base_register = Registers::try_from((instruction >> 6) & 0x7).unwrap(); // Can't break because its maximum value is 8 (111)
    state.register_write(Registers::Pc, state.register_read(base_register));
    Ok(())
}

/// Save the value of the program counter in register 7 and increment the program counter
/// by an singn extended offset or set its value to the one in the base register
/// * Immediate mode (JSR):    |OP_Code (0100)|1 (Mode)|PCOffset (11)|
/// * Register mode (JSRR):    |OP_Code (0100)|0 (Mode)|00|BaseR (3)|000000|
pub(crate) fn jump_to_subrutine(instruction: u16, state: &mut State) -> Result<(), Errors> {
    state.register_write(Registers::R7, state.register_read(Registers::Pc));
    let mode = (instruction >> 11) & 1;
    if mode == 1 {
        let offset = sign_extend(instruction & 0x7FF, 11);
        state.register_write(
            Registers::Pc,
            u16::wrapping_add(state.register_read(Registers::Pc), offset),
        );
    } else {
        let base_register = Registers::try_from((instruction >> 6) & 0x7).unwrap(); // Can't break because its maximum value is 8 (111)
        state.register_write(Registers::Pc, state.register_read(base_register));
    }
    Ok(())
}

/// Read the value from the memory location at progam counter + sign extended offset and write it in the destination registry
/// * Instruction: |OP_Code (0010)|DR (3)|PCOffset (9)|<br>
///   When finished update flags
pub(crate) fn load(instruction: u16, state: &mut State) -> Result<(), Errors> {
    let sign_extended_offset = sign_extend(instruction & 0x1FF, 9);
    let destination_register = Registers::try_from((instruction >> 9) & 0x7).unwrap(); // Can't break because its maximum value is 8 (111)
    let memory_index =
        u16::wrapping_add(state.register_read(Registers::Pc), sign_extended_offset) as usize;
    let value = state.memory_read(memory_index);
    state.register_write(destination_register, value);
    update_flags(destination_register, &mut state.registers);
    Ok(())
}

/// Load a value from memory to the destination register. <br>
/// The memory direction is given by the value inside the base register and the sign extended offset
/// * Instruction: |OP_Code (0110)|DR (3)|BaseR (3)|Offset (6)|<br>
///   When finished update flags
pub(crate) fn load_register(instruction: u16, state: &mut State) -> Result<(), Errors> {
    let sign_extended_offset = sign_extend(instruction & 0x3F, 6);
    let base_register = Registers::try_from(instruction >> 6 & 0x7).unwrap(); // Can't break because its maximum value is 8 (111)
    let destination_register = Registers::try_from(instruction >> 9 & 0x7).unwrap(); // Can't break because its maximum value is 8 (111)
    let memory_index =
        u16::wrapping_add(state.register_read(base_register), sign_extended_offset) as usize;
    let value = state.memory_read(memory_index);
    state.register_write(destination_register, value);
    update_flags(destination_register, &mut state.registers);
    Ok(())
}

/// Load a memory address into a register
/// * Instruction: |OP_Code (1110)|DR (3)|Offset (9)|<br>
///   When finished update flags
pub(crate) fn load_effective_address(instruction: u16, state: &mut State) -> Result<(), Errors> {
    let sign_extended_offset = sign_extend(instruction & 0x1FF, 9);
    let destination_register = Registers::try_from((instruction >> 9) & 0x7).unwrap(); // Can't break because its maximum value is 8 (111)
    let address = u16::wrapping_add(state.register_read(Registers::Pc), sign_extended_offset);
    state.register_write(destination_register, address);
    update_flags(destination_register, &mut state.registers);
    Ok(())
}

/// Calculate the bitwise complement of the source registry and save it in the destination registry
/// * Instruction: |OP_Code (1001)|DR (3)|SR (3)|1|11111|<br>
///   When finished update flags
pub(crate) fn not(instruction: u16, state: &mut State) -> Result<(), Errors> {
    let source_registry = Registers::try_from((instruction >> 6) & 0x7).unwrap(); // Can't break because its maximum value is 8 (111)
    let destination_registry = Registers::try_from((instruction >> 9) & 0x7).unwrap(); // Can't break because its maximum value is 8 (111)
    state.register_write(
        destination_registry,
        !(state.register_read(source_registry)),
    );
    update_flags(destination_registry, &mut state.registers);
    Ok(())
}

/// Store the contents of a source register into a specific location in memory
/// * Instruction: |OP_Code (0011)|SR (3)|PCOffset (9)|<br>
pub(crate) fn store(instruction: u16, state: &mut State) -> Result<(), Errors> {
    let sign_extended_offset = sign_extend(instruction & 0x1FF, 9);
    let source_register = Registers::try_from((instruction >> 9) & 0x7).unwrap(); // Can't break because its maximum value is 8 (111)
    let memory_address =
        u16::wrapping_add(state.register_read(Registers::Pc), sign_extended_offset) as usize;
    state.memory_write(memory_address, state.register_read(source_register));
    Ok(())
}

/// The instruction takes the memory address containing the memory location where the source register's value should be stored and stores it.
/// * Instruction: |OP_Code (1011)|SR (3)|PCOffset (9)|<br>
pub(crate) fn store_indirect(instruction: u16, state: &mut State) -> Result<(), Errors> {
    let sign_extended_offset = sign_extend(instruction & 0x1FF, 9);
    let source_register = Registers::try_from((instruction >> 9) & 0x7).unwrap(); // Can't break because its maximum value is 8 (111)
    let memory_address =
        u16::wrapping_add(state.register_read(Registers::Pc), sign_extended_offset) as usize;
    let actual_address = state.memory_read(memory_address) as usize;
    state.memory_write(actual_address, state.register_read(source_register));
    Ok(())
}
/// Store the register in memory, the address is calculated using the base register's content and a sign extended offset
/// * Instruction: |OP_Code (0111)|SR (3)|BaseR (3)|Offset (6)|<br>
pub(crate) fn store_register(instruction: u16, state: &mut State) -> Result<(), Errors> {
    let sign_extended_offset = sign_extend(instruction & 0x3F, 6);
    let base_register = Registers::try_from(instruction >> 6 & 0x7).unwrap(); // Can't break because its maximum value is 8 (111)
    let source_register = Registers::try_from(instruction >> 9 & 0x7).unwrap(); // Can't break because its maximum value is 8 (111)
    let memory_address =
        u16::wrapping_add(state.register_read(base_register), sign_extended_offset) as usize;
    state.memory_write(memory_address, state.register_read(source_register));
    Ok(())
}

/// Given a trap instruction call the correct routine
/// * Instruction: |OP_Code (1111)|0000|TrapVect (8)|<br>
pub(crate) fn trap(instruction: u16, state: &mut State) -> Result<(), Errors> {
    let routine = Traps::try_from(instruction & 0xFF)?;
    match routine {
        Traps::Getc => trap_routine_getc(state)?,
        Traps::Out => trap_routine_out(state)?,
        Traps::Puts => trap_routine_puts(state),
        Traps::In => trap_routine_in(state)?,
        Traps::Putsp => trap_routine_putsp(state),
        Traps::Halt => trap_routine_halt(state),
    };
    Ok(())
}

/// Prints HALT and stops executing the program
fn trap_routine_halt(state: &mut State) {
    print!("HALT");
    state.running = false;
}

/// Output a string in big endian, for doing this take the memory address from the R0 register,
/// read the value in that memory position, if its different from 0x0 then print the less significant byte first
/// and if the more significant byte is different from 0x0 print it. It continues reading from the next memory position until it finds a 0x0
fn trap_routine_putsp(state: &mut State) {
    let mut address = state.register_read(Registers::R0) as usize;
    let mut character = state.memory_read(address);
    while character != NULL_WORD {
        if let Some(char1) = char::from_u32((character & 0xFF) as u32) {
            print!("{}", char1);
        } else {
            break;
        };
        let char2 = character >> 8;
        if char2 != NULL_WORD {
            if let Some(c2) = char::from_u32(char2 as u32) {
                print!("{}", c2);
            }
        }
        // Fetch next character
        address += 1;
        character = state.memory_read(address);
    }
}

/// Prompt for input character.
/// Print a line asking the user to enter a character, read the character, save it in register 0 and update the flags.
fn trap_routine_in(state: &mut State) -> Result<(), Errors> {
    print!("Enter character: ");
    let input = 0_u8;
    match stdin().read_exact(&mut [input]) {
        Ok(_) => print!("{}", input),
        Err(_) => return Err(Errors::Trap(Traps::In)),
    };
    state.register_write(Registers::R0, input as u16);
    update_flags(Registers::R0, &mut state.registers);
    Ok(())
}

/// Reads a character from register 0 and prints it
fn trap_routine_out(state: &State) -> Result<(), Errors> {
    let character = state.register_read(Registers::R0);
    if let Some(char) = char::from_u32(character as u32) {
        print!("{}", char);
    } else {
        return Err(Errors::Trap(Traps::Out));
    };
    Ok(())
}

/// Reads a single character from the keyboard and save it in the Register 0
fn trap_routine_getc(state: &mut State) -> Result<(), Errors> {
    let mut input = [0u8];
    match stdin().read_exact(&mut input) {
        Ok(_) => state.register_write(Registers::R0, input[0] as u16),
        Err(_) => return Err(Errors::Trap(Traps::Getc)),
    };
    update_flags(Registers::R0, &mut state.registers);
    Ok(())
}

/// Print a string from memory
/// Each memory position will represent one char, start reading memory at the address in the register R0, print the read character
/// and continue reading the next memory position
fn trap_routine_puts(state: &mut State) {
    let mut address = state.register_read(Registers::R0) as usize;
    let mut character = state.memory_read(address);
    while character != NULL_WORD {
        if let Some(char_char) = char::from_u32(character as u32) {
            print!("{}", char_char);
        } else {
            break;
        };
        // Fetch next character
        address += 1;
        character = state.memory_read(address);
    }
    let _ = stdout().flush();
}

/// Receives a register and the current registers status.
/// Update the RCond register acording to the value of the register passed by argument
fn update_flags(register: Registers, registers: &mut [u16; 10]) {
    if registers[register] == 0 {
        registers[Registers::Flags] = Flags::Zro as u16;
    }
    // If the left-most bit is a 1 then the number is negative
    else if registers[register] >> 15 == 1 {
        registers[Registers::Flags] = Flags::Neg as u16;
    } else {
        registers[Registers::Flags] = Flags::Pos as u16;
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
