// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Maps syntax highlighting tokens to terminal colors.
//!
//! This module provides the mapping from Synoptic token types to Edit's 
//! IndexedColor system, with support for different color themes and 
//! terminal capabilities.

use crate::framebuffer::IndexedColor;
use std::collections::HashMap;

/// Maps token types to colors for syntax highlighting.
#[derive(Debug, Clone)]
pub struct ColorMapper {
    /// Mapping from token type to color
    token_colors: HashMap<String, IndexedColor>,
    /// Whether to use 256-color mode (vs 16-color mode)
    use_256_colors: bool,
}

impl Default for ColorMapper {
    fn default() -> Self {
        Self::new(true)
    }
}

impl ColorMapper {
    /// Creates a new color mapper with the default theme.
    pub fn new(use_256_colors: bool) -> Self {
        let mut mapper = Self {
            token_colors: HashMap::new(),
            use_256_colors,
        };
        mapper.load_default_theme();
        mapper
    }

    /// Loads the default color theme.
    fn load_default_theme(&mut self) {
        if self.use_256_colors {
            // 256-color theme with rich colors
            self.token_colors.insert("keyword".to_string(), IndexedColor::Blue);
            self.token_colors.insert("type".to_string(), IndexedColor::Cyan);
            self.token_colors.insert("string".to_string(), IndexedColor::Green);
            self.token_colors.insert("comment".to_string(), IndexedColor::BrightBlack);
            self.token_colors.insert("number".to_string(), IndexedColor::Magenta);
            self.token_colors.insert("boolean".to_string(), IndexedColor::Magenta);
            self.token_colors.insert("attribute".to_string(), IndexedColor::Yellow);
            self.token_colors.insert("builtin".to_string(), IndexedColor::BrightCyan);
            self.token_colors.insert("decorator".to_string(), IndexedColor::BrightYellow);
            self.token_colors.insert("regex".to_string(), IndexedColor::Red);
            self.token_colors.insert("operator".to_string(), IndexedColor::White);
            self.token_colors.insert("punctuation".to_string(), IndexedColor::BrightBlack);
            self.token_colors.insert("function".to_string(), IndexedColor::BrightBlue);
            self.token_colors.insert("variable".to_string(), IndexedColor::White);
            self.token_colors.insert("constant".to_string(), IndexedColor::BrightMagenta);
            self.token_colors.insert("error".to_string(), IndexedColor::BrightRed);
        } else {
            // 16-color theme for basic terminals
            self.token_colors.insert("keyword".to_string(), IndexedColor::Blue);
            self.token_colors.insert("type".to_string(), IndexedColor::Cyan);
            self.token_colors.insert("string".to_string(), IndexedColor::Green);
            self.token_colors.insert("comment".to_string(), IndexedColor::BrightBlack);
            self.token_colors.insert("number".to_string(), IndexedColor::Yellow);
            self.token_colors.insert("boolean".to_string(), IndexedColor::Yellow);
            self.token_colors.insert("attribute".to_string(), IndexedColor::Yellow);
            self.token_colors.insert("builtin".to_string(), IndexedColor::Cyan);
            self.token_colors.insert("decorator".to_string(), IndexedColor::Yellow);
            self.token_colors.insert("regex".to_string(), IndexedColor::Red);
            self.token_colors.insert("operator".to_string(), IndexedColor::White);
            self.token_colors.insert("punctuation".to_string(), IndexedColor::White);
            self.token_colors.insert("function".to_string(), IndexedColor::Blue);
            self.token_colors.insert("variable".to_string(), IndexedColor::White);
            self.token_colors.insert("constant".to_string(), IndexedColor::Yellow);
            self.token_colors.insert("error".to_string(), IndexedColor::Red);
        }
    }

    /// Gets the color for a given token type.
    pub fn get_color(&self, token_type: &str) -> IndexedColor {
        self.token_colors
            .get(token_type)
            .copied()
            .unwrap_or(IndexedColor::White)
    }

    /// Sets a custom color for a token type.
    pub fn set_color(&mut self, token_type: String, color: IndexedColor) {
        self.token_colors.insert(token_type, color);
    }

    /// Resets the color mapping to the default theme.
    pub fn reset_to_default(&mut self) {
        self.token_colors.clear();
        self.load_default_theme();
    }

    /// Returns whether 256-color mode is enabled.
    pub fn is_256_color_mode(&self) -> bool {
        self.use_256_colors
    }

    /// Sets whether to use 256-color mode.
    pub fn set_256_color_mode(&mut self, use_256_colors: bool) {
        if self.use_256_colors != use_256_colors {
            self.use_256_colors = use_256_colors;
            self.reset_to_default();
        }
    }

    /// Gets all configured token types.
    pub fn token_types(&self) -> Vec<&String> {
        self.token_colors.keys().collect()
    }

    /// Loads a custom theme from a configuration.
    pub fn load_theme(&mut self, theme: HashMap<String, IndexedColor>) {
        self.token_colors = theme;
    }

    /// Exports the current theme as a configuration.
    pub fn export_theme(&self) -> HashMap<String, IndexedColor> {
        self.token_colors.clone()
    }
}

use once_cell::sync::Lazy;
use std::sync::Mutex;

/// Global color mapper instance for syntax highlighting.
static COLOR_MAPPER: Lazy<Mutex<ColorMapper>> = Lazy::new(|| {
    Mutex::new(ColorMapper::default())
});

/// Gets the global color mapper instance.
pub fn global_color_mapper() -> std::sync::MutexGuard<'static, ColorMapper> {
    COLOR_MAPPER.lock().unwrap()
}

/// Gets a mutable reference to the global color mapper instance.
/// This is the same as global_color_mapper() since MutexGuard provides mutable access.
pub fn global_color_mapper_mut() -> std::sync::MutexGuard<'static, ColorMapper> {
    COLOR_MAPPER.lock().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_colors() {
        let mapper = ColorMapper::new(true);
        
        assert_eq!(mapper.get_color("keyword"), IndexedColor::Blue);
        assert_eq!(mapper.get_color("string"), IndexedColor::Green);
        assert_eq!(mapper.get_color("comment"), IndexedColor::BrightBlack);
        assert_eq!(mapper.get_color("unknown"), IndexedColor::White);
    }

    #[test]
    fn test_custom_colors() {
        let mut mapper = ColorMapper::new(true);
        
        mapper.set_color("keyword".to_string(), IndexedColor::BrightRed);
        assert_eq!(mapper.get_color("keyword"), IndexedColor::BrightRed);
    }

    #[test]
    fn test_16_color_mode() {
        let mapper = ColorMapper::new(false);
        
        // In 16-color mode, some colors should be simplified
        assert_eq!(mapper.get_color("number"), IndexedColor::Yellow);
    }

    #[test]
    fn test_theme_export_import() {
        let mut mapper = ColorMapper::new(true);
        mapper.set_color("custom".to_string(), IndexedColor::BrightBlue);
        
        let theme = mapper.export_theme();
        
        let mut mapper2 = ColorMapper::new(true);
        mapper2.load_theme(theme);
        
        assert_eq!(mapper2.get_color("custom"), IndexedColor::BrightBlue);
    }
}