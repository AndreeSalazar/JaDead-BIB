// ============================================================
// Java Type Checker for JaDead-BIB 💀☕
// ============================================================
// Maps Java static types to ADeadOp native representations
// Java has static types from the start -> massive advantage over Python
// ============================================================

use super::ja_ast::JaType;
use crate::middle::ir::IRType;

pub struct JaTypeChecker;

impl JaTypeChecker {
    pub fn new() -> Self {
        Self
    }

    /// Map Java AST Type to Native ADeadOp Type
    pub fn resolve_type(&self, ty: &JaType) -> Result<IRType, String> {
        match ty {
            JaType::Boolean => Ok(IRType::I8),
            JaType::Byte => Ok(IRType::I8),
            JaType::Char => Ok(IRType::I16),
            JaType::Short => Ok(IRType::I16),
            JaType::Int => Ok(IRType::I64), // ADeadOp standardizes primitives natively to I64 (64-bit words)
            JaType::Long => Ok(IRType::I64),
            JaType::Float => Ok(IRType::F32),
            JaType::Double => Ok(IRType::F64),
            JaType::Void => Ok(IRType::Void),

            JaType::Class(name) => {
                if name == "String" || name == "java.lang.String" {
                    // Strings are raw pointers to .data or Heap allocated char slices
                    Ok(IRType::Ptr)
                } else if name == "Object" || name == "java.lang.Object" {
                    // Object references are just Pointers
                    Ok(IRType::Ptr)
                } else {
                    // Custom object reference (Struct/Class instance pointer)
                    Ok(IRType::Ptr)
                }
            }

            JaType::Array(_) => {
                // T[] -> HeapAlloc array pointer (like ArrayList under the hood)
                Ok(IRType::Ptr)
            }

            JaType::Generic { base, .. } => {
                // List<T> -> HeapAlloc + vtable pointer
                // Actual monomorphization handles the inner types during IR Gen
                // For now, at type checking base level, it's considered a pointer.
                if base == "List" || base == "java.util.List"
                    || base == "ArrayList" || base == "java.util.ArrayList" {
                    Ok(IRType::Ptr)
                } else {
                    Ok(IRType::Ptr)
                }
            }

            JaType::Var | JaType::Inferred => {
                Err("Inferred types must be resolved during AST passes before Native mapping".to_string())
            }

            JaType::Wildcard { .. } => {
                Err("Wildcards must be erased or monomorphized before Native mapping".to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_resolution() {
        let checker = JaTypeChecker::new();
        assert_eq!(checker.resolve_type(&JaType::Int).unwrap(), IRType::I64);
        assert_eq!(checker.resolve_type(&JaType::Double).unwrap(), IRType::F64);
        assert_eq!(checker.resolve_type(&JaType::Boolean).unwrap(), IRType::I8);
    }

    #[test]
    fn test_reference_resolution() {
        let checker = JaTypeChecker::new();
        assert_eq!(checker.resolve_type(&JaType::Class("String".to_string())).unwrap(), IRType::Ptr);
        assert_eq!(checker.resolve_type(&JaType::Array(Box::new(JaType::Int))).unwrap(), IRType::Ptr);
    }
}
