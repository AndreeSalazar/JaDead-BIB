// ============================================================
// Java Lexer for JaDead-BIB 💀☕
// ============================================================
// Tokenizes Java source code into JaToken stream
// Full syntax up to Java 21+
// Pure Rust — no external dependencies
// ============================================================

/// Java token types
#[derive(Debug, Clone, PartialEq)]
pub enum JaToken {
    // ── Keywords ──────────────────────────────────────────
    Abstract, Assert, Boolean, Break, Byte, Case, Catch, Char, Class, Const,
    Continue, Default, Do, Double, Else, Enum, Extends, Final, Finally, Float,
    For, Goto, If, Implements, Import, Instanceof, Int, Interface, Long, Native,
    New, Package, Private, Protected, Public, Return, Short, Static, Strictfp,
    Super, Switch, Synchronized, This, Throw, Throws, Transient, Try, Void,
    Volatile, While,

    // ── Modern Java Keywords (Contextual & Reserved) ──────
    Record,     // Java 16+
    Sealed,     // Java 17+
    Permits,    // Java 17+
    Yield,      // Java 14+
    Var,        // Java 10+
    Exports, Module, Opens, Provides, Requires, To, Uses, With, // Java 9 Modules

    // ── Literals ──────────────────────────────────────────
    Identifier(String),
    IntLiteral(i64),
    LongLiteral(i64),
    FloatLiteral(f64),
    DoubleLiteral(f64),
    CharLiteral(char),
    StringLiteral(String),
    TextBlock(String),   // Java 15+ """..."""
    True,
    False,
    Null,

    // ── Operators ────────────────────────────────────────
    Assign,          // =
    Plus,            // +
    Minus,           // -
    Star,            // *
    Slash,           // /
    Percent,         // %
    PlusPlus,        // ++
    MinusMinus,      // --
    EqEq,            // ==
    NotEq,           // !=
    Greater,         // >
    Less,            // <
    GreaterEq,       // >=
    LessEq,          // <=
    Not,             // !
    AndAnd,          // &&
    OrOr,            // ||
    Ampersand,       // &
    Pipe,            // |
    Caret,           // ^
    Tilde,           // ~
    LShift,          // <<
    RShift,          // >>
    URShift,         // >>>

    // ── Assignment Operators ─────────────────────────────
    PlusAssign,      // +=
    MinusAssign,     // -=
    StarAssign,      // *=
    SlashAssign,     // /=
    PercentAssign,   // %=
    AmpAssign,       // &=
    PipeAssign,      // |=
    CaretAssign,     // ^=
    LShiftAssign,    // <<=
    RShiftAssign,    // >>=
    URShiftAssign,   // >>>=

    // ── Delimiters & Specials ────────────────────────────
    LParen,          // (
    RParen,          // )
    LBrace,          // {
    RBrace,          // }
    LBracket,        // [
    RBracket,        // ]
    Semicolon,       // ;
    Comma,           // ,
    Dot,             // .
    Arrow,           // -> (Lambda)
    Colon,           // :
    DoubleColon,     // :: (Method Ref)
    Ellipsis,        // ... (Varargs)
    At,              // @ (Annotations)
    Question,        // ? (Ternary / Wildcard generics)

    // ── Control ──────────────────────────────────────────
    Eof,
}

/// Java Lexer — whitespace and comment ignoring tokenizer
pub struct JaLexer {
    source: Vec<char>,
    pos: usize,
    pub line: usize,
    pub col: usize,
}

impl JaLexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 0,
        }
    }

    /// Tokenize entire source into token stream
    pub fn tokenize(&mut self) -> Vec<JaToken> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token();
            if tok == JaToken::Eof {
                tokens.push(JaToken::Eof);
                break;
            }
            tokens.push(tok);
        }
        tokens
    }

    /// Get next token
    pub fn next_token(&mut self) -> JaToken {
        self.skip_whitespace_and_comments();

        if self.pos >= self.source.len() {
            return JaToken::Eof;
        }

        let ch = self.source[self.pos];

        // Text blocks (Java 15+)
        if ch == '"' && self.peek_at(1) == Some('"') && self.peek_at(2) == Some('"') {
            return self.read_text_block();
        }

        // String Literals
        if ch == '"' {
            return self.read_string();
        }

        // Char Literals
        if ch == '\'' {
            return self.read_char();
        }

        // Numbers (Int, Long, Float, Double)
        if ch.is_ascii_digit() || (ch == '.' && self.peek_at(1).map_or(false, |c| c.is_ascii_digit())) {
            return self.read_number();
        }

        // Identifiers and Keywords
        if ch.is_ascii_alphabetic() || ch == '_' || ch == '$' {
            return self.read_identifier();
        }

        // Operators and Delimiters
        self.read_operator()
    }

    // ── Helpers ──────────────────────────────────────────

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.source.get(self.pos + offset).copied()
    }

    fn advance(&mut self) -> char {
        let ch = self.source[self.pos];
        if ch == '\n' {
            self.line += 1;
            self.col = 0;
        } else {
            self.col += 1;
        }
        self.pos += 1;
        ch
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            if self.pos >= self.source.len() { break; }
            let ch = self.source[self.pos];

            if ch.is_whitespace() {
                self.advance();
            } else if ch == '/' && self.peek_at(1) == Some('/') {
                // Line comment
                while self.pos < self.source.len() && self.source[self.pos] != '\n' {
                    self.advance();
                }
            } else if ch == '/' && self.peek_at(1) == Some('*') {
                // Block comment
                self.advance(); // /
                self.advance(); // *
                while self.pos < self.source.len() {
                    if self.source[self.pos] == '*' && self.peek_at(1) == Some('/') {
                        self.advance(); // *
                        self.advance(); // /
                        break;
                    }
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    fn read_text_block(&mut self) -> JaToken {
        self.advance(); // "
        self.advance(); // "
        self.advance(); // "
        
        // Skip optional newline immediately following """
        if self.pos < self.source.len() && self.source[self.pos] == '\n' {
            self.advance();
        }

        let mut content = String::new();
        while self.pos < self.source.len() {
            if self.source[self.pos] == '"' && self.peek_at(1) == Some('"') && self.peek_at(2) == Some('"') {
                self.advance(); // "
                self.advance(); // "
                self.advance(); // "
                break;
            }
            content.push(self.advance());
        }
        JaToken::TextBlock(content)
    }

    fn read_string(&mut self) -> JaToken {
        self.advance(); // consume "
        let mut content = String::new();
        while self.pos < self.source.len() && self.source[self.pos] != '"' {
            if self.source[self.pos] == '\\' && self.pos + 1 < self.source.len() {
                self.advance();
                let escaped = self.advance();
                match escaped {
                    'n' => content.push('\n'),
                    't' => content.push('\t'),
                    'r' => content.push('\r'),
                    '\\' => content.push('\\'),
                    '"' => content.push('"'),
                    _ => { content.push('\\'); content.push(escaped); }
                }
            } else {
                content.push(self.advance());
            }
        }
        if self.pos < self.source.len() {
            self.advance(); // consume "
        }
        JaToken::StringLiteral(content)
    }

    fn read_char(&mut self) -> JaToken {
        self.advance(); // consume '
        let mut c = '\0';
        if self.pos < self.source.len() {
            if self.source[self.pos] == '\\' && self.pos + 1 < self.source.len() {
                self.advance();
                let escaped = self.advance();
                c = match escaped {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '\\' => '\\',
                    '\'' => '\'',
                    _ => escaped
                };
            } else {
                c = self.advance();
            }
        }
        if self.pos < self.source.len() && self.source[self.pos] == '\'' {
            self.advance(); // consume '
        }
        JaToken::CharLiteral(c)
    }

    fn read_number(&mut self) -> JaToken {
        let start = self.pos;
        let mut is_hex = false;
        let mut is_float = false;

        // Hex prefix
        if self.source[self.pos] == '0' && matches!(self.peek_at(1), Some('x') | Some('X')) {
            is_hex = true;
            self.advance();
            self.advance();
        }

        while self.pos < self.source.len() {
            let c = self.source[self.pos];
            if is_hex && c.is_ascii_hexdigit() || c == '_' {
                self.advance();
            } else if !is_hex && (c.is_ascii_digit() || c == '_') {
                self.advance();
            } else if !is_hex && c == '.' {
                is_float = true;
                self.advance();
            } else if !is_hex && matches!(c, 'e' | 'E') {
                is_float = true;
                self.advance();
                if matches!(self.peek_at(0), Some('+') | Some('-')) {
                    self.advance();
                }
            } else {
                break;
            }
        }

        let mut suffix = ' ';
        if self.pos < self.source.len() && matches!(self.source[self.pos], 'l' | 'L' | 'f' | 'F' | 'd' | 'D') {
            suffix = self.advance().to_ascii_uppercase();
        }

        let s: String = self.source[start..self.pos].iter().filter(|c| **c != '_').collect();

        if is_float || suffix == 'F' || suffix == 'D' {
            let mut num_str = s;
            if num_str.ends_with('F') || num_str.ends_with('D') {
                num_str.pop();
            }
            let val = num_str.parse::<f64>().unwrap_or(0.0);
            if suffix == 'F' {
                JaToken::FloatLiteral(val)
            } else {
                JaToken::DoubleLiteral(val)
            }
        } else {
            let mut num_str = s;
            if num_str.ends_with('L') {
                num_str.pop();
            }
            
            let radix = if is_hex { 16 } else if num_str.starts_with('0') && num_str.len() > 1 && !num_str.contains('.') { 8 } else { 10 };
            let clean_str = if is_hex { &num_str[2..] } else { &num_str };
            
            let val = i64::from_str_radix(clean_str, radix).unwrap_or(0);
            if suffix == 'L' {
                JaToken::LongLiteral(val)
            } else {
                JaToken::IntLiteral(val)
            }
        }
    }

    fn read_identifier(&mut self) -> JaToken {
        let start = self.pos;
        while self.pos < self.source.len()
            && (self.source[self.pos].is_ascii_alphanumeric() || self.source[self.pos] == '_' || self.source[self.pos] == '$')
        {
            self.advance();
        }
        let word: String = self.source[start..self.pos].iter().collect();

        match word.as_str() {
            "abstract" => JaToken::Abstract, "assert" => JaToken::Assert, "boolean" => JaToken::Boolean,
            "break" => JaToken::Break, "byte" => JaToken::Byte, "case" => JaToken::Case, "catch" => JaToken::Catch,
            "char" => JaToken::Char, "class" => JaToken::Class, "const" => JaToken::Const, "continue" => JaToken::Continue,
            "default" => JaToken::Default, "do" => JaToken::Do, "double" => JaToken::Double, "else" => JaToken::Else,
            "enum" => JaToken::Enum, "extends" => JaToken::Extends, "final" => JaToken::Final, "finally" => JaToken::Finally,
            "float" => JaToken::Float, "for" => JaToken::For, "goto" => JaToken::Goto, "if" => JaToken::If,
            "implements" => JaToken::Implements, "import" => JaToken::Import, "instanceof" => JaToken::Instanceof,
            "int" => JaToken::Int, "interface" => JaToken::Interface, "long" => JaToken::Long, "native" => JaToken::Native,
            "new" => JaToken::New, "package" => JaToken::Package, "private" => JaToken::Private, "protected" => JaToken::Protected,
            "public" => JaToken::Public, "return" => JaToken::Return, "short" => JaToken::Short, "static" => JaToken::Static,
            "strictfp" => JaToken::Strictfp, "super" => JaToken::Super, "switch" => JaToken::Switch, "synchronized" => JaToken::Synchronized,
            "this" => JaToken::This, "throw" => JaToken::Throw, "throws" => JaToken::Throws, "transient" => JaToken::Transient,
            "try" => JaToken::Try, "void" => JaToken::Void, "volatile" => JaToken::Volatile, "while" => JaToken::While,
            "record" => JaToken::Record, "sealed" => JaToken::Sealed, "permits" => JaToken::Permits, "yield" => JaToken::Yield,
            "var" => JaToken::Var, "exports" => JaToken::Exports, "module" => JaToken::Module, "opens" => JaToken::Opens,
            "provides" => JaToken::Provides, "requires" => JaToken::Requires, "to" => JaToken::To, "uses" => JaToken::Uses,
            "with" => JaToken::With, "true" => JaToken::True, "false" => JaToken::False, "null" => JaToken::Null,
            _ => JaToken::Identifier(word),
        }
    }

    fn read_operator(&mut self) -> JaToken {
        let ch = self.advance();
        match ch {
            '(' => JaToken::LParen,
            ')' => JaToken::RParen,
            '{' => JaToken::LBrace,
            '}' => JaToken::RBrace,
            '[' => JaToken::LBracket,
            ']' => JaToken::RBracket,
            ';' => JaToken::Semicolon,
            ',' => JaToken::Comma,
            '@' => JaToken::At,
            '~' => JaToken::Tilde,
            '?' => JaToken::Question,
            ':' => {
                if self.peek_at(0) == Some(':') { self.advance(); JaToken::DoubleColon }
                else { JaToken::Colon }
            }
            '.' => {
                if self.peek_at(0) == Some('.') && self.peek_at(1) == Some('.') {
                    self.advance(); self.advance(); JaToken::Ellipsis
                } else { JaToken::Dot }
            }
            '+' => {
                if self.peek_at(0) == Some('+') { self.advance(); JaToken::PlusPlus }
                else if self.peek_at(0) == Some('=') { self.advance(); JaToken::PlusAssign }
                else { JaToken::Plus }
            }
            '-' => {
                if self.peek_at(0) == Some('-') { self.advance(); JaToken::MinusMinus }
                else if self.peek_at(0) == Some('>') { self.advance(); JaToken::Arrow }
                else if self.peek_at(0) == Some('=') { self.advance(); JaToken::MinusAssign }
                else { JaToken::Minus }
            }
            '*' => {
                if self.peek_at(0) == Some('=') { self.advance(); JaToken::StarAssign }
                else { JaToken::Star }
            }
            '/' => {
                if self.peek_at(0) == Some('=') { self.advance(); JaToken::SlashAssign }
                else { JaToken::Slash }
            }
            '%' => {
                if self.peek_at(0) == Some('=') { self.advance(); JaToken::PercentAssign }
                else { JaToken::Percent }
            }
            '=' => {
                if self.peek_at(0) == Some('=') { self.advance(); JaToken::EqEq }
                else { JaToken::Assign }
            }
            '!' => {
                if self.peek_at(0) == Some('=') { self.advance(); JaToken::NotEq }
                else { JaToken::Not }
            }
            '<' => {
                if self.peek_at(0) == Some('<') {
                    self.advance();
                    if self.peek_at(0) == Some('=') { self.advance(); JaToken::LShiftAssign }
                    else { JaToken::LShift }
                } else if self.peek_at(0) == Some('=') { self.advance(); JaToken::LessEq }
                else { JaToken::Less }
            }
            '>' => {
                if self.peek_at(0) == Some('>') {
                    self.advance();
                    if self.peek_at(0) == Some('>') {
                        self.advance();
                        if self.peek_at(0) == Some('=') { self.advance(); JaToken::URShiftAssign }
                        else { JaToken::URShift }
                    } else if self.peek_at(0) == Some('=') { self.advance(); JaToken::RShiftAssign }
                    else { JaToken::RShift }
                } else if self.peek_at(0) == Some('=') { self.advance(); JaToken::GreaterEq }
                else { JaToken::Greater }
            }
            '&' => {
                if self.peek_at(0) == Some('&') { self.advance(); JaToken::AndAnd }
                else if self.peek_at(0) == Some('=') { self.advance(); JaToken::AmpAssign }
                else { JaToken::Ampersand }
            }
            '|' => {
                if self.peek_at(0) == Some('|') { self.advance(); JaToken::OrOr }
                else if self.peek_at(0) == Some('=') { self.advance(); JaToken::PipeAssign }
                else { JaToken::Pipe }
            }
            '^' => {
                if self.peek_at(0) == Some('=') { self.advance(); JaToken::CaretAssign }
                else { JaToken::Caret }
            }
            _ => JaToken::Identifier(ch.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let mut lexer = JaLexer::new("int x = 42;");
        let tokens = lexer.tokenize();
        assert!(tokens.contains(&JaToken::Int));
        assert!(tokens.contains(&JaToken::Identifier("x".to_string())));
        assert!(tokens.contains(&JaToken::Assign));
        assert!(tokens.contains(&JaToken::IntLiteral(42)));
        assert!(tokens.contains(&JaToken::Semicolon));
    }

    #[test]
    fn test_modern_features() {
        let mut lexer = JaLexer::new("var r = new Record(1, 2);");
        let tokens = lexer.tokenize();
        assert!(tokens.contains(&JaToken::Var));
        assert!(tokens.contains(&JaToken::New));
    }

    #[test]
    fn test_text_blocks() {
        let mut lexer = JaLexer::new("\"\"\"\nHello \n World!\"\"\"");
        let tokens = lexer.tokenize();
        assert!(tokens.iter().any(|t| matches!(t, JaToken::TextBlock(s) if s.contains("Hello \n World!"))));
    }

    #[test]
    fn test_lambda_arrows() {
        let mut lexer = JaLexer::new("(x) -> x * 2");
        let tokens = lexer.tokenize();
        assert!(tokens.contains(&JaToken::Arrow));
    }
}
