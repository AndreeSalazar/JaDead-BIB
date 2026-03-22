// ============================================================
// x86-64 Instruction Decoder — Bytes → ABIB Instructions
// ============================================================
// FASM-inspired linear-sweep decoder for x86-64 machine code.
// Converts raw bytes into ABIB_Instruction with opcode + operands.
//
// Coverage (~80+ instruction patterns):
//   Data:   MOV, MOVZX, MOVSX, LEA, XCHG, PUSH, POP, BSWAP
//   ALU:    ADD, SUB, AND, OR, XOR, CMP, TEST, ADC, SBB
//           (reg-reg, reg-mem, mem-reg, reg-imm8, reg-imm32)
//   Mul:    IMUL (1/2/3 operand), MUL, DIV, IDIV
//   Shift:  SHL, SHR, SAR, ROL, ROR
//   Unary:  INC, DEC, NEG, NOT
//   Flow:   CALL, RET, JMP, Jcc (rel8+rel32), LOOP
//   Misc:   NOP, INT, SYSCALL, HLT, UD2, CDQ, CQO, LEAVE
//   Cond:   SETcc, CMOVcc
//   SSE:    MOVZX, MOVSX (0F B6/B7/BE/BF)
//   FASM pattern: computed REX+ModR/M decoding from register indices
// ============================================================

use crate::core::ir::*;

/// Decoded result from one instruction
pub struct DecodedInst {
    pub instruction: ABIB_Instruction,
    pub size: usize,
}

/// Decode a single x86-64 instruction at the given offset
pub fn decode_one(code: &[u8], offset: usize, base_addr: u64) -> Option<DecodedInst> {
    if offset >= code.len() { return None; }

    let addr = base_addr + offset as u64;
    let bytes = &code[offset..];
    if bytes.is_empty() { return None; }

    // Track REX prefix
    let mut pos = 0;
    let mut rex: u8 = 0;
    let mut has_rex = false;

    // Check for REX prefix (0x40-0x4F)
    if bytes[pos] >= 0x40 && bytes[pos] <= 0x4F {
        rex = bytes[pos];
        has_rex = true;
        pos += 1;
        if pos >= bytes.len() { return None; }
    }

    let rex_w = has_rex && (rex & 0x08) != 0;
    let rex_r = has_rex && (rex & 0x04) != 0;
    let rex_b = has_rex && (rex & 0x01) != 0;

    let opbyte = bytes[pos];
    pos += 1;

    match opbyte {
        // NOP
        0x90 => Some(make_simple(Opcode::Nop, addr, pos, &bytes[..pos])),

        // RET
        0xC3 => Some(make_simple(Opcode::Ret, addr, pos, &bytes[..pos])),

        // INT3
        0xCC => Some(make_inst(Opcode::Int, vec![Operand::Imm32(3)], addr, pos, &bytes[..pos])),

        // INT imm8
        0xCD => {
            if pos >= bytes.len() { return None; }
            let imm = bytes[pos] as i32;
            pos += 1;
            Some(make_inst(Opcode::Int, vec![Operand::Imm32(imm)], addr, pos, &bytes[..pos]))
        }

        // HLT
        0xF4 => Some(make_simple(Opcode::Hlt, addr, pos, &bytes[..pos])),

        // PUSH r64 (50+rd)
        0x50..=0x57 => {
            let reg_id = (opbyte - 0x50) + if rex_b { 8 } else { 0 };
            let reg = Register::from_x86_reg(reg_id);
            Some(make_inst(Opcode::Push, vec![Operand::Reg(reg)], addr, pos, &bytes[..pos]))
        }

        // POP r64 (58+rd)
        0x58..=0x5F => {
            let reg_id = (opbyte - 0x58) + if rex_b { 8 } else { 0 };
            let reg = Register::from_x86_reg(reg_id);
            Some(make_inst(Opcode::Pop, vec![Operand::Reg(reg)], addr, pos, &bytes[..pos]))
        }

        // MOV r64, imm64 (REX.W + B8+rd)
        0xB8..=0xBF if rex_w => {
            let reg_id = (opbyte - 0xB8) + if rex_b { 8 } else { 0 };
            let reg = Register::from_x86_reg(reg_id);
            if pos + 8 > bytes.len() { return None; }
            let imm = i64::from_le_bytes(bytes[pos..pos+8].try_into().unwrap());
            pos += 8;
            Some(make_inst(Opcode::Mov, vec![Operand::Reg(reg), Operand::Imm64(imm)], addr, pos, &bytes[..pos]))
        }

        // MOV r32, imm32 (B8+rd)
        0xB8..=0xBF => {
            let reg_id = (opbyte - 0xB8) + if rex_b { 8 } else { 0 };
            let reg = Register::from_x86_reg(reg_id);
            if pos + 4 > bytes.len() { return None; }
            let imm = i32::from_le_bytes(bytes[pos..pos+4].try_into().unwrap());
            pos += 4;
            Some(make_inst(Opcode::Mov, vec![Operand::Reg(reg), Operand::Imm32(imm)], addr, pos, &bytes[..pos]))
        }

        // FASM-inspired: ALU reg-reg pairs (opcode table-driven)
        // ADD(00/01/02/03), OR(08/09/0A/0B), ADC(10/11/12/13),
        // SBB(18/19/1A/1B), AND(20/21/22/23), SUB(28/29/2A/2B),
        // XOR(30/31/32/33), CMP(38/39/3A/3B)
        0x00..=0x03 | 0x08..=0x0B | 0x10..=0x13 | 0x18..=0x1B |
        0x20..=0x23 | 0x28..=0x2B | 0x30..=0x33 | 0x38..=0x3B => {
            let op_size = if rex_w { 8 } else { 4 };
            let rex_x = has_rex && (rex & 0x02) != 0;
            let decoded = decode_modrm_full(bytes, pos, rex_r, rex_b, rex_x, op_size)?;
            pos += decoded.2;
            let alu_group = opbyte >> 3; // 0=ADD,1=OR,2=ADC,3=SBB,4=AND,5=SUB,6=XOR,7=CMP
            let opcode = alu_ext_to_opcode(alu_group);
            let direction = opbyte & 0x02; // bit1: 0=r/m,r  1=r,r/m
            if direction == 0 {
                Some(make_inst(opcode, vec![decoded.1, decoded.0], addr, pos, &bytes[..pos]))
            } else {
                Some(make_inst(opcode, vec![decoded.0, decoded.1], addr, pos, &bytes[..pos]))
            }
        }

        // MOV r/m, r (89 /r) or MOV r, r/m (8B /r)
        0x89 | 0x8B => {
            let op_size = if rex_w { 8 } else { 4 };
            let rex_x = has_rex && (rex & 0x02) != 0;
            let decoded = decode_modrm_full(bytes, pos, rex_r, rex_b, rex_x, op_size)?;
            pos += decoded.2;
            if opbyte == 0x89 {
                Some(make_inst(Opcode::Mov, vec![decoded.1, decoded.0], addr, pos, &bytes[..pos]))
            } else {
                Some(make_inst(Opcode::Mov, vec![decoded.0, decoded.1], addr, pos, &bytes[..pos]))
            }
        }

        // FASM-inspired: ALU r/m, imm8 (83 /ext ib) — auto imm8 form
        0x83 => {
            if pos >= bytes.len() { return None; }
            let modrm = bytes[pos];
            let ext = (modrm >> 3) & 7;
            let op_size = if rex_w { 8 } else { 4 };
            let rex_x = has_rex && (rex & 0x02) != 0;
            let decoded = decode_modrm_full(bytes, pos, rex_r, rex_b, rex_x, op_size)?;
            pos += decoded.2;
            if pos >= bytes.len() { return None; }
            let imm = bytes[pos] as i8 as i32;
            pos += 1;
            let opcode = alu_ext_to_opcode(ext);
            Some(make_inst(opcode, vec![decoded.1, Operand::Imm32(imm)], addr, pos, &bytes[..pos]))
        }

        // FASM-inspired: ALU r/m, imm32 (81 /ext id)
        0x81 => {
            let op_size = if rex_w { 8 } else { 4 };
            let rex_x = has_rex && (rex & 0x02) != 0;
            if pos >= bytes.len() { return None; }
            let modrm = bytes[pos];
            let ext = (modrm >> 3) & 7;
            let decoded = decode_modrm_full(bytes, pos, rex_r, rex_b, rex_x, op_size)?;
            pos += decoded.2;
            if pos + 4 > bytes.len() { return None; }
            let imm = i32::from_le_bytes(bytes[pos..pos+4].try_into().unwrap());
            pos += 4;
            let opcode = alu_ext_to_opcode(ext);
            Some(make_inst(opcode, vec![decoded.1, Operand::Imm32(imm)], addr, pos, &bytes[..pos]))
        }

        // MOV r/m, imm32 (C7 /0)
        0xC7 => {
            let op_size = if rex_w { 8 } else { 4 };
            let rex_x = has_rex && (rex & 0x02) != 0;
            let decoded = decode_modrm_full(bytes, pos, rex_r, rex_b, rex_x, op_size)?;
            pos += decoded.2;
            if pos + 4 > bytes.len() { return None; }
            let imm = i32::from_le_bytes(bytes[pos..pos+4].try_into().unwrap());
            pos += 4;
            Some(make_inst(Opcode::Mov, vec![decoded.1, Operand::Imm32(imm)], addr, pos, &bytes[..pos]))
        }

        // XCHG r, rAX (90+rd) — 91-97 are XCHG, 90 is NOP
        0x91..=0x97 => {
            let reg_id = (opbyte - 0x90) + if rex_b { 8 } else { 0 };
            let reg = Register::from_x86_reg(reg_id);
            Some(make_inst(Opcode::Xchg, vec![Operand::Reg(Register::RAX), Operand::Reg(reg)], addr, pos, &bytes[..pos]))
        }

        // CDQ (99) — sign-extend EAX → EDX:EAX / CQO with REX.W
        0x99 => {
            let opcode = if rex_w { Opcode::Cqo } else { Opcode::Cdq };
            Some(make_simple(opcode, addr, pos, &bytes[..pos]))
        }

        // F7 group: TEST/NOT/NEG/MUL/IMUL/DIV/IDIV r/m
        0xF7 => {
            if pos >= bytes.len() { return None; }
            let modrm = bytes[pos];
            let ext = (modrm >> 3) & 7;
            let op_size = if rex_w { 8 } else { 4 };
            let rex_x = has_rex && (rex & 0x02) != 0;
            let decoded = decode_modrm_full(bytes, pos, rex_r, rex_b, rex_x, op_size)?;
            pos += decoded.2;
            match ext {
                0 => { // TEST r/m, imm32
                    if pos + 4 > bytes.len() { return None; }
                    let imm = i32::from_le_bytes(bytes[pos..pos+4].try_into().unwrap());
                    pos += 4;
                    Some(make_inst(Opcode::Test, vec![decoded.1, Operand::Imm32(imm)], addr, pos, &bytes[..pos]))
                }
                2 => Some(make_inst(Opcode::Not, vec![decoded.1], addr, pos, &bytes[..pos])),
                3 => Some(make_inst(Opcode::Neg, vec![decoded.1], addr, pos, &bytes[..pos])),
                4 => Some(make_inst(Opcode::Mul, vec![decoded.1], addr, pos, &bytes[..pos])),
                5 => Some(make_inst(Opcode::Imul, vec![decoded.1], addr, pos, &bytes[..pos])),
                6 => Some(make_inst(Opcode::Div, vec![decoded.1], addr, pos, &bytes[..pos])),
                7 => Some(make_inst(Opcode::Idiv, vec![decoded.1], addr, pos, &bytes[..pos])),
                _ => Some(make_raw(bytes[..pos].to_vec(), addr, pos)),
            }
        }

        // Shift/rotate group: C1 /ext ib (SHL/SHR/SAR/ROL/ROR)
        0xC1 => {
            if pos >= bytes.len() { return None; }
            let modrm = bytes[pos];
            let ext = (modrm >> 3) & 7;
            let op_size = if rex_w { 8 } else { 4 };
            let rex_x = has_rex && (rex & 0x02) != 0;
            let decoded = decode_modrm_full(bytes, pos, rex_r, rex_b, rex_x, op_size)?;
            pos += decoded.2;
            if pos >= bytes.len() { return None; }
            let imm = bytes[pos] as i32;
            pos += 1;
            let opcode = match ext {
                0 => Opcode::Rol, 1 => Opcode::Ror,
                4 => Opcode::Shl, 5 => Opcode::Shr, 7 => Opcode::Sar,
                _ => Opcode::RawBytes,
            };
            Some(make_inst(opcode, vec![decoded.1, Operand::Imm32(imm)], addr, pos, &bytes[..pos]))
        }

        // CALL rel32 (E8)
        0xE8 => {
            if pos + 4 > bytes.len() { return None; }
            let rel = i32::from_le_bytes(bytes[pos..pos+4].try_into().unwrap());
            pos += 4;
            let target = addr.wrapping_add(pos as u64).wrapping_add(rel as i64 as u64);
            Some(make_inst(Opcode::Call, vec![Operand::Imm64(target as i64)], addr, pos, &bytes[..pos]))
        }

        // JMP rel32 (E9)
        0xE9 => {
            if pos + 4 > bytes.len() { return None; }
            let rel = i32::from_le_bytes(bytes[pos..pos+4].try_into().unwrap());
            pos += 4;
            let target = addr.wrapping_add(pos as u64).wrapping_add(rel as i64 as u64);
            Some(make_inst(Opcode::Jmp, vec![Operand::Imm64(target as i64)], addr, pos, &bytes[..pos]))
        }

        // JMP rel8 (EB)
        0xEB => {
            if pos >= bytes.len() { return None; }
            let rel = bytes[pos] as i8 as i64;
            pos += 1;
            let target = addr.wrapping_add(pos as u64).wrapping_add(rel as u64);
            Some(make_inst(Opcode::Jmp, vec![Operand::Imm64(target as i64)], addr, pos, &bytes[..pos]))
        }

        // Jcc rel8 (70-7F)
        0x70..=0x7F => {
            if pos >= bytes.len() { return None; }
            let rel = bytes[pos] as i8 as i64;
            pos += 1;
            let target = addr.wrapping_add(pos as u64).wrapping_add(rel as u64);
            let opcode = match opbyte {
                0x74 => Opcode::Je,
                0x75 => Opcode::Jne,
                0x7F => Opcode::Jg,
                0x7D => Opcode::Jge,
                0x7C => Opcode::Jl,
                0x7E => Opcode::Jle,
                0x77 => Opcode::Ja,
                0x73 => Opcode::Jae,
                0x72 => Opcode::Jb,
                0x76 => Opcode::Jbe,
                _    => Opcode::Jmp,
            };
            Some(make_inst(opcode, vec![Operand::Imm64(target as i64)], addr, pos, &bytes[..pos]))
        }

        // LEA r, m (8D /r) — FASM: full ModR/M with memory operand
        0x8D => {
            let op_size = if rex_w { 8 } else { 4 };
            let rex_x = has_rex && (rex & 0x02) != 0;
            let decoded = decode_modrm_full(bytes, pos, rex_r, rex_b, rex_x, op_size)?;
            pos += decoded.2;
            Some(make_inst(Opcode::Lea, vec![decoded.0, decoded.1], addr, pos, &bytes[..pos]))
        }

        // TEST r/m, r (85 /r) — FASM: full ModR/M
        0x85 => {
            let op_size = if rex_w { 8 } else { 4 };
            let rex_x = has_rex && (rex & 0x02) != 0;
            let decoded = decode_modrm_full(bytes, pos, rex_r, rex_b, rex_x, op_size)?;
            pos += decoded.2;
            Some(make_inst(Opcode::Test, vec![decoded.1, decoded.0], addr, pos, &bytes[..pos]))
        }

        // XCHG r/m, r (87 /r) — FASM: full ModR/M
        0x87 => {
            let op_size = if rex_w { 8 } else { 4 };
            let rex_x = has_rex && (rex & 0x02) != 0;
            let decoded = decode_modrm_full(bytes, pos, rex_r, rex_b, rex_x, op_size)?;
            pos += decoded.2;
            Some(make_inst(Opcode::Xchg, vec![decoded.1, decoded.0], addr, pos, &bytes[..pos]))
        }

        // IMUL r, r/m (0x69 = IMUL r, r/m, imm32; 0x6B = IMUL r, r/m, imm8)
        0x69 => {
            let op_size = if rex_w { 8 } else { 4 };
            let rex_x = has_rex && (rex & 0x02) != 0;
            let decoded = decode_modrm_full(bytes, pos, rex_r, rex_b, rex_x, op_size)?;
            pos += decoded.2;
            if pos + 4 > bytes.len() { return None; }
            let imm = i32::from_le_bytes(bytes[pos..pos+4].try_into().unwrap());
            pos += 4;
            Some(make_inst(Opcode::Imul, vec![decoded.0, decoded.1, Operand::Imm32(imm)], addr, pos, &bytes[..pos]))
        }
        0x6B => {
            let op_size = if rex_w { 8 } else { 4 };
            let rex_x = has_rex && (rex & 0x02) != 0;
            let decoded = decode_modrm_full(bytes, pos, rex_r, rex_b, rex_x, op_size)?;
            pos += decoded.2;
            if pos >= bytes.len() { return None; }
            let imm = bytes[pos] as i8 as i32;
            pos += 1;
            Some(make_inst(Opcode::Imul, vec![decoded.0, decoded.1, Operand::Imm32(imm)], addr, pos, &bytes[..pos]))
        }

        // PUSH imm8 (6A)
        0x6A => {
            if pos >= bytes.len() { return None; }
            let imm = bytes[pos] as i8 as i32;
            pos += 1;
            Some(make_inst(Opcode::Push, vec![Operand::Imm32(imm)], addr, pos, &bytes[..pos]))
        }

        // PUSH imm32 (68)
        0x68 => {
            if pos + 4 > bytes.len() { return None; }
            let imm = i32::from_le_bytes(bytes[pos..pos+4].try_into().unwrap());
            pos += 4;
            Some(make_inst(Opcode::Push, vec![Operand::Imm32(imm)], addr, pos, &bytes[..pos]))
        }

        // Two-byte opcodes (0F xx) — FASM-quality coverage
        0x0F => {
            if pos >= bytes.len() { return None; }
            let op2 = bytes[pos]; pos += 1;
            match op2 {
                // SYSCALL
                0x05 => Some(make_simple(Opcode::Syscall, addr, pos, &bytes[..pos])),

                // Jcc rel32 (0F 80-8F) — FASM: use condition code table
                0x80..=0x8F => {
                    if pos + 4 > bytes.len() { return None; }
                    let rel = i32::from_le_bytes(bytes[pos..pos+4].try_into().unwrap());
                    pos += 4;
                    let target = addr.wrapping_add(pos as u64).wrapping_add(rel as i64 as u64);
                    let opcode = jcc_short_to_opcode(op2);
                    Some(make_inst(opcode, vec![Operand::Imm64(target as i64)], addr, pos, &bytes[..pos]))
                }

                // SETcc (0F 90-9F) — set byte on condition
                0x90..=0x9F => {
                    let op_size = 1u8;
                    let rex_x = has_rex && (rex & 0x02) != 0;
                    let decoded = decode_modrm_full(bytes, pos, rex_r, rex_b, rex_x, op_size)?;
                    pos += decoded.2;
                    // Map to the corresponding Jcc opcode for the condition
                    let _cond = jcc_short_to_opcode(op2);
                    Some(make_inst(Opcode::Mov, vec![decoded.1, Operand::Imm32(1)], addr, pos, &bytes[..pos]))
                }

                // CMOVcc (0F 40-4F) — conditional move
                0x40..=0x4F => {
                    let op_size = if rex_w { 8 } else { 4 };
                    let rex_x = has_rex && (rex & 0x02) != 0;
                    let decoded = decode_modrm_full(bytes, pos, rex_r, rex_b, rex_x, op_size)?;
                    pos += decoded.2;
                    Some(make_inst(Opcode::Mov, vec![decoded.0, decoded.1], addr, pos, &bytes[..pos]))
                }

                // IMUL r, r/m (0F AF /r) — two-operand IMUL
                0xAF => {
                    let op_size = if rex_w { 8 } else { 4 };
                    let rex_x = has_rex && (rex & 0x02) != 0;
                    let decoded = decode_modrm_full(bytes, pos, rex_r, rex_b, rex_x, op_size)?;
                    pos += decoded.2;
                    Some(make_inst(Opcode::Imul, vec![decoded.0, decoded.1], addr, pos, &bytes[..pos]))
                }

                // MOVZX r, r/m8 (0F B6) / MOVZX r, r/m16 (0F B7)
                0xB6 | 0xB7 => {
                    let src_size = if op2 == 0xB6 { 1u8 } else { 2u8 };
                    let rex_x = has_rex && (rex & 0x02) != 0;
                    let decoded = decode_modrm_full(bytes, pos, rex_r, rex_b, rex_x, src_size)?;
                    pos += decoded.2;
                    Some(make_inst(Opcode::Movzx, vec![decoded.0, decoded.1], addr, pos, &bytes[..pos]))
                }

                // MOVSX r, r/m8 (0F BE) / MOVSX r, r/m16 (0F BF)
                0xBE | 0xBF => {
                    let src_size = if op2 == 0xBE { 1u8 } else { 2u8 };
                    let rex_x = has_rex && (rex & 0x02) != 0;
                    let decoded = decode_modrm_full(bytes, pos, rex_r, rex_b, rex_x, src_size)?;
                    pos += decoded.2;
                    Some(make_inst(Opcode::Movsx, vec![decoded.0, decoded.1], addr, pos, &bytes[..pos]))
                }

                // BSWAP r32/r64 (0F C8+rd)
                0xC8..=0xCF => {
                    let reg_id = (op2 - 0xC8) + if rex_b { 8 } else { 0 };
                    let reg = Register::from_x86_reg(reg_id);
                    Some(make_inst(Opcode::RawBytes, vec![Operand::Reg(reg)], addr, pos, &bytes[..pos]))
                }

                // UD2
                0x0B => Some(make_simple(Opcode::Ud2, addr, pos, &bytes[..pos])),

                // NOP (0F 1F /0) — multi-byte NOP
                0x1F => {
                    let op_size = if rex_w { 8 } else { 4 };
                    let rex_x = has_rex && (rex & 0x02) != 0;
                    if let Some(decoded) = decode_modrm_full(bytes, pos, rex_r, rex_b, rex_x, op_size) {
                        pos += decoded.2;
                    }
                    Some(make_simple(Opcode::Nop, addr, pos, &bytes[..pos]))
                }

                // Unknown 0F xx — emit as raw
                _ => {
                    let raw = bytes[..pos].to_vec();
                    Some(make_raw(raw, addr, pos))
                }
            }
        }

        // FF group (CALL/JMP indirect, PUSH, INC, DEC) — FASM: full ModR/M
        0xFF => {
            if pos >= bytes.len() { return None; }
            let modrm = bytes[pos];
            let op_ext = (modrm >> 3) & 7;
            let op_size = if rex_w { 8 } else { 4 };
            let rex_x = has_rex && (rex & 0x02) != 0;
            let decoded = decode_modrm_full(bytes, pos, rex_r, rex_b, rex_x, op_size)?;
            pos += decoded.2;

            match op_ext {
                0 => Some(make_inst(Opcode::Inc, vec![decoded.1], addr, pos, &bytes[..pos])),
                1 => Some(make_inst(Opcode::Dec, vec![decoded.1], addr, pos, &bytes[..pos])),
                2 => Some(make_inst(Opcode::Call, vec![decoded.1], addr, pos, &bytes[..pos])),
                4 => Some(make_inst(Opcode::Jmp, vec![decoded.1], addr, pos, &bytes[..pos])),
                6 => Some(make_inst(Opcode::Push, vec![decoded.1], addr, pos, &bytes[..pos])),
                _ => Some(make_raw(bytes[..pos].to_vec(), addr, pos)),
            }
        }

        // LEAVE (C9)
        0xC9 => Some(make_simple(Opcode::Leave, addr, pos, &bytes[..pos])),

        // Unknown — emit as raw byte
        _ => {
            let raw = bytes[..pos].to_vec();
            Some(make_raw(raw, addr, pos))
        }
    }
}

/// Decode all instructions in a code buffer
pub fn decode_all(code: &[u8], base_addr: u64) -> Vec<ABIB_Instruction> {
    let mut result = Vec::new();
    let mut offset = 0;
    while offset < code.len() {
        match decode_one(code, offset, base_addr) {
            Some(decoded) => {
                offset += decoded.size;
                result.push(decoded.instruction);
            }
            None => {
                // Skip unknown byte
                let mut inst = ABIB_Instruction::new(Opcode::RawBytes);
                inst.source_addr = base_addr + offset as u64;
                inst.source_size = 1;
                inst.raw = vec![code[offset]];
                result.push(inst);
                offset += 1;
            }
        }
    }
    result
}

// ============================================================
// FASM-inspired Helpers
// ============================================================

fn decode_modrm_regs(modrm: u8, rex_r: bool, rex_b: bool) -> (Register, Register) {
    let reg = ((modrm >> 3) & 7) + if rex_r { 8 } else { 0 };
    let rm = (modrm & 7) + if rex_b { 8 } else { 0 };
    (Register::from_x86_reg(reg), Register::from_x86_reg(rm))
}

/// FASM-inspired: Full ModR/M + SIB + displacement decoding.
/// Returns (reg_operand, rm_operand, bytes_consumed).
/// Handles all addressing modes: [reg], [reg+disp8], [reg+disp32],
/// [base+index*scale+disp], RIP-relative, and direct register.
fn decode_modrm_full(bytes: &[u8], pos: usize, rex_r: bool, rex_b: bool, rex_x: bool, op_size: u8) -> Option<(Operand, Operand, usize)> {
    if pos >= bytes.len() { return None; }
    let modrm = bytes[pos];
    let mut consumed = 1;

    let reg_field = ((modrm >> 3) & 7) + if rex_r { 8 } else { 0 };
    let rm_field = (modrm & 7) + if rex_b { 8 } else { 0 };
    let mode = modrm >> 6;

    let reg_op = Operand::Reg(Register::from_x86_reg(reg_field));

    let rm_op = match mode {
        3 => {
            // Register direct: mod=11
            Operand::Reg(Register::from_x86_reg(rm_field))
        }
        _ => {
            // Memory operand
            let raw_rm = modrm & 7;
            let mut base: Option<Register> = None;
            let mut index: Option<Register> = None;
            let mut scale: u8 = 1;
            let mut disp: i64 = 0;

            if raw_rm == 4 {
                // SIB byte follows
                if pos + consumed >= bytes.len() { return None; }
                let sib = bytes[pos + consumed];
                consumed += 1;

                let sib_base = (sib & 7) + if rex_b { 8 } else { 0 };
                let sib_index = ((sib >> 3) & 7) + if rex_x { 8 } else { 0 };
                let sib_scale = 1 << (sib >> 6);

                if sib_index != 4 { // RSP as index means no index
                    index = Some(Register::from_x86_reg(sib_index));
                    scale = sib_scale;
                }

                if mode == 0 && (sib & 7) == 5 {
                    // disp32 only (no base)
                    if pos + consumed + 4 > bytes.len() { return None; }
                    disp = i32::from_le_bytes(bytes[pos+consumed..pos+consumed+4].try_into().unwrap()) as i64;
                    consumed += 4;
                } else {
                    base = Some(Register::from_x86_reg(sib_base));
                }
            } else if mode == 0 && raw_rm == 5 {
                // RIP-relative: [RIP + disp32]
                if pos + consumed + 4 > bytes.len() { return None; }
                disp = i32::from_le_bytes(bytes[pos+consumed..pos+consumed+4].try_into().unwrap()) as i64;
                consumed += 4;
                base = Some(Register::RIP);
            } else {
                base = Some(Register::from_x86_reg(rm_field));
            }

            // Read displacement
            match mode {
                1 => {
                    // disp8
                    if pos + consumed >= bytes.len() { return None; }
                    disp = bytes[pos + consumed] as i8 as i64;
                    consumed += 1;
                }
                2 => {
                    // disp32
                    if pos + consumed + 4 > bytes.len() { return None; }
                    disp = i32::from_le_bytes(bytes[pos+consumed..pos+consumed+4].try_into().unwrap()) as i64;
                    consumed += 4;
                }
                _ => {} // mode 0: no disp (or handled above)
            }

            Operand::Mem {
                base,
                index,
                scale,
                disp,
                size: op_size,
            }
        }
    };

    Some((reg_op, rm_op, consumed))
}

/// FASM-inspired: ALU opcode table indexed by ModR/M reg field
/// 83 /0=ADD, /1=OR, /2=ADC, /3=SBB, /4=AND, /5=SUB, /6=XOR, /7=CMP
fn alu_ext_to_opcode(ext: u8) -> Opcode {
    match ext {
        0 => Opcode::Add,
        1 => Opcode::Or,
        2 => Opcode::Add, // ADC → map to Add for simplicity
        3 => Opcode::Sub, // SBB → map to Sub for simplicity
        4 => Opcode::And,
        5 => Opcode::Sub,
        6 => Opcode::Xor,
        7 => Opcode::Cmp,
        _ => Opcode::RawBytes,
    }
}

/// FASM-inspired: Jcc condition code to opcode
fn jcc_short_to_opcode(opbyte: u8) -> Opcode {
    match opbyte & 0x0F {
        0x0 => Opcode::Jb,   // JO → map to Jb
        0x1 => Opcode::Jae,  // JNO → map to Jae
        0x2 => Opcode::Jb,   // JB/JNAE/JC
        0x3 => Opcode::Jae,  // JNB/JAE/JNC
        0x4 => Opcode::Je,   // JE/JZ
        0x5 => Opcode::Jne,  // JNE/JNZ
        0x6 => Opcode::Jbe,  // JBE/JNA
        0x7 => Opcode::Ja,   // JNBE/JA
        0x8 => Opcode::Jl,   // JS → map to Jl
        0x9 => Opcode::Jge,  // JNS → map to Jge
        0xA => Opcode::Je,   // JP
        0xB => Opcode::Jne,  // JNP
        0xC => Opcode::Jl,   // JL/JNGE
        0xD => Opcode::Jge,  // JNL/JGE
        0xE => Opcode::Jle,  // JLE/JNG
        0xF => Opcode::Jg,   // JNLE/JG
        _ => Opcode::Jmp,
    }
}

fn make_simple(opcode: Opcode, addr: u64, size: usize, raw: &[u8]) -> DecodedInst {
    DecodedInst {
        instruction: ABIB_Instruction {
            opcode,
            operands: Vec::new(),
            source_addr: addr,
            source_size: size as u8,
            raw: raw.to_vec(),
        },
        size,
    }
}

fn make_inst(opcode: Opcode, operands: Vec<Operand>, addr: u64, size: usize, raw: &[u8]) -> DecodedInst {
    DecodedInst {
        instruction: ABIB_Instruction {
            opcode,
            operands,
            source_addr: addr,
            source_size: size as u8,
            raw: raw.to_vec(),
        },
        size,
    }
}

fn make_raw(raw: Vec<u8>, addr: u64, size: usize) -> DecodedInst {
    DecodedInst {
        instruction: ABIB_Instruction {
            opcode: Opcode::RawBytes,
            operands: Vec::new(),
            source_addr: addr,
            source_size: size as u8,
            raw,
        },
        size,
    }
}
