use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct QueueInfo {
    pub url: String,
    pub name: String,
    pub approximate_messages: i64,
    pub approximate_messages_not_visible: i64,
    pub approximate_messages_delayed: i64,
    #[allow(dead_code)]
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct QueueDetails {
    pub arn: Option<String>,
    pub created_timestamp: Option<i64>,
    #[allow(dead_code)]
    pub last_modified_timestamp: Option<i64>,
    pub message_retention_period: Option<i32>,
    pub visibility_timeout: Option<i32>,
    pub maximum_message_size: Option<i32>,
    pub delay_seconds: Option<i32>,
}

impl Default for QueueDetails {
    fn default() -> Self {
        Self {
            arn: None,
            created_timestamp: None,
            last_modified_timestamp: None,
            message_retention_period: None,
            visibility_timeout: None,
            maximum_message_size: None,
            delay_seconds: None,
        }
    }
}
