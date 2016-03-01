enum_from_primitive! {
    #[derive(Debug)]
    pub enum Opcode {
        Php = 0x08,
        Bpl = 0x10,
        Clc = 0x18,
        Jsr = 0x20,
        Bit = 0x24,
        Plp = 0x28,
        And = 0x29,
        Sec = 0x38,
        Pha = 0x48,
        Jmp = 0x4c,
        Bvc = 0x50,
        Rts = 0x60,
        Pla = 0x68,
        Bvs = 0x70,
        Sei = 0x78,
        Sta = 0x85,
        Stx = 0x86,
        Bcc = 0x90,
        Ldx = 0xa2,
        Lda = 0xa9,
        Bcs = 0xb0,
        Cmp = 0xc9,
        Bne = 0xd0,
        Cld = 0xd8,
        Nop = 0xea,
        Beq = 0xf0,
        Sed = 0xf8
    }
}