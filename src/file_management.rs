use std::{fs::File, io::Read, path::Path};

use crate::{Errors, State};
/// Given a file path open the file and write its instruction in little endian in the memory
pub(crate) fn read_file_to_memory(string_path: &String, state: &mut State) -> Result<(), Errors> {
    // Open file on that path
    let path = Path::new(string_path);
    let mut file = File::open(path)?;
    // Initialize a BufReader and a line iterator to read the file line by line
    let mut buffer = Vec::new();
    let read_amount = file.read_to_end(&mut buffer)?;
    let origin = u16::from_be_bytes([buffer[0], buffer[1]]) as usize;
    let max_memory = state.memory.len() - origin;
    let mut buffer_offset = 2;
    let mut memory_offset = 0;
    loop {
        if buffer_offset == read_amount - 1 {
            state.memory_write(
                origin + memory_offset,
                u16::from_be_bytes([buffer[buffer_offset], 0]),
            );
            break;
        }
        if memory_offset >= max_memory {
            return Err(Errors::BadImageSize);
        }
        if buffer_offset >= read_amount {
            break;
        }
        state.memory_write(
            origin + memory_offset,
            u16::from_be_bytes([buffer[buffer_offset], buffer[buffer_offset + 1]]),
        );
        memory_offset += 1;
        buffer_offset += 2;
    }
    Ok(())
}
