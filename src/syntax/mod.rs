// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Syntax highlighting infrastructure for the Edit text editor.
//!
//! This module provides syntax highlighting capabilities using the Synoptic library.
//! It includes language detection, highlighter management, and token processing.

pub mod language;
pub mod highlighter;
pub mod performance;
pub mod color_mapper;
pub mod render_bridge;

#[cfg(test)]
mod performance_test;

pub use language::{Language, LanguageConfig, LanguageDetector};
pub use highlighter::{SyntaxHighlighter, HighlightingService, TokenInfo, HighlightingState, global_highlighting_service};
pub use performance::{
    PerformanceBaseline, PerformanceMeasurement, FileSizeCategory, LineLengthCategory,
    FileLoadingMetrics, MemoryMetrics, HighlightingPerformanceMetrics, SystemResourceMetrics,
    create_test_session, run_baseline_test
};
pub use color_mapper::{ColorMapper, global_color_mapper, global_color_mapper_mut};
pub use render_bridge::{
    register_buffer_highlighting, unregister_buffer_highlighting, get_line_tokens,
    get_line_tokens_with_viewport, process_background_highlighting, has_background_work,
    update_viewport_tracking
};
