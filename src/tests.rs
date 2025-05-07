#[cfg(test)]
pub mod tests{
    use crate::*;
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
    fn store_indirect_test(){
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::Rcount as usize],
        };
        state.memory[25] = 50;
        state.registers[Registers::Rr4] = 777;
        store_indirect(0x3819, &mut state);
        assert_eq!(state.memory[50], 777);
    }
    
    #[test]
    fn store_register_test(){
        let mut state = State {
            memory: [0; MEM_MAX],
            registers: [0; Registers::Rcount as usize],
        };
        state.registers[Registers::Rr4] = 20;
        state.registers[Registers::Rr5] = 50;
        store_register(0x7B3B, &mut state);
        assert_eq!(state.memory[15],50);
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
        // Directly load a value from memory to register 2, PC = 20
        load(0x25F6, &mut state);
        assert_eq!(state.registers[Registers::Rr2], 50);
        // Load from register 2 with an offset of 15 to register 3, PC = 20
        load_register(0x668f, &mut state);
        assert_eq!(state.registers[Registers::Rr3], 777);
        // ADD 30 to the PC and save it in register 0, PC = 20
        load_effective_address(0xE21E, &mut state);
        assert_eq!(state.registers[Registers::Rr1], 50);
        // Doing a not in register 3, PC = 20
        not(0x96FF,&mut state);
        assert_eq!(state.registers[Registers::Rr3],0xFCF6);
        assert_eq!(state.registers[Registers::Rcond], Flags::Neg as u16);
        // Save the value from register 3 in memory with an offset of 25, PC = 20
        store(0x3619,&mut state);
        assert_eq!(state.memory[45],0xFCF6);
        // Indirect storage of the value of register 1, PC = 20
        store_indirect(0xB21E, &mut state);
        assert_eq!(state.memory[25689],50);
        // Store the value of Rr3 using store register from R1
        store_register(0x767B, &mut state);
        assert_eq!(state.memory[45],0xFCF6);
    }
    
}
