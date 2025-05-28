// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Bridge between syntax highlighting and text rendering.
//!
//! This module provides a way to associate highlighting state with text buffers
//! during rendering, allowing the framebuffer to apply syntax highlighting colors.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::buffer::TextBuffer;
use crate::syntax::{HighlightingState, TokenInfo, global_highlighting_service};

// A registry that maps TextBuffer instances to their highlighting states.
// This allows the rendering code to access highlighting information without
// modifying the TextBuffer interface.
thread_local! {
    static BUFFER_HIGHLIGHTING_REGISTRY: RefCell<HashMap<usize, Rc<RefCell<HighlightingState>>>> = RefCell::new(HashMap::new());
}

/// Associates a highlighting state with a text buffer.
/// The association is based on the buffer's memory address.
pub fn register_buffer_highlighting(buffer: &TextBuffer, state: Rc<RefCell<HighlightingState>>) {
    let buffer_id = buffer as *const _ as usize;
    BUFFER_HIGHLIGHTING_REGISTRY.with(|registry| {
        registry.borrow_mut().insert(buffer_id, state);
    });
}

/// Removes the highlighting state association for a text buffer.
pub fn unregister_buffer_highlighting(buffer: &TextBuffer) {
    let buffer_id = buffer as *const _ as usize;
    BUFFER_HIGHLIGHTING_REGISTRY.with(|registry| {
        registry.borrow_mut().remove(&buffer_id);
    });
}

/// Gets the highlighting state for a text buffer, if one is registered.
pub fn get_buffer_highlighting(buffer: &TextBuffer) -> Option<Rc<RefCell<HighlightingState>>> {
    let buffer_id = buffer as *const _ as usize;
    BUFFER_HIGHLIGHTING_REGISTRY.with(|registry| {
        registry.borrow().get(&buffer_id).cloned()
    })
}

/// Gets syntax highlighting tokens for a specific line in a buffer.
/// Returns None if no highlighting is available for the buffer.
pub fn get_line_tokens(buffer: &TextBuffer, line_content: &str, line_number: usize) -> Option<Vec<TokenInfo>> {
    let state_rc = get_buffer_highlighting(buffer)?;
    let mut state = state_rc.borrow_mut();
    
    if !state.enabled {
        return None;
    }
    
    // Get the highlighting service and highlight the line
    let mut service = global_highlighting_service();
    match service.highlight_line(&mut state, line_content, line_number) {
        Ok(tokens) => Some(tokens),
        Err(_) => None,
    }
}

/// Clears all buffer-highlighting associations.
/// This should be called when the application is shutting down or resetting.
pub fn clear_all_highlighting_associations() {
    BUFFER_HIGHLIGHTING_REGISTRY.with(|registry| {
        registry.borrow_mut().clear();
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::Language;

    #[test]
    fn test_buffer_highlighting_registry() {
        let buffer = TextBuffer::new(false).unwrap();
        let state = Rc::new(RefCell::new(HighlightingState::new(Language::Rust)));
        
        // Register the highlighting
        register_buffer_highlighting(&buffer, state.clone());
        
        // Retrieve it
        let retrieved = get_buffer_highlighting(&buffer);
        assert!(retrieved.is_some());
        
        // Unregister it
        unregister_buffer_highlighting(&buffer);
        
        // Should be gone
        let retrieved = get_buffer_highlighting(&buffer);
        assert!(retrieved.is_none());
    }
}