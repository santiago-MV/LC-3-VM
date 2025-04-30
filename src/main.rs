fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod test {
    use crate::main;

    #[test]
    fn test() {
        main();
    }
}
