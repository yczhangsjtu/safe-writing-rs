use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Status of a tool call
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToolCallStatus {
    Pending,      // Waiting for user confirmation
    Confirmed,    // User confirmed, ready to execute
    Executed,     // Successfully executed
    Failed,       // Execution failed
    Cancelled,    // User cancelled
}

/// A tool call from the assistant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub status: ToolCallStatus,
}

/// A message in the session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    pub id: usize,
    pub role: String,  // "user", "assistant", "tool"
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,  // For tool response messages
    pub timestamp: i64,
    pub compressed: bool,  // Whether this message was replaced by summary
}

/// The session containing conversation history
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Session {
    pub next_message_id: usize,
    pub messages: Vec<SessionMessage>,
    pub summary: Option<String>,  // Compressed summary of earlier messages
    pub summary_cutoff_id: Option<usize>,  // Messages with id < this are compressed
}

impl Session {
    pub fn new() -> Self {
        Self::default()
    }

    fn current_timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
    }

    /// Add a user message
    pub fn add_user_message(&mut self, content: String) -> SessionMessage {
        let message = SessionMessage {
            id: self.next_message_id,
            role: "user".to_string(),
            content,
            tool_calls: None,
            tool_call_id: None,
            timestamp: Self::current_timestamp(),
            compressed: false,
        };
        self.next_message_id += 1;
        self.messages.push(message.clone());
        message
    }

    /// Add an assistant message (possibly with tool calls)
    pub fn add_assistant_message(&mut self, content: String, tool_calls: Option<Vec<ToolCall>>) -> SessionMessage {
        let message = SessionMessage {
            id: self.next_message_id,
            role: "assistant".to_string(),
            content,
            tool_calls,
            tool_call_id: None,
            timestamp: Self::current_timestamp(),
            compressed: false,
        };
        self.next_message_id += 1;
        self.messages.push(message.clone());
        message
    }

    /// Add a tool result message
    pub fn add_tool_result(&mut self, tool_call_id: String, result: String) -> SessionMessage {
        let message = SessionMessage {
            id: self.next_message_id,
            role: "tool".to_string(),
            content: result,
            tool_calls: None,
            tool_call_id: Some(tool_call_id),
            timestamp: Self::current_timestamp(),
            compressed: false,
        };
        self.next_message_id += 1;
        self.messages.push(message.clone());
        message
    }

    /// Get messages for API (excluding compressed ones)
    pub fn get_messages_for_api(&self) -> Vec<SessionMessage> {
        self.messages.iter()
            .filter(|m| !m.compressed)
            .cloned()
            .collect()
    }

    /// Compress old messages into a summary
    pub fn compress(&mut self, summary: String) {
        // Mark all current messages as compressed
        for msg in &mut self.messages {
            msg.compressed = true;
        }
        self.summary_cutoff_id = Some(self.next_message_id);
        self.summary = Some(summary);
    }

    /// Edit a message's content
    pub fn edit_message(&mut self, id: usize, new_content: String) -> Result<(), String> {
        let msg = self.messages.iter_mut().find(|m| m.id == id)
            .ok_or_else(|| format!("Message with id {} not found", id))?;
        msg.content = new_content;
        Ok(())
    }

    /// Delete a message
    pub fn delete_message(&mut self, id: usize) -> Result<SessionMessage, String> {
        let index = self.messages.iter().position(|m| m.id == id)
            .ok_or_else(|| format!("Message with id {} not found", id))?;
        Ok(self.messages.remove(index))
    }

    /// Clear all messages
    pub fn clear(&mut self) {
        self.messages.clear();
        self.summary = None;
        self.summary_cutoff_id = None;
    }

    /// Get pending tool calls (those needing confirmation)
    pub fn get_pending_tool_calls(&self) -> Vec<&ToolCall> {
        self.messages.iter()
            .filter_map(|m| m.tool_calls.as_ref())
            .flat_map(|tc| tc.iter())
            .filter(|tc| tc.status == ToolCallStatus::Pending)
            .collect()
    }

    /// Confirm a tool call
    pub fn confirm_tool_call(&mut self, tool_call_id: &str) -> Result<(), String> {
        for msg in &mut self.messages {
            if let Some(tool_calls) = &mut msg.tool_calls {
                for tc in tool_calls {
                    if tc.id == tool_call_id {
                        tc.status = ToolCallStatus::Confirmed;
                        return Ok(());
                    }
                }
            }
        }
        Err(format!("Tool call {} not found", tool_call_id))
    }

    /// Cancel a tool call
    pub fn cancel_tool_call(&mut self, tool_call_id: &str) -> Result<(), String> {
        for msg in &mut self.messages {
            if let Some(tool_calls) = &mut msg.tool_calls {
                for tc in tool_calls {
                    if tc.id == tool_call_id {
                        tc.status = ToolCallStatus::Cancelled;
                        return Ok(());
                    }
                }
            }
        }
        Err(format!("Tool call {} not found", tool_call_id))
    }

    /// Mark a tool call as executed
    pub fn mark_tool_executed(&mut self, tool_call_id: &str) -> Result<(), String> {
        for msg in &mut self.messages {
            if let Some(tool_calls) = &mut msg.tool_calls {
                for tc in tool_calls {
                    if tc.id == tool_call_id {
                        tc.status = ToolCallStatus::Executed;
                        return Ok(());
                    }
                }
            }
        }
        Err(format!("Tool call {} not found", tool_call_id))
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_default() {
        let session = Session::default();
        assert!(session.messages.is_empty());
        assert!(session.summary.is_none());
    }

    #[test]
    fn test_add_user_message() {
        let mut session = Session::new();
        let msg = session.add_user_message("Hello".to_string());
        assert_eq!(msg.id, 0);
        assert_eq!(msg.role, "user");
        assert_eq!(session.messages.len(), 1);
    }

    #[test]
    fn test_add_assistant_with_tools() {
        let mut session = Session::new();
        let tool_call = ToolCall {
            id: "call_1".to_string(),
            tool_name: "read_passage".to_string(),
            arguments: serde_json::json!({"index": 0}),
            status: ToolCallStatus::Pending,
        };
        let msg = session.add_assistant_message("Let me read".to_string(), Some(vec![tool_call]));
        assert!(msg.tool_calls.is_some());
        assert_eq!(session.get_pending_tool_calls().len(), 1);
    }

    #[test]
    fn test_compress() {
        let mut session = Session::new();
        session.add_user_message("Hello".to_string());
        session.add_assistant_message("Hi".to_string(), None);
        session.compress("Summary of conversation".to_string());

        assert!(session.summary.is_some());
        assert!(session.messages.iter().all(|m| m.compressed));
        assert_eq!(session.get_messages_for_api().len(), 0);
    }

    #[test]
    fn test_json_roundtrip() {
        let mut session = Session::new();
        session.add_user_message("Test".to_string());
        session.add_assistant_message("Response".to_string(), None);

        let json = session.to_json();
        let parsed = Session::from_json(&json).unwrap();
        assert_eq!(parsed.messages.len(), 2);
    }
}