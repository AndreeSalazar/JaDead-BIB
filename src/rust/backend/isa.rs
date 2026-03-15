// ============================================================
// Instruction Set Architecture (ISA) for JaDead-BIB 💀☕
// ============================================================
// Translates ADeadOp SSA IR directly into x86-64 Machine Code.
// This bypasses bytecode, Javac, and JVM entirely.
// ============================================================

use crate::middle::ir::*;

pub struct ISATranslator {
    // Machine block bytes
    code: Vec<u8>,
}

impl ISATranslator {
    pub fn new() -> Self {
        Self { code: Vec::new() }
    }

    /// Compile ADeadOp IR to raw x86-64 opcode bytes
    pub fn translate(&mut self, _module: &IRModule) -> Result<Vec<u8>, String> {
        self.code.clear();
        
        // 1. Prologue Setup (push rbp, mov rbp, rsp)
        self.code.extend_from_slice(&[0x55, 0x48, 0x89, 0xE5]);
        
        // 2. Here we map IRInstruction to actual MOV, ADD, SUB, CALL
        // Note: Full robust implementation uses register allocator inherited from PyDead-BIB/ADead-BIB.
        
        #[allow(unused_variables, unused_mut)]
        let mut gc_scopes_opened = 0;
        #[allow(unused_variables, unused_mut)]
        let mut gc_pools_allocated = 0;

        // Traverse IR to map explicit instructions
        for func in &_module.functions {
             for instr in &func.body {
                 if let IRInstruction::PrintStr(s) = instr {
                     // Leak string temporarily to get absolute memory pointer
                     let boxed_str = Box::leak(s.clone().into_boxed_str());
                     let ptr = boxed_str.as_ptr() as u64;
                     let len = boxed_str.len() as u64;
                     let fn_ptr = crate::backend::jit::jdb_print_str as *const () as u64;

                     // mov rcx, ptr
                     self.code.push(0x48); self.code.push(0xB9);
                     self.code.extend_from_slice(&ptr.to_le_bytes());
                     // mov rdx, len
                     self.code.push(0x48); self.code.push(0xBA);
                     self.code.extend_from_slice(&len.to_le_bytes());
                     // mov rax, fn_ptr
                     self.code.push(0x48); self.code.push(0xB8);
                     self.code.extend_from_slice(&fn_ptr.to_le_bytes());
                     
                     // shadow space for Windows x64 ABI (sub rsp, 32)
                     self.code.extend_from_slice(&[0x48, 0x83, 0xEC, 0x20]);
                     // call rax
                     self.code.extend_from_slice(&[0xFF, 0xD0]);
                     // add rsp, 32
                     self.code.extend_from_slice(&[0x48, 0x83, 0xC4, 0x20]);
                 }
             }
        }

        self.code.extend_from_slice(&[
            0xB8, 0x0A, 0x00, 0x00, 0x00, // mov eax, 10
            0xC9,                         // leave
            0xC3                          // ret
        ]); // Stub execution for basic runs

        Ok(self.code.clone())
    }
}
