// ============================================================
// Java Preprocessor for JaDead-BIB 💀☕
// ============================================================
// 1. Handles BOM and encoding (UTF-8 standard)
// 2. Processes JaDead-BIB compiler directives (e.g. @Native)
// 3. Optional: Initial comment stripping (though lexer handles it too)
// ============================================================

pub struct JaPreprocessor {
    source: String,
}

impl JaPreprocessor {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
        }
    }

    /// Run the preprocessor pipeline
    pub fn process(&mut self) -> Result<String, String> {
        self.remove_bom();
        self.normalize_line_endings();
        self.process_compiler_directives()?;
        
        Ok(self.source.clone())
    }

    /// Removes UTF-8 BOM if present
    fn remove_bom(&mut self) {
        if self.source.starts_with("\u{FEFF}") {
            self.source = self.source[3..].to_string();
        }
    }

    /// Normalize all line endings to \n
    fn normalize_line_endings(&mut self) {
        self.source = self.source.replace("\r\n", "\n").replace("\r", "\n");
    }

    /// Look for specific compiler directives like // @dead-bib:inline
    /// or annotation based hints before passing to Lexer
    fn process_compiler_directives(&mut self) -> Result<(), String> {
        let mut processed_lines = Vec::new();

        for line in self.source.lines() {
            let trimmed = line.trim();
            
            // Example: FastOS explicit memory hinting via comments
            if trimmed.starts_with("// @dead-bib:no-bounds-check") {
                // In a full implementation, this would emit a pragma token to the lexer
                processed_lines.push("/* $PRAGMA_NO_BOUNDS_CHECK$ */");
            } else if trimmed.starts_with("// @dead-bib:inline") {
                processed_lines.push("/* $PRAGMA_FORCE_INLINE$ */");
            } else {
                processed_lines.push(line);
            }
        }

        self.source = processed_lines.join("\n");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bom_removal() {
        let mut pre = JaPreprocessor::new("\u{FEFF}public class Test {}");
        let result = pre.process().unwrap();
        assert_eq!(result, "public class Test {}");
    }

    #[test]
    fn test_pragma_injection() {
        let mut pre = JaPreprocessor::new("// @dead-bib:inline\npublic void test() {}");
        let result = pre.process().unwrap();
        assert!(result.contains("/* $PRAGMA_FORCE_INLINE$ */"));
    }
}
