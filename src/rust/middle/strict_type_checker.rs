// ============================================================
// Strict Type Checker — "Respetar Bits"
// FORTRAN 1957 + Ada 1983 + ADead-BIB 2025
// ============================================================
// Type Strictness ULTRA Agresivo para C y C++
// Sin conversión implícita NUNCA.
// Dev debe ser EXPLÍCITO siempre.
// Sin bypass posible.
// ============================================================

use std::fmt;

/// C Type representation for strict checking
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CType {
    // Signed integers
    Int8,
    Int16,
    Int32,
    Int64,
    // Unsigned integers
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    // Floating point
    Float32,
    Float64,
    // Other primitives
    Char,
    Bool,
    Void,
    // Composite
    Pointer(Box<CType>),
    Array(Box<CType>, usize),
    Struct(String),
    // Unknown (for incomplete inference)
    Unknown,
}

impl CType {
    /// Returns the CPU unit used for this type
    /// ALU for integers, FPU for floats
    pub fn cpu_unit(&self) -> &'static str {
        match self {
            CType::Float32 | CType::Float64 => "FPU (XMM)",
            _ => "ALU (RAX/EAX)",
        }
    }

    /// Returns the bit representation description
    pub fn bit_representation(&self) -> &'static str {
        match self {
            CType::Float32 => "IEEE 754 32-bit",
            CType::Float64 => "IEEE 754 64-bit",
            CType::Int8 => "complemento a 2, 8-bit",
            CType::Int16 => "complemento a 2, 16-bit",
            CType::Int32 => "complemento a 2, 32-bit",
            CType::Int64 => "complemento a 2, 64-bit",
            CType::UInt8 => "binario sin signo, 8-bit",
            CType::UInt16 => "binario sin signo, 16-bit",
            CType::UInt32 => "binario sin signo, 32-bit",
            CType::UInt64 => "binario sin signo, 64-bit",
            CType::Char => "ASCII/UTF-8, 8-bit",
            CType::Bool => "booleano, 8-bit",
            CType::Void => "sin representación",
            CType::Pointer(_) => "dirección de memoria, 64-bit",
            CType::Array(_, _) => "secuencia contigua",
            CType::Struct(_) => "estructura compuesta",
            CType::Unknown => "tipo desconocido",
        }
    }

    /// Returns the register used for this type
    pub fn register(&self) -> &'static str {
        match self {
            CType::Float32 | CType::Float64 => "XMM0-XMM15",
            CType::Int8 | CType::UInt8 | CType::Char | CType::Bool => "AL/BL/CL/DL",
            CType::Int16 | CType::UInt16 => "AX/BX/CX/DX",
            CType::Int32 | CType::UInt32 => "EAX/EBX/ECX/EDX",
            _ => "RAX/RBX/RCX/RDX",
        }
    }

    /// Size in bytes
    pub fn size_bytes(&self) -> usize {
        match self {
            CType::Int8 | CType::UInt8 | CType::Char | CType::Bool => 1,
            CType::Int16 | CType::UInt16 => 2,
            CType::Int32 | CType::UInt32 | CType::Float32 => 4,
            CType::Int64 | CType::UInt64 | CType::Float64 | CType::Pointer(_) => 8,
            CType::Void => 0,
            CType::Array(inner, count) => inner.size_bytes() * count,
            CType::Struct(_) => 8, // Default, should be looked up
            CType::Unknown => 8,
        }
    }

    /// Is this a signed integer type?
    pub fn is_signed(&self) -> bool {
        matches!(self, CType::Int8 | CType::Int16 | CType::Int32 | CType::Int64)
    }

    /// Is this an unsigned integer type?
    pub fn is_unsigned(&self) -> bool {
        matches!(self, CType::UInt8 | CType::UInt16 | CType::UInt32 | CType::UInt64)
    }

    /// Is this a floating point type?
    pub fn is_float(&self) -> bool {
        matches!(self, CType::Float32 | CType::Float64)
    }

    /// Is this an integer type (signed or unsigned)?
    pub fn is_integer(&self) -> bool {
        self.is_signed() || self.is_unsigned()
    }

    /// Is this a pointer type?
    pub fn is_pointer(&self) -> bool {
        matches!(self, CType::Pointer(_))
    }

    /// Convert from frontend Type to CType
    pub fn from_frontend_type(t: &crate::frontend::types::Type) -> Self {
        use crate::frontend::types::Type;
        match t {
            Type::I8 => CType::Int8,
            Type::I16 => CType::Int16,
            Type::I32 => CType::Int32,
            Type::I64 => CType::Int64,
            Type::U8 => CType::UInt8,
            Type::U16 => CType::UInt16,
            Type::U32 => CType::UInt32,
            Type::U64 => CType::UInt64,
            Type::F32 => CType::Float32,
            Type::F64 => CType::Float64,
            Type::Bool => CType::Bool,
            Type::Void => CType::Void,
            Type::Pointer(inner) => CType::Pointer(Box::new(CType::from_frontend_type(inner))),
            Type::Array(inner, Some(n)) => CType::Array(Box::new(CType::from_frontend_type(inner)), *n),
            Type::Array(inner, None) => CType::Pointer(Box::new(CType::from_frontend_type(inner))),
            Type::Struct(name) | Type::Class(name) | Type::Named(name) => CType::Struct(name.clone()),
            _ => CType::Unknown,
        }
    }
}

impl fmt::Display for CType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CType::Int8 => write!(f, "int8"),
            CType::Int16 => write!(f, "int16"),
            CType::Int32 => write!(f, "int32"),
            CType::Int64 => write!(f, "int64"),
            CType::UInt8 => write!(f, "uint8"),
            CType::UInt16 => write!(f, "uint16"),
            CType::UInt32 => write!(f, "uint32"),
            CType::UInt64 => write!(f, "uint64"),
            CType::Float32 => write!(f, "float32"),
            CType::Float64 => write!(f, "float64"),
            CType::Char => write!(f, "char"),
            CType::Bool => write!(f, "bool"),
            CType::Void => write!(f, "void"),
            CType::Pointer(inner) => write!(f, "{}*", inner),
            CType::Array(inner, n) => write!(f, "{}[{}]", inner, n),
            CType::Struct(name) => write!(f, "struct {}", name),
            CType::Unknown => write!(f, "unknown"),
        }
    }
}

/// Result of type compatibility check
#[derive(Debug, Clone)]
pub enum TypeCompatResult {
    /// Types are compatible, result type is provided
    Ok(CType),
    /// Type mismatch (e.g., int + float)
    Mismatch {
        left: CType,
        right: CType,
        op: String,
        suggestions: Vec<String>,
    },
    /// Signed/unsigned mixing
    SignedUnsignedMix {
        signed: CType,
        unsigned: CType,
        suggestions: Vec<String>,
    },
    /// Narrowing conversion (e.g., double → int)
    NarrowingConversion {
        from: CType,
        to: CType,
        suggestion: String,
    },
    /// Implicit cast (e.g., void* → int*)
    ImplicitCast {
        from: CType,
        to: CType,
        suggestion: String,
    },
    /// Integer overflow detected at compile time
    IntegerOverflow {
        typ: CType,
        operation: String,
        suggestion: String,
    },
}

impl TypeCompatResult {
    pub fn is_error(&self) -> bool {
        !matches!(self, TypeCompatResult::Ok(_))
    }
}

/// Check if two types are compatible for a binary operation
/// Returns Ok(result_type) if compatible, or an error variant if not
pub fn check_types_compatible(left: &CType, right: &CType, op: &str) -> TypeCompatResult {
    // Unknown types pass through (incomplete inference)
    if *left == CType::Unknown || *right == CType::Unknown {
        return TypeCompatResult::Ok(CType::Unknown);
    }

    // Same type is always compatible
    if left == right {
        return TypeCompatResult::Ok(left.clone());
    }

    // Special case: char + char = int32 (C standard)
    if *left == CType::Char && *right == CType::Char {
        return TypeCompatResult::Ok(CType::Int32);
    }

    // Special case: bool + bool = int32 (C standard)
    if *left == CType::Bool && *right == CType::Bool {
        return TypeCompatResult::Ok(CType::Int32);
    }

    // BLOCKED: signed vs unsigned mixing
    if (left.is_signed() && right.is_unsigned()) || (left.is_unsigned() && right.is_signed()) {
        let (signed, unsigned) = if left.is_signed() {
            (left.clone(), right.clone())
        } else {
            (right.clone(), left.clone())
        };
        return TypeCompatResult::SignedUnsignedMix {
            signed,
            unsigned,
            suggestions: vec![
                format!("(unsigned)left {} right", op),
                format!("left {} (int)right", op),
            ],
        };
    }

    // BLOCKED: int vs float mixing
    if (left.is_integer() && right.is_float()) || (left.is_float() && right.is_integer()) {
        return TypeCompatResult::Mismatch {
            left: left.clone(),
            right: right.clone(),
            op: op.to_string(),
            suggestions: vec![
                format!("(float)left {} right", op),
                format!("left {} (int)right", op),
            ],
        };
    }

    // BLOCKED: float32 vs float64 mixing
    if (*left == CType::Float32 && *right == CType::Float64)
        || (*left == CType::Float64 && *right == CType::Float32)
    {
        return TypeCompatResult::Mismatch {
            left: left.clone(),
            right: right.clone(),
            op: op.to_string(),
            suggestions: vec![
                format!("(double)left {} right", op),
                format!("(float)left {} (float)right", op),
            ],
        };
    }

    // BLOCKED: different integer sizes
    if left.is_integer() && right.is_integer() && left.size_bytes() != right.size_bytes() {
        return TypeCompatResult::Mismatch {
            left: left.clone(),
            right: right.clone(),
            op: op.to_string(),
            suggestions: vec![
                format!("({})left {} right", right, op),
                format!("left {} ({})right", op, left),
            ],
        };
    }

    // BLOCKED: pointer arithmetic with incompatible types
    if left.is_pointer() && right.is_pointer() && left != right {
        return TypeCompatResult::Mismatch {
            left: left.clone(),
            right: right.clone(),
            op: op.to_string(),
            suggestions: vec!["Ensure pointer types match".to_string()],
        };
    }

    // Default: block anything else that doesn't match
    TypeCompatResult::Mismatch {
        left: left.clone(),
        right: right.clone(),
        op: op.to_string(),
        suggestions: vec![
            "Use explicit cast".to_string(),
            "Verify types before operating".to_string(),
        ],
    }
}

/// Check if an assignment is valid (no implicit narrowing)
pub fn check_assignment_compatible(target: &CType, source: &CType) -> TypeCompatResult {
    // Unknown types pass through
    if *target == CType::Unknown || *source == CType::Unknown {
        return TypeCompatResult::Ok(target.clone());
    }

    // Same type is always compatible
    if target == source {
        return TypeCompatResult::Ok(target.clone());
    }

    // BLOCKED: narrowing from float to int
    if target.is_integer() && source.is_float() {
        return TypeCompatResult::NarrowingConversion {
            from: source.clone(),
            to: target.clone(),
            suggestion: format!("Use explicit cast: ({})value", target),
        };
    }

    // BLOCKED: narrowing from double to float
    if *target == CType::Float32 && *source == CType::Float64 {
        return TypeCompatResult::NarrowingConversion {
            from: source.clone(),
            to: target.clone(),
            suggestion: "(float)value".to_string(),
        };
    }

    // BLOCKED: narrowing from larger int to smaller int
    if target.is_integer() && source.is_integer() && target.size_bytes() < source.size_bytes() {
        return TypeCompatResult::NarrowingConversion {
            from: source.clone(),
            to: target.clone(),
            suggestion: format!("Use explicit cast: ({})value", target),
        };
    }

    // BLOCKED: implicit void* to T* cast
    if let CType::Pointer(inner_target) = target {
        if let CType::Pointer(inner_source) = source {
            if **inner_source == CType::Void && **inner_target != CType::Void {
                return TypeCompatResult::ImplicitCast {
                    from: source.clone(),
                    to: target.clone(),
                    suggestion: format!("Use explicit cast: ({}*)ptr", inner_target),
                };
            }
        }
    }

    // BLOCKED: signed/unsigned mismatch in assignment
    if (target.is_signed() && source.is_unsigned()) || (target.is_unsigned() && source.is_signed())
    {
        let (signed, unsigned) = if target.is_signed() {
            (target.clone(), source.clone())
        } else {
            (source.clone(), target.clone())
        };
        return TypeCompatResult::SignedUnsignedMix {
            signed,
            unsigned,
            suggestions: vec![format!("Use explicit cast: ({})value", target)],
        };
    }

    // Widening conversions are OK (int32 → int64, float → double)
    if target.is_integer() && source.is_integer() && target.size_bytes() >= source.size_bytes() {
        // Same signedness, widening is OK
        if (target.is_signed() && source.is_signed())
            || (target.is_unsigned() && source.is_unsigned())
        {
            return TypeCompatResult::Ok(target.clone());
        }
    }

    if *target == CType::Float64 && *source == CType::Float32 {
        return TypeCompatResult::Ok(target.clone());
    }

    // Default: block
    TypeCompatResult::Mismatch {
        left: target.clone(),
        right: source.clone(),
        op: "=".to_string(),
        suggestions: vec![format!("Use explicit cast: ({})value", target)],
    }
}

/// Check for compile-time integer overflow
pub fn check_overflow(typ: &CType, op: &str, left: i64, right: i64) -> Option<TypeCompatResult> {
    let overflow = match (typ, op) {
        (CType::Int32, "+") => (left as i32).checked_add(right as i32).is_none(),
        (CType::Int32, "-") => (left as i32).checked_sub(right as i32).is_none(),
        (CType::Int32, "*") => (left as i32).checked_mul(right as i32).is_none(),
        (CType::Int64, "+") => left.checked_add(right).is_none(),
        (CType::Int64, "-") => left.checked_sub(right).is_none(),
        (CType::Int64, "*") => left.checked_mul(right).is_none(),
        (CType::UInt32, "+") => (left as u32).checked_add(right as u32).is_none(),
        (CType::UInt32, "-") => (left as u32).checked_sub(right as u32).is_none(),
        (CType::UInt32, "*") => (left as u32).checked_mul(right as u32).is_none(),
        (CType::UInt64, "+") => (left as u64).checked_add(right as u64).is_none(),
        (CType::UInt64, "-") => (left as u64).checked_sub(right as u64).is_none(),
        (CType::UInt64, "*") => (left as u64).checked_mul(right as u64).is_none(),
        _ => false,
    };

    if overflow {
        Some(TypeCompatResult::IntegerOverflow {
            typ: typ.clone(),
            operation: format!("{} {} {}", left, op, right),
            suggestion: if typ.is_signed() {
                format!("Use wider type: (int64_t){} {} {}", left, op, right)
            } else {
                format!("Use wider type: (uint64_t){} {} {}", left, op, right)
            },
        })
    } else {
        None
    }
}

/// Format a detailed error message in FORTRAN 1957 style
pub fn format_error_message(result: &TypeCompatResult, file: &str, line: usize, col: usize) -> String {
    match result {
        TypeCompatResult::Ok(_) => String::new(),
        
        TypeCompatResult::Mismatch { left, right, op, suggestions } => {
            format!(
r#"[UB] TypeMismatch detectado — ADead-BIB
     archivo: {}
     línea: {}, columna: {}

     operación: {} {} {}

     lado izquierdo:
       tipo: {} ({})
       unidad CPU: {}
       registro: {}

     lado derecho:
       tipo: {} ({})
       unidad CPU: {}
       registro: {}

     problema:
       {} y {} son unidades diferentes
       bits con representación incompatible
       mezclar = UB garantizado

     soluciones:
{}

     filosofía:
       "los bits merecen respeto"
       "FORTRAN lo supo en 1957"
       "ADead-BIB lo aplica en 2025"

     compilación bloqueada 💀
     Binary Is Binary — ADead-BIB 💀🦈"#,
                file, line, col,
                left, op, right,
                left, left.bit_representation(),
                left.cpu_unit(),
                left.register(),
                right, right.bit_representation(),
                right.cpu_unit(),
                right.register(),
                left.cpu_unit(), right.cpu_unit(),
                suggestions.iter().map(|s| format!("       {}      ← opción", s)).collect::<Vec<_>>().join("\n")
            )
        }
        
        TypeCompatResult::SignedUnsignedMix { signed, unsigned, suggestions } => {
            format!(
r#"[UB] SignedUnsignedMix detectado — ADead-BIB
     archivo: {}
     línea: {}

     signed:   {}
               bits: representación con signo
               valor negativo posible

     unsigned: {}
               bits: representación sin signo
               siempre >= 0

     problema:
       C convierte signed a unsigned implícitamente
       -1 se convierte a 4,294,967,295 (INCORRECTO)
       bug silencioso clásico

     soluciones:
{}

     compilación bloqueada 💀"#,
                file, line,
                signed,
                unsigned,
                suggestions.iter().map(|s| format!("       {}", s)).collect::<Vec<_>>().join("\n")
            )
        }
        
        TypeCompatResult::NarrowingConversion { from, to, suggestion } => {
            format!(
r#"[UB] NarrowingConversion detectado — ADead-BIB
     archivo: {}
     línea: {}

     de:   {} ({} bytes)
     a:    {} ({} bytes)

     problema:
       pérdida de datos posible
       {} bits → {} bits

     solución:
       {}

     compilación bloqueada 💀"#,
                file, line,
                from, from.size_bytes(),
                to, to.size_bytes(),
                from.size_bytes() * 8, to.size_bytes() * 8,
                suggestion
            )
        }
        
        TypeCompatResult::ImplicitCast { from, to, suggestion } => {
            format!(
r#"[UB] ImplicitCast detectado — ADead-BIB
     archivo: {}
     línea: {}

     de:   {}
     a:    {}

     problema:
       cast implícito de puntero
       comportamiento indefinido posible

     solución:
       {}

     compilación bloqueada 💀"#,
                file, line,
                from, to,
                suggestion
            )
        }
        
        TypeCompatResult::IntegerOverflow { typ, operation, suggestion } => {
            format!(
r#"[UB] IntegerOverflow detectado — ADead-BIB
     archivo: {}
     línea: {}

     tipo: {}
     operación: {}

     problema:
       GCC ignoraría esto silenciosamente
       ADead-BIB NO permite silencio
       resultado indefinido en C estándar

     solución:
       {}

     compilación bloqueada 💀"#,
                file, line,
                typ, operation,
                suggestion
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_type_compatible() {
        let result = check_types_compatible(&CType::Int32, &CType::Int32, "+");
        assert!(matches!(result, TypeCompatResult::Ok(CType::Int32)));
    }

    #[test]
    fn test_int_float_mismatch() {
        let result = check_types_compatible(&CType::Int32, &CType::Float32, "+");
        assert!(matches!(result, TypeCompatResult::Mismatch { .. }));
    }

    #[test]
    fn test_signed_unsigned_mix() {
        let result = check_types_compatible(&CType::Int32, &CType::UInt32, "<");
        assert!(matches!(result, TypeCompatResult::SignedUnsignedMix { .. }));
    }

    #[test]
    fn test_float_double_mismatch() {
        let result = check_types_compatible(&CType::Float32, &CType::Float64, "+");
        assert!(matches!(result, TypeCompatResult::Mismatch { .. }));
    }

    #[test]
    fn test_int32_int64_mismatch() {
        let result = check_types_compatible(&CType::Int32, &CType::Int64, "+");
        assert!(matches!(result, TypeCompatResult::Mismatch { .. }));
    }

    #[test]
    fn test_narrowing_double_to_int() {
        let result = check_assignment_compatible(&CType::Int32, &CType::Float64);
        assert!(matches!(result, TypeCompatResult::NarrowingConversion { .. }));
    }

    #[test]
    fn test_implicit_void_ptr_cast() {
        let void_ptr = CType::Pointer(Box::new(CType::Void));
        let int_ptr = CType::Pointer(Box::new(CType::Int32));
        let result = check_assignment_compatible(&int_ptr, &void_ptr);
        assert!(matches!(result, TypeCompatResult::ImplicitCast { .. }));
    }

    #[test]
    fn test_overflow_detection() {
        let result = check_overflow(&CType::Int32, "+", i32::MAX as i64, 1);
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), TypeCompatResult::IntegerOverflow { .. }));
    }

    #[test]
    fn test_no_overflow() {
        let result = check_overflow(&CType::Int32, "+", 5, 10);
        assert!(result.is_none());
    }

    #[test]
    fn test_char_plus_char() {
        let result = check_types_compatible(&CType::Char, &CType::Char, "+");
        assert!(matches!(result, TypeCompatResult::Ok(CType::Int32)));
    }

    #[test]
    fn test_widening_ok() {
        let result = check_assignment_compatible(&CType::Int64, &CType::Int32);
        assert!(matches!(result, TypeCompatResult::Ok(_)));
    }
}
