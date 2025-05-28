// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Performance baseline establishment script for Phase 0.
//!
//! This script runs comprehensive performance tests to establish baseline
//! measurements for syntax highlighting operations, file loading, and memory usage.

use std::time::Instant;
use edit::syntax::{run_baseline_test, global_highlighting_service};

fn main() {
    println!("=== Edit Syntax Highlighting Performance Baseline ===");
    println!("Phase 0 - Establishing performance baseline\n");

    // Test file set representing typical usage
    let test_files = vec![
        "test_small.rs",       // Small Rust file
        "test_medium.js",      // Medium JavaScript file
        "test_large.py",       // Large Python file
        "test_json.json",      // JSON configuration
        "test_markdown.md",    // Markdown documentation
        "test_typescript.ts",  // TypeScript code
        "test_css.css",        // CSS styles
        "test_html.html",      // HTML markup
        "test_yaml.yml",       // YAML configuration
        "test_toml.toml",      // TOML configuration
    ];

    println!("Running baseline tests with {} test files...\n", test_files.len());

    // Run the comprehensive baseline test
    let start_time = Instant::now();
    let measurement = run_baseline_test(&test_files);
    let total_time = start_time.elapsed();

    println!("Baseline test completed in {:.2}s\n", total_time.as_secs_f64());

    // Generate and display the performance report
    let report = measurement.generate_report();
    println!("{}", report);

    // Check if performance meets Phase 0 requirements
    let (meets_requirements, issues) = measurement.meets_requirements();
    
    if meets_requirements {
        println!("✅ All performance requirements met!");
        println!("Phase 0 baseline successfully established.");
    } else {
        println!("⚠️  Performance issues detected:");
        for issue in &issues {
            println!("  - {}", issue);
        }
        println!("\nRecommendations:");
        println!("  - Review highlighting algorithm efficiency");
        println!("  - Optimize token generation and caching");
        println!("  - Consider reducing memory overhead");
    }

    // Test syntax highlighting service integration
    println!("\n=== Testing Syntax Highlighting Service Integration ===");
    test_service_integration();

    println!("\n=== Performance Baseline Complete ===");
}

fn test_service_integration() {
    let mut service = global_highlighting_service();
    
    // Test creating highlighting states for different languages
    let test_cases = vec![
        ("main.rs", "Rust"),
        ("app.js", "JavaScript"),
        ("script.py", "Python"),
        ("config.json", "JSON"),
        ("README.md", "Markdown"),
        ("style.css", "CSS"),
        ("index.html", "HTML"),
        ("data.yaml", "YAML"),
        ("config.toml", "TOML"),
        ("app.ts", "TypeScript"),
        ("unknown.xyz", "PlainText"),
    ];

    println!("Testing language detection and state creation:");
    
    for (filename, expected_lang) in test_cases {
        let mut state = service.create_highlighting_state(filename);
        println!("  {} -> {} (enabled: {})", filename, expected_lang, state.enabled);
        
        // Test highlighting a sample line
        let sample_line = match expected_lang {
            "Rust" => "fn main() { println!(\"Hello, world!\"); }",
            "JavaScript" => "console.log('Hello, world!');",
            "Python" => "print('Hello, world!')",
            "JSON" => "{ \"message\": \"Hello, world!\" }",
            _ => "Hello, world!",
        };
        
        let start = Instant::now();
        match service.highlight_line(&mut state, sample_line, 0) {
            Ok(tokens) => {
                let duration = start.elapsed();
                println!("    Highlighted {} chars -> {} tokens in {}μs", 
                    sample_line.len(), tokens.len(), duration.as_micros());
            }
            Err(e) => {
                println!("    Error highlighting: {}", e);
            }
        }
    }
    
    // Test performance with various line lengths
    println!("\nTesting line length performance:");
    let mut rust_state = service.create_highlighting_state("test.rs");
    
    let extra_long_line = "x".repeat(1000);
    let test_lines = vec![
        "fn main() {",  // Short line
        "pub fn complex_function(param1: &str, param2: Option<i32>, param3: Vec<String>) -> Result<(), Error> {",  // Normal line
        "let very_long_variable_name_that_demonstrates_long_line_handling = some_complex_expression_with_many_method_calls().and_then(|result| result.map(|x| x.to_string())).unwrap_or_else(|| default_value.clone());",  // Long line
        &extra_long_line,  // Extra long line
    ];
    
    for (i, line) in test_lines.iter().enumerate() {
        let start = Instant::now();
        match service.highlight_line(&mut rust_state, line, i) {
            Ok(tokens) => {
                let duration = start.elapsed();
                println!("  Line {} chars: {} tokens in {}μs", 
                    line.len(), tokens.len(), duration.as_micros());
            }
            Err(e) => {
                println!("  Error with {}-char line: {}", line.len(), e);
            }
        }
    }
    
    println!("\nService integration test completed.");
}
