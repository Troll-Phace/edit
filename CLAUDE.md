# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# CODEBASE OVERVIEW

`edit` is a modern terminal text editor written in Rust, inspired by MS-DOS Editor. It prioritizes small binary size, high performance, and minimal dependencies.

## Essential Commands

**Build:**
```bash
cargo build --config .cargo/release.toml --release
```

**Run:**
```bash
./target/release/edit [OPTIONS] [FILE[:LINE[:COLUMN]]]
```

**Test:**
```bash
cargo test
```

**Benchmarks:**
```bash
cargo bench
```

## Architecture

### Core Design Principles
- **No line indexing**: The buffer doesn't track line breaks. Line navigation uses SIMD-optimized search achieving >100GB/s performance.
- **Immediate mode UI**: Custom TUI framework with framebuffer-based rendering for efficient terminal updates.
- **Minimal dependencies**: Only external dependency is `synoptic` for syntax highlighting.

### Key Components

**Buffer System** (`src/buffer/`):
- Gap buffer implementation without line tracking
- O(n) line seeking using SIMD operations
- Navigation logic in `navigation.rs`

**UI Framework** (`src/tui.rs`):
- Immediate mode design
- Video game-style framebuffer in `src/framebuffer.rs`
- Efficient diffing for terminal updates

**Platform Abstraction** (`src/sys/`):
- Unix and Windows implementations
- Raw terminal I/O handling
- Platform-specific optimizations

**Unicode Support** (`src/unicode/`):
- UTF-8 validation and manipulation
- Grapheme clustering
- Terminal width measurement

**Performance** (`src/simd/`):
- SIMD-optimized memchr2, memrchr2, memset
- Custom implementations for line searching

**Syntax Highlighting** (`src/syntax/`):
- Language detection and highlighting via Synoptic
- Performance-focused implementation

### Binary Structure
- Main application: `src/bin/edit/`
- Baseline binary: `src/bin/baseline.rs`

## Development Notes

- **Rust Version**: Requires nightly toolchain (configured in `rust-toolchain.toml`)
- **Release Profile**: Custom release configuration strips ~65% of binary size
- **Platform Builds**: Windows includes manifest, Unix links against libc
- **Code Style**: Run `cargo fmt` before committing

# Current Project Implementation Instructions

## Synoptic Syntax Highlighting Integration - MVP Production Requirements

## System Prompt

the codebase you are working with is a terminal text editor called Edit, which is a lightweight, fast, and efficient text editor designed for terminal use. It is built in Rust and focuses on simplicity, performance, and extensibility. The codebase includes features like a TUI (Text User Interface), file handling, and basic text editing capabilities. The goal is to enhance Edit with syntax highlighting capabilities using the Synoptic library while maintaining its core principles.

## Project Overview

Implement syntax highlighting capabilities for the Microsoft Edit terminal text editor using the Synoptic library. This MVP will focus on adding basic but effective syntax highlighting for common programming languages while maintaining Edit's philosophy of simplicity and performance.

**MVP Goals:**

- Basic syntax highlighting for 8-10 common programming languages
- Real-time highlighting during editing with minimal performance impact
- Automatic language detection based on file extensions
- Integration with existing TUI color scheme and theming
- Optional toggle to enable/disable highlighting
- Maintain Edit's sub-100ms response time for all operations

---

## Technology Stack Overview

### Core Highlighting Engine

**Synoptic v2.2.9**

- **Lightweight Design**: Only 3 main dependencies, perfect for Edit's minimal approach
- **Regex-Based**: Simple, predictable highlighting rules without complex parsing overhead
- **Incremental Updates**: Designed for text editors with efficient re-highlighting on edits
- **Built-in Language Support**: Pre-configured rules for major programming languages
- **Performance**: Fast enough for real-time highlighting without blocking the UI thread

### Integration Architecture

**Direct TUI Integration**

- **Framebuffer Integration**: Highlight tokens applied during TUI rendering phase
- **Color System**: Leverages Edit's existing IndexedColor system for terminal compatibility
- **Memory Efficient**: Minimal memory overhead with line-based token caching
- **Thread Safe**: All highlighting operations compatible with Edit's single-threaded model

### Language Detection

**Extension-Based Detection**

- **File Extension Mapping**: Simple, reliable mapping from file extensions to language types
- **Fallback Handling**: Graceful degradation for unknown file types
- **User Override**: Manual language selection when auto-detection fails
- **Performance**: Zero-cost detection using hash maps for O(1) lookups

### Supported Languages (MVP)

**Priority Tier 1 (Phase 1)**
- **Rust**: Primary target language with comprehensive highlighting
- **JavaScript/TypeScript**: Web development support
- **Python**: Data science and general programming
- **JSON**: Configuration files and data interchange

**Priority Tier 2 (Phase 2)**
- **HTML/CSS**: Web development markup and styling
- **Markdown**: Documentation and README files
- **TOML/YAML**: Configuration file formats
- **SQL**: Database query highlighting

### Key Libraries and Dependencies

#### Core Highlighting
- `synoptic = "2.2.9"` - Main syntax highlighting engine
- `regex = "1.10"` - Pattern matching for language detection
- `once_cell = "1.19"` - Lazy static initialization for language configs

#### Integration Components
- Existing `edit::tui` - TUI rendering framework integration
- Existing `edit::framebuffer` - Color and styling system
- Existing `edit::buffer` - Text buffer and line access

### Architecture Patterns

**Lazy Loading Pattern**
- Language configurations loaded on-demand when first needed
- Reduces startup time and memory usage for unused languages
- Thread-safe initialization using `once_cell::sync::Lazy`

**Incremental Highlighting Strategy**
- Only re-highlight modified lines and affected context
- Background highlighting for large files beyond viewport
- Viewport-priority highlighting for responsive user experience

**Color Mapping System**
- Synoptic token types mapped to Edit's IndexedColor palette
- Terminal capability detection for 8-bit vs 24-bit color support
- User-configurable color themes through existing theme system

**Performance-First Design**
- Sub-millisecond highlighting for typical line lengths (< 200 characters)
- Chunked processing for very long lines to prevent UI blocking
- Memory pooling for token allocations to reduce GC pressure

This technology stack provides a lightweight, performant foundation for syntax highlighting that integrates seamlessly with Edit's existing architecture while maintaining its speed and simplicity principles.

---

## Phase 0: Foundation & Language Detection

### Objectives

- Add Synoptic dependency to the project
- Implement basic file extension to language mapping
- Create highlighting infrastructure without visual changes
- Establish performance baseline and testing framework

### Requirements

#### 1. Dependency Integration

- Add Synoptic crate to Cargo.toml with appropriate feature flags
- Verify compatibility with existing dependencies
- Configure cargo features for optimal binary size
- Update build configuration if needed

#### 2. Language Detection System

- Create language detection module with extension mapping
- Implement fallback handling for unknown file types
- Add manual language override capability
- Create language configuration struct

#### 3. Infrastructure Setup

- Create syntax highlighting service module
- Implement lazy initialization for language parsers
- Add highlighting state to document structure
- Create mock highlighting pipeline (no visual output yet)

#### 4. Performance Baseline

- Establish performance benchmarks for file loading
- Measure memory usage baseline before highlighting
- Create test suite for language detection accuracy
- Document performance requirements and SLAs

### Completion Requirements Checklist

- [ ] Synoptic v2.2.9 added to dependencies without build errors
- [ ] Language detection correctly identifies 15+ file extensions
- [ ] Unknown file types handled gracefully without panics
- [ ] Manual language override works for test files
- [ ] Highlighting service module compiles and initializes
- [ ] Language parsers load lazily without startup performance impact
- [ ] Document structure extended with highlighting state
- [ ] Mock highlighting pipeline processes files without errors
- [ ] Performance benchmarks established and documented
- [ ] Memory usage measurements recorded for baseline
- [ ] Test suite passes 100% for language detection
- [ ] No regressions in existing Edit functionality

### Libraries/Frameworks/APIs for this Phase

- **Synoptic**: Core syntax highlighting engine
- **once_cell**: Lazy static initialization
- **regex**: Pattern matching for file extensions

---

## Phase 1: Basic Highlighting Implementation

### Objectives

- Implement visual syntax highlighting for Tier 1 languages
- Integrate highlighting with TUI rendering system
- Add color mapping from Synoptic tokens to Edit colors
- Ensure highlighting works in real-time during editing

### Requirements

#### 1. Core Highlighting Engine

- Implement Synoptic highlighter initialization for Rust, JavaScript, Python, JSON
- Create token-to-color mapping system using Edit's IndexedColor
- Integrate highlighting with framebuffer rendering pipeline
- Add per-line token caching for performance

#### 2. TUI Integration

- Modify text rendering to apply syntax highlighting colors
- Ensure highlighting respects terminal color capabilities
- Maintain cursor positioning accuracy with colored text
- Handle text selection highlighting correctly

#### 3. Real-time Highlighting

- Implement incremental re-highlighting on text changes
- Add highlighting for newly typed content
- Handle line insertions and deletions correctly
- Optimize for common editing operations (typing, backspace)

#### 4. Color Scheme Integration

- Map Synoptic token types to semantic color roles
- Respect user's terminal theme and color preferences
- Provide fallback colors for limited color terminals
- Ensure sufficient contrast for accessibility

#### 5. Performance Optimization

- Limit highlighting to visible viewport for large files
- Implement line-level caching to avoid redundant work
- Add timeout protection for extremely long lines
- Monitor and maintain sub-100ms response times

### Completion Requirements Checklist

- [ ] Rust syntax highlighting displays correctly (keywords, strings, comments)
- [ ] JavaScript highlighting works for functions, variables, strings
- [ ] Python highlighting shows keywords, strings, comments properly
- [ ] JSON highlighting displays keys, values, and structure
- [ ] Colors map correctly to Edit's existing color scheme
- [ ] Highlighting appears in real-time during typing
- [ ] Text selection works correctly over highlighted text
- [ ] Cursor positioning remains accurate with highlighting
- [ ] Performance remains under 50ms for typical lines (< 200 chars)
- [ ] Large files (> 1000 lines) load without blocking UI
- [ ] Memory usage increases by less than 50MB for typical files
- [ ] No visual artifacts or rendering glitches
- [ ] Highlighting toggles on/off correctly via settings

### Libraries/Frameworks/APIs for this Phase

- **Synoptic**: Syntax highlighting with built-in language rules
- **Edit TUI Framework**: Existing rendering and color systems
- **Edit Framebuffer**: Color application and text rendering

---

## Phase 2: Extended Language Support

### Objectives

- Add Tier 2 languages (HTML, CSS, Markdown, YAML, TOML, SQL)
- Implement user configuration for highlighting preferences
- Add language-specific optimization and fine-tuning
- Create comprehensive test coverage for all supported languages

### Requirements

#### 1. Additional Language Support

- Add HTML highlighting with tag, attribute, and content recognition
- Implement CSS highlighting for selectors, properties, and values
- Add Markdown highlighting for headers, links, code blocks, emphasis
- Support YAML and TOML configuration file highlighting
- Implement SQL highlighting for keywords, strings, and comments

#### 2. Configuration System

- Add highlighting enable/disable toggle to settings
- Implement per-language highlighting preferences
- Add color customization options for token types
- Create highlighting intensity/detail level settings

#### 3. Language-Specific Optimizations

- Fine-tune regex patterns for better accuracy
- Add language-specific token priorities
- Implement context-aware highlighting where beneficial
- Optimize performance for language-specific patterns

#### 4. Quality Assurance

- Create comprehensive test files for each language
- Add visual regression testing for highlighting accuracy
- Implement automated performance testing
- Add edge case handling for malformed syntax

### Completion Requirements Checklist

- [ ] HTML highlighting works for tags, attributes, and nested content
- [ ] CSS highlighting displays selectors, properties, and values correctly
- [ ] Markdown highlighting shows headers, emphasis, links, and code blocks
- [ ] YAML highlighting works for keys, values, and structure
- [ ] TOML highlighting displays sections, keys, and values
- [ ] SQL highlighting shows keywords, strings, and operators
- [ ] Settings panel includes highlighting enable/disable toggle
- [ ] Per-language preferences save and load correctly
- [ ] Color customization affects highlighting display
- [ ] All languages maintain performance requirements (< 100ms)
- [ ] Test coverage reaches 90%+ for highlighting logic
- [ ] Visual regression tests pass for all languages
- [ ] Memory usage remains reasonable (< 100MB additional)
- [ ] Edge cases handled gracefully without crashes

### Libraries/Frameworks/APIs for this Phase

- **Synoptic**: Extended language rule sets
- **Edit Settings**: Configuration persistence and UI
- **Test Framework**: Automated testing infrastructure

---

## Phase 3: Advanced Features & User Experience

### Objectives

- Add advanced highlighting features like nested languages
- Implement smart highlighting based on file content
- Add performance monitoring and optimization
- Create user documentation and help system

### Requirements

#### 1. Advanced Highlighting Features

- Support for embedded languages (JavaScript in HTML, SQL in strings)
- Add bracket/parentheses matching highlighting
- Implement TODO/FIXME comment highlighting
- Add URL and email highlighting in strings and comments

#### 2. Smart Content Detection

- Implement content-based language detection as fallback
- Add shebang line parsing for script files
- Support for mixed-content files (e.g., frontmatter in Markdown)
- Add heuristic detection for ambiguous extensions

#### 3. Performance Monitoring

- Add performance metrics collection for highlighting operations
- Implement automatic performance degradation for slow files
- Add memory usage monitoring and alerts
- Create performance profiling tools for debugging

#### 4. User Experience Enhancements

- Add syntax error indication (visual markers for parsing errors)
- Implement semantic highlighting hints where possible
- Add highlighting preview in file picker
- Create keyboard shortcuts for highlighting controls

#### 5. Documentation and Help

- Create user guide for syntax highlighting features
- Add tooltips and help text for highlighting settings
- Document performance characteristics and limitations
- Create troubleshooting guide for common issues

### Completion Requirements Checklist

- [ ] Embedded language highlighting works (JS in HTML, CSS in HTML)
- [ ] Bracket matching highlights corresponding pairs
- [ ] TODO/FIXME comments are visually distinct
- [ ] URLs and emails highlight in strings and comments
- [ ] Content-based detection works for ambiguous files
- [ ] Shebang parsing correctly identifies script languages
- [ ] Mixed-content files handle multiple syntaxes
- [ ] Performance metrics track highlighting operation times
- [ ] Automatic degradation activates for slow operations (> 200ms)
- [ ] Memory monitoring alerts on excessive usage (> 200MB)
- [ ] Syntax error indicators display appropriately
- [ ] File picker shows highlighting preview
- [ ] Keyboard shortcuts work for highlighting controls
- [ ] User guide explains all highlighting features
- [ ] Settings tooltips provide helpful information
- [ ] Troubleshooting guide addresses common problems

### Libraries/Frameworks/APIs for this Phase

- **Synoptic**: Advanced highlighting features
- **Edit Help System**: Documentation integration
- **Performance Monitoring**: Metrics collection and analysis

---

## Phase 4: Integration Testing & Polish

### Objectives

- Comprehensive integration testing with Edit's existing features
- Performance optimization and memory leak prevention
- Bug fixes and edge case handling
- Prepare for production deployment

### Requirements

#### 1. Integration Testing

- Test highlighting with all existing Edit features (search, replace, etc.)
- Verify compatibility with different terminal types and color modes
- Test with various file sizes and edge cases
- Ensure highlighting works correctly with undo/redo operations

#### 2. Performance Optimization

- Profile and optimize hot paths in highlighting code
- Implement memory pooling for frequently allocated objects
- Add caching for expensive operations
- Optimize regex compilation and reuse

#### 3. Robustness & Error Handling

- Add comprehensive error handling for malformed files
- Implement graceful degradation for unsupported content
- Add recovery mechanisms for highlighting failures
- Ensure thread safety for all highlighting operations

#### 4. Production Readiness

- Code review and security audit of highlighting system
- Performance regression testing
- Memory leak detection and prevention
- Final bug fixes and stability improvements

#### 5. Documentation Completion

- Complete API documentation for highlighting system
- Update user manual with highlighting features
- Create developer guide for extending language support
- Document configuration options and performance tuning

### Completion Requirements Checklist

- [ ] Search functionality works correctly with highlighted text
- [ ] Replace operations preserve highlighting appropriately
- [ ] Undo/redo maintains highlighting state correctly
- [ ] All terminal types display highlighting without issues
- [ ] 8-bit and 24-bit color modes both work correctly
- [ ] Files up to 10MB highlight without performance degradation
- [ ] Edge cases (empty files, binary files) handled gracefully
- [ ] Memory usage remains stable during extended use
- [ ] No memory leaks detected in 24-hour stress test
- [ ] Regex patterns compiled once and reused efficiently
- [ ] Error handling prevents crashes from malformed syntax
- [ ] Highlighting failures degrade gracefully to plain text
- [ ] All operations remain thread-safe
- [ ] Security audit completed with no vulnerabilities found
- [ ] Performance regression tests pass all benchmarks
- [ ] API documentation complete and accurate
- [ ] User manual updated with highlighting sections
- [ ] Developer guide enables easy language additions

### Libraries/Frameworks/APIs for this Phase

- **Testing Framework**: Comprehensive test suite
- **Profiling Tools**: Performance analysis and optimization
- **Documentation Tools**: API docs and user guides

---

## Phase 5: Deployment & Monitoring

### Objectives

- Deploy syntax highlighting as part of Edit release
- Implement telemetry for performance and usage monitoring
- Create support documentation and troubleshooting guides
- Plan for future enhancements and community contributions

### Requirements

#### 1. Deployment Preparation

- Final build testing across all supported platforms
- Package highlighting system with appropriate feature flags
- Create deployment checklist and rollback procedures
- Prepare release notes highlighting new features

#### 2. Telemetry & Monitoring

- Implement opt-in usage analytics for highlighting features
- Add performance monitoring for real-world usage
- Create dashboards for highlighting system health
- Set up alerts for performance degradation or errors

#### 3. Support Infrastructure

- Create FAQ for common highlighting questions
- Develop troubleshooting flowchart for support team
- Document known limitations and workarounds
- Prepare customer communication materials

#### 4. Future Planning

- Design plugin architecture for community language additions
- Plan performance improvements for next version
- Identify opportunities for AI-assisted highlighting
- Create roadmap for advanced features (LSP integration, etc.)

### Completion Requirements Checklist

- [ ] Highlighting system packages correctly on Windows, macOS, Linux
- [ ] Feature flags allow enabling/disabling highlighting at build time
- [ ] Release notes accurately describe highlighting capabilities
- [ ] Rollback procedures tested and documented
- [ ] Usage analytics collect appropriate data with user consent
- [ ] Performance monitoring tracks key metrics in production
- [ ] Health dashboards display highlighting system status
- [ ] Alerts trigger appropriately for performance issues
- [ ] FAQ addresses 90% of anticipated user questions
- [ ] Troubleshooting guide enables quick issue resolution
- [ ] Known limitations documented with workarounds
- [ ] Customer communication materials ready for release
- [ ] Plugin architecture designed for future extensibility
- [ ] Performance improvement plans documented
- [ ] AI enhancement opportunities identified
- [ ] Feature roadmap created for next 6-12 months

### Libraries/Frameworks/APIs for this Phase

- **Deployment Tools**: Build and packaging systems
- **Telemetry Framework**: Usage and performance analytics
- **Monitoring Systems**: Health dashboards and alerting

---

## Additional MVP Requirements

### Performance Targets

- **Highlighting Latency**: < 50ms for lines under 200 characters
- **Memory Overhead**: < 100MB additional for typical usage
- **File Loading**: No degradation in Edit's current load times
- **Large File Performance**: Files up to 5MB highlight without UI blocking
- **Startup Impact**: < 10ms additional startup time
- **Memory Leaks**: Zero memory leaks in 24-hour continuous operation

### Platform Support

- **Operating Systems**: Windows 10+, macOS 11+, Ubuntu 20.04+
- **Terminal Emulators**: All terminals supported by Edit
- **Color Support**: 8-bit, 24-bit, and monochrome terminals
- **Rust Version**: Compatible with Edit's current MSRV

### Quality Requirements

- **Test Coverage**: > 90% code coverage for highlighting system
- **Performance Regression**: < 5% degradation in existing Edit operations
- **Memory Safety**: All highlighting code must be memory-safe
- **Error Recovery**: Graceful handling of all syntax edge cases
- **Accessibility**: Sufficient color contrast for visually impaired users

### Configuration Requirements

- **Default State**: Syntax highlighting enabled by default
- **Toggle Control**: Easy on/off toggle in settings menu
- **Performance Mode**: Automatic degradation for slow operations
- **Color Customization**: Basic color scheme customization
- **Language Override**: Manual language selection when needed

### Documentation Requirements

- **User Guide**: Complete documentation of highlighting features
- **Performance Guide**: Best practices for optimal performance
- **Language Reference**: List of supported languages and features
- **Developer Guide**: Instructions for adding new languages
- **Troubleshooting**: Common issues and solutions

---

## Success Metrics

### Functional Success Metrics

- [ ] 95%+ of common file types highlight correctly on first attempt
- [ ] Language detection accuracy > 98% for supported extensions
- [ ] Zero crashes caused by syntax highlighting in normal usage
- [ ] Highlighting toggles on/off within 100ms
- [ ] All supported languages display with correct colors

### Performance Success Metrics

- [ ] Highlighting adds < 50ms to file opening for files under 1MB
- [ ] Memory usage increases by < 100MB for typical development sessions
- [ ] No measurable impact on Edit's existing performance benchmarks
- [ ] Large files (> 10K lines) remain responsive during editing
- [ ] Startup time increases by < 10ms with highlighting enabled

### User Experience Success Metrics

- [ ] Settings interface is intuitive and requires no documentation
- [ ] Color schemes provide sufficient contrast in all terminal types
- [ ] Highlighting enhances readability without causing visual fatigue
- [ ] Users can successfully disable highlighting if needed
- [ ] Error states provide clear guidance for resolution

### Technical Success Metrics

- [ ] Test suite maintains > 90% code coverage
- [ ] No memory leaks detected in automated testing
- [ ] All code passes security review without major findings
- [ ] Performance regression tests pass within 5% tolerance
- [ ] Documentation completeness score > 95%

This MVP delivers a robust, performant syntax highlighting system that enhances Edit's capabilities while maintaining its core principles of simplicity, speed, and reliability. The phased approach ensures thorough testing and optimization at each stage, leading to a production-ready feature that users will appreciate without compromising Edit's exceptional performance characteristics.