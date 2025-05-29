// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Syntax highlighting integration for text buffer rendering.

use crate::framebuffer::Framebuffer;
use crate::helpers::{CoordType, Rect};
use crate::syntax::{get_line_tokens, get_line_tokens_with_viewport, global_color_mapper, TokenInfo};
use crate::buffer::TextBuffer;

/// Renders a line of text with syntax highlighting to the framebuffer.
/// 
/// This function replaces the standard `fb.replace_text` call with multiple
/// calls that apply appropriate colors to each syntax token.
pub fn render_highlighted_line(
    fb: &mut Framebuffer,
    buffer: &TextBuffer,
    line_content: &str,
    line_number: usize,
    y: CoordType,
    left: CoordType,
    right: CoordType,
) {
    render_highlighted_line_with_viewport(fb, buffer, line_content, line_number, y, left, right, None, None);
}

/// Renders a line of text with syntax highlighting to the framebuffer with viewport information.
/// 
/// This function replaces the standard `fb.replace_text` call with multiple
/// calls that apply appropriate colors to each syntax token.
pub fn render_highlighted_line_with_viewport(
    fb: &mut Framebuffer,
    buffer: &TextBuffer,
    line_content: &str,
    line_number: usize,
    y: CoordType,
    left: CoordType,
    right: CoordType,
    viewport_start: Option<usize>,
    viewport_end: Option<usize>,
) {
    // Try to get syntax tokens for this line with viewport information
    let tokens = if let (Some(start), Some(end)) = (viewport_start, viewport_end) {
        get_line_tokens_with_viewport(buffer, line_content, line_number, start, end)
    } else {
        // Fallback to basic highlighting without viewport tracking
        get_line_tokens(buffer, line_content, line_number)
    };
    
    if let Some(tokens) = tokens {
        if !tokens.is_empty() {
            render_with_tokens(fb, &tokens, y, left, right);
            return;
        }
    }
    
    // Fallback to normal rendering without highlighting
    fb.replace_text(y, left, right, line_content);
}

/// Renders a line using syntax highlighting tokens.
fn render_with_tokens(
    fb: &mut Framebuffer,
    tokens: &[TokenInfo],
    y: CoordType,
    left: CoordType,
    right: CoordType,
) {
    let color_mapper = global_color_mapper();
    let mut current_x = left;
    
    for token in tokens {
        if current_x >= right {
            break;
        }
        
        let token_right = (current_x + token.text.chars().count() as CoordType).min(right);
        
        // Apply the token's text
        fb.replace_text(y, current_x, token_right, &token.text);
        
        // Apply the token's color if it has a type
        if let Some(ref kind) = token.kind {
            let color = color_mapper.get_color(kind);
            let color_rgba = fb.indexed(color);
            fb.blend_fg(Rect { left: current_x, top: y, right: token_right, bottom: y + 1 }, color_rgba);
        }
        
        current_x = token_right;
    }
    
    // Clear any remaining space on the line
    if current_x < right {
        fb.replace_text(y, current_x, right, "");
    }
}

