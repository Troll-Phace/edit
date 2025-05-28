// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Syntax highlighting infrastructure for the Edit text editor.
//!
//! This module provides syntax highlighting capabilities using the Synoptic library.
//! It includes language detection, highlighter management, and token processing.

pub mod language;
pub mod highlighter;
pub mod performance;

pub use language::{Language, LanguageConfig, LanguageDetector};
pub use highlighter::{SyntaxHighlighter, HighlightingService, TokenInfo, HighlightingState, global_highlighting_service};
pub use performance::{
    PerformanceBaseline, PerformanceMeasurement, FileSizeCategory, LineLengthCategory,
    FileLoadingMetrics, MemoryMetrics, HighlightingPerformanceMetrics, SystemResourceMetrics,
    create_test_session, run_baseline_test
};
