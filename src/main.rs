enum Registers{
    RR0,
    RR1,
    RR2,
    RR3,
    RR4,
    RR5,
    RR6,
    RR7,
    RPC,
    RCOND,
    RCOUNT
}

fn main() {
    // Initialize empty memory
    let mut memory = [0_u16;2_usize.pow(16)];
    memory[0] = Registers::RPC as u16;
    print!("{:?}",memory[0]);
}

#[cfg(test)]
mod test {
    use crate::main;

    #[test]
    fn test() {
        main();
    }
}
