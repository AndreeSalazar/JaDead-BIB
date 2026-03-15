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

    fn parse_type_params(&mut self) -> Vec<String> {
        let mut params = Vec::new();
        if self.match_token(&JaToken::Less) {
            if let Ok(name) = self.parse_identifier() {
                params.push(name);
                // Skip optional bounds: extends Foo
                if self.match_token(&JaToken::Extends) {
                    let _ = self.parse_type(); // consume bound type
                }
            }
            while self.match_token(&JaToken::Comma) {
                if let Ok(name) = self.parse_identifier() {
                    params.push(name);
                    if self.match_token(&JaToken::Extends) {
                        let _ = self.parse_type();
                    }
                }
            }
            let _ = self.consume(&JaToken::Greater, "closing > for type params");
        }
        params
    }

    fn parse_class_decl(&mut self, modifiers: Vec<JaModifier>) -> Result<JaTypeDecl, String> {
        self.consume(&JaToken::Class, "class keyword")?;
        let name = self.parse_identifier()?;
        
        let type_params = self.parse_type_params();
        
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
        let type_params = self.parse_type_params();
        
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
        let type_params = self.parse_type_params();
        
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
        if self.check(&JaToken::LBrace) { return Ok(JaStmt::Block(self.parse_block()?)); }
        if self.match_token(&JaToken::If) { return self.parse_if_stmt(); }
        if self.match_token(&JaToken::While) { return self.parse_while_stmt(); }
        if self.match_token(&JaToken::Do) { return self.parse_do_while_stmt(); }
        if self.match_token(&JaToken::For) { return self.parse_for_stmt(); }
        if self.match_token(&JaToken::Switch) { return self.parse_switch_stmt(); }
        if self.match_token(&JaToken::Try) { return self.parse_try_stmt(); }
        if self.match_token(&JaToken::Return) {
            let expr = if self.match_token(&JaToken::Semicolon) { None } else {
                let e = self.parse_expr()?;
                self.consume(&JaToken::Semicolon, "after return stmt")?;
                Some(e)
            };
            return Ok(JaStmt::Return(expr));
        }
        if self.match_token(&JaToken::Break) {
            let label = if let Some(JaToken::Identifier(_)) = self.peek() {
                Some(self.parse_identifier()?)
            } else { None };
            self.consume(&JaToken::Semicolon, "after break")?;
            return Ok(JaStmt::Break(label));
        }
        if self.match_token(&JaToken::Continue) {
            let label = if let Some(JaToken::Identifier(_)) = self.peek() {
                Some(self.parse_identifier()?)
            } else { None };
            self.consume(&JaToken::Semicolon, "after continue")?;
            return Ok(JaStmt::Continue(label));
        }
        if self.match_token(&JaToken::Throw) {
            let expr = self.parse_expr()?;
            self.consume(&JaToken::Semicolon, "after throw")?;
            return Ok(JaStmt::Throw(expr));
        }
        if self.match_token(&JaToken::Semicolon) {
            return Ok(JaStmt::Empty);
        }

        if self.is_local_var_decl() {
            let ty = self.parse_type()?;
            let name = self.parse_identifier()?;
            let mut init = None;
            if self.match_token(&JaToken::Assign) {
                init = Some(self.parse_expr()?);
            }
            self.consume(&JaToken::Semicolon, "after local var decl")?;
            return Ok(JaStmt::LocalVarDecl { ty, declarators: vec![JaVarDeclarator { name, init }] });
        }

        let expr = self.parse_expr()?;
        self.consume(&JaToken::Semicolon, "after statement")?;
        Ok(JaStmt::Expr(expr))
    }

    fn parse_if_stmt(&mut self) -> Result<JaStmt, String> {
        self.consume(&JaToken::LParen, "if condition start")?;
        let cond = self.parse_expr()?;
        self.consume(&JaToken::RParen, "if condition end")?;
        let then_branch = Box::new(self.parse_stmt()?);
        let else_branch = if self.match_token(&JaToken::Else) {
            Some(Box::new(self.parse_stmt()?))
        } else { None };
        Ok(JaStmt::If { cond, then_branch, else_branch })
    }

    fn parse_while_stmt(&mut self) -> Result<JaStmt, String> {
        self.consume(&JaToken::LParen, "while cond start")?;
        let cond = self.parse_expr()?;
        self.consume(&JaToken::RParen, "while cond end")?;
        let body = Box::new(self.parse_stmt()?);
        Ok(JaStmt::While { cond, body })
    }

    fn parse_do_while_stmt(&mut self) -> Result<JaStmt, String> {
        let body = Box::new(self.parse_stmt()?);
        self.consume(&JaToken::While, "do-while 'while' keyword")?;
        self.consume(&JaToken::LParen, "do-while cond start")?;
        let cond = self.parse_expr()?;
        self.consume(&JaToken::RParen, "do-while cond end")?;
        self.consume(&JaToken::Semicolon, "after do-while")?;
        Ok(JaStmt::DoWhile { body, cond })
    }

    fn parse_for_stmt(&mut self) -> Result<JaStmt, String> {
        self.consume(&JaToken::LParen, "for start")?;

        // Check for enhanced for (for-each): Type name : iterable
        if self.is_for_each() {
            let ty = self.parse_type()?;
            let name = self.parse_identifier()?;
            self.consume(&JaToken::Colon, "for-each colon")?;
            let iterable = self.parse_expr()?;
            self.consume(&JaToken::RParen, "for-each end")?;
            let body = Box::new(self.parse_stmt()?);
            return Ok(JaStmt::ForEach { ty, name, iterable, body });
        }

        let init = if self.match_token(&JaToken::Semicolon) { None } else {
            if self.is_local_var_decl() {
                let ty = self.parse_type()?;
                let name = self.parse_identifier()?;
                let mut var_init = None;
                if self.match_token(&JaToken::Assign) { var_init = Some(self.parse_expr()?); }
                self.consume(&JaToken::Semicolon, "for init")?;
                Some(Box::new(JaStmt::LocalVarDecl { ty, declarators: vec![JaVarDeclarator { name, init: var_init }] }))
            } else {
                let expr = self.parse_expr()?;
                self.consume(&JaToken::Semicolon, "for expr init")?;
                Some(Box::new(JaStmt::Expr(expr)))
            }
        };
        let cond = if self.check(&JaToken::Semicolon) { None } else { Some(self.parse_expr()?) };
        self.consume(&JaToken::Semicolon, "for cond")?;
        
        let mut update = Vec::new();
        if !self.check(&JaToken::RParen) {
            update.push(self.parse_expr()?);
            while self.match_token(&JaToken::Comma) {
                update.push(self.parse_expr()?);
            }
        }
        self.consume(&JaToken::RParen, "for end")?;
        
        let body = Box::new(self.parse_stmt()?);
        Ok(JaStmt::For { init, cond, update, body })
    }

    fn is_for_each(&self) -> bool {
        // Enhanced for: Type Identifier : expr
        let mut p = self.pos;
        // Skip type
        match self.tokens.get(p) {
            Some(JaToken::Int) | Some(JaToken::Long) | Some(JaToken::Float) | Some(JaToken::Double) |
            Some(JaToken::Boolean) | Some(JaToken::Char) | Some(JaToken::Byte) | Some(JaToken::Short) => { p += 1; }
            Some(JaToken::Identifier(_)) => {
                p += 1;
                while let Some(JaToken::Dot) = self.tokens.get(p) {
                    p += 1;
                    if let Some(JaToken::Identifier(_)) = self.tokens.get(p) { p += 1; } else { return false; }
                }
                // Skip generics
                if let Some(JaToken::Less) = self.tokens.get(p) {
                    let mut depth = 1; p += 1;
                    while depth > 0 && p < self.tokens.len() {
                        match self.tokens.get(p) {
                            Some(JaToken::Less) => depth += 1,
                            Some(JaToken::Greater) => depth -= 1,
                            _ => {}
                        }
                        p += 1;
                    }
                }
            }
            _ => return false,
        }
        // Skip array brackets
        while let Some(JaToken::LBracket) = self.tokens.get(p) {
            p += 1;
            if let Some(JaToken::RBracket) = self.tokens.get(p) { p += 1; } else { return false; }
        }
        // Should be Identifier then Colon
        if let Some(JaToken::Identifier(_)) = self.tokens.get(p) {
            p += 1;
            if let Some(JaToken::Colon) = self.tokens.get(p) {
                return true;
            }
        }
        false
    }

    fn parse_switch_stmt(&mut self) -> Result<JaStmt, String> {
        self.consume(&JaToken::LParen, "switch expr start")?;
        let expr = self.parse_expr()?;
        self.consume(&JaToken::RParen, "switch expr end")?;
        self.consume(&JaToken::LBrace, "switch body start")?;
        
        let mut cases = Vec::new();
        while !self.check(&JaToken::RBrace) && !self.check(&JaToken::Eof) {
            let mut labels = Vec::new();
            let mut is_arrow = false;

            if self.match_token(&JaToken::Case) {
                labels.push(self.parse_expr()?);
                while self.match_token(&JaToken::Comma) {
                    labels.push(self.parse_expr()?);
                }
                if self.match_token(&JaToken::Arrow) {
                    is_arrow = true;
                } else {
                    self.consume(&JaToken::Colon, "after case label")?;
                }
            } else if self.match_token(&JaToken::Default) {
                // labels stays empty = default
                if self.match_token(&JaToken::Arrow) {
                    is_arrow = true;
                } else {
                    self.consume(&JaToken::Colon, "after default label")?;
                }
            } else {
                return Err(format!("Expected case or default in switch, got {:?}", self.peek()));
            }

            let mut body = Vec::new();
            if is_arrow {
                if self.check(&JaToken::LBrace) {
                    body.push(JaStmt::Block(self.parse_block()?));
                } else {
                    let e = self.parse_expr()?;
                    self.consume(&JaToken::Semicolon, "after arrow case expr")?;
                    body.push(JaStmt::Expr(e));
                }
            } else {
                while !self.check(&JaToken::Case) && !self.check(&JaToken::Default) 
                    && !self.check(&JaToken::RBrace) && !self.check(&JaToken::Eof) {
                    body.push(self.parse_stmt()?);
                }
            }
            cases.push(JaSwitchCase { labels, is_arrow, body });
        }
        self.consume(&JaToken::RBrace, "switch body end")?;
        Ok(JaStmt::Switch { expr, cases })
    }

    fn parse_try_stmt(&mut self) -> Result<JaStmt, String> {
        // Try-with-resources
        let mut resources = Vec::new();
        if self.match_token(&JaToken::LParen) {
            while !self.check(&JaToken::RParen) {
                let ty = self.parse_type()?;
                let name = self.parse_identifier()?;
                self.consume(&JaToken::Assign, "resource init")?;
                let init_expr = self.parse_expr()?;
                resources.push(JaLocalVarDecl { ty, name, init: Some(init_expr) });
                if !self.match_token(&JaToken::Semicolon) {
                    break;
                }
            }
            self.consume(&JaToken::RParen, "try resources end")?;
        }

        let body = self.parse_block()?;

        let mut catches = Vec::new();
        while self.match_token(&JaToken::Catch) {
            self.consume(&JaToken::LParen, "catch param start")?;
            let mut types = Vec::new();
            types.push(self.parse_type()?);
            while self.match_token(&JaToken::Pipe) {
                types.push(self.parse_type()?);
            }
            let param_name = self.parse_identifier()?;
            self.consume(&JaToken::RParen, "catch param end")?;
            let catch_body = self.parse_block()?;
            catches.push(JaCatchClause { types, param_name, body: catch_body });
        }

        let finally_block = if self.match_token(&JaToken::Finally) {
            Some(self.parse_block()?)
        } else { None };

        Ok(JaStmt::Try { resources, body, catches, finally_block })
    }

    fn is_local_var_decl(&self) -> bool {
        let mut p = self.pos;
        let mut ok = false;
        if let Some(tok) = self.tokens.get(p) {
            match tok {
                JaToken::Int | JaToken::Long | JaToken::Float | JaToken::Double | JaToken::Boolean | JaToken::Char | JaToken::Byte | JaToken::Short => {
                    ok = true; p += 1;
                }
                JaToken::Var => {
                    ok = true; p += 1;
                }
                JaToken::Identifier(_) => {
                    p += 1;
                    while let Some(JaToken::Dot) = self.tokens.get(p) {
                        p += 1;
                        if let Some(JaToken::Identifier(_)) = self.tokens.get(p) { p += 1; } else { return false; }
                    }
                    if let Some(JaToken::Less) = self.tokens.get(p) {
                        let mut depth = 1; p += 1;
                        while depth > 0 && p < self.tokens.len() {
                            match self.tokens.get(p).unwrap() {
                                JaToken::Less => depth += 1,
                                JaToken::Greater => depth -= 1,
                                _ => {}
                            }
                            p += 1;
                        }
                    }
                    ok = true;
                }
                _ => return false,
            }
        }
        if !ok { return false; }
        while let Some(JaToken::LBracket) = self.tokens.get(p) {
            p += 1;
            if let Some(JaToken::RBracket) = self.tokens.get(p) { p += 1; } else { return false; }
        }
        if let Some(JaToken::Identifier(_)) = self.tokens.get(p) { return true; }
        false
    }

    fn parse_expr(&mut self) -> Result<JaExpr, String> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<JaExpr, String> {
        let left = self.parse_ternary()?;
        let assign_op = match self.peek() {
            Some(JaToken::Assign) => Some(JaAssignOp::Assign),
            Some(JaToken::PlusAssign) => Some(JaAssignOp::AddAssign),
            Some(JaToken::MinusAssign) => Some(JaAssignOp::SubAssign),
            Some(JaToken::StarAssign) => Some(JaAssignOp::MulAssign),
            Some(JaToken::SlashAssign) => Some(JaAssignOp::DivAssign),
            Some(JaToken::PercentAssign) => Some(JaAssignOp::RemAssign),
            Some(JaToken::AmpAssign) => Some(JaAssignOp::AndAssign),
            Some(JaToken::PipeAssign) => Some(JaAssignOp::OrAssign),
            Some(JaToken::CaretAssign) => Some(JaAssignOp::XorAssign),
            Some(JaToken::LShiftAssign) => Some(JaAssignOp::ShlAssign),
            Some(JaToken::RShiftAssign) => Some(JaAssignOp::ShrAssign),
            Some(JaToken::URShiftAssign) => Some(JaAssignOp::UShrAssign),
            _ => None,
        };
        if let Some(op) = assign_op {
            self.advance();
            let right = self.parse_assignment()?;
            Ok(JaExpr::Assign { op, target: Box::new(left), value: Box::new(right) })
        } else {
            Ok(left)
        }
    }

    fn parse_ternary(&mut self) -> Result<JaExpr, String> {
        let expr = self.parse_logical_or()?;
        if self.match_token(&JaToken::Question) {
            let true_expr = self.parse_expr()?;
            self.consume(&JaToken::Colon, "ternary ':' expected")?;
            let false_expr = self.parse_ternary()?;
            Ok(JaExpr::Ternary { cond: Box::new(expr), true_expr: Box::new(true_expr), false_expr: Box::new(false_expr) })
        } else {
            Ok(expr)
        }
    }

    fn parse_logical_or(&mut self) -> Result<JaExpr, String> {
        let mut expr = self.parse_logical_and()?;
        while self.match_token(&JaToken::OrOr) {
            let right = self.parse_logical_and()?;
            expr = JaExpr::Binary { op: JaBinOp::Or, left: Box::new(expr), right: Box::new(right) };
        }
        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> Result<JaExpr, String> {
        let mut expr = self.parse_bitwise_or()?;
        while self.match_token(&JaToken::AndAnd) {
            let right = self.parse_bitwise_or()?;
            expr = JaExpr::Binary { op: JaBinOp::And, left: Box::new(expr), right: Box::new(right) };
        }
        Ok(expr)
    }

    fn parse_bitwise_or(&mut self) -> Result<JaExpr, String> {
        let mut expr = self.parse_bitwise_xor()?;
        while self.check(&JaToken::Pipe) {
            self.advance();
            let right = self.parse_bitwise_xor()?;
            expr = JaExpr::Binary { op: JaBinOp::BitOr, left: Box::new(expr), right: Box::new(right) };
        }
        Ok(expr)
    }

    fn parse_bitwise_xor(&mut self) -> Result<JaExpr, String> {
        let mut expr = self.parse_bitwise_and()?;
        while self.match_token(&JaToken::Caret) {
            let right = self.parse_bitwise_and()?;
            expr = JaExpr::Binary { op: JaBinOp::BitXor, left: Box::new(expr), right: Box::new(right) };
        }
        Ok(expr)
    }

    fn parse_bitwise_and(&mut self) -> Result<JaExpr, String> {
        let mut expr = self.parse_equality()?;
        while self.check(&JaToken::Ampersand) {
            self.advance();
            let right = self.parse_equality()?;
            expr = JaExpr::Binary { op: JaBinOp::BitAnd, left: Box::new(expr), right: Box::new(right) };
        }
        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<JaExpr, String> {
        let mut expr = self.parse_relational()?;
        while let Some(tok) = self.peek() {
            let op = match tok {
                JaToken::EqEq => JaBinOp::Eq,
                JaToken::NotEq => JaBinOp::Neq,
                _ => break,
            };
            self.advance();
            let right = self.parse_relational()?;
            expr = JaExpr::Binary { op, left: Box::new(expr), right: Box::new(right) };
        }
        Ok(expr)
    }

    fn parse_relational(&mut self) -> Result<JaExpr, String> {
        let mut expr = self.parse_shift()?;
        loop {
            if let Some(tok) = self.peek() {
                match tok {
                    JaToken::Less | JaToken::Greater | JaToken::LessEq | JaToken::GreaterEq => {
                        let op = match self.peek().unwrap() {
                            JaToken::Less => JaBinOp::Lt,
                            JaToken::Greater => JaBinOp::Gt,
                            JaToken::LessEq => JaBinOp::Le,
                            JaToken::GreaterEq => JaBinOp::Ge,
                            _ => unreachable!(),
                        };
                        self.advance();
                        let right = self.parse_shift()?;
                        expr = JaExpr::Binary { op, left: Box::new(expr), right: Box::new(right) };
                    }
                    JaToken::Instanceof => {
                        self.advance();
                        let ty = self.parse_type()?;
                        let pattern_name = if let Some(JaToken::Identifier(_)) = self.peek() {
                            Some(self.parse_identifier()?)
                        } else { None };
                        expr = JaExpr::Instanceof { expr: Box::new(expr), ty, pattern_name };
                    }
                    _ => break,
                }
            } else { break; }
        }
        Ok(expr)
    }

    fn parse_shift(&mut self) -> Result<JaExpr, String> {
        let mut expr = self.parse_additive()?;
        while let Some(tok) = self.peek() {
            let op = match tok {
                JaToken::LShift => JaBinOp::Shl,
                JaToken::RShift => JaBinOp::Shr,
                JaToken::URShift => JaBinOp::UShr,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive()?;
            expr = JaExpr::Binary { op, left: Box::new(expr), right: Box::new(right) };
        }
        Ok(expr)
    }

    fn parse_additive(&mut self) -> Result<JaExpr, String> {
        let mut expr = self.parse_multiplicative()?;
        while let Some(tok) = self.peek() {
            let op = match tok {
                JaToken::Plus => JaBinOp::Add,
                JaToken::Minus => JaBinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            expr = JaExpr::Binary { op, left: Box::new(expr), right: Box::new(right) };
        }
        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> Result<JaExpr, String> {
        let mut expr = self.parse_unary()?;
        while let Some(tok) = self.peek() {
            let op = match tok {
                JaToken::Star => JaBinOp::Mul,
                JaToken::Slash => JaBinOp::Div,
                JaToken::Percent => JaBinOp::Rem,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            expr = JaExpr::Binary { op, left: Box::new(expr), right: Box::new(right) };
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<JaExpr, String> {
        if self.match_token(&JaToken::Minus) {
            let expr = self.parse_unary()?;
            return Ok(JaExpr::Unary { op: JaUnaryOp::Minus, expr: Box::new(expr), is_postfix: false });
        }
        if self.match_token(&JaToken::Plus) {
            let expr = self.parse_unary()?;
            return Ok(JaExpr::Unary { op: JaUnaryOp::Plus, expr: Box::new(expr), is_postfix: false });
        }
        if self.match_token(&JaToken::Not) {
            let expr = self.parse_unary()?;
            return Ok(JaExpr::Unary { op: JaUnaryOp::Not, expr: Box::new(expr), is_postfix: false });
        }
        if self.match_token(&JaToken::Tilde) {
            let expr = self.parse_unary()?;
            return Ok(JaExpr::Unary { op: JaUnaryOp::BitNot, expr: Box::new(expr), is_postfix: false });
        }
        if self.match_token(&JaToken::PlusPlus) {
            let expr = self.parse_unary()?;
            return Ok(JaExpr::Unary { op: JaUnaryOp::Inc, expr: Box::new(expr), is_postfix: false });
        }
        if self.match_token(&JaToken::MinusMinus) {
            let expr = self.parse_unary()?;
            return Ok(JaExpr::Unary { op: JaUnaryOp::Dec, expr: Box::new(expr), is_postfix: false });
        }

        // Cast: (Type) expr
        if self.check(&JaToken::LParen) && self.is_cast() {
            self.advance(); // consume (
            let ty = self.parse_type()?;
            self.consume(&JaToken::RParen, "cast closing )")?;
            let expr = self.parse_unary()?;
            return Ok(JaExpr::Cast { ty, expr: Box::new(expr) });
        }

        let mut expr = self.parse_primary_or_access()?;
        // Postfix operators
        if self.match_token(&JaToken::PlusPlus) {
            expr = JaExpr::Unary { op: JaUnaryOp::Inc, expr: Box::new(expr), is_postfix: true };
        } else if self.match_token(&JaToken::MinusMinus) {
            expr = JaExpr::Unary { op: JaUnaryOp::Dec, expr: Box::new(expr), is_postfix: true };
        }
        Ok(expr)
    }

    fn is_cast(&self) -> bool {
        // Heuristic: (Type) is a cast if the next token after ( is a primitive or known class type
        // followed by ) and then an expression
        if self.peek() != Some(&JaToken::LParen) { return false; }
        let mut p = self.pos + 1;
        match self.tokens.get(p) {
            Some(JaToken::Int) | Some(JaToken::Long) | Some(JaToken::Float) | Some(JaToken::Double) |
            Some(JaToken::Boolean) | Some(JaToken::Char) | Some(JaToken::Byte) | Some(JaToken::Short) => {
                p += 1;
                // Allow array dimensions
                while let Some(JaToken::LBracket) = self.tokens.get(p) {
                    p += 1;
                    if let Some(JaToken::RBracket) = self.tokens.get(p) { p += 1; } else { return false; }
                }
                if let Some(JaToken::RParen) = self.tokens.get(p) { return true; }
            }
            _ => {}
        }
        false
    }

    fn parse_primary_or_access(&mut self) -> Result<JaExpr, String> {
        let mut expr = match self.peek() {
            Some(JaToken::IntLiteral(_)) => {
                if let JaToken::IntLiteral(v) = self.advance().unwrap() {
                    JaExpr::IntLiteral(*v)
                } else { unreachable!() }
            }
            Some(JaToken::LongLiteral(_)) => {
                if let JaToken::LongLiteral(v) = self.advance().unwrap() {
                    JaExpr::LongLiteral(*v)
                } else { unreachable!() }
            }
            Some(JaToken::FloatLiteral(_)) => {
                if let JaToken::FloatLiteral(v) = self.advance().unwrap() {
                    JaExpr::FloatLiteral(*v)
                } else { unreachable!() }
            }
            Some(JaToken::DoubleLiteral(_)) => {
                if let JaToken::DoubleLiteral(v) = self.advance().unwrap() {
                    JaExpr::DoubleLiteral(*v)
                } else { unreachable!() }
            }
            Some(JaToken::CharLiteral(_)) => {
                if let JaToken::CharLiteral(v) = self.advance().unwrap() {
                    JaExpr::CharLiteral(*v)
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
            Some(JaToken::Super) => { self.advance(); JaExpr::Name("super".to_string()) }
            Some(JaToken::LParen) => {
                self.advance(); // consume (
                let inner = self.parse_expr()?;
                self.consume(&JaToken::RParen, "closing parenthesis")?;
                inner
            }
            Some(JaToken::New) => {
                self.advance();
                
                // Parse base type exactly manually to avoid parse_type consuming empty brackets []
                let ty = match self.peek() {
                    Some(JaToken::Int) => { self.advance(); JaType::Int }
                    Some(JaToken::Long) => { self.advance(); JaType::Long }
                    Some(JaToken::Float) => { self.advance(); JaType::Float }
                    Some(JaToken::Double) => { self.advance(); JaType::Double }
                    Some(JaToken::Boolean) => { self.advance(); JaType::Boolean }
                    Some(JaToken::Char) => { self.advance(); JaType::Char }
                    Some(JaToken::Byte) => { self.advance(); JaType::Byte }
                    Some(JaToken::Short) => { self.advance(); JaType::Short }
                    Some(JaToken::Identifier(_)) => {
                        let name = self.parse_qualified_name()?;
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
                    _ => return Err(format!("Expected type after new, got: {:?}", self.peek())),
                };

                if self.match_token(&JaToken::LBracket) {
                    let mut dimensions = Vec::new();
                    if !self.check(&JaToken::RBracket) {
                        dimensions.push(Some(self.parse_expr()?));
                    } else {
                        dimensions.push(None);
                    }
                    self.consume(&JaToken::RBracket, "closing bracket in array alloc")?;
                    
                    // Allow multi-dimensional empty brackets logically
                    while self.match_token(&JaToken::LBracket) {
                        self.consume(&JaToken::RBracket, "closing bracket empty dimension in array alloc")?;
                        dimensions.push(None);
                    }
                    
                    JaExpr::NewArray { ty, dimensions, init: None }
                } else if self.match_token(&JaToken::LParen) {
                    let mut args = Vec::new();
                    if !self.check(&JaToken::RParen) {
                        args.push(self.parse_expr()?);
                        while self.match_token(&JaToken::Comma) {
                            args.push(self.parse_expr()?);
                        }
                    }
                    self.consume(&JaToken::RParen, "closing paren obj alloc")?;
                    JaExpr::NewObject { ty, args, body: None }
                } else {
                    return Err("Expected [ or ( after new".to_string());
                }
            }
            Some(JaToken::Identifier(_)) => {
                let name = self.parse_identifier()?;
                if self.match_token(&JaToken::LParen) {
                    let mut args = Vec::new();
                    if !self.check(&JaToken::RParen) {
                        args.push(self.parse_expr()?);
                        while self.match_token(&JaToken::Comma) {
                            args.push(self.parse_expr()?);
                        }
                    }
                    self.consume(&JaToken::RParen, "end of implicit method args")?;
                    JaExpr::MethodCall { target: None, name, type_args: vec![], args }
                } else {
                    JaExpr::Name(name)
                }
            }
            _ => return Err(format!("Unimplemented expression parsing at {:?}", self.peek()))
        };

        // Handle suffix operators `.`, method calls `()`, and array indexes `[]`
        loop {
            if self.match_token(&JaToken::Dot) {
                let field_or_method = self.parse_identifier()?;
                if self.match_token(&JaToken::LParen) {
                    let mut args = Vec::new();
                    if !self.check(&JaToken::RParen) {
                        args.push(self.parse_expr()?);
                        while self.match_token(&JaToken::Comma) {
                            args.push(self.parse_expr()?);
                        }
                    }
                    self.consume(&JaToken::RParen, "end of method args")?;
                    expr = JaExpr::MethodCall { target: Some(Box::new(expr)), name: field_or_method, type_args: vec![], args };
                } else {
                    expr = JaExpr::FieldAccess { target: Box::new(expr), field: field_or_method };
                }
            } else if self.match_token(&JaToken::LBracket) {
                let index = self.parse_expr()?;
                self.consume(&JaToken::RBracket, "closing bracket in array access")?;
                expr = JaExpr::ArrayAccess { array: Box::new(expr), index: Box::new(index) };
            } else {
                break;
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

    #[test]
    fn test_parse_do_while() {
        let code = "class A { void f() { do { int x = 1; } while (x > 0); } }";
        let lexer = JaLexer::new(code);
        let mut parser = JaParser::new(lexer);
        let unit = parser.parse_compilation_unit().unwrap();
        assert_eq!(unit.declarations.len(), 1);
    }

    #[test]
    fn test_parse_switch_statement() {
        let code = "class A { void f() { switch (x) { case 1: break; case 2: break; default: break; } } }";
        let lexer = JaLexer::new(code);
        let mut parser = JaParser::new(lexer);
        let unit = parser.parse_compilation_unit().unwrap();
        assert_eq!(unit.declarations.len(), 1);
    }

    #[test]
    fn test_parse_try_catch_finally() {
        let code = "class A { void f() { try { int x = 1; } catch (Exception e) { int y = 2; } finally { int z = 3; } } }";
        let lexer = JaLexer::new(code);
        let mut parser = JaParser::new(lexer);
        let unit = parser.parse_compilation_unit().unwrap();
        assert_eq!(unit.declarations.len(), 1);
    }

    #[test]
    fn test_parse_for_each() {
        let code = "class A { void f() { for (int x : arr) { System.out.println(x); } } }";
        let lexer = JaLexer::new(code);
        let mut parser = JaParser::new(lexer);
        let unit = parser.parse_compilation_unit().unwrap();
        assert_eq!(unit.declarations.len(), 1);
    }

    #[test]
    fn test_parse_ternary() {
        let code = "class A { void f() { int x = a > b ? a : b; } }";
        let lexer = JaLexer::new(code);
        let mut parser = JaParser::new(lexer);
        let unit = parser.parse_compilation_unit().unwrap();
        assert_eq!(unit.declarations.len(), 1);
    }

    #[test]
    fn test_parse_logical_operators() {
        let code = "class A { void f() { boolean r = a && b || c; } }";
        let lexer = JaLexer::new(code);
        let mut parser = JaParser::new(lexer);
        let unit = parser.parse_compilation_unit().unwrap();
        assert_eq!(unit.declarations.len(), 1);
    }

    #[test]
    fn test_parse_bitwise_operators() {
        let code = "class A { void f() { int r = a & b | c ^ d; int s = x << 2; int t = ~y; } }";
        let lexer = JaLexer::new(code);
        let mut parser = JaParser::new(lexer);
        let unit = parser.parse_compilation_unit().unwrap();
        assert_eq!(unit.declarations.len(), 1);
    }

    #[test]
    fn test_parse_instanceof() {
        let code = "class A { void f() { if (x instanceof String s) { return; } } }";
        let lexer = JaLexer::new(code);
        let mut parser = JaParser::new(lexer);
        let unit = parser.parse_compilation_unit().unwrap();
        assert_eq!(unit.declarations.len(), 1);
    }

    #[test]
    fn test_parse_cast() {
        let code = "class A { void f() { int x = (int) 3.14; } }";
        let lexer = JaLexer::new(code);
        let mut parser = JaParser::new(lexer);
        let unit = parser.parse_compilation_unit().unwrap();
        assert_eq!(unit.declarations.len(), 1);
    }

    #[test]
    fn test_parse_type_params() {
        let code = "class Container<T, U extends Comparable> { T value; }";
        let lexer = JaLexer::new(code);
        let mut parser = JaParser::new(lexer);
        let unit = parser.parse_compilation_unit().unwrap();
        match &unit.declarations[0] {
            JaTypeDecl::Class { name, type_params, .. } => {
                assert_eq!(name, "Container");
                assert_eq!(type_params.len(), 2);
                assert_eq!(type_params[0], "T");
                assert_eq!(type_params[1], "U");
            }
            _ => panic!("Expected generic class"),
        }
    }

    #[test]
    fn test_parse_break_continue() {
        let code = "class A { void f() { while (true) { if (x > 0) break; continue; } } }";
        let lexer = JaLexer::new(code);
        let mut parser = JaParser::new(lexer);
        let unit = parser.parse_compilation_unit().unwrap();
        assert_eq!(unit.declarations.len(), 1);
    }

    #[test]
    fn test_parse_decrement() {
        let code = "class A { void f() { int x = 10; x--; --x; } }";
        let lexer = JaLexer::new(code);
        let mut parser = JaParser::new(lexer);
        let unit = parser.parse_compilation_unit().unwrap();
        assert_eq!(unit.declarations.len(), 1);
    }

    #[test]
    fn test_parse_var_decl() {
        let code = "class A { void f() { var x = 42; } }";
        let lexer = JaLexer::new(code);
        let mut parser = JaParser::new(lexer);
        let unit = parser.parse_compilation_unit().unwrap();
        assert_eq!(unit.declarations.len(), 1);
    }

    #[test]
    fn test_parse_enum() {
        let code = "enum Color { RED, GREEN, BLUE }";
        let lexer = JaLexer::new(code);
        let mut parser = JaParser::new(lexer);
        let unit = parser.parse_compilation_unit().unwrap();
        match &unit.declarations[0] {
            JaTypeDecl::Enum { name, constants, .. } => {
                assert_eq!(name, "Color");
                assert_eq!(constants.len(), 3);
            }
            _ => panic!("Expected enum"),
        }
    }

    #[test]
    fn test_parse_all_assign_ops() {
        let code = "class A { void f() { x += 1; x -= 2; x *= 3; x /= 4; x %= 5; x &= 6; x |= 7; x ^= 8; } }";
        let lexer = JaLexer::new(code);
        let mut parser = JaParser::new(lexer);
        let unit = parser.parse_compilation_unit().unwrap();
        assert_eq!(unit.declarations.len(), 1);
    }
}
