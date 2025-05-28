// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.
// Contributed by Anthony Grimaldi, 2025

//! Language detection and configuration for syntax highlighting.
//!
//! This module provides functionality to detect programming languages based on
//! file extensions and manage language-specific configuration for syntax highlighting.

use std::collections::HashMap;
use std::path::Path;
use once_cell::sync::Lazy;

/// Supported programming languages for syntax highlighting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    /// Rust programming language
    Rust,
    /// JavaScript programming language
    JavaScript,
    /// TypeScript programming language
    TypeScript,
    /// Python programming language
    Python,
    /// JSON data format
    Json,
    /// HTML markup language
    Html,
    /// CSS stylesheet language
    Css,
    /// Markdown markup language
    Markdown,
    /// YAML data format
    Yaml,
    /// TOML configuration format
    Toml,
    /// SQL query language
    Sql,
    /// Plain text (no highlighting)
    PlainText,
}

impl Language {
    /// Returns the display name of the language.
    pub fn display_name(self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::JavaScript => "JavaScript",
            Language::TypeScript => "TypeScript",
            Language::Python => "Python",
            Language::Json => "JSON",
            Language::Html => "HTML",
            Language::Css => "CSS",
            Language::Markdown => "Markdown",
            Language::Yaml => "YAML",
            Language::Toml => "TOML",
            Language::Sql => "SQL",
            Language::PlainText => "Plain Text",
        }
    }

    /// Returns the primary file extension for this language.
    pub fn primary_extension(self) -> &'static str {
        match self {
            Language::Rust => "rs",
            Language::JavaScript => "js",
            Language::TypeScript => "ts",
            Language::Python => "py",
            Language::Json => "json",
            Language::Html => "html",
            Language::Css => "css",
            Language::Markdown => "md",
            Language::Yaml => "yaml",
            Language::Toml => "toml",
            Language::Sql => "sql",
            Language::PlainText => "txt",
        }
    }

    /// Returns whether this language is supported in the current phase.
    /// Phase 0: Infrastructure only
    /// Phase 1: Tier 1 languages (Rust, JavaScript, Python, JSON)
    /// Phase 2: Tier 2 languages (HTML, CSS, Markdown, YAML, TOML, SQL)
    pub fn is_tier_1(self) -> bool {
        matches!(self, Language::Rust | Language::JavaScript | Language::TypeScript | Language::Python | Language::Json)
    }

    /// Returns whether this language is supported in Phase 2.
    pub fn is_tier_2(self) -> bool {
        matches!(self, Language::Html | Language::Css | Language::Markdown | Language::Yaml | Language::Toml | Language::Sql)
    }
}

/// Configuration for a specific language's syntax highlighting.
#[derive(Debug, Clone)]
pub struct LanguageConfig {
    /// The language this configuration applies to
    pub language: Language,
    /// Whether highlighting is enabled for this language
    pub enabled: bool,
    /// Whether this language supports multiline tokens (like block comments)
    pub supports_multiline: bool,
    /// Tab width for this language (used for indentation-sensitive languages)
    pub tab_width: usize,
}

impl LanguageConfig {
    /// Creates a new language configuration with default settings.
    pub fn new(language: Language) -> Self {
        Self {
            language,
            enabled: true,
            supports_multiline: true,
            tab_width: 4,
        }
    }

    /// Creates a language configuration with highlighting disabled.
    pub fn disabled(language: Language) -> Self {
        Self {
            language,
            enabled: false,
            supports_multiline: true,
            tab_width: 4,
        }
    }
}

/// Global mapping of file extensions to programming languages.
static EXTENSION_MAP: Lazy<HashMap<&'static str, Language>> = Lazy::new(|| {
    let mut map = HashMap::new();
    
    // Tier 1 languages (Phase 1)
    map.insert("rs", Language::Rust);
    map.insert("js", Language::JavaScript);
    map.insert("mjs", Language::JavaScript);
    map.insert("cjs", Language::JavaScript);
    map.insert("jsx", Language::JavaScript);
    map.insert("ts", Language::TypeScript);
    map.insert("tsx", Language::TypeScript);
    map.insert("py", Language::Python);
    map.insert("pyw", Language::Python);
    map.insert("pyi", Language::Python);
    map.insert("json", Language::Json);
    map.insert("jsonc", Language::Json);
    map.insert("json5", Language::Json);
    
    // Tier 2 languages (Phase 2)
    map.insert("html", Language::Html);
    map.insert("htm", Language::Html);
    map.insert("xhtml", Language::Html);
    map.insert("css", Language::Css);
    map.insert("scss", Language::Css);
    map.insert("sass", Language::Css);
    map.insert("less", Language::Css);
    map.insert("md", Language::Markdown);
    map.insert("markdown", Language::Markdown);
    map.insert("mdown", Language::Markdown);
    map.insert("mkd", Language::Markdown);
    map.insert("yaml", Language::Yaml);
    map.insert("yml", Language::Yaml);
    map.insert("toml", Language::Toml);
    map.insert("sql", Language::Sql);
    map.insert("sqlite", Language::Sql);
    map.insert("mysql", Language::Sql);
    map.insert("pgsql", Language::Sql);
    
    // Common text file extensions
    map.insert("txt", Language::PlainText);
    map.insert("text", Language::PlainText);
    
    map
});

/// Language detector that can identify programming languages from file paths.
#[derive(Debug, Default)]
pub struct LanguageDetector {
    /// Manual language overrides for specific files
    overrides: HashMap<String, Language>,
}

impl LanguageDetector {
    /// Creates a new language detector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Detects the programming language from a file path.
    /// 
    /// This function uses the following detection strategy:
    /// 1. Check for manual override
    /// 2. Extract file extension and look up in extension map
    /// 3. Fall back to PlainText if no match found
    /// 
    /// # Arguments
    /// 
    /// * `path` - The file path to analyze
    /// 
    /// # Returns
    /// 
    /// The detected language, or `Language::PlainText` if detection fails.
    pub fn detect_language<P: AsRef<Path>>(&self, path: P) -> Language {
        let path = path.as_ref();
        let path_str = path.to_string_lossy();

        // Check for manual override first
        if let Some(&language) = self.overrides.get(path_str.as_ref()) {
            return language;
        }

        // Extract file extension
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                let ext_lower = ext_str.to_lowercase();
                if let Some(&language) = EXTENSION_MAP.get(ext_lower.as_str()) {
                    return language;
                }
            }
        }

        // Fallback to plain text
        Language::PlainText
    }

    /// Sets a manual language override for a specific file path.
    /// 
    /// # Arguments
    /// 
    /// * `path` - The file path to override
    /// * `language` - The language to use for this file
    pub fn set_language_override<P: AsRef<Path>>(&mut self, path: P, language: Language) {
        let path_str = path.as_ref().to_string_lossy().into_owned();
        self.overrides.insert(path_str, language);
    }

    /// Removes a manual language override for a specific file path.
    /// 
    /// # Arguments
    /// 
    /// * `path` - The file path to remove the override for
    /// 
    /// # Returns
    /// 
    /// The previously set language override, if any.
    pub fn remove_language_override<P: AsRef<Path>>(&mut self, path: P) -> Option<Language> {
        let path_str = path.as_ref().to_string_lossy();
        self.overrides.remove(path_str.as_ref())
    }

    /// Returns all currently registered language overrides.
    pub fn get_overrides(&self) -> &HashMap<String, Language> {
        &self.overrides
    }

    /// Clears all language overrides.
    pub fn clear_overrides(&mut self) {
        self.overrides.clear();
    }

    /// Returns all supported file extensions for a given language.
    /// 
    /// # Arguments
    /// 
    /// * `language` - The language to get extensions for
    /// 
    /// # Returns
    /// 
    /// A vector of file extensions (without the dot) that map to this language.
    pub fn get_extensions_for_language(&self, language: Language) -> Vec<&'static str> {
        EXTENSION_MAP
            .iter()
            .filter_map(|(&ext, &lang)| if lang == language { Some(ext) } else { None })
            .collect()
    }

    /// Returns the total number of supported file extensions.
    pub fn supported_extension_count() -> usize {
        EXTENSION_MAP.len()
    }

    /// Returns all supported languages.
    pub fn supported_languages() -> Vec<Language> {
        let mut languages: Vec<Language> = EXTENSION_MAP.values().copied().collect();
        languages.sort_by_key(|lang| lang.display_name());
        languages.dedup();
        languages
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection_basic() {
        let detector = LanguageDetector::new();
        
        assert_eq!(detector.detect_language("test.rs"), Language::Rust);
        assert_eq!(detector.detect_language("script.js"), Language::JavaScript);
        assert_eq!(detector.detect_language("app.ts"), Language::TypeScript);
        assert_eq!(detector.detect_language("main.py"), Language::Python);
        assert_eq!(detector.detect_language("config.json"), Language::Json);
        assert_eq!(detector.detect_language("readme.txt"), Language::PlainText);
    }

    #[test]
    fn test_language_detection_case_insensitive() {
        let detector = LanguageDetector::new();
        
        assert_eq!(detector.detect_language("TEST.RS"), Language::Rust);
        assert_eq!(detector.detect_language("Script.JS"), Language::JavaScript);
        assert_eq!(detector.detect_language("Config.JSON"), Language::Json);
    }

    #[test]
    fn test_language_detection_complex_paths() {
        let detector = LanguageDetector::new();
        
        assert_eq!(detector.detect_language("/path/to/file.rs"), Language::Rust);
        assert_eq!(detector.detect_language("./relative/path.py"), Language::Python);
        assert_eq!(detector.detect_language("C:\\Windows\\file.js"), Language::JavaScript);
    }

    #[test]
    fn test_language_detection_no_extension() {
        let detector = LanguageDetector::new();
        
        assert_eq!(detector.detect_language("Makefile"), Language::PlainText);
        assert_eq!(detector.detect_language("README"), Language::PlainText);
        assert_eq!(detector.detect_language("noext"), Language::PlainText);
    }

    #[test]
    fn test_language_overrides() {
        let mut detector = LanguageDetector::new();
        
        // Initially should detect as plain text
        assert_eq!(detector.detect_language("special_file"), Language::PlainText);
        
        // Set override to Rust
        detector.set_language_override("special_file", Language::Rust);
        assert_eq!(detector.detect_language("special_file"), Language::Rust);
        
        // Remove override
        let removed = detector.remove_language_override("special_file");
        assert_eq!(removed, Some(Language::Rust));
        assert_eq!(detector.detect_language("special_file"), Language::PlainText);
    }

    #[test]
    fn test_language_config() {
        let config = LanguageConfig::new(Language::Rust);
        assert_eq!(config.language, Language::Rust);
        assert!(config.enabled);
        assert!(config.supports_multiline);
        assert_eq!(config.tab_width, 4);

        let disabled_config = LanguageConfig::disabled(Language::Python);
        assert_eq!(disabled_config.language, Language::Python);
        assert!(!disabled_config.enabled);
    }

    #[test]
    fn test_extension_mapping_completeness() {
        // Ensure we have at least 15+ extensions as required by Phase 0
        assert!(LanguageDetector::supported_extension_count() >= 15);
        
        // Test some specific mappings
        let detector = LanguageDetector::new();
        assert_eq!(detector.detect_language("app.jsx"), Language::JavaScript);
        assert_eq!(detector.detect_language("types.tsx"), Language::TypeScript);
        assert_eq!(detector.detect_language("config.toml"), Language::Toml);
        assert_eq!(detector.detect_language("data.yaml"), Language::Yaml);
    }

    #[test]
    fn test_language_tier_classification() {
        assert!(Language::Rust.is_tier_1());
        assert!(Language::JavaScript.is_tier_1());
        assert!(Language::Python.is_tier_1());
        assert!(Language::Json.is_tier_1());
        
        assert!(Language::Html.is_tier_2());
        assert!(Language::Css.is_tier_2());
        assert!(Language::Markdown.is_tier_2());
        assert!(Language::Yaml.is_tier_2());
        
        assert!(!Language::PlainText.is_tier_1());
        assert!(!Language::PlainText.is_tier_2());
    }

    #[test]
    fn test_supported_languages() {
        let languages = LanguageDetector::supported_languages();
        assert!(!languages.is_empty());
        
        // Check that we have all expected tier 1 languages
        assert!(languages.contains(&Language::Rust));
        assert!(languages.contains(&Language::JavaScript));
        assert!(languages.contains(&Language::Python));
        assert!(languages.contains(&Language::Json));
    }
}
