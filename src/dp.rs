#[cfg(test)]
mod tests {
    use crate::decoder::{decode, Op};

    #[test]
    fn test_mov_imm() {
        // MOV Rd, #imm: 00100_Rd_iiiiiiii
        let inst = decode(0b00100_010_11111111, None);
        assert_eq!(inst.op, Op::MOV);
        assert_eq!(inst.rd, 2);
        assert_eq!(inst.imm, 255);
    }

    #[test]
    fn test_add_imm() {
        // ADD Rd, #imm: 00110_Rd_iiiiiiii
        let inst = decode(0b00110_001_00000101, None);
        assert_eq!(inst.op, Op::ADD);
        assert_eq!(inst.rd, 1);
        assert_eq!(inst.imm, 5);
    }

    #[test]
    fn test_sub_imm() {
        // SUB Rd, #imm: 00111_Rd_iiiiiiii
        let inst = decode(0b00111_000_00000011, None);
        assert_eq!(inst.op, Op::SUB);
        assert_eq!(inst.rd, 0);
        assert_eq!(inst.imm, 3);
    }

    #[test]
    fn test_b_unconditional() {
        // B imm11: 11100_iiiiiiiiiii
        let inst = decode(0b11100_00001010101, None);
        assert_eq!(inst.op, Op::B);
        assert_eq!(inst.imm, 170); // 85 * 2
    }

    #[test]
    fn test_push() {
        // PUSH {reg_list, LR}: 10110_M_0_reglist (M=1 means LR)
        // Correct binary: bit 8 (M) = 1
        let inst = decode(0b10110_1_01_10101010, None);
        assert_eq!(inst.op, Op::PUSH);
        assert_eq!(inst.reg_list, 0b1_10101010); // 426
    }

    #[test]
    fn test_pop() {
        // POP {reg_list, PC}: 10111_P_0_reglist (P=1 means PC)
        // Correct binary: bit 8 (P) = 1
        let inst = decode(0b10111_1_01_01010101, None);
        assert_eq!(inst.op, Op::POP);
        assert_eq!(inst.reg_list, 0b1000_0000_01010101); // 32853
    }

    #[test]
    fn test_and_register() {
        // AND Rd, Rs: 010000_0000_Rs_Rd
        // Format: 0100_0000_00Rs_Rddd
        let inst = decode(0b0100_0000_0000_0000, None);
        assert_eq!(inst.op, Op::AND);
    }

    #[test]
    fn test_mul_register() {
        // MUL Rd, Rs: 010000_1101_Rs_Rd
        let inst = decode(0b0100_0000_1101_0000, None);
        assert_eq!(inst.op, Op::MUL);
    }

    #[test]
    fn test_lsl_register() {
        // LSL Rd, Rs: 010000_0010_Rs_Rd
        let inst = decode(0b0100_0000_0010_0000, None);
        assert_eq!(inst.op, Op::LSL);
    }

    #[test]
    fn test_svc() {
        // SVC imm8: 1101_1111_iiiiiiii
        let inst = decode(0b1101_1111_11111111, None);
        assert_eq!(inst.op, Op::SVC);
        assert_eq!(inst.imm, 0xFF);
    }
}
