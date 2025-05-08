use crate::State;
use crate::check_key;
pub enum MemoryMappedRegisters {
    MrKbsr = 0xFE00,
    MrKbdr = 0xFE02,
}

pub fn memory_write(adress: usize, value: u16, state: &mut State) {
    state.memory[adress] = value;
}

pub fn memory_read(address: usize, state: &mut State) -> u16 {
    if address == MemoryMappedRegisters::MrKbsr as usize {
        match check_key() {
            Ok(rv) => {
                state.memory[MemoryMappedRegisters::MrKbsr as usize] = 1 << 15;
                state.memory[MemoryMappedRegisters::MrKbdr as usize] = rv
            }
            Err(_) => state.memory[MemoryMappedRegisters::MrKbsr as usize] = 0,
        };
    }
    state.memory[address]
}
