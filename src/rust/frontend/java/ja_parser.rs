// ============================================================
// Java Parser for JaDead-BIB 💀☕
// ============================================================
// Recursive Descent Parser for Java 8 → 21+
// Generates JaCompilationUnit from JaToken stream
// ============================================================

use super::ja_lexer::{JaLexer, JaToken};
use super::ja_ast::*;

pub struct JaParser {
    tokens: Vec<JaToken>,
    pos: usize,
}

impl JaParser {
    pub fn new(mut lexer: JaLexer) -> Self {
        Self {
            tokens: lexer.tokenize(),
            pos: 0,
        }
    }

    // ── Entry Point ──────────────────────────────────────────

    pub fn parse_compilation_unit(&mut self) -> Result<JaCompilationUnit, String> {
        let package = self.parse_package_decl()?;
        let mut imports = Vec::new();
        while self.peek() == Some(&JaToken::Import) {
            imports.push(self.parse_import_decl()?);
        }

        let mut declarations = Vec::new();
        while self.peek() != Some(&JaToken::Eof) && self.peek().is_some() {
            declarations.push(self.parse_type_decl()?);
        }

        Ok(JaCompilationUnit {
            package,
            imports,
            declarations,
        })
    }

    // ── Helper Methods ───────────────────────────────────────

    fn peek(&self) -> Option<&JaToken> {
        self.tokens.get(self.pos)
    }

    fn peek_nth(&self, offset: usize) -> Option<&JaToken> {
        self.tokens.get(self.pos + offset)
    }

    fn advance(&mut self) -> Option<&JaToken> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn check(&self, expected: &JaToken) -> bool {
        self.peek() == Some(expected)
    }

    fn match_token(&mut self, expected: &JaToken) -> bool {
        if self.check(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, expected: &JaToken, msg: &str) -> Result<(), String> {
        if self.check(expected) {
            self.advance();
            Ok(())
        } else {
            Err(format!("Parser Error: {} - Expected {:?}", msg, expected))
        }
    }

    fn parse_identifier(&mut self) -> Result<String, String> {
        match self.advance() {
            Some(JaToken::Identifier(name)) => Ok(name.clone()),
            _ => Err("Expected identifier".to_string()),
        }
    }

    // ── Top Level Parsing ────────────────────────────────────

    fn parse_package_decl(&mut self) -> Result<Option<JaPackageDecl>, String> {
        if self.match_token(&JaToken::Package) {
            let name = self.parse_qualified_name()?;
            self.consume(&JaToken::Semicolon, "after package name")?;
            Ok(Some(JaPackageDecl { name }))
        } else {
            Ok(None)
        }
    }

    fn parse_import_decl(&mut self) -> Result<JaImportDecl, String> {
        self.consume(&JaToken::Import, "import declaration")?;
        let mut is_static = false;
        if self.match_token(&JaToken::Static) {
            is_static = true;
        }

        let mut name = self.parse_identifier()?;
        let mut is_asterisk = false;

        while self.match_token(&JaToken::Dot) {
            if self.match_token(&JaToken::Star) {
                is_asterisk = true;
                break;
            } else {
                name.push('.');
                name.push_str(&self.parse_identifier()?);
            }
        }

        self.consume(&JaToken::Semicolon, "after import declaration")?;
        Ok(JaImportDecl { name, is_static, is_asterisk })
    }

    fn parse_qualified_name(&mut self) -> Result<String, String> {
        let mut name = self.parse_identifier()?;
        while self.match_token(&JaToken::Dot) {
            name.push('.');
            name.push_str(&self.parse_identifier()?);
        }
        Ok(name)
    }

    // ── Type Declarations ────────────────────────────────────

    fn parse_modifiers(&mut self) -> Vec<JaModifier> {
        let mut modifiers = Vec::new();
        loop {
            match self.peek() {
                Some(JaToken::Public) => { self.advance(); modifiers.push(JaModifier::Public); }
                Some(JaToken::Private) => { self.advance(); modifiers.push(JaModifier::Private); }
                Some(JaToken::Protected) => { self.advance(); modifiers.push(JaModifier::Protected); }
                Some(JaToken::Static) => { self.advance(); modifiers.push(JaModifier::Static); }
                Some(JaToken::Final) => { self.advance(); modifiers.push(JaModifier::Final); }
                Some(JaToken::Abstract) => { self.advance(); modifiers.push(JaModifier::Abstract); }
                Some(JaToken::Sealed) => { self.advance(); modifiers.push(JaModifier::Sealed); }
                Some(JaToken::Strictfp) => { self.advance(); modifiers.push(JaModifier::Strictfp); }
                Some(JaToken::Native) => { self.advance(); modifiers.push(JaModifier::Native); }
                Some(JaToken::Transient) => { self.advance(); modifiers.push(JaModifier::Transient); }
                Some(JaToken::Volatile) => { self.advance(); modifiers.push(JaModifier::Volatile); }
                Some(JaToken::Synchronized) => { self.advance(); modifiers.push(JaModifier::Synchronized); }
                Some(JaToken::Default) => { self.advance(); modifiers.push(JaModifier::Default); }
                _ => break,
            }
        }
        modifiers
    }

    fn parse_type_decl(&mut self) -> Result<JaTypeDecl, String> {
        let modifiers = self.parse_modifiers();

        match self.peek() {
            Some(JaToken::Class) => self.parse_class_decl(modifiers),
            Some(JaToken::Interface) => self.parse_interface_decl(modifiers),
            Some(JaToken::Record) => self.parse_record_decl(modifiers),
            Some(JaToken::Enum) => self.parse_enum_decl(modifiers),
            _ => Err(format!("Expected type declaration (class/interface/enum/record) at token {:?}", self.peek())),
        }
    }

    fn parse_class_decl(&mut self, modifiers: Vec<JaModifier>) -> Result<JaTypeDecl, String> {
        self.consume(&JaToken::Class, "class keyword")?;
        let name = self.parse_identifier()?;
        
        // Type params placeholder
        let type_params = Vec::new(); // TODO: <T,U>
        
        let mut extends = None;
        if self.match_token(&JaToken::Extends) {
            extends = Some(self.parse_type()?);
        }

        let mut implements = Vec::new();
        if self.match_token(&JaToken::Implements) {
            implements.push(self.parse_type()?);
            while self.match_token(&JaToken::Comma) {
                implements.push(self.parse_type()?);
            }
        }

        let mut permits = Vec::new();
        if self.match_token(&JaToken::Permits) {
            permits.push(self.parse_type()?);
            while self.match_token(&JaToken::Comma) {
                permits.push(self.parse_type()?);
            }
        }

        self.consume(&JaToken::LBrace, "class body start")?;
        let mut body = Vec::new();
        while !self.check(&JaToken::RBrace) && !self.check(&JaToken::Eof) {
            body.push(self.parse_class_member()?);
        }
        self.consume(&JaToken::RBrace, "class body end")?;

        Ok(JaTypeDecl::Class { name, modifiers, type_params, extends, implements, permits, body })
    }

    fn parse_interface_decl(&mut self, modifiers: Vec<JaModifier>) -> Result<JaTypeDecl, String> {
        self.consume(&JaToken::Interface, "interface keyword")?;
        let name = self.parse_identifier()?;
        let type_params = Vec::new(); 
        
        let mut extends = Vec::new();
        if self.match_token(&JaToken::Extends) {
            extends.push(self.parse_type()?);
            while self.match_token(&JaToken::Comma) {
                extends.push(self.parse_type()?);
            }
        }

        let mut permits = Vec::new();
        if self.match_token(&JaToken::Permits) {
            permits.push(self.parse_type()?);
            while self.match_token(&JaToken::Comma) {
                permits.push(self.parse_type()?);
            }
        }

        self.consume(&JaToken::LBrace, "interface body start")?;
        let mut body = Vec::new();
        while !self.check(&JaToken::RBrace) && !self.check(&JaToken::Eof) {
            body.push(self.parse_class_member()?); // Interface methods
        }
        self.consume(&JaToken::RBrace, "interface body end")?;

        Ok(JaTypeDecl::Interface { name, modifiers, type_params, extends, permits, body })
    }

    fn parse_record_decl(&mut self, modifiers: Vec<JaModifier>) -> Result<JaTypeDecl, String> {
        self.consume(&JaToken::Record, "record keyword")?;
        let name = self.parse_identifier()?;
        let type_params = Vec::new();
        
        // Record components: (int x, String y)
        self.consume(&JaToken::LParen, "record components start")?;
        let mut components = Vec::new();
        if !self.check(&JaToken::RParen) {
            let ty = self.parse_type()?;
            let c_name = self.parse_identifier()?;
            components.push(JaRecordComponent { name: c_name, ty });
            
            while self.match_token(&JaToken::Comma) {
                let ty = self.parse_type()?;
                let c_name = self.parse_identifier()?;
                components.push(JaRecordComponent { name: c_name, ty });
            }
        }
        self.consume(&JaToken::RParen, "record components end")?;
        
        let mut implements = Vec::new();
        if self.match_token(&JaToken::Implements) {
            implements.push(self.parse_type()?);
            while self.match_token(&JaToken::Comma) {
                implements.push(self.parse_type()?);
            }
        }

        self.consume(&JaToken::LBrace, "record body start")?;
        let mut body = Vec::new();
        while !self.check(&JaToken::RBrace) && !self.check(&JaToken::Eof) {
            body.push(self.parse_class_member()?);
        }
        self.consume(&JaToken::RBrace, "record body end")?;

        Ok(JaTypeDecl::Record { name, modifiers, type_params, components, implements, body })
    }

    fn parse_enum_decl(&mut self, modifiers: Vec<JaModifier>) -> Result<JaTypeDecl, String> {
        self.consume(&JaToken::Enum, "enum keyword")?;
        let name = self.parse_identifier()?;
        
        let mut implements = Vec::new();
        if self.match_token(&JaToken::Implements) {
            implements.push(self.parse_type()?);
            while self.match_token(&JaToken::Comma) {
                implements.push(self.parse_type()?);
            }
        }

        self.consume(&JaToken::LBrace, "enum body start")?;
        
        let mut constants = Vec::new();
        while !self.check(&JaToken::Semicolon) && !self.check(&JaToken::RBrace) {
            let c_name = self.parse_identifier()?;
            let mut args = Vec::new();
            if self.match_token(&JaToken::LParen) {
                if !self.check(&JaToken::RParen) {
                    args.push(self.parse_expr()?);
                    while self.match_token(&JaToken::Comma) {
                        args.push(self.parse_expr()?);
                    }
                }
                self.consume(&JaToken::RParen, "enum constant args end")?;
            }
            constants.push(JaEnumConstant { name: c_name, args });
            
            if !self.match_token(&JaToken::Comma) {
                break;
            }
        }
        
        if self.match_token(&JaToken::Semicolon) {
            // Read rest of body
        }
        
        let mut body = Vec::new();
        while !self.check(&JaToken::RBrace) && !self.check(&JaToken::Eof) {
            body.push(self.parse_class_member()?);
        }
        self.consume(&JaToken::RBrace, "enum body end")?;

        Ok(JaTypeDecl::Enum { name, modifiers, implements, constants, body })
    }

    // ── Class Members ────────────────────────────────────────

    fn parse_class_member(&mut self) -> Result<JaClassMember, String> {
        let modifiers = self.parse_modifiers();
        
        // Constructor vs Method vs Field
        // 1. check if it's nested class/interface/enum/record
        match self.peek() {
            Some(JaToken::Class) | Some(JaToken::Interface) | Some(JaToken::Record) | Some(JaToken::Enum) => {
                return Ok(JaClassMember::NestedType(self.parse_type_decl()?));
            }
            _ => {}
        }
        
        // static { ... } block
        if modifiers.contains(&JaModifier::Static) && self.check(&JaToken::LBrace) {
            let block = self.parse_block()?;
            return Ok(JaClassMember::Initializer(block, true));
        } else if modifiers.is_empty() && self.check(&JaToken::LBrace) {
            let block = self.parse_block()?;
            return Ok(JaClassMember::Initializer(block, false));
        }

        // Could be Constructor or Method/Field. Read Type.
        // A constructor has no return type. Lookahead constraint: Constructor is Identifier followed by (
        if let Some(JaToken::Identifier(_)) = self.peek() {
            if self.peek_nth(1) == Some(&JaToken::LParen) {
                // Constructor
                let name = self.parse_identifier()?;
                let params = self.parse_params()?;
                
                let mut throws = Vec::new();
                if self.match_token(&JaToken::Throws) {
                    throws.push(self.parse_type()?);
                    while self.match_token(&JaToken::Comma) {
                        throws.push(self.parse_type()?);
                    }
                }

                let body = self.parse_block()?;
                return Ok(JaClassMember::Constructor { name, modifiers, params, throws, body });
            }
        }

        // Otherwise it's a Type for Method/Field
        let ty = self.parse_type()?;
        let name = self.parse_identifier()?;

        if self.check(&JaToken::LParen) {
            // Method
            let params = self.parse_params()?;
            
            let mut throws = Vec::new();
            if self.match_token(&JaToken::Throws) {
                throws.push(self.parse_type()?);
                while self.match_token(&JaToken::Comma) {
                    throws.push(self.parse_type()?);
                }
            }

            let body = if self.match_token(&JaToken::Semicolon) {
                None
            } else {
                Some(self.parse_block()?)
            };

            return Ok(JaClassMember::Method { name, return_type: ty, modifiers, type_params: vec![], params, throws, body });
        } else {
            // Field
            let mut init = None;
            if self.match_token(&JaToken::Assign) {
                init = Some(self.parse_expr()?);
            }
            self.consume(&JaToken::Semicolon, "end of field")?;
            return Ok(JaClassMember::Field { name, ty, modifiers, init });
        }
    }

    fn parse_params(&mut self) -> Result<Vec<JaParam>, String> {
        self.consume(&JaToken::LParen, "start of parameters")?;
        let mut params = Vec::new();
        if !self.check(&JaToken::RParen) {
            params.push(self.parse_param()?);
            while self.match_token(&JaToken::Comma) {
                params.push(self.parse_param()?);
            }
        }
        self.consume(&JaToken::RParen, "end of parameters")?;
        Ok(params)
    }

    fn parse_param(&mut self) -> Result<JaParam, String> {
        let is_final = self.match_token(&JaToken::Final);
        let ty = self.parse_type()?;
        
        let mut is_varargs = false;
        if self.match_token(&JaToken::Ellipsis) {
            is_varargs = true;
        }

        let name = self.parse_identifier()?;
        Ok(JaParam { name, ty, is_final, is_varargs })
    }

    // ── Types ────────────────────────────────────────────────

    fn parse_type(&mut self) -> Result<JaType, String> {
        let mut base_ty = match self.peek() {
            Some(JaToken::Int) => { self.advance(); JaType::Int }
            Some(JaToken::Long) => { self.advance(); JaType::Long }
            Some(JaToken::Float) => { self.advance(); JaType::Float }
            Some(JaToken::Double) => { self.advance(); JaType::Double }
            Some(JaToken::Boolean) => { self.advance(); JaType::Boolean }
            Some(JaToken::Char) => { self.advance(); JaType::Char }
            Some(JaToken::Byte) => { self.advance(); JaType::Byte }
            Some(JaToken::Short) => { self.advance(); JaType::Short }
            Some(JaToken::Void) => { self.advance(); JaType::Void }
            Some(JaToken::Var) => { self.advance(); JaType::Var }
            Some(JaToken::Identifier(_)) => {
                let name = self.parse_qualified_name()?;
                // Check generics
                if self.match_token(&JaToken::Less) {
                    let mut type_args = Vec::new();
                    type_args.push(self.parse_type()?);
                    while self.match_token(&JaToken::Comma) {
                        type_args.push(self.parse_type()?);
                    }
                    self.consume(&JaToken::Greater, "generic closing >")?;
                    JaType::Generic { base: name, type_args }
                } else {
                    JaType::Class(name)
                }
            }
            _ => return Err(format!("Expected type, got: {:?}", self.peek())),
        };

        // Arrays []
        while self.match_token(&JaToken::LBracket) {
            self.consume(&JaToken::RBracket, "closing bracket in array type")?;
            base_ty = JaType::Array(Box::new(base_ty));
        }

        Ok(base_ty)
    }

    // ── Statements & Expresions ──────────────────────────────

    fn parse_block(&mut self) -> Result<JaBlock, String> {
        self.consume(&JaToken::LBrace, "{")?;
        let mut stmts = Vec::new();
        while !self.check(&JaToken::RBrace) && !self.check(&JaToken::Eof) {
            stmts.push(self.parse_stmt()?);
        }
        self.consume(&JaToken::RBrace, "}")?;
        Ok(JaBlock { stmts })
    }

    fn parse_stmt(&mut self) -> Result<JaStmt, String> {
        // Very basic stub to build AST: Blocks, Ifs, Returns, Exprs
        if self.check(&JaToken::LBrace) {
            return Ok(JaStmt::Block(self.parse_block()?));
        }
        
        if self.match_token(&JaToken::Return) {
            let expr = if self.match_token(&JaToken::Semicolon) { None } else {
                let e = self.parse_expr()?;
                self.consume(&JaToken::Semicolon, "after return stmt")?;
                Some(e)
            };
            return Ok(JaStmt::Return(expr));
        }

        let expr = self.parse_expr()?;
        if self.match_token(&JaToken::Assign) {
            let val = self.parse_expr()?;
            self.consume(&JaToken::Semicolon, "after assignment")?;
            return Ok(JaStmt::Expr(JaExpr::Assign { 
                op: JaAssignOp::Assign, 
                target: Box::new(expr), 
                value: Box::new(val) 
            }));
        }

        self.consume(&JaToken::Semicolon, "after statement")?;
        Ok(JaStmt::Expr(expr))
    }

    fn parse_expr(&mut self) -> Result<JaExpr, String> {
        self.parse_primary_or_access()
    }

    fn parse_primary_or_access(&mut self) -> Result<JaExpr, String> {
        let mut expr = match self.peek() {
            Some(JaToken::IntLiteral(_)) => {
                if let JaToken::IntLiteral(v) = self.advance().unwrap() {
                    JaExpr::IntLiteral(*v)
                } else { unreachable!() }
            }
            Some(JaToken::StringLiteral(_)) | Some(JaToken::TextBlock(_)) => {
                if let JaToken::StringLiteral(v) | JaToken::TextBlock(v) = self.advance().unwrap() {
                    JaExpr::StringLiteral(v.clone())
                } else { unreachable!() }
            }
            Some(JaToken::True) => { self.advance(); JaExpr::BooleanLiteral(true) }
            Some(JaToken::False) => { self.advance(); JaExpr::BooleanLiteral(false) }
            Some(JaToken::Null) => { self.advance(); JaExpr::Null }
            Some(JaToken::This) => { self.advance(); JaExpr::Name("this".to_string()) }
            Some(JaToken::Identifier(_)) => {
                let name = self.parse_identifier()?;
                if self.match_token(&JaToken::LParen) {
                    // Implicit method call: atacar()
                    self.consume(&JaToken::RParen, "end of implicit method args")?;
                    JaExpr::MethodCall { target: None, name, type_args: vec![], args: vec![] }
                } else {
                    JaExpr::Name(name)
                }
            }
            _ => return Err(format!("Unimplemented expression parsing at {:?}", self.peek()))
        };

        // Handle . field accesses or method calls
        while self.match_token(&JaToken::Dot) {
            let field_or_method = self.parse_identifier()?;
            if self.match_token(&JaToken::LParen) {
                self.consume(&JaToken::RParen, "end of method args")?;
                expr = JaExpr::MethodCall { target: Some(Box::new(expr)), name: field_or_method, type_args: vec![], args: vec![] };
            } else {
                expr = JaExpr::FieldAccess { target: Box::new(expr), field: field_or_method };
            }
        }

        Ok(expr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_record() {
        let code = "public record Point(int x, int y) {}";
        let lexer = JaLexer::new(code);
        let mut parser = JaParser::new(lexer);
        let unit = parser.parse_compilation_unit().unwrap();
        
        assert_eq!(unit.declarations.len(), 1);
        match &unit.declarations[0] {
            JaTypeDecl::Record { name, components, .. } => {
                assert_eq!(name, "Point");
                assert_eq!(components.len(), 2);
                assert_eq!(components[0].name, "x");
                assert_eq!(components[1].name, "y");
            }
            _ => panic!("Expected record"),
        }
    }

    #[test]
    fn test_parse_class_with_methods() {
        let code = "class Jugador { int vida; public void atacar() { return; } }";
        let lexer = JaLexer::new(code);
        let mut parser = JaParser::new(lexer);
        let unit = parser.parse_compilation_unit().unwrap();
        
        assert_eq!(unit.declarations.len(), 1);
        match &unit.declarations[0] {
            JaTypeDecl::Class { name, body, .. } => {
                assert_eq!(name, "Jugador");
                assert_eq!(body.len(), 2);
            }
            _ => panic!("Expected class"),
        }
    }
}
