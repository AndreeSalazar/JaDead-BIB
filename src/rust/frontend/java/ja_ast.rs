// ============================================================
// Java AST for JaDead-BIB 💀☕
// ============================================================
// Represents Java programs before lowering to ADeadOp IR
// Supports Java 8 → 21+ syntax
// ============================================================

/// Java Data Types
#[derive(Debug, Clone, PartialEq)]
pub enum JaType {
    // Primitives
    Int, Long, Float, Double, Boolean, Char, Byte, Short,
    Void,
    
    // Objects and Arrays
    Class(String),
    Array(Box<JaType>),
    Generic {
        base: String,
        type_args: Vec<JaType>,
    },
    
    // Type Inference / Wildcards
    Var,
    Wildcard {
        extends: Option<Box<JaType>>,
        super_type: Option<Box<JaType>>,
    },
    
    Inferred,
}

/// Compilation Unit (A single .java file)
#[derive(Debug, Clone)]
pub struct JaCompilationUnit {
    pub package: Option<JaPackageDecl>,
    pub imports: Vec<JaImportDecl>,
    pub declarations: Vec<JaTypeDecl>,
}

/// Package Declaration
#[derive(Debug, Clone)]
pub struct JaPackageDecl {
    pub name: String,
}

/// Import Declaration
#[derive(Debug, Clone)]
pub struct JaImportDecl {
    pub name: String,
    pub is_static: bool,
    pub is_asterisk: bool,
}

/// Type Declarations (Class, Interface, Record, Enum, Annotation)
#[derive(Debug, Clone)]
pub enum JaTypeDecl {
    Class {
        name: String,
        modifiers: Vec<JaModifier>,
        type_params: Vec<String>,
        extends: Option<JaType>,
        implements: Vec<JaType>,
        permits: Vec<JaType>, // Java 17+ Sealed
        body: Vec<JaClassMember>,
    },
    Interface {
        name: String,
        modifiers: Vec<JaModifier>,
        type_params: Vec<String>,
        extends: Vec<JaType>,
        permits: Vec<JaType>,
        body: Vec<JaClassMember>,
    },
    Record {
        name: String,
        modifiers: Vec<JaModifier>,
        type_params: Vec<String>,
        components: Vec<JaRecordComponent>,
        implements: Vec<JaType>,
        body: Vec<JaClassMember>,
    },
    Enum {
        name: String,
        modifiers: Vec<JaModifier>,
        implements: Vec<JaType>,
        constants: Vec<JaEnumConstant>,
        body: Vec<JaClassMember>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum JaModifier {
    Public, Protected, Private,
    Static, Final, Abstract,
    Synchronized, Volatile, Transient, Native,
    Strictfp, Default, Sealed, NonSealed,
}

#[derive(Debug, Clone)]
pub struct JaRecordComponent {
    pub name: String,
    pub ty: JaType,
}

#[derive(Debug, Clone)]
pub struct JaEnumConstant {
    pub name: String,
    pub args: Vec<JaExpr>,
}

/// Class Members (Fields, Methods, Constructors, Nested Types)
#[derive(Debug, Clone)]
pub enum JaClassMember {
    Field {
        name: String,
        ty: JaType,
        modifiers: Vec<JaModifier>,
        init: Option<JaExpr>,
    },
    Method {
        name: String,
        return_type: JaType,
        modifiers: Vec<JaModifier>,
        type_params: Vec<String>,
        params: Vec<JaParam>,
        throws: Vec<JaType>,
        body: Option<JaBlock>, // None if abstract/native
    },
    Constructor {
        name: String,
        modifiers: Vec<JaModifier>,
        params: Vec<JaParam>,
        throws: Vec<JaType>,
        body: JaBlock,
    },
    NestedType(JaTypeDecl),
    Initializer(JaBlock, bool), // static block
}

#[derive(Debug, Clone)]
pub struct JaParam {
    pub name: String,
    pub ty: JaType,
    pub is_varargs: bool,
    pub is_final: bool,
}

/// Statements
#[derive(Debug, Clone)]
pub enum JaStmt {
    Block(JaBlock),
    Expr(JaExpr),
    LocalVarDecl {
        ty: JaType,
        declarators: Vec<JaVarDeclarator>,
    },
    If {
        cond: JaExpr,
        then_branch: Box<JaStmt>,
        else_branch: Option<Box<JaStmt>>,
    },
    While {
        cond: JaExpr,
        body: Box<JaStmt>,
    },
    DoWhile {
        body: Box<JaStmt>,
        cond: JaExpr,
    },
    For {
        init: Option<Box<JaStmt>>, // simplified, usually LocalVarDecl or Expr
        cond: Option<JaExpr>,
        update: Vec<JaExpr>,
        body: Box<JaStmt>,
    },
    ForEach {
        ty: JaType,
        name: String,
        iterable: JaExpr,
        body: Box<JaStmt>,
    },
    Break(Option<String>),
    Continue(Option<String>),
    Return(Option<JaExpr>),
    Throw(JaExpr),
    Try {
        resources: Vec<JaLocalVarDecl>, // Try-with-resources
        body: JaBlock,
        catches: Vec<JaCatchClause>,
        finally_block: Option<JaBlock>,
    },
    Switch {
        expr: JaExpr,
        cases: Vec<JaSwitchCase>,
    },
    Yield(JaExpr), // Java 14+
    Synchronized {
        lock: JaExpr,
        body: JaBlock,
    },
    Empty,
}

#[derive(Debug, Clone)]
pub struct JaBlock {
    pub stmts: Vec<JaStmt>,
}

#[derive(Debug, Clone)]
pub struct JaLocalVarDecl {
    pub ty: JaType,
    pub name: String,
    pub init: Option<JaExpr>,
}

#[derive(Debug, Clone)]
pub struct JaVarDeclarator {
    pub name: String,
    pub init: Option<JaExpr>,
}

#[derive(Debug, Clone)]
pub struct JaCatchClause {
    pub types: Vec<JaType>,
    pub param_name: String,
    pub body: JaBlock,
}

#[derive(Debug, Clone)]
pub struct JaSwitchCase {
    pub labels: Vec<JaExpr>, // empty means Default
    pub is_arrow: bool,      // case x -> vs case x:
    pub body: Vec<JaStmt>,   // if arrow, usually single expr/block
}

/// Expressions
#[derive(Debug, Clone)]
pub enum JaExpr {
    // Literals
    IntLiteral(i64),
    LongLiteral(i64),
    FloatLiteral(f64),
    DoubleLiteral(f64),
    CharLiteral(char),
    StringLiteral(String),
    BooleanLiteral(bool),
    Null,
    
    // Names and Access
    Name(String),
    FieldAccess {
        target: Box<JaExpr>,
        field: String,
    },
    ArrayAccess {
        array: Box<JaExpr>,
        index: Box<JaExpr>,
    },
    
    // Method/Constructor calls
    MethodCall {
        target: Option<Box<JaExpr>>,
        name: String,
        type_args: Vec<JaType>,
        args: Vec<JaExpr>,
    },
    NewObject {
        ty: JaType,
        args: Vec<JaExpr>,
        body: Option<Vec<JaClassMember>>, // Anonymous subclass
    },
    NewArray {
        ty: JaType,
        dimensions: Vec<Option<JaExpr>>,
        init: Option<Vec<JaExpr>>,
    },
    
    // Operations
    Binary {
        op: JaBinOp,
        left: Box<JaExpr>,
        right: Box<JaExpr>,
    },
    Unary {
        op: JaUnaryOp,
        expr: Box<JaExpr>,
        is_postfix: bool,
    },
    Assign {
        op: JaAssignOp,
        target: Box<JaExpr>,
        value: Box<JaExpr>,
    },
    Ternary {
        cond: Box<JaExpr>,
        true_expr: Box<JaExpr>,
        false_expr: Box<JaExpr>,
    },
    Instanceof {
        expr: Box<JaExpr>,
        ty: JaType,
        pattern_name: Option<String>, // Java 16+ Pattern Matching
    },
    Cast {
        ty: JaType,
        expr: Box<JaExpr>,
    },
    
    // Modern Features
    Lambda {
        params: Vec<JaParam>,
        body: Box<JaStmt>, // Could be block or expression
    },
    MethodReference {
        target: Box<JaExpr>,
        name: String,
    },
    SwitchExpr {
        expr: Box<JaExpr>,
        cases: Vec<JaSwitchCase>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JaBinOp {
    Add, Sub, Mul, Div, Rem,
    Shl, Shr, UShr,
    BitAnd, BitOr, BitXor,
    And, Or,
    Eq, Neq, Lt, Gt, Le, Ge,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JaUnaryOp {
    Plus, Minus, Not, BitNot,
    Inc, Dec,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JaAssignOp {
    Assign,
    AddAssign, SubAssign, MulAssign, DivAssign, RemAssign,
    ShlAssign, ShrAssign, UShrAssign,
    AndAssign, OrAssign, XorAssign,
}
