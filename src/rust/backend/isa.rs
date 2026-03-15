// ============================================================
// Instruction Set Architecture (ISA) for JaDead-BIB 💀☕
// ============================================================
// Translates ADeadOp SSA IR directly into x86-64 Machine Code.
// This bypasses bytecode, Javac, and JVM entirely.
// ============================================================

use crate::middle::ir::*;
use std::collections::HashMap;

pub struct ISATranslator {
    code: Vec<u8>,
}

struct Patch {
    offset: usize,
    target_label: String,
    is_conditional: bool,
}

impl ISATranslator {
    pub fn new() -> Self {
        Self { code: Vec::new() }
    }

    pub fn translate(&mut self, _module: &IRModule) -> Result<Vec<u8>, String> {
        self.code.clear();
        
        // 1. Prologue (push rbp, mov rbp, rsp, sub rsp, 128)
        self.code.extend_from_slice(&[0x55, 0x48, 0x89, 0xE5]);
        self.code.extend_from_slice(&[0x48, 0x81, 0xEC, 0x80, 0x00, 0x00, 0x00]);
        
        let mut labels: HashMap<String, usize> = HashMap::new();
        let mut patches = Vec::new();
        let mut var_map: HashMap<String, i32> = HashMap::new();
        let mut next_offset = 0;

        for func in &_module.functions {
             for instr in &func.body {
                 self.emit_root(instr, &mut var_map, &mut next_offset, &mut labels, &mut patches);
             }
        }

        // Pass 2: Patch Labels
        for patch in patches {
            if let Some(&target) = labels.get(&patch.target_label) {
                let rel32 = (target as isize) - ((patch.offset + 4) as isize);
                let rel32_bytes = (rel32 as i32).to_le_bytes();
                for i in 0..4 {
                    self.code[patch.offset + i] = rel32_bytes[i];
                }
            }
        }

        // Epilogue
        self.code.extend_from_slice(&[0xB8, 0x00, 0x00, 0x00, 0x00]); // mov eax, 0
        self.code.push(0xC9); // leave
        self.code.push(0xC3); // ret

        Ok(self.code.clone())
    }

    fn emit_root(&mut self, instr: &IRInstruction, var_map: &mut HashMap<String, i32>, next_offset: &mut i32, labels: &mut HashMap<String, usize>, patches: &mut Vec<Patch>) {
        match instr {
            IRInstruction::Label(name) => {
                labels.insert(name.clone(), self.code.len());
            }
            IRInstruction::Jump(name) => {
                self.code.push(0xE9); // jmp rel32
                patches.push(Patch { offset: self.code.len(), target_label: name.clone(), is_conditional: false });
                self.code.extend_from_slice(&[0,0,0,0]);
            }
            IRInstruction::BranchIfFalse(name) => {
                // cmp rax, 0
                self.code.extend_from_slice(&[0x48, 0x83, 0xF8, 0x00]);
                // je rel32
                self.code.extend_from_slice(&[0x0F, 0x84]);
                patches.push(Patch { offset: self.code.len(), target_label: name.clone(), is_conditional: true });
                self.code.extend_from_slice(&[0,0,0,0]);
            }
            IRInstruction::VarDecl { name, .. } => {
                if !var_map.contains_key(name) {
                    *next_offset += 8;
                    var_map.insert(name.clone(), *next_offset);
                }
            }
            IRInstruction::Store(name) => {
                if !var_map.contains_key(name) {
                    *next_offset += 8;
                    var_map.insert(name.clone(), *next_offset);
                }
                let off = *var_map.get(name).unwrap();
                self.code.extend_from_slice(&[0x48, 0x89, 0x45]); // mov [rbp - off], rax
                self.code.push((256 - off) as u8);
            }
            IRInstruction::PrintStr(s) => {
                let boxed_str = Box::leak(s.clone().into_boxed_str());
                let ptr = boxed_str.as_ptr() as u64;
                let len = boxed_str.len() as u64;
                let fn_ptr = crate::backend::jit::jdb_print_str as *const () as u64;

                self.code.extend_from_slice(&[0x48, 0xB9]);
                self.code.extend_from_slice(&ptr.to_le_bytes());
                self.code.extend_from_slice(&[0x48, 0xBA]);
                self.code.extend_from_slice(&len.to_le_bytes());
                self.code.extend_from_slice(&[0x48, 0xB8]);
                self.code.extend_from_slice(&fn_ptr.to_le_bytes());
                
                self.code.extend_from_slice(&[0x48, 0x81, 0xEC, 0x20, 0x00, 0x00, 0x00]);
                self.code.extend_from_slice(&[0xFF, 0xD0]);
                self.code.extend_from_slice(&[0x48, 0x81, 0xC4, 0x20, 0x00, 0x00, 0x00]);
            }
            IRInstruction::PrintInt => {
                let fn_ptr = crate::backend::jit::jdb_print_int as *const () as u64;
                self.code.extend_from_slice(&[0x48, 0x89, 0xC1]); // mov rcx, rax
                self.code.extend_from_slice(&[0x48, 0xB8]);
                self.code.extend_from_slice(&fn_ptr.to_le_bytes());
                
                self.code.extend_from_slice(&[0x48, 0x81, 0xEC, 0x20, 0x00, 0x00, 0x00]);
                self.code.extend_from_slice(&[0xFF, 0xD0]);
                self.code.extend_from_slice(&[0x48, 0x81, 0xC4, 0x20, 0x00, 0x00, 0x00]);
            }
            IRInstruction::GCPlusScopeEnter { .. } | IRInstruction::GCPlusScopeExit { .. } |
            IRInstruction::GCPlusLoopAlloc { .. } | IRInstruction::GCPlusLoopReuse { .. } | 
            IRInstruction::GCPlusLoopFree { .. } | IRInstruction::GCPlusEscapeCheck { .. } => {}
            _ => {
                self.emit_expr(instr, var_map, next_offset, labels, patches);
            }
        }
    }

    fn emit_expr(&mut self, instr: &IRInstruction, var_map: &mut HashMap<String, i32>, next_offset: &mut i32, labels: &mut HashMap<String, usize>, patches: &mut Vec<Patch>) {
        match instr {
            IRInstruction::LoadConst(IRConstValue::Int(v)) => {
                self.code.extend_from_slice(&[0x48, 0xB8]);
                self.code.extend_from_slice(&v.to_le_bytes());
            }
            IRInstruction::LoadConst(IRConstValue::Bool(b)) => {
                let v = if *b { 1i64 } else { 0i64 };
                self.code.extend_from_slice(&[0x48, 0xB8]);
                self.code.extend_from_slice(&v.to_le_bytes());
            }
            IRInstruction::Load(name) => {
                let off = *var_map.get(name).unwrap_or(&0);
                self.code.extend_from_slice(&[0x48, 0x8B, 0x45]); // mov rax, [rbp - off]
                self.code.push((256 - off) as u8);
            }
            IRInstruction::BinOp { op, left, right } => {
                self.emit_expr(left, var_map, next_offset, labels, patches);
                self.code.push(0x50); // push rax
                self.emit_expr(right, var_map, next_offset, labels, patches);
                self.code.extend_from_slice(&[0x48, 0x89, 0xC3]); // mov rbx, rax
                self.code.push(0x58); // pop rax
                
                match op {
                    IROp::Add => self.code.extend_from_slice(&[0x48, 0x01, 0xD8]),
                    IROp::Sub => self.code.extend_from_slice(&[0x48, 0x29, 0xD8]),
                    IROp::Mul => self.code.extend_from_slice(&[0x48, 0x0F, 0xAF, 0xC3]),
                    IROp::Div | IROp::Mod => {
                        self.code.extend_from_slice(&[0x48, 0x99]); // cqo
                        self.code.extend_from_slice(&[0x48, 0xF7, 0xFB]); // idiv rbx
                        if matches!(op, IROp::Mod) {
                            self.code.extend_from_slice(&[0x48, 0x89, 0xD0]); // mov rax, rdx
                        }
                    }
                    _ => {}
                }
            }
            IRInstruction::Compare { op, left, right } => {
                self.emit_expr(left, var_map, next_offset, labels, patches);
                self.code.push(0x50);
                self.emit_expr(right, var_map, next_offset, labels, patches);
                self.code.extend_from_slice(&[0x48, 0x89, 0xC3]);
                self.code.push(0x58);
                self.code.extend_from_slice(&[0x48, 0x39, 0xD8]); // cmp rax, rbx
                
                match op {
                    IRCmpOp::Eq => self.code.extend_from_slice(&[0x0F, 0x94, 0xC0]),
                    IRCmpOp::Ne => self.code.extend_from_slice(&[0x0F, 0x95, 0xC0]),
                    IRCmpOp::Lt => self.code.extend_from_slice(&[0x0F, 0x9C, 0xC0]),
                    IRCmpOp::Gt => self.code.extend_from_slice(&[0x0F, 0x9F, 0xC0]),
                    IRCmpOp::Le => self.code.extend_from_slice(&[0x0F, 0x9E, 0xC0]),
                    IRCmpOp::Ge => self.code.extend_from_slice(&[0x0F, 0x9D, 0xC0]),
                    _ => {}
                }
                self.code.extend_from_slice(&[0x48, 0x0F, 0xB6, 0xC0]); // movzb rax, al
            }
            _ => {}
        }
    }
}
