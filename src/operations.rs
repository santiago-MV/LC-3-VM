use crate::{Flags, Registers, State};
/// Binary add operation with 2 possible encodings
/// The number between the () indicates the amount of bits of that field or its value
/// * Register mode:    |OP_Code (0001)|DR (3)|SR1 (3)|0|00|SR2 (3)|
/// * Immediate mode:   |OP_Code (0001)|DR (3)|SR1 (3)|1| IMMR5 (5)|
pub(crate) fn add(value: u16, state: &mut State) {
    // Shift right so that the 3 dr bits are in the less significant position, do an binary and operation with 3 ones (0x7) to take their value
    let dr = (value >> 9) & 0x7;
    let sr1 = (value >> 6) & 0x7;
    let mode = (value >> 5) & 0x1;
    if mode == 1 {
        let imm5 = sign_extend((value) & 0x1F, 5);
        state.registers[dr as usize] = u16::wrapping_add(state.registers[sr1 as usize], imm5);
    } else {
        let sr2 = value & 0x7;
        state.registers[dr as usize] =
            u16::wrapping_add(state.registers[sr1 as usize], state.registers[sr2 as usize]);
    }
    update_flags(dr, &mut state.registers);
}
/// Load the data from a memory location into the destination register
/// The number between the () indicates the amount of bits of that field or its value
/// Instruction: | OP_Code (1010)| DR (3)| PCOffset9 (9)|
pub(crate) fn load_indirect(value: u16, state: &mut State) {
    let dr = value >> 9 & 0x7; // Take the 3 DR bits
    let pc_offset = sign_extend(value & 0x1FF, 9); // Take the 9 PCOffset bits and sign_extend them
    let memory_index = u16::wrapping_add(state.registers[Registers::Rpc], pc_offset);
    state.registers[dr as usize] = state.memory[memory_index as usize];
    update_flags(dr, &mut state.registers);
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
    use crate::{Flags, Registers, State, MEM_MAX};

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
    fn integration_test(){
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::Rcount as usize],
        };
        state.memory[50] = 25;
        state.registers[Registers::Rpc] = 10;
        load_indirect(0b1010_1010_0010_1000, &mut state); // Move the value from register 40 positions from PC to the register 5
        assert_eq!(state.registers[Registers::Rr5],25);
        add(0b0001_0101_0111_0010,&mut state); // Add -14 to register 5 and save it in register 2
        assert_eq!(state.registers[Registers::Rr2],11);

    }
}
