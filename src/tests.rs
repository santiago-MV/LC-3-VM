#[cfg(test)]
pub mod tests {
    use crate::*;

    #[test]
    fn add_test_mode_0() {
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::InstRet as usize],
            running: true,
        };
        let _ = add(0x1E41, &mut state);
        assert_eq!(state.registers[7], 0);
        assert_eq!(state.registers[Registers::Flags], Flags::Zro as u16);
        state.registers[1] = 2;
        let _ = add(0x1E01, &mut state);
        assert_eq!(state.registers[7], 2);
        assert_eq!(state.registers[Registers::Flags], Flags::Pos as u16);
    }

    #[test]
    fn add_test_mode_1() {
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::InstRet as usize],
            running: true,
        };
        let _ = add(0x1E61, &mut state);
        assert_eq!(state.registers[7], 1);
        assert_eq!(state.registers[Registers::Flags], Flags::Pos as u16);
        let _ = add(0x1E3F, &mut state);
        assert_eq!(state.registers[7], 0xFFFF);
        assert_eq!(state.registers[Registers::Flags], Flags::Neg as u16);
    }

    #[test]
    fn load_indirect_test() {
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; 10],
            running: true,
        };
        state.memory[20] = 7890;
        state.memory[7890] = 5;
        state.registers[Registers::Pc] = 5;
        let _ = load_indirect(0xA40F, &mut state);
        assert_eq!(state.registers[Registers::R2], 5);
        assert_eq!(state.registers[Registers::Flags], Flags::Pos as u16);
        state.registers[Registers::Pc] = 25;
        let _ = load_indirect(0xA1FB, &mut state);
        assert_eq!(state.registers[Registers::R0], 5);
        assert_eq!(state.registers[Registers::Flags], Flags::Pos as u16);
    }

    #[test]
    fn and_test_mode_0() {
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::InstRet as usize],
            running: true,
        };
        state.registers[Registers::R5] = 0xFFFF;
        state.registers[Registers::R6] = 0x000F;
        let _ = and(0x5F46, &mut state);
        assert_eq!(state.registers[Registers::R7], 0x000F);
        assert_eq!(state.registers[Registers::Flags], Flags::Pos as u16);
    }

    #[test]
    fn and_test_mode_1() {
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::InstRet as usize],
            running: true,
        };
        state.registers[Registers::R5] = 0xFFFF;
        let _ = and(0x5F66, &mut state);
        assert_eq!(state.registers[Registers::R7], 0x0006);
        assert_eq!(state.registers[Registers::Flags], Flags::Pos as u16);
        let _ = and(0x5F76, &mut state);
        assert_eq!(state.registers[Registers::R7], 0xFFF6);
        assert_eq!(state.registers[Registers::Flags], Flags::Neg as u16);
    }

    #[test]
    fn conditional_branch_test() {
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::InstRet as usize],
            running: true,
        };
        state.registers[Registers::Flags] = Flags::Neg as u16; // Flag Neg = 1
        conditional_branch(0x805, &mut state); // Test Flag Neg
        conditional_branch(0x405, &mut state); // Test Flag Zero
        conditional_branch(0x205, &mut state); // Test Flag Pos
        assert_eq!(state.registers[Registers::Pc], 5);
        state.registers[Registers::Flags] = Flags::Zro as u16; // Flag Zro = 1
        conditional_branch(0x805, &mut state); // Test Flag Neg
        conditional_branch(0x405, &mut state); // Test Flag Zero
        conditional_branch(0x205, &mut state); // Test Flag Pos
        assert_eq!(state.registers[Registers::Pc], 10);
        state.registers[Registers::Flags] = Flags::Pos as u16; // Flag Pos = 1
        conditional_branch(0x805, &mut state); // Test Flag Neg
        conditional_branch(0x405, &mut state); // Test Flag Zero
        conditional_branch(0x205, &mut state); // Test Flag Pos
        assert_eq!(state.registers[Registers::Pc], 15);
        conditional_branch(0xFFB, &mut state); // Add -5 if any of the flags is active
        assert_eq!(state.registers[Registers::Pc], 10);
    }

    #[test]
    fn jump_test() {
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::InstRet as usize],
            running: true,
        };
        state.registers[Registers::R5] = 25;
        let _ = jump(0xC140, &mut state);
        assert_eq!(state.registers[Registers::Pc], 25);
    }

    #[test]
    fn jump_to_subrutine_test() {
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::InstRet as usize],
            running: true,
        };
        state.registers[Registers::Pc] = 15;
        let _ = jump_to_subrutine(0x4FFB, &mut state);
        assert_eq!(state.registers[Registers::Pc], 10);
        assert_eq!(state.registers[Registers::R7], 15);
        state.registers[Registers::R5] = 50;
        let _ = jump_to_subrutine(0x4140, &mut state);
        assert_eq!(state.registers[Registers::R7], 10);
        assert_eq!(state.registers[Registers::Pc], 50);
    }

    #[test]
    fn load_test() {
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::InstRet as usize],
            running: true,
        };
        state.memory[50] = 70;
        let _ = load(0x2E32, &mut state);
        assert_eq!(state.registers[Registers::R7], 70);
        assert_eq!(state.registers[Registers::Flags], Flags::Pos as u16);
    }

    #[test]
    fn load_register_test() {
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::InstRet as usize],
            running: true,
        };
        state.memory[50] = 78;
        state.registers[Registers::R2] = 25;
        let _ = load_register(0x6A99, &mut state);
        assert_eq!(state.registers[Registers::R5], 78);
        assert_eq!(state.registers[Registers::Flags], Flags::Pos as u16);
    }

    #[test]
    fn load_effective_address_test() {
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::InstRet as usize],
            running: true,
        };
        state.registers[Registers::Pc] = 15;
        let _ = load_effective_address(0xE21F, &mut state);
        assert_eq!(state.registers[Registers::R1], 46);
    }

    #[test]
    fn not_test() {
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::InstRet as usize],
            running: true,
        };
        state.registers[Registers::R5] = 0x00FF;
        let _ = not(0x977F, &mut state);
        assert_eq!(state.registers[Registers::R3], 0xFF00);
        assert_eq!(state.registers[Registers::Flags], Flags::Neg as u16);
        let _ = not(0x96FF, &mut state);
        assert_eq!(state.registers[Registers::R3], 0xFF);
        assert_eq!(state.registers[Registers::Flags], Flags::Pos as u16);
    }

    #[test]
    fn store_test() {
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::InstRet as usize],
            running: true,
        };
        state.registers[Registers::R4] = 777;
        let _ = store(0x3819, &mut state);
        assert_eq!(state.memory[25], 777);
    }

    #[test]
    fn store_indirect_test() {
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::InstRet as usize],
            running: true,
        };
        state.memory[25] = 50;
        state.registers[Registers::R4] = 777;
        let _ = store_indirect(0x3819, &mut state);
        assert_eq!(state.memory[50], 777);
    }

    #[test]
    fn store_register_test() {
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::InstRet as usize],
            running: true,
        };
        state.registers[Registers::R4] = 20;
        state.registers[Registers::R5] = 50;
        let _ = store_register(0x7B3B, &mut state);
        assert_eq!(state.memory[15], 50);
    }
}
