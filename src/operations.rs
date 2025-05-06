use crate::{Flags, Registers, State};
/// Binary ADD operation with 2 possible encodings
/// The number between the () indicates the amount of bits of that field or its value
/// * Register mode:    |OP_Code (0001)|DR (3)|SR1 (3)|0|00|SR2 (3)|
/// * Immediate mode:   |OP_Code (0001)|DR (3)|SR1 (3)|1| IMMR5 (5)|
pub(crate) fn add(instruction: u16, state: &mut State) {
    // Shift right so that the 3 dr bits are in the less significant position, do an binary and operation with 3 ones (0x7) to take their value
    let destination_register = (instruction >> 9) & 0x7;
    let source_register_1 = (instruction >> 6) & 0x7;
    let mode = (instruction >> 5) & 0x1;
    if mode == 1 {
        let value_to_add = sign_extend((instruction) & 0x1F, 5);
        state.registers[destination_register as usize] =
            u16::wrapping_add(state.registers[source_register_1 as usize], value_to_add);
    } else {
        let source_register_2 = instruction & 0x7;
        state.registers[destination_register as usize] = u16::wrapping_add(
            state.registers[source_register_1 as usize],
            state.registers[source_register_2 as usize],
        );
    }
    update_flags(destination_register, &mut state.registers);
}
/// Load the data from a memory location into the destination register
/// The number between the () indicates the amount of bits of that field or its value
/// Instruction: | OP_Code (1010)| DR (3)| PCOffset9 (9)|
pub(crate) fn load_indirect(instruction: u16, state: &mut State) {
    let destination_register = instruction >> 9 & 0x7; // Take the 3 DR bits
    let pc_offset = sign_extend(instruction & 0x1FF, 9); // Take the 9 PCOffset bits and sign_extend them
    let memory_index = u16::wrapping_add(state.registers[Registers::Rpc], pc_offset);
    state.registers[destination_register as usize] = state.memory[memory_index as usize];
    update_flags(destination_register, &mut state.registers);
}
/// Binary AND operation with two possible encodings
/// The number between the () indicates the amount of bits of that field or its value
/// * Register mode:    |OP_Code (0101)|DR (3)|SR1 (3)|0|00|SR2 (3)|
/// * Immediate mode:   |OP_Code (0101)|DR (3)|SR1 (3)|1| IMMR5 (5)|
pub(crate) fn and(instruction: u16, state: &mut State) {
    let destination_register = (instruction >> 9) & 0x7;
    let source_register_1 = (instruction >> 6) & 0x7;
    let mode = (instruction >> 5) & 0x1;
    if mode == 1 {
        let value_to_and = sign_extend((instruction) & 0x1F, 5);
        state.registers[destination_register as usize] =
            state.registers[source_register_1 as usize] & value_to_and;
    } else {
        let source_register_2 = instruction & 0x7;
        state.registers[destination_register as usize] = state.registers
            [source_register_1 as usize]
            & state.registers[source_register_2 as usize];
    }
    update_flags(destination_register, &mut state.registers);
}

/// Conditional branch operator
/// Instruction: |OP_Code (0000)| n (1)| z (1)| p (1)| PCOffset9 (9)|<br>
/// The three bits after the OP_Code set which flags will be tested
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

fn update_flags(register: u16, registers: &mut [u16; 10]) {
    if registers[Registers::from(register)] == 0 {
        registers[Registers::Rcond] = Flags::Zro as u16;
    }
    // If the left-most bit is a 1 then the number is negative
    else if registers[Registers::from(register)] >> 15 == 1 {
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

#[cfg(test)]
mod test {
    use crate::{Flags, MEM_MAX, Registers, State};

    use super::*;

    #[test]
    fn add_test_mode_0() {
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::Rcount as usize],
        };
        add(0b0001_1110_0100_0001, &mut state);
        assert_eq!(state.registers[7], 0);
        assert_eq!(state.registers[Registers::Rcond], Flags::Zro as u16);
        state.registers[1] = 2;
        add(0b0001_1110_0000_0001, &mut state);
        assert_eq!(state.registers[7], 2);
        assert_eq!(state.registers[Registers::Rcond], Flags::Pos as u16);
    }
    #[test]
    fn add_test_mode_1() {
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::Rcount as usize],
        };
        add(0b0001_1110_0110_0001, &mut state);
        assert_eq!(state.registers[7], 1);
        assert_eq!(state.registers[Registers::Rcond], Flags::Pos as u16);
        add(0b0001_1110_0011_1111, &mut state);
        assert_eq!(state.registers[7], 0xFFFF);
        assert_eq!(state.registers[Registers::Rcond], Flags::Neg as u16);
    }
    #[test]
    fn load_indirect_test() {
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; 10],
        };
        state.memory[20] = 5;
        state.registers[Registers::Rpc] = 5;
        load_indirect(0b1010_0100_0000_1111, &mut state);
        assert_eq!(state.registers[Registers::Rr2], 5);
        assert_eq!(state.registers[Registers::Rcond], Flags::Pos as u16);
        state.registers[Registers::Rpc] = 25;
        load_indirect(0b1010_0001_1111_1011, &mut state);
        assert_eq!(state.registers[Registers::Rr0], 5);
        assert_eq!(state.registers[Registers::Rcond], Flags::Pos as u16);
    }

    #[test]
    fn and_test_mode_0() {
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::Rcount as usize],
        };
        state.registers[Registers::Rr5] = 0xFFFF;
        state.registers[Registers::Rr6] = 0x000F;
        and(0b0101_1111_0100_0110, &mut state);
        assert_eq!(state.registers[Registers::Rr7], 0x000F);
        assert_eq!(state.registers[Registers::Rcond], Flags::Pos as u16);
    }

    #[test]
    fn and_test_mode_1() {
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::Rcount as usize],
        };
        state.registers[Registers::Rr5] = 0xFFFF;
        and(0b0101_1111_0110_0110, &mut state);
        assert_eq!(state.registers[Registers::Rr7], 0x0006);
        assert_eq!(state.registers[Registers::Rcond], Flags::Pos as u16);
        and(0b0101_1111_0111_0110, &mut state);
        assert_eq!(state.registers[Registers::Rr7], 0xFFF6);
        assert_eq!(state.registers[Registers::Rcond], Flags::Neg as u16);
    }

    #[test]
    fn conditional_branch_test() {
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::Rcount as usize],
        };
        state.registers[Registers::Rcond] = Flags::Neg as u16; // Flag Neg = 1
        conditional_branch(0b0000_1000_0000_0101, &mut state); // Test Flag Neg
        conditional_branch(0b0000_0100_0000_0101, &mut state); // Test Flag Zero
        conditional_branch(0b0000_0010_0000_0101, &mut state); // Test Flag Pos
        assert_eq!(state.registers[Registers::Rpc], 5);
        state.registers[Registers::Rcond] = Flags::Zro as u16; // Flag Zro = 1
        conditional_branch(0b0000_1000_0000_0101, &mut state); // Test Flag Neg
        conditional_branch(0b0000_0100_0000_0101, &mut state); // Test Flag Zero
        conditional_branch(0b0000_0010_0000_0101, &mut state); // Test Flag Pos
        assert_eq!(state.registers[Registers::Rpc], 10);
        state.registers[Registers::Rcond] = Flags::Pos as u16; // Flag Pos = 1
        conditional_branch(0b0000_1000_0000_0101, &mut state); // Test Flag Neg
        conditional_branch(0b0000_0100_0000_0101, &mut state); // Test Flag Zero
        conditional_branch(0b0000_0010_0000_0101, &mut state); // Test Flag Pos
        assert_eq!(state.registers[Registers::Rpc], 15);
        conditional_branch(0b0000_1111_1111_1011, &mut state); // Add -5 if any of the flags is active
        assert_eq!(state.registers[Registers::Rpc], 10);
    }

    #[test]
    fn integration_test() {
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::Rcount as usize],
        };
        state.memory[50] = 25;
        state.registers[Registers::Rpc] = 10;
        load_indirect(0b1010_1010_0010_1000, &mut state); // Move the value from register 40 positions from PC to the register 5
        assert_eq!(state.registers[Registers::Rr5], 25);
        add(0b0001_0101_0111_0010, &mut state); // Add -14 to register 5 and save it in register 2
        assert_eq!(state.registers[Registers::Rr2], 11);
        and(0b_0101_0100_1011_0100, &mut state);
        assert_eq!(state.registers[Registers::Rr2], 0x0000);
        assert_eq!(state.registers[Registers::Rcond], Flags::Zro as u16);
        conditional_branch(0b0000_0100_0000_0101, &mut state);
        assert_eq!(state.registers[Registers::Rpc], 15);
    }
}
