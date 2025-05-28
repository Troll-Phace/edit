// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Performance tests for syntax highlighting to ensure sub-100ms response times.

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};
    use crate::syntax::{HighlightingService, Language};

    /// Maximum acceptable time for highlighting a single line
    const MAX_LINE_TIME: Duration = Duration::from_millis(100);
    
    /// Maximum acceptable time for highlighting a typical file viewport (50 lines)
    const MAX_VIEWPORT_TIME: Duration = Duration::from_millis(500);

    /// Test data for different languages
    struct TestCase {
        language: Language,
        filename: &'static str,
        lines: Vec<&'static str>,
    }

    fn get_test_cases() -> Vec<TestCase> {
        vec![
            TestCase {
                language: Language::Rust,
                filename: "test.rs",
                lines: vec![
                    "use std::collections::HashMap;",
                    "use std::time::{Duration, Instant};",
                    "",
                    "/// A complex function with various Rust syntax elements",
                    "pub fn process_data<T: Clone + Debug>(input: &[T], config: Config) -> Result<Vec<T>, Error> {",
                    "    let mut results = Vec::with_capacity(input.len());",
                    "    let start_time = Instant::now();",
                    "    ",
                    "    for (index, item) in input.iter().enumerate() {",
                    "        // Process each item with error handling",
                    "        match validate_item(item) {",
                    "            Ok(validated) => {",
                    "                let processed = transform(validated, &config)?;",
                    "                results.push(processed);",
                    "            }",
                    "            Err(e) => {",
                    "                eprintln!(\"Error processing item {}: {:?}\", index, e);",
                    "                return Err(Error::ValidationFailed(index));",
                    "            }",
                    "        }",
                    "    }",
                    "    ",
                    "    let duration = start_time.elapsed();",
                    "    println!(\"Processing completed in {:?}\", duration);",
                    "    Ok(results)",
                    "}",
                    "",
                    "impl<T> MyTrait for Vec<T> where T: Display + 'static {",
                    "    type Output = String;",
                    "    const MAX_SIZE: usize = 1024;",
                    "    ",
                    "    async fn process(&mut self) -> Self::Output {",
                    "        self.iter().map(|x| format!(\"{}\", x)).collect::<Vec<_>>().join(\", \")",
                    "    }",
                    "}",
                ],
            },
            TestCase {
                language: Language::JavaScript,
                filename: "test.js",
                lines: vec![
                    "import { useState, useEffect } from 'react';",
                    "import axios from 'axios';",
                    "",
                    "/**",
                    " * A React component with various JavaScript/JSX syntax",
                    " */",
                    "export default function DataDashboard({ userId, apiKey }) {",
                    "    const [data, setData] = useState(null);",
                    "    const [loading, setLoading] = useState(true);",
                    "    const [error, setError] = useState(null);",
                    "    ",
                    "    useEffect(() => {",
                    "        const fetchData = async () => {",
                    "            try {",
                    "                const response = await axios.get(`/api/users/${userId}/data`, {",
                    "                    headers: { 'Authorization': `Bearer ${apiKey}` }",
                    "                });",
                    "                ",
                    "                const processedData = response.data.map(item => ({",
                    "                    ...item,",
                    "                    timestamp: new Date(item.timestamp),",
                    "                    value: parseFloat(item.value) || 0",
                    "                }));",
                    "                ",
                    "                setData(processedData);",
                    "            } catch (err) {",
                    "                console.error('Failed to fetch data:', err);",
                    "                setError(err.message || 'Unknown error occurred');",
                    "            } finally {",
                    "                setLoading(false);",
                    "            }",
                    "        };",
                    "        ",
                    "        fetchData();",
                    "    }, [userId, apiKey]);",
                    "    ",
                    "    if (loading) return <div className=\"spinner\">Loading...</div>;",
                    "    if (error) return <div className=\"error\">Error: {error}</div>;",
                    "    ",
                    "    return (",
                    "        <div className=\"dashboard\">",
                    "            <h1>User Dashboard</h1>",
                    "            {data?.map((item, index) => (",
                    "                <DataCard key={item.id || index} {...item} />",
                    "            ))}",
                    "        </div>",
                    "    );",
                    "}",
                ],
            },
            TestCase {
                language: Language::Python,
                filename: "test.py",
                lines: vec![
                    "#!/usr/bin/env python3",
                    "\"\"\"A Python module demonstrating various syntax elements.\"\"\"",
                    "",
                    "import asyncio",
                    "import json",
                    "from typing import List, Optional, Dict, Any",
                    "from dataclasses import dataclass, field",
                    "from datetime import datetime, timedelta",
                    "",
                    "@dataclass",
                    "class Configuration:",
                    "    \"\"\"Configuration settings for the application.\"\"\"",
                    "    api_key: str",
                    "    timeout: int = 30",
                    "    retry_count: int = 3",
                    "    features: List[str] = field(default_factory=list)",
                    "    metadata: Dict[str, Any] = field(default_factory=dict)",
                    "",
                    "class DataProcessor:",
                    "    \"\"\"Processes data with various Python features.\"\"\"",
                    "    ",
                    "    def __init__(self, config: Configuration):",
                    "        self.config = config",
                    "        self._cache = {}",
                    "        self._session = None",
                    "    ",
                    "    async def process_batch(self, items: List[Dict[str, Any]]) -> List[Dict[str, Any]]:",
                    "        \"\"\"Process a batch of items asynchronously.\"\"\"",
                    "        results = []",
                    "        ",
                    "        async with self._get_session() as session:",
                    "            tasks = [self._process_item(session, item) for item in items]",
                    "            completed = await asyncio.gather(*tasks, return_exceptions=True)",
                    "            ",
                    "            for i, result in enumerate(completed):",
                    "                if isinstance(result, Exception):",
                    "                    print(f\"Error processing item {i}: {result}\")",
                    "                    continue",
                    "                results.append(result)",
                    "        ",
                    "        return results",
                    "    ",
                    "    @staticmethod",
                    "    def validate_data(data: Any) -> bool:",
                    "        \"\"\"Validate input data using pattern matching.\"\"\"",
                    "        match data:",
                    "            case {'type': 'user', 'id': int(user_id), 'name': str(name)}:",
                    "                return user_id > 0 and len(name) > 0",
                    "            case {'type': 'product', 'price': float(price) | int(price)}:",
                    "                return price >= 0",
                    "            case _:",
                    "                return False",
                    "",
                    "    def _transform_item(self, item: Dict[str, Any]) -> Dict[str, Any]:",
                    "        \"\"\"Transform an item using various Python features.\"\"\"",
                    "        # List comprehension with conditional",
                    "        filtered_keys = [k for k, v in item.items() if v is not None]",
                    "        ",
                    "        # Dictionary comprehension",
                    "        transformed = {",
                    "            k: v * 2 if isinstance(v, (int, float)) else v.upper() if isinstance(v, str) else v",
                    "            for k, v in item.items()",
                    "            if k in filtered_keys",
                    "        }",
                    "        ",
                    "        # F-string formatting",
                    "        transformed['processed_at'] = f\"{datetime.now():%Y-%m-%d %H:%M:%S}\"",
                    "        ",
                    "        # Walrus operator",
                    "        if (total := sum(v for v in transformed.values() if isinstance(v, (int, float)))) > 100:",
                    "            transformed['high_value'] = True",
                    "            transformed['total'] = total",
                    "        ",
                    "        return transformed",
                ],
            },
        ]
    }

    /// Test highlighting performance for individual lines
    #[test]
    fn test_single_line_performance() {
        let mut service = HighlightingService::new();
        let test_cases = get_test_cases();
        
        for test_case in test_cases {
            let mut state = service.create_highlighting_state(test_case.filename);
            
            // Warm up the highlighter
            for line in &test_case.lines[..5.min(test_case.lines.len())] {
                let _ = service.highlight_line(&mut state, line, 0);
            }
            
            // Test each line
            for (line_number, line) in test_case.lines.iter().enumerate() {
                let start = Instant::now();
                let result = service.highlight_line(&mut state, line, line_number);
                let duration = start.elapsed();
                
                assert!(result.is_ok(), "Highlighting failed for {:?} line {}", test_case.language, line_number);
                assert!(
                    duration < MAX_LINE_TIME,
                    "Line {} of {:?} took {:?} (max allowed: {:?})",
                    line_number,
                    test_case.language,
                    duration,
                    MAX_LINE_TIME
                );
            }
        }
    }

    /// Test highlighting performance for a typical viewport (50 lines)
    #[test]
    fn test_viewport_performance() {
        let mut service = HighlightingService::new();
        let test_cases = get_test_cases();
        
        for test_case in test_cases {
            let mut state = service.create_highlighting_state(test_case.filename);
            
            // Simulate highlighting a viewport of lines
            let start = Instant::now();
            
            for (line_number, line) in test_case.lines.iter().enumerate() {
                let _ = service.highlight_line(&mut state, line, line_number);
            }
            
            let duration = start.elapsed();
            
            assert!(
                duration < MAX_VIEWPORT_TIME,
                "Viewport highlighting for {:?} took {:?} (max allowed: {:?})",
                test_case.language,
                duration,
                MAX_VIEWPORT_TIME
            );
        }
    }

    /// Test cache performance (second pass should be much faster)
    #[test]
    fn test_cache_performance() {
        let mut service = HighlightingService::new();
        let test_case = &get_test_cases()[0]; // Use Rust test case
        let mut state = service.create_highlighting_state(test_case.filename);
        
        // First pass - populate cache
        let start_first = Instant::now();
        for (line_number, line) in test_case.lines.iter().enumerate() {
            let _ = service.highlight_line(&mut state, line, line_number);
        }
        let duration_first = start_first.elapsed();
        
        // Second pass - should use cache
        let start_second = Instant::now();
        for (line_number, line) in test_case.lines.iter().enumerate() {
            let _ = service.highlight_line(&mut state, line, line_number);
        }
        let duration_second = start_second.elapsed();
        
        // Cache should make second pass at least 10x faster
        assert!(
            duration_second < duration_first / 10,
            "Cache performance not sufficient. First: {:?}, Second: {:?}",
            duration_first,
            duration_second
        );
        
        // Verify cache hits
        assert_eq!(state.metrics.cache_hits, test_case.lines.len());
        // The cache hit ratio should be at least 0.5 (50%)
        // We have N cache misses from the first pass and N cache hits from the second pass
        // So the ratio should be N / (N + N) = 0.5
        assert!(state.metrics.cache_hit_ratio() >= 0.5, 
            "Cache hit ratio {} is too low", state.metrics.cache_hit_ratio());
    }

    /// Test performance with extremely long lines
    #[test]
    fn test_long_line_performance() {
        let mut service = HighlightingService::new();
        let mut state = service.create_highlighting_state("test.rs");
        
        // Create a very long line
        let long_line = "let data = vec![".to_string() + 
            &(0..1000).map(|i| format!("{}", i)).collect::<Vec<_>>().join(", ") + 
            "];";
        
        let start = Instant::now();
        let result = service.highlight_line(&mut state, &long_line, 0);
        let duration = start.elapsed();
        
        assert!(result.is_ok());
        
        // Should still complete reasonably quickly even for long lines
        assert!(
            duration < Duration::from_millis(200),
            "Long line took {:?}",
            duration
        );
    }

    /// Test performance metrics collection
    #[test]
    fn test_metrics_collection() {
        let mut service = HighlightingService::new();
        let test_case = &get_test_cases()[0];
        let mut state = service.create_highlighting_state(test_case.filename);
        
        // Reset metrics
        service.reset_metrics();
        state.metrics.reset();
        
        // Highlight some lines
        for (line_number, line) in test_case.lines.iter().enumerate().take(10) {
            let _ = service.highlight_line(&mut state, line, line_number);
        }
        
        // Check metrics
        assert_eq!(state.metrics.lines_highlighted, 10);
        assert!(state.metrics.tokens_generated > 0);
        assert!(state.metrics.avg_time_per_line > Duration::from_nanos(0));
        assert!(state.metrics.max_line_time > Duration::from_nanos(0));
        assert!(state.metrics.max_line_time < MAX_LINE_TIME);
        
        // Global metrics should also be updated
        let global_metrics = service.global_metrics();
        assert_eq!(global_metrics.lines_highlighted, 10);
    }
}