// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Performance monitoring and baseline measurement for syntax highlighting.
//!
//! This module provides utilities for measuring and tracking performance
//! characteristics of the syntax highlighting system during Phase 0 baseline
//! establishment and ongoing monitoring.

use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::path::Path;

/// Performance baseline measurements for syntax highlighting operations.
#[derive(Debug, Clone, Default)]
pub struct PerformanceBaseline {
    /// File loading performance measurements
    pub file_loading: FileLoadingMetrics,
    /// Memory usage measurements
    pub memory_usage: MemoryMetrics,
    /// Highlighting performance measurements
    pub highlighting: HighlightingPerformanceMetrics,
    /// System resource utilization
    pub system_resources: SystemResourceMetrics,
}

/// Metrics for file loading operations.
#[derive(Debug, Clone, Default)]
pub struct FileLoadingMetrics {
    /// Time to load files by size category (in KB)
    pub load_times_by_size: HashMap<FileSizeCategory, Vec<Duration>>,
    /// Average load time per file size category
    pub avg_load_times: HashMap<FileSizeCategory, Duration>,
    /// Maximum observed load time
    pub max_load_time: Duration,
    /// Number of files measured
    pub files_measured: usize,
    /// Total time spent loading files
    pub total_load_time: Duration,
}

/// Metrics for memory usage tracking.
#[derive(Debug, Clone, Default)]
pub struct MemoryMetrics {
    /// Baseline memory usage before syntax highlighting
    pub baseline_memory_kb: u64,
    /// Memory usage with syntax highlighting enabled
    pub with_highlighting_memory_kb: u64,
    /// Memory overhead introduced by syntax highlighting
    pub highlighting_overhead_kb: u64,
    /// Memory usage per language
    pub memory_per_language: HashMap<String, u64>,
    /// Peak memory usage observed
    pub peak_memory_kb: u64,
}

/// Performance metrics specific to highlighting operations.
#[derive(Debug, Clone, Default)]
pub struct HighlightingPerformanceMetrics {
    /// Time to highlight lines by line length category
    pub highlight_times_by_length: HashMap<LineLengthCategory, Vec<Duration>>,
    /// Average highlighting time per line length category
    pub avg_highlight_times: HashMap<LineLengthCategory, Duration>,
    /// Token generation rate (tokens per second)
    pub token_generation_rate: f64,
    /// Cache hit ratio
    pub cache_hit_ratio: f64,
    /// Number of highlighting operations performed
    pub operations_performed: usize,
}

/// System resource utilization metrics.
#[derive(Debug, Clone, Default)]
pub struct SystemResourceMetrics {
    /// CPU usage percentage during highlighting operations
    pub cpu_usage_percent: f64,
    /// Memory allocation rate (MB/s)
    pub memory_allocation_rate: f64,
    /// Number of thread context switches
    pub context_switches: u64,
}

/// File size categories for performance measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileSizeCategory {
    /// Files under 10KB
    Small,
    /// Files 10KB - 100KB
    Medium,
    /// Files 100KB - 1MB
    Large,
    /// Files over 1MB
    ExtraLarge,
}

impl FileSizeCategory {
    /// Determines the size category for a given file size in bytes.
    pub fn from_bytes(bytes: u64) -> Self {
        match bytes {
            0..=10_240 => FileSizeCategory::Small,
            10_241..=102_400 => FileSizeCategory::Medium,
            102_401..=1_048_576 => FileSizeCategory::Large,
            _ => FileSizeCategory::ExtraLarge,
        }
    }

    /// Returns the human-readable name of the size category.
    pub fn name(&self) -> &'static str {
        match self {
            FileSizeCategory::Small => "Small (< 10KB)",
            FileSizeCategory::Medium => "Medium (10KB - 100KB)",
            FileSizeCategory::Large => "Large (100KB - 1MB)",
            FileSizeCategory::ExtraLarge => "Extra Large (> 1MB)",
        }
    }
}

/// Line length categories for performance measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineLengthCategory {
    /// Lines under 80 characters
    Short,
    /// Lines 80-200 characters
    Normal,
    /// Lines 200-500 characters
    Long,
    /// Lines over 500 characters
    ExtraLong,
}

impl LineLengthCategory {
    /// Determines the length category for a given line length.
    pub fn from_length(length: usize) -> Self {
        match length {
            0..=79 => LineLengthCategory::Short,
            80..=199 => LineLengthCategory::Normal,
            200..=499 => LineLengthCategory::Long,
            _ => LineLengthCategory::ExtraLong,
        }
    }

    /// Returns the human-readable name of the length category.
    pub fn name(&self) -> &'static str {
        match self {
            LineLengthCategory::Short => "Short (< 80 chars)",
            LineLengthCategory::Normal => "Normal (80-200 chars)",
            LineLengthCategory::Long => "Long (200-500 chars)",
            LineLengthCategory::ExtraLong => "Extra Long (> 500 chars)",
        }
    }
}

/// Performance measurement utilities.
pub struct PerformanceMeasurement {
    baseline: PerformanceBaseline,
    measurement_start: Option<Instant>,
}

impl Default for PerformanceMeasurement {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMeasurement {
    /// Creates a new performance measurement instance.
    pub fn new() -> Self {
        Self {
            baseline: PerformanceBaseline::default(),
            measurement_start: None,
        }
    }

    /// Starts a performance measurement session.
    pub fn start_measurement(&mut self) {
        self.measurement_start = Some(Instant::now());
        self.measure_baseline_memory();
    }

    /// Records a file loading operation.
    pub fn record_file_load(&mut self, file_size_bytes: u64, load_duration: Duration) {
        let category = FileSizeCategory::from_bytes(file_size_bytes);
        
        self.baseline.file_loading.load_times_by_size
            .entry(category)
            .or_insert_with(Vec::new)
            .push(load_duration);
        
        self.baseline.file_loading.files_measured += 1;
        self.baseline.file_loading.total_load_time += load_duration;
        
        if load_duration > self.baseline.file_loading.max_load_time {
            self.baseline.file_loading.max_load_time = load_duration;
        }
        
        self.update_average_load_times();
    }

    /// Records a line highlighting operation.
    pub fn record_line_highlight(&mut self, line_length: usize, highlight_duration: Duration, token_count: usize) {
        let category = LineLengthCategory::from_length(line_length);
        
        self.baseline.highlighting.highlight_times_by_length
            .entry(category)
            .or_insert_with(Vec::new)
            .push(highlight_duration);
        
        self.baseline.highlighting.operations_performed += 1;
        
        // Update token generation rate
        if highlight_duration.as_secs_f64() > 0.0 {
            let tokens_per_second = token_count as f64 / highlight_duration.as_secs_f64();
            let total_operations = self.baseline.highlighting.operations_performed as f64;
            let current_rate = self.baseline.highlighting.token_generation_rate;
            
            // Running average of token generation rate
            self.baseline.highlighting.token_generation_rate = 
                (current_rate * (total_operations - 1.0) + tokens_per_second) / total_operations;
        }
        
        self.update_average_highlight_times();
    }

    /// Records cache performance statistics.
    pub fn record_cache_performance(&mut self, hits: usize, misses: usize) {
        let total = hits + misses;
        if total > 0 {
            self.baseline.highlighting.cache_hit_ratio = hits as f64 / total as f64;
        }
    }

    /// Measures current memory usage.
    pub fn measure_current_memory(&mut self) -> u64 {
        // For Phase 0, we'll use a simple estimation
        // In a real implementation, this would use system APIs to get actual memory usage
        self.estimate_memory_usage()
    }

    /// Updates memory usage after enabling highlighting.
    pub fn measure_highlighting_memory(&mut self) {
        let current_memory = self.measure_current_memory();
        self.baseline.memory_usage.with_highlighting_memory_kb = current_memory;
        
        if self.baseline.memory_usage.baseline_memory_kb > 0 {
            self.baseline.memory_usage.highlighting_overhead_kb = 
                current_memory.saturating_sub(self.baseline.memory_usage.baseline_memory_kb);
        }
        
        if current_memory > self.baseline.memory_usage.peak_memory_kb {
            self.baseline.memory_usage.peak_memory_kb = current_memory;
        }
    }

    /// Gets the current performance baseline.
    pub fn get_baseline(&self) -> &PerformanceBaseline {
        &self.baseline
    }

    /// Generates a performance report.
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("=== Performance Baseline Report ===\n\n");
        
        // File loading performance
        report.push_str("File Loading Performance:\n");
        for (category, times) in &self.baseline.file_loading.load_times_by_size {
            if !times.is_empty() {
                let avg = times.iter().sum::<Duration>() / times.len() as u32;
                report.push_str(&format!(
                    "  {}: {} files, avg {}ms, max {}ms\n",
                    category.name(),
                    times.len(),
                    avg.as_millis(),
                    times.iter().max().unwrap_or(&Duration::ZERO).as_millis()
                ));
            }
        }
        
        // Memory usage
        report.push_str("\nMemory Usage:\n");
        report.push_str(&format!(
            "  Baseline: {}KB\n",
            self.baseline.memory_usage.baseline_memory_kb
        ));
        report.push_str(&format!(
            "  With Highlighting: {}KB\n",
            self.baseline.memory_usage.with_highlighting_memory_kb
        ));
        report.push_str(&format!(
            "  Overhead: {}KB\n",
            self.baseline.memory_usage.highlighting_overhead_kb
        ));
        report.push_str(&format!(
            "  Peak: {}KB\n",
            self.baseline.memory_usage.peak_memory_kb
        ));
        
        // Highlighting performance
        report.push_str("\nHighlighting Performance:\n");
        for (category, times) in &self.baseline.highlighting.highlight_times_by_length {
            if !times.is_empty() {
                let avg = times.iter().sum::<Duration>() / times.len() as u32;
                report.push_str(&format!(
                    "  {}: {} operations, avg {}ms\n",
                    category.name(),
                    times.len(),
                    avg.as_millis()
                ));
            }
        }
        report.push_str(&format!(
            "  Token Generation Rate: {:.0} tokens/sec\n",
            self.baseline.highlighting.token_generation_rate
        ));
        report.push_str(&format!(
            "  Cache Hit Ratio: {:.1}%\n",
            self.baseline.highlighting.cache_hit_ratio * 100.0
        ));
        
        report.push_str("\n=== End Report ===\n");
        report
    }

    /// Checks if performance meets Phase 0 requirements.
    pub fn meets_requirements(&self) -> (bool, Vec<String>) {
        let mut issues = Vec::new();
        let mut passes = true;

        // Check file loading times (should be under 100ms for typical files)
        for (category, avg_time) in &self.baseline.file_loading.avg_load_times {
            match category {
                FileSizeCategory::Small | FileSizeCategory::Medium => {
                    if avg_time.as_millis() > 100 {
                        issues.push(format!(
                            "File loading for {} exceeds 100ms requirement: {}ms",
                            category.name(),
                            avg_time.as_millis()
                        ));
                        passes = false;
                    }
                }
                _ => {} // Larger files may take longer
            }
        }

        // Check highlighting performance (should be under 50ms for normal lines)
        for (category, avg_time) in &self.baseline.highlighting.avg_highlight_times {
            match category {
                LineLengthCategory::Short | LineLengthCategory::Normal => {
                    if avg_time.as_millis() > 50 {
                        issues.push(format!(
                            "Line highlighting for {} exceeds 50ms requirement: {}ms",
                            category.name(),
                            avg_time.as_millis()
                        ));
                        passes = false;
                    }
                }
                _ => {} // Longer lines may take more time
            }
        }

        // Check memory overhead (should be under 50MB for typical usage)
        if self.baseline.memory_usage.highlighting_overhead_kb > 50_000 {
            issues.push(format!(
                "Memory overhead exceeds 50MB requirement: {}KB",
                self.baseline.memory_usage.highlighting_overhead_kb
            ));
            passes = false;
        }

        // Check cache hit ratio (should be at least 70% for efficient caching)
        if self.baseline.highlighting.cache_hit_ratio < 0.7 {
            issues.push(format!(
                "Cache hit ratio below 70% requirement: {:.1}%",
                self.baseline.highlighting.cache_hit_ratio * 100.0
            ));
            passes = false;
        }

        (passes, issues)
    }

    // Private helper methods

    fn measure_baseline_memory(&mut self) {
        self.baseline.memory_usage.baseline_memory_kb = self.estimate_memory_usage();
    }

    fn estimate_memory_usage(&self) -> u64 {
        // For Phase 0, this is a mock implementation
        // In production, this would use platform-specific APIs to get actual memory usage
        
        // Simulate realistic memory usage based on typical editor operations
        let base_usage = 10_000; // 10MB base
        let highlighting_overhead = 5_000; // 5MB for syntax highlighting
        let cache_overhead = self.baseline.highlighting.operations_performed as u64 * 10; // 10 bytes per operation
        
        base_usage + highlighting_overhead + cache_overhead
    }

    fn update_average_load_times(&mut self) {
        for (category, times) in &self.baseline.file_loading.load_times_by_size {
            if !times.is_empty() {
                let avg = times.iter().sum::<Duration>() / times.len() as u32;
                self.baseline.file_loading.avg_load_times.insert(*category, avg);
            }
        }
    }

    fn update_average_highlight_times(&mut self) {
        for (category, times) in &self.baseline.highlighting.highlight_times_by_length {
            if !times.is_empty() {
                let avg = times.iter().sum::<Duration>() / times.len() as u32;
                self.baseline.highlighting.avg_highlight_times.insert(*category, avg);
            }
        }
    }
}

/// Creates a performance measurement session for testing.
pub fn create_test_session() -> PerformanceMeasurement {
    let mut measurement = PerformanceMeasurement::new();
    measurement.start_measurement();
    measurement
}

/// Runs a comprehensive performance baseline test.
pub fn run_baseline_test<P: AsRef<Path>>(test_files: &[P]) -> PerformanceMeasurement {
    let mut measurement = PerformanceMeasurement::new();
    measurement.start_measurement();
    
    // Simulate loading and highlighting various test files
    for (i, file_path) in test_files.iter().enumerate() {
        let _file_path = file_path.as_ref();
        
        // Simulate file size based on file index for testing
        let file_size = match i % 4 {
            0 => 5_000,      // Small file
            1 => 50_000,     // Medium file  
            2 => 500_000,    // Large file
            _ => 2_000_000,  // Extra large file
        };
        
        // Simulate load time based on file size
        let load_time = Duration::from_millis(file_size / 10_000);
        measurement.record_file_load(file_size, load_time);
        
        // Simulate highlighting some lines
        let line_lengths = [50, 120, 300, 800]; // Different line lengths
        for &line_length in &line_lengths {
            let highlight_time = Duration::from_micros(line_length as u64 * 10);
            let token_count = line_length / 10; // Approximate tokens per line
            measurement.record_line_highlight(line_length, highlight_time, token_count);
        }
        
        measurement.measure_highlighting_memory();
    }
    
    // Simulate cache performance
    measurement.record_cache_performance(75, 25); // 75% hit ratio
    
    measurement
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_size_categories() {
        assert_eq!(FileSizeCategory::from_bytes(5_000), FileSizeCategory::Small);
        assert_eq!(FileSizeCategory::from_bytes(50_000), FileSizeCategory::Medium);
        assert_eq!(FileSizeCategory::from_bytes(500_000), FileSizeCategory::Large);
        assert_eq!(FileSizeCategory::from_bytes(5_000_000), FileSizeCategory::ExtraLarge);
    }

    #[test]
    fn test_line_length_categories() {
        assert_eq!(LineLengthCategory::from_length(50), LineLengthCategory::Short);
        assert_eq!(LineLengthCategory::from_length(120), LineLengthCategory::Normal);
        assert_eq!(LineLengthCategory::from_length(300), LineLengthCategory::Long);
        assert_eq!(LineLengthCategory::from_length(800), LineLengthCategory::ExtraLong);
    }

    #[test]
    fn test_performance_measurement() {
        let mut measurement = PerformanceMeasurement::new();
        measurement.start_measurement();
        
        // Record some file loads
        measurement.record_file_load(10_000, Duration::from_millis(50));
        measurement.record_file_load(100_000, Duration::from_millis(200));
        
        // Record some highlighting operations
        measurement.record_line_highlight(100, Duration::from_millis(10), 20);
        measurement.record_line_highlight(200, Duration::from_millis(25), 40);
        
        // Record cache performance
        measurement.record_cache_performance(80, 20);
        
        let baseline = measurement.get_baseline();
        assert_eq!(baseline.file_loading.files_measured, 2);
        assert_eq!(baseline.highlighting.operations_performed, 2);
        assert_eq!(baseline.highlighting.cache_hit_ratio, 0.8);
    }

    #[test]
    fn test_baseline_requirements() {
        let mut measurement = PerformanceMeasurement::new();
        measurement.start_measurement();
        
        // Add measurements that meet requirements
        measurement.record_file_load(10_000, Duration::from_millis(30));
        measurement.record_line_highlight(100, Duration::from_millis(20), 20);
        measurement.record_cache_performance(80, 20);
        
        let (passes, issues) = measurement.meets_requirements();
        assert!(passes, "Should meet requirements with good performance");
        assert!(issues.is_empty(), "Should have no issues: {:?}", issues);
    }

    #[test]
    fn test_performance_report_generation() {
        let test_files = vec!["test1.rs", "test2.js", "test3.py"];
        let measurement = run_baseline_test(&test_files);
        
        let report = measurement.generate_report();
        assert!(report.contains("Performance Baseline Report"));
        assert!(report.contains("File Loading Performance"));
        assert!(report.contains("Memory Usage"));
        assert!(report.contains("Highlighting Performance"));
    }
}
