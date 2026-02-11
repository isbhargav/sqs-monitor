use crate::aws::sqs::SqsClient;
use crate::types::{QueueDetails, QueueInfo};
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::time::Duration;

pub struct App {
    pub queues: Vec<QueueInfo>,
    all_queues: Vec<QueueInfo>,
    pub selected_index: usize,
    pub selected_details: Option<QueueDetails>,
    pub last_refresh: Option<DateTime<Utc>>,
    pub refresh_interval: Duration,
    pub status_message: String,
    pub should_quit: bool,
    pub filter_non_empty: bool,
    pub awaiting_purge_confirmation: bool,
    pub purge_in_progress: bool,
    sqs_client: SqsClient,
}

impl App {
    pub async fn new() -> Result<Self> {
        let sqs_client = SqsClient::new().await?;
        Ok(Self {
            queues: Vec::new(),
            all_queues: Vec::new(),
            selected_index: 0,
            selected_details: None,
            last_refresh: None,
            refresh_interval: Duration::from_secs(30),
            status_message: "Initializing...".to_string(),
            should_quit: false,
            filter_non_empty: false,
            awaiting_purge_confirmation: false,
            purge_in_progress: false,
            sqs_client,
        })
    }

    pub async fn refresh_queues(&mut self) -> Result<()> {
        self.status_message = "Refreshing queues...".to_string();

        match self.sqs_client.list_queues().await {
            Ok(mut queues) => {
                // Sort queues by message count in descending order
                queues.sort_by(|a, b| b.approximate_messages.cmp(&a.approximate_messages));

                self.all_queues = queues;
                self.apply_filter();
                self.last_refresh = Some(Utc::now());

                let total_count = self.all_queues.len();
                let filtered_count = self.queues.len();
                self.status_message = if self.filter_non_empty {
                    format!(
                        "Connected to AWS | {} of {} queues (non-empty only)",
                        filtered_count, total_count
                    )
                } else {
                    format!("Connected to AWS | {} queues found", total_count)
                };

                // Reset selection if needed
                if self.selected_index >= self.queues.len() && !self.queues.is_empty() {
                    self.selected_index = 0;
                }

                // Refresh details for selected queue
                if !self.queues.is_empty() && self.selected_index < self.queues.len() {
                    self.refresh_selected_details().await?;
                }
            }
            Err(e) => {
                self.status_message = format!("Error: {}", e);
            }
        }

        Ok(())
    }

    pub async fn refresh_selected_details(&mut self) -> Result<()> {
        if let Some(queue) = self.queues.get(self.selected_index) {
            match self.sqs_client.get_queue_details(&queue.url).await {
                Ok(details) => {
                    self.selected_details = Some(details);
                }
                Err(e) => {
                    self.status_message = format!("Error fetching details: {}", e);
                }
            }
        }
        Ok(())
    }

    pub fn next_queue(&mut self) {
        if !self.queues.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.queues.len();
        }
    }

    pub fn previous_queue(&mut self) {
        if !self.queues.is_empty() {
            if self.selected_index > 0 {
                self.selected_index -= 1;
            } else {
                self.selected_index = self.queues.len() - 1;
            }
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn selected_queue(&self) -> Option<&QueueInfo> {
        self.queues.get(self.selected_index)
    }

    pub fn toggle_filter(&mut self) {
        self.filter_non_empty = !self.filter_non_empty;
        self.apply_filter();

        // Reset selection if needed
        if self.selected_index >= self.queues.len() && !self.queues.is_empty() {
            self.selected_index = 0;
        }

        let total_count = self.all_queues.len();
        let filtered_count = self.queues.len();
        self.status_message = if self.filter_non_empty {
            format!(
                "Filter: ON | {} of {} queues (non-empty only)",
                filtered_count, total_count
            )
        } else {
            format!("Filter: OFF | {} queues shown", total_count)
        };
    }

    fn apply_filter(&mut self) {
        if self.filter_non_empty {
            self.queues = self
                .all_queues
                .iter()
                .filter(|q| q.approximate_messages > 0)
                .cloned()
                .collect();
        } else {
            self.queues = self.all_queues.clone();
        }
    }

    pub fn request_purge_confirmation(&mut self) {
        if let Some(queue_name) = self.selected_queue().map(|q| q.name.clone()) {
            self.awaiting_purge_confirmation = true;
            self.status_message = format!(
                "Purge queue '{}'? Press Y to confirm, N to cancel",
                queue_name
            );
        }
    }

    pub fn begin_purge(&mut self) -> Option<(String, String)> {
        self.awaiting_purge_confirmation = false;

        if let Some(queue) = self.selected_queue() {
            let queue_name = queue.name.clone();
            let queue_url = queue.url.clone();

            self.purge_in_progress = true;
            self.status_message = format!("Purging queue '{}'...", queue_name);
            Some((queue_url, queue_name))
        } else {
            None
        }
    }

    pub async fn execute_purge(&mut self, queue_url: &str, queue_name: &str) -> Result<()> {
        match self.sqs_client.purge_queue(queue_url).await {
            Ok(_) => {
                self.status_message = format!("Queue '{}' purged successfully", queue_name);
                // Refresh queues to show updated counts
                self.refresh_queues().await?;
            }
            Err(e) => {
                self.status_message = format!("Failed to purge queue '{}': {}", queue_name, e);
            }
        }

        self.purge_in_progress = false;
        Ok(())
    }

    pub fn cancel_purge(&mut self) {
        self.awaiting_purge_confirmation = false;
        self.status_message = "Purge cancelled".to_string();
    }
}
