// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Syntax highlighting service and infrastructure.
//!
//! This module provides the core syntax highlighting functionality using Synoptic,
//! including lazy initialization, token processing, and performance monitoring.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::{Duration, Instant};
use once_cell::sync::Lazy;
use synoptic::Highlighter;

use crate::syntax::language::{Language, LanguageConfig, LanguageDetector};

/// Information about a highlighted token in the document.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenInfo {
    /// The text content of the token
    pub text: String,
    /// The token type/kind for color mapping
    pub kind: Option<String>,
    /// The byte offset where this token starts in the line
    pub start_offset: usize,
    /// The byte offset where this token ends in the line
    pub end_offset: usize,
}

impl TokenInfo {
    /// Creates a new token info with the given parameters.
    pub fn new(text: String, kind: Option<String>, start_offset: usize, end_offset: usize) -> Self {
        Self {
            text,
            kind,
            start_offset,
            end_offset,
        }
    }

    /// Creates a token info for plain text (no highlighting).
    pub fn plain_text(text: String, start_offset: usize, end_offset: usize) -> Self {
        Self::new(text, None, start_offset, end_offset)
    }

    /// Creates a token info for highlighted text.
    pub fn highlighted(text: String, kind: String, start_offset: usize, end_offset: usize) -> Self {
        Self::new(text, Some(kind), start_offset, end_offset)
    }

    /// Returns true if this token should be highlighted.
    pub fn is_highlighted(&self) -> bool {
        self.kind.is_some()
    }

    /// Returns the length of the token text in bytes.
    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// Returns true if the token is empty.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

/// Performance metrics for syntax highlighting operations.
#[derive(Debug, Clone, Default)]
pub struct HighlightingMetrics {
    /// Total time spent on highlighting operations
    pub total_time: Duration,
    /// Number of lines highlighted
    pub lines_highlighted: usize,
    /// Number of tokens generated
    pub tokens_generated: usize,
    /// Average time per line
    pub avg_time_per_line: Duration,
    /// Maximum time for a single line
    pub max_line_time: Duration,
    /// Number of highlighting cache hits
    pub cache_hits: usize,
    /// Number of highlighting cache misses
    pub cache_misses: usize,
}

impl HighlightingMetrics {
    /// Updates metrics with a new line highlighting operation.
    pub fn record_line_highlight(&mut self, duration: Duration, token_count: usize) {
        self.total_time += duration;
        self.lines_highlighted += 1;
        self.tokens_generated += token_count;
        
        if self.lines_highlighted > 0 {
            self.avg_time_per_line = self.total_time / self.lines_highlighted as u32;
        }
        
        if duration > self.max_line_time {
            self.max_line_time = duration;
        }
    }

    /// Records a cache hit.
    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }

    /// Records a cache miss.
    pub fn record_cache_miss(&mut self) {
        self.cache_misses += 1;
    }

    /// Returns the cache hit ratio (0.0 to 1.0).
    pub fn cache_hit_ratio(&self) -> f64 {
        let total_requests = self.cache_hits + self.cache_misses;
        if total_requests == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total_requests as f64
        }
    }

    /// Resets all metrics to zero.
    pub fn reset(&mut self) {
        *self = Default::default();
    }
}

/// State information for syntax highlighting of a document.
#[derive(Debug, Clone)]
pub struct HighlightingState {
    /// The detected language for this document
    pub language: Language,
    /// Configuration for this language
    pub config: LanguageConfig,
    /// Whether highlighting is currently enabled
    pub enabled: bool,
    /// Performance metrics for this document
    pub metrics: HighlightingMetrics,
    /// Cache of highlighted tokens per line (line_number -> tokens)
    token_cache: HashMap<usize, Vec<TokenInfo>>,
    /// Cache validity tracking (line_number -> content_hash)
    cache_validity: HashMap<usize, u64>,
    /// Track which lines need re-highlighting
    dirty_lines: HashSet<usize>,
    /// Track if entire document needs re-highlighting
    needs_full_rehighlight: bool,
    /// Current viewport information (start line, end line)
    viewport: Option<(usize, usize)>,
    /// Background highlighting queue (ordered by priority)
    background_queue: Vec<usize>,
    /// Lines currently being highlighted in background
    background_in_progress: HashSet<usize>,
    /// Maximum number of lines to highlight per background processing cycle
    background_batch_size: usize,
    /// Distance from viewport to pre-highlight (lines above and below)
    background_lookahead: usize,
}

impl HighlightingState {
    /// Creates a new highlighting state for the given language.
    pub fn new(language: Language) -> Self {
        Self {
            language,
            config: LanguageConfig::new(language),
            enabled: true,
            metrics: HighlightingMetrics::default(),
            token_cache: HashMap::new(),
            cache_validity: HashMap::new(),
            dirty_lines: HashSet::new(),
            needs_full_rehighlight: false,
            viewport: None,
            background_queue: Vec::new(),
            background_in_progress: HashSet::new(),
            background_batch_size: 10, // Process 10 lines per background cycle
            background_lookahead: 50,  // Pre-highlight 50 lines ahead/behind viewport
        }
    }

    /// Creates a highlighting state with highlighting disabled.
    pub fn disabled(language: Language) -> Self {
        Self {
            language,
            config: LanguageConfig::disabled(language),
            enabled: false,
            metrics: HighlightingMetrics::default(),
            token_cache: HashMap::new(),
            cache_validity: HashMap::new(),
            dirty_lines: HashSet::new(),
            needs_full_rehighlight: false,
            viewport: None,
            background_queue: Vec::new(),
            background_in_progress: HashSet::new(),
            background_batch_size: 10,
            background_lookahead: 50,
        }
    }

    /// Checks if tokens are cached for the given line with the given content hash.
    pub fn has_cached_tokens(&self, line_number: usize, content_hash: u64) -> bool {
        self.cache_validity.get(&line_number) == Some(&content_hash) &&
        self.token_cache.contains_key(&line_number)
    }

    /// Gets cached tokens for the given line.
    pub fn get_cached_tokens(&self, line_number: usize) -> Option<&Vec<TokenInfo>> {
        self.token_cache.get(&line_number)
    }

    /// Caches tokens for the given line with the given content hash.
    pub fn cache_tokens(&mut self, line_number: usize, content_hash: u64, tokens: Vec<TokenInfo>) {
        self.token_cache.insert(line_number, tokens);
        self.cache_validity.insert(line_number, content_hash);
    }

    /// Invalidates cache for the given line.
    pub fn invalidate_line_cache(&mut self, line_number: usize) {
        self.token_cache.remove(&line_number);
        self.cache_validity.remove(&line_number);
    }

    /// Invalidates cache for a range of lines.
    pub fn invalidate_line_range_cache(&mut self, start_line: usize, end_line: usize) {
        for line_number in start_line..=end_line {
            self.invalidate_line_cache(line_number);
        }
    }

    /// Clears all cached tokens.
    pub fn clear_cache(&mut self) {
        self.token_cache.clear();
        self.cache_validity.clear();
    }

    /// Returns the size of the token cache.
    pub fn cache_size(&self) -> usize {
        self.token_cache.len()
    }

    /// Mark a line as needing re-highlighting.
    pub fn mark_line_dirty(&mut self, line_number: usize) {
        self.dirty_lines.insert(line_number);
        self.invalidate_line_cache(line_number);
    }

    /// Mark a range of lines as needing re-highlighting.
    pub fn mark_lines_dirty(&mut self, start_line: usize, end_line: usize) {
        for line in start_line..=end_line {
            self.dirty_lines.insert(line);
        }
        self.invalidate_line_range_cache(start_line, end_line);
    }

    /// Mark the entire document as needing re-highlighting.
    pub fn mark_document_dirty(&mut self) {
        self.needs_full_rehighlight = true;
        self.clear_cache();
        self.dirty_lines.clear();
    }

    /// Check if a line needs re-highlighting.
    pub fn is_line_dirty(&self, line_number: usize) -> bool {
        self.needs_full_rehighlight || self.dirty_lines.contains(&line_number)
    }

    /// Clear dirty state for a line after re-highlighting.
    pub fn clear_line_dirty(&mut self, line_number: usize) {
        self.dirty_lines.remove(&line_number);
    }

    /// Clear all dirty state.
    pub fn clear_all_dirty(&mut self) {
        self.dirty_lines.clear();
        self.needs_full_rehighlight = false;
    }

    /// Updates the current viewport and schedules background highlighting for nearby lines.
    /// 
    /// # Arguments
    /// 
    /// * `start_line` - First visible line in the viewport
    /// * `end_line` - Last visible line in the viewport (exclusive)
    pub fn update_viewport(&mut self, start_line: usize, end_line: usize) {
        let new_viewport = (start_line, end_line);
        
        // Only update if viewport has changed significantly
        if let Some((old_start, old_end)) = self.viewport {
            // If the change is small (< 5 lines), don't rebuild the queue
            if (start_line.abs_diff(old_start) < 5) && (end_line.abs_diff(old_end) < 5) {
                return;
            }
        }
        
        self.viewport = Some(new_viewport);
        self.rebuild_background_queue();
    }

    /// Rebuilds the background highlighting queue based on current viewport.
    fn rebuild_background_queue(&mut self) {
        self.background_queue.clear();
        
        let (start_line, end_line) = match self.viewport {
            Some(viewport) => viewport,
            None => return,
        };

        // Calculate the range to pre-highlight
        let background_start = start_line.saturating_sub(self.background_lookahead);
        let background_end = end_line + self.background_lookahead;

        // Add lines to the queue, prioritizing by distance from viewport
        let mut candidates = Vec::new();
        
        // Lines above viewport (closer to viewport = higher priority)
        for line in background_start..start_line {
            if !self.has_valid_cache(line) && !self.background_in_progress.contains(&line) {
                let distance = start_line - line;
                candidates.push((distance, line));
            }
        }
        
        // Lines below viewport (closer to viewport = higher priority)
        for line in end_line..background_end {
            if !self.has_valid_cache(line) && !self.background_in_progress.contains(&line) {
                let distance = line - end_line + 1;
                candidates.push((distance, line));
            }
        }
        
        // Sort by distance (closer lines first)
        candidates.sort_by_key(|(distance, _)| *distance);
        
        // Add to queue
        self.background_queue = candidates.into_iter().map(|(_, line)| line).collect();
    }

    /// Checks if a line has valid cached tokens (without checking content hash).
    fn has_valid_cache(&self, line_number: usize) -> bool {
        self.token_cache.contains_key(&line_number)
    }

    /// Gets the next batch of lines to highlight in the background.
    /// Returns a vector of line numbers that should be highlighted.
    pub fn get_background_batch(&mut self) -> Vec<usize> {
        let batch_size = self.background_batch_size.min(self.background_queue.len());
        let batch: Vec<usize> = self.background_queue.drain(..batch_size).collect();
        
        // Mark these lines as in progress
        for &line in &batch {
            self.background_in_progress.insert(line);
        }
        
        batch
    }

    /// Marks a line as completed for background highlighting.
    pub fn complete_background_line(&mut self, line_number: usize) {
        self.background_in_progress.remove(&line_number);
    }

    /// Returns true if there are lines waiting for background highlighting.
    pub fn has_background_work(&self) -> bool {
        !self.background_queue.is_empty()
    }

    /// Returns the number of lines in the background queue.
    pub fn background_queue_size(&self) -> usize {
        self.background_queue.len()
    }

    /// Sets the background highlighting batch size.
    pub fn set_background_batch_size(&mut self, size: usize) {
        self.background_batch_size = size.max(1).min(50); // Limit between 1 and 50
    }

    /// Sets the background highlighting lookahead distance.
    pub fn set_background_lookahead(&mut self, distance: usize) {
        self.background_lookahead = distance.min(200); // Limit to 200 lines
        
        // Rebuild queue if viewport is set
        if self.viewport.is_some() {
            self.rebuild_background_queue();
        }
    }

    /// Gets current viewport information.
    pub fn get_viewport(&self) -> Option<(usize, usize)> {
        self.viewport
    }

    /// Handles text insertion by updating line caches and background highlighting queue.
    /// This method shifts cached tokens for lines after the insertion point.
    pub fn handle_text_insert(&mut self, start_line: usize, lines_added: usize) {
        if lines_added == 0 {
            return;
        }

        // Shift cached tokens for lines after the insertion point
        let mut new_token_cache = HashMap::new();
        let mut new_cache_validity = HashMap::new();
        let mut new_dirty_lines = HashSet::new();

        for (&line_num, tokens) in &self.token_cache {
            if line_num >= start_line {
                // Shift line numbers down by the number of lines added
                new_token_cache.insert(line_num + lines_added, tokens.clone());
            } else {
                new_token_cache.insert(line_num, tokens.clone());
            }
        }

        for (&line_num, &hash) in &self.cache_validity {
            if line_num >= start_line {
                new_cache_validity.insert(line_num + lines_added, hash);
            } else {
                new_cache_validity.insert(line_num, hash);
            }
        }

        for &line_num in &self.dirty_lines {
            if line_num >= start_line {
                new_dirty_lines.insert(line_num + lines_added);
            } else {
                new_dirty_lines.insert(line_num);
            }
        }

        self.token_cache = new_token_cache;
        self.cache_validity = new_cache_validity;
        self.dirty_lines = new_dirty_lines;

        // Mark the insertion area as dirty
        for line in start_line..(start_line + lines_added) {
            self.dirty_lines.insert(line);
        }

        // Update background highlighting queue if viewport is active
        if self.viewport.is_some() {
            self.rebuild_background_queue();
        }
    }

    /// Handles text deletion by updating line caches and background highlighting queue.
    /// This method shifts cached tokens for lines after the deletion point.
    pub fn handle_text_delete(&mut self, start_line: usize, lines_deleted: usize) {
        if lines_deleted == 0 {
            return;
        }

        // Remove cached tokens for deleted lines and shift remaining lines
        let mut new_token_cache = HashMap::new();
        let mut new_cache_validity = HashMap::new();
        let mut new_dirty_lines = HashSet::new();

        for (&line_num, tokens) in &self.token_cache {
            if line_num < start_line {
                // Lines before deletion remain unchanged
                new_token_cache.insert(line_num, tokens.clone());
            } else if line_num >= start_line + lines_deleted {
                // Lines after deletion are shifted up
                new_token_cache.insert(line_num - lines_deleted, tokens.clone());
            }
            // Lines within the deleted range are not copied (removed)
        }

        for (&line_num, &hash) in &self.cache_validity {
            if line_num < start_line {
                new_cache_validity.insert(line_num, hash);
            } else if line_num >= start_line + lines_deleted {
                new_cache_validity.insert(line_num - lines_deleted, hash);
            }
        }

        for &line_num in &self.dirty_lines {
            if line_num < start_line {
                new_dirty_lines.insert(line_num);
            } else if line_num >= start_line + lines_deleted {
                new_dirty_lines.insert(line_num - lines_deleted);
            }
        }

        self.token_cache = new_token_cache;
        self.cache_validity = new_cache_validity;
        self.dirty_lines = new_dirty_lines;

        // Mark the deletion point as dirty
        self.dirty_lines.insert(start_line);

        // Clear background highlighting work in progress for deleted lines
        self.background_in_progress.retain(|&line| line < start_line || line >= start_line + lines_deleted);

        // Update background highlighting queue if viewport is active
        if self.viewport.is_some() {
            self.rebuild_background_queue();
        }
    }
}

/// A wrapper around Synoptic's Highlighter with additional functionality.
#[derive(Debug)]
pub struct SyntaxHighlighter {
    /// The underlying Synoptic highlighter
    highlighter: Option<Highlighter>,
    /// The language this highlighter is configured for
    language: Language,
    /// Whether the highlighter has been initialized
    initialized: bool,
}

impl SyntaxHighlighter {
    /// Creates a new syntax highlighter for the given language.
    /// Note: The highlighter is not initialized until first use (lazy initialization).
    pub fn new(language: Language) -> Self {
        Self {
            highlighter: None,
            language,
            initialized: false,
        }
    }

    /// Initializes the highlighter for the given language.
    /// This is called automatically on first use.
    fn initialize(&mut self) -> Result<(), String> {
        if self.initialized {
            return Ok(());
        }

        // Try to create a Synoptic highlighter from extension
        let extension = self.language.primary_extension();
        match synoptic::from_extension(extension, 4) { // 4-space tabs
            Some(highlighter) => {
                self.highlighter = Some(highlighter);
                self.initialized = true;
                Ok(())
            }
            None => {
                // Create a highlighter manually for languages not supported by Synoptic
                let mut highlighter = Highlighter::new(4); // 4-space tabs
                
                // Add comprehensive highlighting rules for Phase 1 languages
                match self.language {
                    Language::Rust => {
                        // Keywords
                        highlighter.keyword("keyword", r"\b(as|async|await|break|const|continue|crate|dyn|else|enum|extern|false|fn|for|if|impl|in|let|loop|match|mod|move|mut|pub|ref|return|self|Self|static|struct|super|trait|true|type|unsafe|use|where|while)\b");
                        
                        // Types
                        highlighter.keyword("type", r"\b(bool|char|f32|f64|i8|i16|i32|i64|i128|isize|str|u8|u16|u32|u64|u128|usize|String|Vec|Option|Result|Box|Rc|Arc)\b");
                        
                        // Strings
                        highlighter.bounded_interp("string", "\"", "\"", "\\{", "\\}", true);
                        highlighter.keyword("string", r"'[^']*'");
                        highlighter.keyword("string", r##"r#*".*?"#*"##);
                        
                        // Comments
                        highlighter.keyword("comment", r"//.*$");
                        highlighter.bounded("comment", r"/\*", r"\*/", false);
                        
                        // Numbers
                        highlighter.keyword("number", r"\b\d+(\.\d+)?([eE][+-]?\d+)?(f32|f64|i8|i16|i32|i64|i128|isize|u8|u16|u32|u64|u128|usize)?\b");
                        
                        // Attributes
                        highlighter.keyword("attribute", r"#\[.*?\]");
                    }
                    Language::JavaScript | Language::TypeScript => {
                        // Keywords
                        highlighter.keyword("keyword", r"\b(async|await|break|case|catch|class|const|continue|debugger|default|delete|do|else|export|extends|finally|for|function|if|import|in|instanceof|let|new|of|return|super|switch|this|throw|try|typeof|var|void|while|with|yield)\b");
                        
                        // Built-in objects
                        highlighter.keyword("type", r"\b(Array|Boolean|Date|Error|Function|JSON|Map|Math|Number|Object|Promise|RegExp|Set|String|Symbol|console|document|window)\b");
                        
                        // Strings
                        highlighter.bounded("string", "\"", "\"", true);
                        highlighter.bounded("string", "'", "'", true);
                        highlighter.bounded_interp("string", "`", "`", "${", "}", true);
                        
                        // Comments
                        highlighter.keyword("comment", r"//.*$");
                        highlighter.bounded("comment", r"/\*", r"\*/", false);
                        
                        // Numbers
                        highlighter.keyword("number", r"\b\d+(\.\d+)?([eE][+-]?\d+)?\b");
                        
                        // Regex
                        highlighter.keyword("regex", r"/[^/\n]+/[gimuy]*");
                    }
                    Language::Python => {
                        // Keywords
                        highlighter.keyword("keyword", r"\b(and|as|assert|async|await|break|class|continue|def|del|elif|else|except|False|finally|for|from|global|if|import|in|is|lambda|None|nonlocal|not|or|pass|raise|return|True|try|while|with|yield)\b");
                        
                        // Built-in functions
                        highlighter.keyword("builtin", r"\b(abs|all|any|ascii|bin|bool|breakpoint|bytearray|bytes|callable|chr|classmethod|compile|complex|delattr|dict|dir|divmod|enumerate|eval|exec|filter|float|format|frozenset|getattr|globals|hasattr|hash|help|hex|id|input|int|isinstance|issubclass|iter|len|list|locals|map|max|memoryview|min|next|object|oct|open|ord|pow|print|property|range|repr|reversed|round|set|setattr|slice|sorted|staticmethod|str|sum|super|tuple|type|vars|zip)\b");
                        
                        // Strings
                        highlighter.bounded("string", "\"\"\"", "\"\"\"", false);
                        highlighter.bounded("string", "'''", "'''", false);
                        highlighter.bounded("string", "\"", "\"", true);
                        highlighter.bounded("string", "'", "'", true);
                        highlighter.keyword("string", r#"[rf]"[^"]*"|[rf]'[^']*'"#);
                        
                        // Comments
                        highlighter.keyword("comment", r"#.*$");
                        
                        // Numbers
                        highlighter.keyword("number", r"\b\d+(\.\d+)?([eE][+-]?\d+)?\b");
                        highlighter.keyword("number", r"\b0[xX][0-9a-fA-F]+\b");
                        highlighter.keyword("number", r"\b0[bB][01]+\b");
                        highlighter.keyword("number", r"\b0[oO][0-7]+\b");
                        
                        // Decorators
                        highlighter.keyword("decorator", r"@\w+");
                    }
                    Language::Json => {
                        // Strings
                        highlighter.bounded("string", "\"", "\"", true);
                        
                        // Numbers
                        highlighter.keyword("number", r"-?\b\d+(\.\d+)?([eE][+-]?\d+)?\b");
                        
                        // Booleans and null
                        highlighter.keyword("boolean", r"\b(true|false|null)\b");
                    }
                    _ => {
                        // For other languages, add basic string and comment highlighting
                        highlighter.bounded("string", "\"", "\"", true);
                        highlighter.keyword("comment", r"//.*$|#.*$");
                    }
                }
                
                self.highlighter = Some(highlighter);
                self.initialized = true;
                Ok(())
            }
        }
    }

    /// Highlights a single line of text and returns the tokens.
    /// Note: For proper context-aware highlighting (e.g., multiline comments),
    /// the entire document should be processed through `highlight_document` first.
    pub fn highlight_line(&mut self, line: &str, _line_number: usize) -> Result<Vec<TokenInfo>, String> {
        // Ensure highlighter is initialized
        self.initialize()?;

        if line.is_empty() {
            return Ok(Vec::new());
        }

        // Get the highlighter
        let highlighter = self.highlighter.as_mut().ok_or("Highlighter not initialized")?;

        // For single-line highlighting, we run the highlighter on just this line
        // This won't handle multiline tokens properly, but is fast for incremental updates
        let lines = vec![line.to_string()];
        highlighter.run(&lines);

        // Get tokens for line 0 (since we're only processing one line)
        let mut tokens = Vec::new();
        let mut current_offset = 0;

        // Process tokens from Synoptic
        for token in highlighter.line(0, line) {
            match token {
                synoptic::TokOpt::Some(text, kind) => {
                    let start = current_offset;
                    let end = start + text.len();
                    
                    tokens.push(TokenInfo::highlighted(
                        text.to_string(),
                        kind.to_string(),
                        start,
                        end,
                    ));
                    
                    current_offset = end;
                }
                synoptic::TokOpt::None(text) => {
                    let start = current_offset;
                    let end = start + text.len();
                    
                    tokens.push(TokenInfo::plain_text(
                        text.to_string(),
                        start,
                        end,
                    ));
                    
                    current_offset = end;
                }
            }
        }

        Ok(tokens)
    }
    
    /// Highlights an entire document and returns tokens for a specific line.
    /// This method provides proper context-aware highlighting for multiline tokens.
    pub fn highlight_document(&mut self, document: &str, line_number: usize) -> Result<Vec<TokenInfo>, String> {
        // Ensure highlighter is initialized
        self.initialize()?;

        // Get the highlighter
        let highlighter = self.highlighter.as_mut().ok_or("Highlighter not initialized")?;

        // Run the highlighter on the entire document
        let lines: Vec<String> = document.lines().map(String::from).collect();
        highlighter.run(&lines);

        // Get the specific line from the document
        let lines: Vec<&str> = document.lines().collect();
        if line_number >= lines.len() {
            return Ok(Vec::new());
        }
        
        let line = lines[line_number];
        
        // Get tokens for the specific line
        let mut tokens = Vec::new();
        let mut current_offset = 0;

        // Process tokens from Synoptic
        for token in highlighter.line(line_number, line) {
            match token {
                synoptic::TokOpt::Some(text, kind) => {
                    let start = current_offset;
                    let end = start + text.len();
                    
                    tokens.push(TokenInfo::highlighted(
                        text.to_string(),
                        kind.to_string(),
                        start,
                        end,
                    ));
                    
                    current_offset = end;
                }
                synoptic::TokOpt::None(text) => {
                    let start = current_offset;
                    let end = start + text.len();
                    
                    tokens.push(TokenInfo::plain_text(
                        text.to_string(),
                        start,
                        end,
                    ));
                    
                    current_offset = end;
                }
            }
        }

        Ok(tokens)
    }


    /// Returns the language this highlighter is configured for.
    pub fn language(&self) -> Language {
        self.language
    }

    /// Returns whether the highlighter has been initialized.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

/// Global syntax highlighting service for the Edit text editor.
#[derive(Debug)]
pub struct HighlightingService {
    /// Language detector for identifying file types
    language_detector: LanguageDetector,
    /// Cache of syntax highlighters per language
    highlighters: HashMap<Language, SyntaxHighlighter>,
    /// Global highlighting configuration
    enabled: bool,
    /// Global performance metrics
    global_metrics: HighlightingMetrics,
    /// Maximum time allowed for highlighting a single line
    line_timeout: Duration,
    /// Maximum line length before skipping highlighting
    max_line_length: usize,
}

impl Default for HighlightingService {
    fn default() -> Self {
        Self::new()
    }
}

impl HighlightingService {
    /// Creates a new highlighting service.
    pub fn new() -> Self {
        Self {
            language_detector: LanguageDetector::new(),
            highlighters: HashMap::new(),
            enabled: true,
            global_metrics: HighlightingMetrics::default(),
            line_timeout: Duration::from_millis(50), // 50ms per line timeout
            max_line_length: 10_000, // Skip highlighting for lines longer than 10k characters
        }
    }

    /// Creates a new highlighting state for a file.
    pub fn create_highlighting_state<P: AsRef<Path>>(&mut self, file_path: P) -> HighlightingState {
        let language = self.language_detector.detect_language(&file_path);
        
        if self.enabled && (language.is_tier_1() || language.is_tier_2()) {
            HighlightingState::new(language)
        } else {
            HighlightingState::disabled(language)
        }
    }

    /// Highlights a single line of text.
    pub fn highlight_line(
        &mut self, 
        state: &mut HighlightingState,
        line: &str, 
        line_number: usize
    ) -> Result<Vec<TokenInfo>, String> {
        if !state.enabled || !self.enabled {
            // Return the entire line as plain text if highlighting is disabled
            return Ok(vec![TokenInfo::plain_text(
                line.to_string(),
                0,
                line.len(),
            )]);
        }

        // Skip highlighting for extremely long lines
        if line.len() > self.max_line_length {
            return Ok(vec![TokenInfo::plain_text(
                line.to_string(),
                0,
                line.len(),
            )]);
        }

        // Calculate content hash for caching
        let content_hash = self.calculate_line_hash(line);
        
        // Check cache first
        if state.has_cached_tokens(line_number, content_hash) {
            state.metrics.record_cache_hit();
            return Ok(state.get_cached_tokens(line_number).unwrap().clone());
        }

        state.metrics.record_cache_miss();

        // Get or create highlighter for this language
        let highlighter = self.highlighters
            .entry(state.language)
            .or_insert_with(|| SyntaxHighlighter::new(state.language));

        // Perform highlighting with timeout protection
        let start_time = Instant::now();
        
        // For now, we perform the highlighting and check the duration after
        // In a production system, you might want to use a separate thread with a timeout
        let tokens = highlighter.highlight_line(line, line_number)?;
        let duration = start_time.elapsed();

        // If highlighting took too long, return plain text and mark line for skipping
        if duration > self.line_timeout {
            // Log that we exceeded timeout (in production, you'd use a proper logging system)
            eprintln!("Syntax highlighting timeout for line {} ({}ms)", line_number, duration.as_millis());
            
            // Return plain text instead
            return Ok(vec![TokenInfo::plain_text(
                line.to_string(),
                0,
                line.len(),
            )]);
        }

        // Update metrics
        state.metrics.record_line_highlight(duration, tokens.len());
        self.global_metrics.record_line_highlight(duration, tokens.len());

        // Cache the result
        state.cache_tokens(line_number, content_hash, tokens.clone());

        Ok(tokens)
    }

    /// Sets a language override for a specific file.
    pub fn set_language_override<P: AsRef<Path>>(&mut self, file_path: P, language: Language) {
        self.language_detector.set_language_override(file_path, language);
    }

    /// Removes a language override for a specific file.
    pub fn remove_language_override<P: AsRef<Path>>(&mut self, file_path: P) -> Option<Language> {
        self.language_detector.remove_language_override(file_path)
    }

    /// Enables or disables syntax highlighting globally.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Returns whether syntax highlighting is globally enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Returns the global performance metrics.
    pub fn global_metrics(&self) -> &HighlightingMetrics {
        &self.global_metrics
    }

    /// Resets all performance metrics.
    pub fn reset_metrics(&mut self) {
        self.global_metrics.reset();
    }

    /// Returns information about supported languages.
    pub fn supported_languages(&self) -> Vec<Language> {
        LanguageDetector::supported_languages()
    }

    /// Returns the number of cached highlighters.
    pub fn cached_highlighter_count(&self) -> usize {
        self.highlighters.len()
    }

    /// Sets the timeout for highlighting a single line.
    pub fn set_line_timeout(&mut self, timeout: Duration) {
        self.line_timeout = timeout;
    }

    /// Gets the current line timeout.
    pub fn line_timeout(&self) -> Duration {
        self.line_timeout
    }

    /// Sets the maximum line length before skipping highlighting.
    pub fn set_max_line_length(&mut self, max_length: usize) {
        self.max_line_length = max_length;
    }

    /// Gets the current maximum line length.
    pub fn max_line_length(&self) -> usize {
        self.max_line_length
    }

    /// Calculates a simple hash for line content caching.
    fn calculate_line_hash(&self, line: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        line.hash(&mut hasher);
        hasher.finish()
    }

    /// Performs background highlighting for a batch of lines.
    /// This method is designed to be called during idle time to pre-highlight
    /// lines near the viewport without blocking the UI.
    /// 
    /// # Arguments
    /// 
    /// * `state` - The highlighting state for the document
    /// * `get_line_content` - A closure that returns the content for a given line number
    /// 
    /// # Returns
    /// 
    /// The number of lines successfully highlighted in the background.
    pub fn highlight_background_batch<F>(
        &mut self,
        state: &mut HighlightingState,
        mut get_line_content: F,
    ) -> usize
    where
        F: FnMut(usize) -> Option<String>,
    {
        if !state.enabled || !self.enabled {
            return 0;
        }

        let batch = state.get_background_batch();
        let mut highlighted_count = 0;

        for line_number in batch {
            // Get the line content
            if let Some(line_content) = get_line_content(line_number) {
                // Skip extremely long lines to avoid blocking
                if line_content.len() <= self.max_line_length {
                    // Calculate content hash
                    let content_hash = self.calculate_line_hash(&line_content);
                    
                    // Skip if already cached with current content
                    if !state.has_cached_tokens(line_number, content_hash) {
                        // Get or create highlighter for this language
                        let highlighter = self.highlighters
                            .entry(state.language)
                            .or_insert_with(|| SyntaxHighlighter::new(state.language));

                        // Perform highlighting with a shorter timeout for background work
                        let start_time = Instant::now();
                        
                        if let Ok(tokens) = highlighter.highlight_line(&line_content, line_number) {
                            let duration = start_time.elapsed();
                            
                            // Use a shorter timeout for background highlighting (half of normal timeout)
                            let background_timeout = self.line_timeout / 2;
                            
                            if duration <= background_timeout {
                                // Cache the result
                                state.cache_tokens(line_number, content_hash, tokens);
                                highlighted_count += 1;
                                
                                // Update metrics (but don't count towards main metrics to avoid skewing)
                                // We could add separate background metrics here if needed
                            }
                        }
                    }
                }
            }
            
            // Mark this line as completed
            state.complete_background_line(line_number);
        }

        highlighted_count
    }

    /// Triggers background highlighting for lines near the current viewport.
    /// This should be called when the viewport changes (e.g., during scrolling).
    /// 
    /// # Arguments
    /// 
    /// * `state` - The highlighting state for the document
    /// * `viewport_start` - First visible line
    /// * `viewport_end` - Last visible line (exclusive)
    pub fn update_viewport(&mut self, state: &mut HighlightingState, viewport_start: usize, viewport_end: usize) {
        state.update_viewport(viewport_start, viewport_end);
    }

    /// Returns true if there is background highlighting work available.
    pub fn has_background_work(&self, state: &HighlightingState) -> bool {
        state.has_background_work()
    }

    /// Sets background highlighting configuration.
    /// This configures the default settings for new highlighting states.
    pub fn configure_background_highlighting(&mut self, batch_size: usize, lookahead: usize) {
        // Store configuration that will be applied to new highlighting states
        // For now, we don't store these globally since each state manages its own configuration
        // The parameters are used during state creation and individual configuration calls
        
        // We could add fields to store these defaults if needed:
        // self.default_batch_size = batch_size.max(1).min(50);
        // self.default_lookahead = lookahead.min(200);
        
        // For now, prevent unused variable warnings by acknowledging the parameters
        let _ = batch_size.max(1).min(50); // Validate range
        let _ = lookahead.min(200); // Validate range
    }
}

/// Global singleton instance of the highlighting service.
static HIGHLIGHTING_SERVICE: Lazy<std::sync::Mutex<HighlightingService>> = 
    Lazy::new(|| std::sync::Mutex::new(HighlightingService::new()));

/// Gets a reference to the global highlighting service.
/// 
/// # Panics
/// 
/// Panics if the service mutex is poisoned.
pub fn global_highlighting_service() -> std::sync::MutexGuard<'static, HighlightingService> {
    HIGHLIGHTING_SERVICE.lock().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_info_creation() {
        let token = TokenInfo::plain_text("hello".to_string(), 0, 5);
        assert_eq!(token.text, "hello");
        assert_eq!(token.kind, None);
        assert!(!token.is_highlighted());
        assert_eq!(token.len(), 5);

        let highlighted = TokenInfo::highlighted("world".to_string(), "keyword".to_string(), 6, 11);
        assert_eq!(highlighted.text, "world");
        assert_eq!(highlighted.kind, Some("keyword".to_string()));
        assert!(highlighted.is_highlighted());
    }

    #[test]
    fn test_highlighting_metrics() {
        let mut metrics = HighlightingMetrics::default();
        
        metrics.record_line_highlight(Duration::from_millis(10), 5);
        metrics.record_line_highlight(Duration::from_millis(20), 3);
        
        assert_eq!(metrics.lines_highlighted, 2);
        assert_eq!(metrics.tokens_generated, 8);
        assert_eq!(metrics.max_line_time, Duration::from_millis(20));
        
        metrics.record_cache_hit();
        metrics.record_cache_miss();
        assert_eq!(metrics.cache_hit_ratio(), 0.5);
    }

    #[test]
    fn test_highlighting_state() {
        let mut state = HighlightingState::new(Language::Rust);
        assert_eq!(state.language, Language::Rust);
        assert!(state.enabled);
        
        let tokens = vec![TokenInfo::plain_text("test".to_string(), 0, 4)];
        state.cache_tokens(0, 12345, tokens.clone());
        
        assert!(state.has_cached_tokens(0, 12345));
        assert_eq!(state.get_cached_tokens(0).unwrap(), &tokens);
        
        state.invalidate_line_cache(0);
        assert!(!state.has_cached_tokens(0, 12345));
    }

    #[test]
    fn test_syntax_highlighter_creation() {
        let highlighter = SyntaxHighlighter::new(Language::Rust);
        assert_eq!(highlighter.language(), Language::Rust);
        assert!(!highlighter.is_initialized());
    }

    #[test]
    fn test_highlighting_service() {
        let mut service = HighlightingService::new();
        
        // Test state creation
        let state = service.create_highlighting_state("test.rs");
        assert_eq!(state.language, Language::Rust);
        assert!(state.enabled);
        
        // Test language override
        service.set_language_override("special_file", Language::Python);
        let state = service.create_highlighting_state("special_file");
        assert_eq!(state.language, Language::Python);
        
        // Test disabling highlighting
        service.set_enabled(false);
        assert!(!service.is_enabled());
    }

    #[test]
    fn test_mock_highlighting() {
        let mut service = HighlightingService::new();
        let mut state = service.create_highlighting_state("test.rs");
        
        let tokens = service.highlight_line(&mut state, "fn main() {", 0).unwrap();
        assert!(!tokens.is_empty());
        
        // Should contain some highlighted tokens for Rust keywords
        let has_keyword = tokens.iter().any(|t| t.kind.as_deref() == Some("keyword"));
        assert!(has_keyword);
    }

    #[test]
    fn test_caching_behavior() {
        let mut service = HighlightingService::new();
        let mut state = service.create_highlighting_state("test.rs");
        
        // First highlight should be a cache miss
        let _tokens1 = service.highlight_line(&mut state, "fn main() {", 0).unwrap();
        assert_eq!(state.metrics.cache_misses, 1);
        assert_eq!(state.metrics.cache_hits, 0);
        
        // Second highlight of same line should be a cache hit
        let _tokens2 = service.highlight_line(&mut state, "fn main() {", 0).unwrap();
        assert_eq!(state.metrics.cache_misses, 1);
        assert_eq!(state.metrics.cache_hits, 1);
        
        // Different line content should be a cache miss
        let _tokens3 = service.highlight_line(&mut state, "let x = 5;", 0).unwrap();
        assert_eq!(state.metrics.cache_misses, 2);
        assert_eq!(state.metrics.cache_hits, 1);
    }
}
