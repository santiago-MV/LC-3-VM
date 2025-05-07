use crate::{Flags, Registers, State};
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
        add(0x1E41, &mut state);
        assert_eq!(state.registers[7], 0);
        assert_eq!(state.registers[Registers::Rcond], Flags::Zro as u16);
        state.registers[1] = 2;
        add(0x1E01, &mut state);
        assert_eq!(state.registers[7], 2);
        assert_eq!(state.registers[Registers::Rcond], Flags::Pos as u16);
    }

    #[test]
    fn add_test_mode_1() {
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::Rcount as usize],
        };
        add(0x1E61, &mut state);
        assert_eq!(state.registers[7], 1);
        assert_eq!(state.registers[Registers::Rcond], Flags::Pos as u16);
        add(0x1E3F, &mut state);
        assert_eq!(state.registers[7], 0xFFFF);
        assert_eq!(state.registers[Registers::Rcond], Flags::Neg as u16);
    }

    #[test]
    fn load_indirect_test() {
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; 10],
        };
        state.memory[20] = 7890;
        state.memory[7890] = 5;
        state.registers[Registers::Rpc] = 5;
        load_indirect(0xA40F, &mut state);
        assert_eq!(state.registers[Registers::Rr2], 5);
        assert_eq!(state.registers[Registers::Rcond], Flags::Pos as u16);
        state.registers[Registers::Rpc] = 25;
        load_indirect(0xA1FB, &mut state);
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
        and(0x5F46, &mut state);
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
        and(0x5F66, &mut state);
        assert_eq!(state.registers[Registers::Rr7], 0x0006);
        assert_eq!(state.registers[Registers::Rcond], Flags::Pos as u16);
        and(0x5F76, &mut state);
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
        conditional_branch(0x805, &mut state); // Test Flag Neg
        conditional_branch(0x405, &mut state); // Test Flag Zero
        conditional_branch(0x205, &mut state); // Test Flag Pos
        assert_eq!(state.registers[Registers::Rpc], 5);
        state.registers[Registers::Rcond] = Flags::Zro as u16; // Flag Zro = 1
        conditional_branch(0x805, &mut state); // Test Flag Neg
        conditional_branch(0x405, &mut state); // Test Flag Zero
        conditional_branch(0x205, &mut state); // Test Flag Pos
        assert_eq!(state.registers[Registers::Rpc], 10);
        state.registers[Registers::Rcond] = Flags::Pos as u16; // Flag Pos = 1
        conditional_branch(0x805, &mut state); // Test Flag Neg
        conditional_branch(0x405, &mut state); // Test Flag Zero
        conditional_branch(0x205, &mut state); // Test Flag Pos
        assert_eq!(state.registers[Registers::Rpc], 15);
        conditional_branch(0xFFB, &mut state); // Add -5 if any of the flags is active
        assert_eq!(state.registers[Registers::Rpc], 10);
    }

    #[test]
    fn jump_test() {
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::Rcount as usize],
        };
        state.registers[Registers::Rr5] = 25;
        jump(0xC140, &mut state);
        assert_eq!(state.registers[Registers::Rpc], 25);
    }

    #[test]
    fn jump_to_subrutine_test() {
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::Rcount as usize],
        };
        state.registers[Registers::Rpc] = 15;
        jump_to_subrutine(0x4FFB, &mut state);
        assert_eq!(state.registers[Registers::Rpc], 10);
        assert_eq!(state.registers[Registers::Rr7], 15);
        state.registers[Registers::Rr5] = 50;
        jump_to_subrutine(0x4140, &mut state);
        assert_eq!(state.registers[Registers::Rr7], 10);
        assert_eq!(state.registers[Registers::Rpc], 50);
    }

    #[test]
    fn load_test() {
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::Rcount as usize],
        };
        state.memory[50] = 70;
        load(0x2E32, &mut state);
        assert_eq!(state.registers[Registers::Rr7], 70);
        assert_eq!(state.registers[Registers::Rcond], Flags::Pos as u16);
    }

    #[test]
    fn load_register_test() {
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::Rcount as usize],
        };
        state.memory[50] = 78;
        state.registers[Registers::Rr2] = 25;
        load_register(0x6A99, &mut state);
        assert_eq!(state.registers[Registers::Rr5], 78);
        assert_eq!(state.registers[Registers::Rcond], Flags::Pos as u16);
    }

    #[test]
    fn load_effective_address_test() {
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::Rcount as usize],
        };
        state.registers[Registers::Rpc] = 15;
        load_effective_address(0xE21F, &mut state);
        assert_eq!(state.registers[Registers::Rr1], 46);
    }

    #[test]
    fn not_test(){
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::Rcount as usize],
        };
        state.registers[Registers::Rr5] = 0x00FF;
        not(0x977F,&mut state);
        assert_eq!(state.registers[Registers::Rr3], 0xFF00);
        assert_eq!(state.registers[Registers::Rcond], Flags::Neg as u16);
        not(0x96FF,&mut state);
        assert_eq!(state.registers[Registers::Rr3], 0xFF);
        assert_eq!(state.registers[Registers::Rcond], Flags::Pos as u16);
    }

    #[test]
    fn store_test(){
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::Rcount as usize],
        };
        state.registers[Registers::Rr4] = 777;
        store(0x3819, &mut state);
        assert_eq!(state.memory[25], 777);
    }

    #[test]
    fn integration_test() {
        // Initializate values
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::Rcount as usize],
        };
        state.memory[50] = 25689;
        state.memory[25689] = 25;
        state.memory[65] = 777;
        state.memory[10] = 50;
        state.registers[Registers::Rpc] = 10;
        // Indirectly insert a value in register 5
        load_indirect(0xAA28, &mut state);
        assert_eq!(state.registers[Registers::Rr5], 25);
        // Take the value from register 5, add -14 to it and save it in register 2
        add(0x1572, &mut state);
        assert_eq!(state.registers[Registers::Rr2], 11);
        // Make an and operation that will clear the value in register 2
        and(0x54B4, &mut state);
        assert_eq!(state.registers[Registers::Rr2], 0x0000);
        assert_eq!(state.registers[Registers::Rcond], Flags::Zro as u16);
        // If the Rcond flag is 0 increase the progam counter
        conditional_branch(0x405, &mut state);
        assert_eq!(state.registers[Registers::Rpc], 15);
        // Set the program counter to the value from register5
        jump(0xC140, &mut state);
        assert_eq!(state.registers[Registers::Rpc], 25);
        // Jump by adding an offset to the PC, previously save its value un register 7
        jump_to_subrutine(0x4FFB, &mut state);
        assert_eq!(state.registers[Registers::Rr7], 25);
        assert_eq!(state.registers[Registers::Rpc], 20);
        // Directly load a value from memory to register 2
        load(0x25F6, &mut state);
        assert_eq!(state.registers[Registers::Rr2], 50);
        // Load from register 2 with an offset of 15 to register 3
        load_register(0x668f, &mut state);
        assert_eq!(state.registers[Registers::Rr3], 777);
        // ADD 30 to the PC and save it in register 0
        load_effective_address(0xE21E, &mut state);
        assert_eq!(state.registers[Registers::Rr1], 50);
        // Doing a not in register 3
        not(0x96FF,&mut state);
        assert_eq!(state.registers[Registers::Rr3],0xFCF6);
        assert_eq!(state.registers[Registers::Rcond], Flags::Neg as u16);
        // Save the value from register 3 in memory with an offset of 25
        store(0x3619,&mut state);
        assert_eq!(state.memory[45],0xFCF6);
    }
}
