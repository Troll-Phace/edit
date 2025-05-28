// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Text change notification system for syntax highlighting updates.
//!
//! This module provides a mechanism to notify the highlighting system
//! when text changes occur in the buffer, enabling incremental re-highlighting.

use crate::syntax::render_bridge;
use crate::buffer::TextBuffer;
use crate::helpers::Point;

/// Notification of a text change in the buffer.
#[derive(Debug, Clone)]
pub struct TextChangeNotification {
    /// The line number where the change started
    pub start_line: usize,
    /// The line number where the change ended
    pub end_line: usize,
    /// The number of lines that were added (positive) or removed (negative)
    pub line_delta: isize,
    /// The type of change that occurred
    pub change_type: TextChangeType,
}

/// Type of text change that occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextChangeType {
    /// Text was inserted
    Insert,
    /// Text was deleted
    Delete,
    /// Text was replaced
    Replace,
    /// Multiple changes occurred (e.g., undo/redo)
    Multiple,
}

impl TextChangeNotification {
    /// Creates a new text change notification.
    pub fn new(
        start_line: usize,
        end_line: usize,
        line_delta: isize,
        change_type: TextChangeType,
    ) -> Self {
        Self {
            start_line,
            end_line,
            line_delta,
            change_type,
        }
    }

    /// Creates a notification for a single-line change.
    pub fn single_line(line: usize, change_type: TextChangeType) -> Self {
        Self::new(line, line, 0, change_type)
    }

    /// Creates a notification for an insertion.
    pub fn insert(start_line: usize, lines_added: usize) -> Self {
        Self::new(
            start_line,
            start_line + lines_added,
            lines_added as isize,
            TextChangeType::Insert,
        )
    }

    /// Creates a notification for a deletion.
    pub fn delete(start_line: usize, lines_deleted: usize) -> Self {
        Self::new(
            start_line,
            start_line,
            -(lines_deleted as isize),
            TextChangeType::Delete,
        )
    }

    /// Creates a notification for a text replacement.
    pub fn replace(start_line: usize, end_line: usize, line_delta: isize) -> Self {
        Self::new(
            start_line,
            end_line,
            line_delta,
            TextChangeType::Replace,
        )
    }
}

/// Notifies the highlighting system about text changes.
pub fn notify_text_change(
    buffer: &TextBuffer,
    notification: &TextChangeNotification,
) {
    // Get the highlighting state for this buffer
    if let Some(state_rc) = render_bridge::get_buffer_highlighting(buffer) {
        let mut state = state_rc.borrow_mut();
        
        match notification.change_type {
            TextChangeType::Insert => {
                let lines_added = notification.line_delta.max(0) as usize;
                state.handle_text_insert(notification.start_line, lines_added);
            }
            TextChangeType::Delete => {
                let lines_deleted = (-notification.line_delta).max(0) as usize;
                state.handle_text_delete(notification.start_line, lines_deleted);
            }
            TextChangeType::Replace | TextChangeType::Multiple => {
                // For replace or multiple changes, invalidate the affected range
                state.mark_lines_dirty(notification.start_line, notification.end_line);
                
                // If lines were added or removed, we need to shift the cache
                if notification.line_delta > 0 {
                    state.handle_text_insert(notification.end_line, notification.line_delta as usize);
                } else if notification.line_delta < 0 {
                    state.handle_text_delete(notification.end_line, (-notification.line_delta) as usize);
                }
            }
        }
    }
}

/// Calculates the line delta between two cursor positions.
pub fn calculate_line_delta(before: Point, after: Point) -> isize {
    after.y as isize - before.y as isize
}

/// Notifies about a text edit operation.
/// This should be called after any text modification in the buffer.
pub fn notify_edit_operation(
    buffer: &TextBuffer,
    cursor_before: Point,
    cursor_after: Point,
    was_deletion: bool,
) {
    let line_delta = calculate_line_delta(cursor_before, cursor_after);
    
    let notification = if was_deletion {
        if line_delta < 0 {
            TextChangeNotification::delete(cursor_after.y as usize, (-line_delta) as usize)
        } else {
            TextChangeNotification::single_line(cursor_before.y as usize, TextChangeType::Delete)
        }
    } else {
        if line_delta > 0 {
            TextChangeNotification::insert(cursor_before.y as usize, line_delta as usize)
        } else {
            TextChangeNotification::single_line(cursor_after.y as usize, TextChangeType::Insert)
        }
    };
    
    notify_text_change(buffer, &notification);
}

/// Notifies about an undo/redo operation.
/// This marks a wider range as dirty since undo/redo can affect multiple lines.
pub fn notify_undo_redo(
    buffer: &TextBuffer,
    start_line: usize,
    end_line: usize,
    line_delta: isize,
) {
    let notification = TextChangeNotification::new(
        start_line,
        end_line,
        line_delta,
        TextChangeType::Multiple,
    );
    
    notify_text_change(buffer, &notification);
}

/// Notifies about a find-and-replace operation.
/// This should be called when text is replaced via find/replace functionality.
#[allow(dead_code)]
pub fn notify_replace_operation(
    buffer: &TextBuffer,
    start_line: usize,
    end_line: usize,
    line_delta: isize,
) {
    let notification = TextChangeNotification::replace(start_line, end_line, line_delta);
    notify_text_change(buffer, &notification);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_change_notification_creation() {
        let notif = TextChangeNotification::single_line(5, TextChangeType::Insert);
        assert_eq!(notif.start_line, 5);
        assert_eq!(notif.end_line, 5);
        assert_eq!(notif.line_delta, 0);
        assert_eq!(notif.change_type, TextChangeType::Insert);

        let notif = TextChangeNotification::insert(10, 3);
        assert_eq!(notif.start_line, 10);
        assert_eq!(notif.end_line, 13);
        assert_eq!(notif.line_delta, 3);
        assert_eq!(notif.change_type, TextChangeType::Insert);

        let notif = TextChangeNotification::delete(20, 2);
        assert_eq!(notif.start_line, 20);
        assert_eq!(notif.end_line, 20);
        assert_eq!(notif.line_delta, -2);
        assert_eq!(notif.change_type, TextChangeType::Delete);

        let notif = TextChangeNotification::replace(15, 18, 1);
        assert_eq!(notif.start_line, 15);
        assert_eq!(notif.end_line, 18);
        assert_eq!(notif.line_delta, 1);
        assert_eq!(notif.change_type, TextChangeType::Replace);
    }

    #[test]
    fn test_calculate_line_delta() {
        let before = Point { x: 0, y: 10 };
        let after = Point { x: 0, y: 15 };
        assert_eq!(calculate_line_delta(before, after), 5);

        let before = Point { x: 0, y: 20 };
        let after = Point { x: 0, y: 18 };
        assert_eq!(calculate_line_delta(before, after), -2);

        let before = Point { x: 0, y: 5 };
        let after = Point { x: 10, y: 5 };
        assert_eq!(calculate_line_delta(before, after), 0);
    }
}