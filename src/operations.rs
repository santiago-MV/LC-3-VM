use crate::{Flags, Registers};

pub fn add(value: u16, registers: &mut [u16; 10]) {
    let destination_resgister: u16 = (value >> 9) & 0x7;
    let source_register_1 = (value >> 6) & 0x7;
    let mode = (value >> 5) & 0x1;
    if mode == 1 {
        let value_to_add = sign_extend((value) & 0x1F, 5);
        registers[destination_resgister as usize] =
            u16::wrapping_add(registers[source_register_1 as usize], value_to_add);
    } else {
        let source_register_2 = value & 0x7;
        registers[destination_resgister as usize] = u16::wrapping_add(
            registers[source_register_1 as usize],
            registers[source_register_2 as usize],
        );
    }
    update_flags(destination_resgister, registers);
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
    // If the value is negative add 1s to complete the 16 bytes
    if (value >> (bit_count - 1)) == 1 {
        x |= 0xFFFF << bit_count;
    }
    // If its positive return the value
    x
}

#[cfg(test)]
mod test {
    use super::add;

    #[test]
    fn add_test_mode_0() {
        let mut registers = [0; 10];
        add(0b1010_1110_0100_0001, &mut registers);
        assert_eq!(registers[7], 0);
        registers[1] = 2;
        add(0b1010_1110_0000_0001, &mut registers);
        assert_eq!(registers[7], 2);
    }
    #[test]
    fn add_test_mode_1() {
        let mut registers = [0; 10];
        add(0b1010_1110_0110_0001, &mut registers);
        assert_eq!(registers[7], 1);
        add(0b1010_1110_0011_1111, &mut registers);
        assert_eq!(registers[7], 0xFFFF);
    }
}
