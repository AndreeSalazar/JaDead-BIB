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
                patches.push(Patch { offset: self.code.len(), target_label: name.clone() });
                self.code.extend_from_slice(&[0,0,0,0]);
            }
            IRInstruction::BranchIfFalse(name) => {
                // cmp rax, 0
                self.code.extend_from_slice(&[0x48, 0x83, 0xF8, 0x00]);
                // je rel32
                self.code.extend_from_slice(&[0x0F, 0x84]);
                patches.push(Patch { offset: self.code.len(), target_label: name.clone() });
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
            IRInstruction::PropertySet { obj, offset } => {
                // value is in rax, we need to save it to [obj_ptr + offset]
                self.code.push(0x50); // push rax (value)
                
                let off_local = *var_map.get(obj).unwrap_or(&0);
                self.code.extend_from_slice(&[0x48, 0x8B, 0x5D]); // mov rbx, [rbp - off_local]
                self.code.push((256 - off_local) as u8);
                
                self.code.push(0x58); // pop rax (value)
                
                // mov [rbx + offset], rax
                if *offset < 128 {
                    self.code.extend_from_slice(&[0x48, 0x89, 0x43, *offset as u8]);
                } else {
                    self.code.extend_from_slice(&[0x48, 0x89, 0x83]);
                    self.code.extend_from_slice(&offset.to_le_bytes());
                }
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
            IRInstruction::Return => {
                // Value is already in RAX from the preceding expression
                self.code.push(0xC9); // leave
                self.code.push(0xC3); // ret
            }
            IRInstruction::ReturnVoid => {
                self.code.push(0xC9); // leave
                self.code.push(0xC3); // ret
            }
            IRInstruction::PrintNewline => {
                let fn_ptr = crate::backend::jit::jdb_print_newline as *const () as u64;
                self.code.extend_from_slice(&[0x48, 0xB8]);
                self.code.extend_from_slice(&fn_ptr.to_le_bytes());
                self.code.extend_from_slice(&[0x48, 0x81, 0xEC, 0x20, 0x00, 0x00, 0x00]);
                self.code.extend_from_slice(&[0xFF, 0xD0]);
                self.code.extend_from_slice(&[0x48, 0x81, 0xC4, 0x20, 0x00, 0x00, 0x00]);
            }
            IRInstruction::PrintFloat => {
                let fn_ptr = crate::backend::jit::jdb_print_float as *const () as u64;
                // xmm0 already has the float value
                self.code.extend_from_slice(&[0x48, 0xB8]);
                self.code.extend_from_slice(&fn_ptr.to_le_bytes());
                self.code.extend_from_slice(&[0x48, 0x81, 0xEC, 0x20, 0x00, 0x00, 0x00]);
                self.code.extend_from_slice(&[0xFF, 0xD0]);
                self.code.extend_from_slice(&[0x48, 0x81, 0xC4, 0x20, 0x00, 0x00, 0x00]);
            }
            IRInstruction::PrintChar => {
                let fn_ptr = crate::backend::jit::jdb_print_char as *const () as u64;
                self.code.extend_from_slice(&[0x48, 0x89, 0xC1]); // mov rcx, rax
                self.code.extend_from_slice(&[0x48, 0xB8]);
                self.code.extend_from_slice(&fn_ptr.to_le_bytes());
                self.code.extend_from_slice(&[0x48, 0x81, 0xEC, 0x20, 0x00, 0x00, 0x00]);
                self.code.extend_from_slice(&[0xFF, 0xD0]);
                self.code.extend_from_slice(&[0x48, 0x81, 0xC4, 0x20, 0x00, 0x00, 0x00]);
            }
            IRInstruction::Break | IRInstruction::Continue => {
                // These are handled at a higher level in a full implementation
                // For now, they are structural markers
            }
            IRInstruction::TryBegin(_) | IRInstruction::TryEnd | IRInstruction::ClearError |
            IRInstruction::FinallyBegin | IRInstruction::FinallyEnd |
            IRInstruction::Raise { .. } | IRInstruction::CheckError(_) => {
                // Exception handling is structural in v1.0
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
            IRInstruction::LoadConst(IRConstValue::Float(f)) => {
                // Load float as i64 bits into rax, then movq xmm0, rax
                let bits = f.to_bits();
                self.code.extend_from_slice(&[0x48, 0xB8]);
                self.code.extend_from_slice(&bits.to_le_bytes());
                // movq xmm0, rax: 66 48 0F 6E C0
                self.code.extend_from_slice(&[0x66, 0x48, 0x0F, 0x6E, 0xC0]);
            }
            IRInstruction::LoadConst(IRConstValue::None) => {
                self.code.extend_from_slice(&[0x48, 0x31, 0xC0]); // xor rax, rax
            }
            IRInstruction::Load(name) => {
                let off = *var_map.get(name).unwrap_or(&0);
                self.code.extend_from_slice(&[0x48, 0x8B, 0x45]); // mov rax, [rbp - off]
                self.code.push((256 - off) as u8);
            }
            IRInstruction::AllocObject { class_name: _, size } => {
                self.code.extend_from_slice(&[0x48, 0xC7, 0xC1]); // mov rcx, imm32 (Arg 1: size)
                self.code.extend_from_slice(&size.to_le_bytes()); 
                
                let fn_ptr = crate::backend::jit::jdb_alloc_obj as *const () as u64;
                self.code.extend_from_slice(&[0x48, 0xB8]); // mov rax, fn_ptr
                self.code.extend_from_slice(&fn_ptr.to_le_bytes());
                self.code.extend_from_slice(&[0x48, 0x81, 0xEC, 0x20, 0x00, 0x00, 0x00]); // shadow space x64 ABI
                self.code.extend_from_slice(&[0xFF, 0xD0]); // call rax
                self.code.extend_from_slice(&[0x48, 0x81, 0xC4, 0x20, 0x00, 0x00, 0x00]); // restore
            }
            IRInstruction::PropertyGet { obj, offset } => {
                let off_local = *var_map.get(obj).unwrap_or(&0);
                self.code.extend_from_slice(&[0x48, 0x8B, 0x45]); // mov rax, [rbp - off_local] (load object ptr)
                self.code.push((256 - off_local) as u8);
                
                // mov rax, [rax + offset]
                if *offset < 128 {
                    self.code.extend_from_slice(&[0x48, 0x8B, 0x40, *offset as u8]);
                } else {
                    self.code.extend_from_slice(&[0x48, 0x8B, 0x80]);
                    self.code.extend_from_slice(&offset.to_le_bytes()); // 32-bit offset
                }
            }
            IRInstruction::AllocArray { ir_type, count } => {
                let mut size = ir_type.byte_size() as u32;
                if size == 0 { size = 8; } // Default fallback for object references
                let element_size: u32 = size;
                
                self.emit_expr(count, var_map, next_offset, labels, patches); // rax = count
                self.code.extend_from_slice(&[0x48, 0x89, 0xC1]); // mov rcx, rax (Arg 1)
                
                self.code.extend_from_slice(&[0x48, 0xC7, 0xC2]); // mov rdx, imm32 (Arg 2)
                self.code.extend_from_slice(&element_size.to_le_bytes()); 
                
                let fn_ptr = crate::backend::jit::jdb_alloc_array as *const () as u64;
                self.code.extend_from_slice(&[0x48, 0xB8]); // mov rax, fn_ptr
                self.code.extend_from_slice(&fn_ptr.to_le_bytes());
                self.code.extend_from_slice(&[0x48, 0x81, 0xEC, 0x20, 0x00, 0x00, 0x00]); // shadow space
                self.code.extend_from_slice(&[0xFF, 0xD0]); // call rax
                self.code.extend_from_slice(&[0x48, 0x81, 0xC4, 0x20, 0x00, 0x00, 0x00]); // restore
            }
            IRInstruction::LoadElement { array, index } => {
                self.emit_expr(array, var_map, next_offset, labels, patches);
                self.code.push(0x50); // push rax (array struct ptr)
                self.emit_expr(index, var_map, next_offset, labels, patches);
                self.code.extend_from_slice(&[0x48, 0x89, 0xC3]); // mov rbx, rax (index)
                self.code.push(0x58); // pop rax (array struct ptr)
                
                // rax = JdbArray ptr, rbx = index
                self.code.extend_from_slice(&[0x48, 0x8B, 0x10]); // mov rdx, [rax] (ptr to buf)
                self.code.extend_from_slice(&[0x44, 0x8B, 0x40, 0x0C]); // mov r8d, dword ptr [rax+12]
                self.code.extend_from_slice(&[0x49, 0x0F, 0xAF, 0xD8]); // imul rbx, r8
                self.code.extend_from_slice(&[0x48, 0x01, 0xDA]); // add rdx, rbx
                self.code.extend_from_slice(&[0x48, 0x8B, 0x02]); // mov rax, [rdx]
            }
            IRInstruction::StoreElement { array, index, value } => {
                self.emit_expr(array, var_map, next_offset, labels, patches);
                self.code.push(0x50); // push rax (array ptr)
                self.emit_expr(index, var_map, next_offset, labels, patches);
                self.code.extend_from_slice(&[0x48, 0x89, 0xC3]); // mov rbx, rax (index)
                self.code.push(0x50); // push rbx (save index)
                
                self.emit_expr(value, var_map, next_offset, labels, patches);
                self.code.extend_from_slice(&[0x48, 0x89, 0xC1]); // mov rcx, rax (value)
                
                self.code.push(0x5B); // pop rbx (index)
                self.code.push(0x58); // pop rax (array ptr)
                
                // rax = array ptr, rbx = index, rcx = value
                self.code.extend_from_slice(&[0x48, 0x8B, 0x10]); // mov rdx, [rax] (ptr to buf)
                self.code.extend_from_slice(&[0x44, 0x8B, 0x40, 0x0C]); // mov r8d, dword ptr [rax+12]
                self.code.extend_from_slice(&[0x49, 0x0F, 0xAF, 0xD8]); // imul rbx, r8
                self.code.extend_from_slice(&[0x48, 0x01, 0xDA]); // add rdx, rbx
                
                self.code.extend_from_slice(&[0x48, 0x89, 0x0A]); // mov [rdx], rcx
            }
            IRInstruction::ArrayLength { array } => {
                self.emit_expr(array, var_map, next_offset, labels, patches);
                // rax = JdbArray ptr
                self.code.extend_from_slice(&[0x8B, 0x40, 0x08]); // mov eax, dword ptr [rax + 8]
                // Zero extend to 64-bit just in case, though mov eax zero-extends implicitly
            }
            IRInstruction::LoadString(s) => {
                let boxed_str = Box::leak(s.clone().into_boxed_str());
                let jdb_str = Box::new(crate::backend::jit::JdbString {
                    ptr: boxed_str.as_ptr(),
                    len: boxed_str.len() as u32,
                });
                let ptr_val = Box::leak(jdb_str) as *const _ as u64;
                self.code.extend_from_slice(&[0x48, 0xB8]);
                self.code.extend_from_slice(&ptr_val.to_le_bytes());
            }
            IRInstruction::Call { func, args } => {
                for arg in args {
                    self.emit_expr(arg, var_map, next_offset, labels, patches);
                    self.code.push(0x50); // push rax
                }
                
                let abi_regs = vec![
                    vec![0x48, 0x89, 0xC1], // mov rcx, rax (Arg 1)
                    vec![0x48, 0x89, 0xC2], // mov rdx, rax (Arg 2)
                    vec![0x49, 0x89, 0xC0], // mov r8, rax  (Arg 3)
                    vec![0x49, 0x89, 0xC1], // mov r9, rax  (Arg 4)
                ];
                
                for i in (0..args.len()).rev() {
                    self.code.push(0x58); // pop rax
                    if i < abi_regs.len() {
                        self.code.extend_from_slice(&abi_regs[i]);
                    }
                }
                
                let fn_ptr = match func.as_str() {
                    "jdb_string_len" => crate::backend::jit::jdb_string_len as *const () as u64,
                    "jdb_string_eq" => crate::backend::jit::jdb_string_eq as *const () as u64,
                    "jdb_string_concat" => crate::backend::jit::jdb_string_concat as *const () as u64,
                    "jdb_print_str" => crate::backend::jit::jdb_print_str as *const () as u64,
                    "jdb_print_obj" => crate::backend::jit::jdb_print_obj as *const () as u64,
                    _ => 0,
                };
                
                if fn_ptr != 0 {
                    self.code.extend_from_slice(&[0x48, 0xB8]); // mov rax, fn_ptr
                    self.code.extend_from_slice(&fn_ptr.to_le_bytes());
                    self.code.extend_from_slice(&[0x48, 0x81, 0xEC, 0x20, 0x00, 0x00, 0x00]); // shadow space x64 ABI
                    self.code.extend_from_slice(&[0xFF, 0xD0]); // call rax
                    self.code.extend_from_slice(&[0x48, 0x81, 0xC4, 0x20, 0x00, 0x00, 0x00]); // restore
                }
            }
            IRInstruction::BinOp { op, left, right } => {
                self.emit_expr(left, var_map, next_offset, labels, patches);
                self.code.push(0x50); // push rax
                self.emit_expr(right, var_map, next_offset, labels, patches);
                self.code.extend_from_slice(&[0x48, 0x89, 0xC3]); // mov rbx, rax
                self.code.push(0x58); // pop rax
                
                match op {
                    IROp::Add => self.code.extend_from_slice(&[0x48, 0x01, 0xD8]),       // add rax, rbx
                    IROp::Sub => self.code.extend_from_slice(&[0x48, 0x29, 0xD8]),       // sub rax, rbx
                    IROp::Mul => self.code.extend_from_slice(&[0x48, 0x0F, 0xAF, 0xC3]),// imul rax, rbx
                    IROp::Div | IROp::Mod => {
                        self.code.extend_from_slice(&[0x48, 0x99]); // cqo
                        self.code.extend_from_slice(&[0x48, 0xF7, 0xFB]); // idiv rbx
                        if matches!(op, IROp::Mod) {
                            self.code.extend_from_slice(&[0x48, 0x89, 0xD0]); // mov rax, rdx
                        }
                    }
                    IROp::Shl => {
                        self.code.extend_from_slice(&[0x48, 0x89, 0xD9]); // mov rcx, rbx
                        self.code.extend_from_slice(&[0x48, 0xD3, 0xE0]); // shl rax, cl
                    }
                    IROp::Shr => {
                        self.code.extend_from_slice(&[0x48, 0x89, 0xD9]); // mov rcx, rbx
                        self.code.extend_from_slice(&[0x48, 0xD3, 0xF8]); // sar rax, cl
                    }
                    IROp::And => self.code.extend_from_slice(&[0x48, 0x21, 0xD8]),       // and rax, rbx
                    IROp::Or  => self.code.extend_from_slice(&[0x48, 0x09, 0xD8]),       // or rax, rbx
                    IROp::Xor => self.code.extend_from_slice(&[0x48, 0x31, 0xD8]),       // xor rax, rbx
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
