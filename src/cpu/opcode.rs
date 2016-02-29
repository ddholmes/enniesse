enum_from_primitive! {
    #[derive(Debug)]
    pub enum Opcode {
        Jmp = 0x4c,
        Ldx = 0xa2,
        Stx = 0x86
    }
}