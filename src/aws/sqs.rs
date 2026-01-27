use anyhow::Result;
use aws_sdk_sqs::Client;
use chrono::Utc;

use crate::types::{QueueDetails, QueueInfo};

pub struct SqsClient {
    client: Client,
}

impl SqsClient {
    pub async fn new() -> Result<Self> {
        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);
        Ok(Self { client })
    }

    pub async fn list_queues(&self) -> Result<Vec<QueueInfo>> {
        let resp = self.client.list_queues().send().await?;

        let mut queues = Vec::new();
        let urls = resp.queue_urls();
        for url in urls {
            let queue_info = self.get_queue_info(url).await?;
            queues.push(queue_info);
        }

        Ok(queues)
    }

    async fn get_queue_info(&self, url: &str) -> Result<QueueInfo> {
        let resp = self
            .client
            .get_queue_attributes()
            .queue_url(url)
            .attribute_names(aws_sdk_sqs::types::QueueAttributeName::ApproximateNumberOfMessages)
            .attribute_names(
                aws_sdk_sqs::types::QueueAttributeName::ApproximateNumberOfMessagesNotVisible,
            )
            .attribute_names(
                aws_sdk_sqs::types::QueueAttributeName::ApproximateNumberOfMessagesDelayed,
            )
            .send()
            .await?;

        let empty_map = std::collections::HashMap::new();
        let attributes = resp.attributes().unwrap_or(&empty_map);

        let approximate_messages = attributes
            .get(&aws_sdk_sqs::types::QueueAttributeName::ApproximateNumberOfMessages)
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(0);

        let approximate_messages_not_visible = attributes
            .get(&aws_sdk_sqs::types::QueueAttributeName::ApproximateNumberOfMessagesNotVisible)
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(0);

        let approximate_messages_delayed = attributes
            .get(&aws_sdk_sqs::types::QueueAttributeName::ApproximateNumberOfMessagesDelayed)
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(0);

        let name = url.rsplit('/').next().unwrap_or("unknown").to_string();

        Ok(QueueInfo {
            url: url.to_string(),
            name,
            approximate_messages,
            approximate_messages_not_visible,
            approximate_messages_delayed,
            last_updated: Utc::now(),
        })
    }

    pub async fn get_queue_details(&self, url: &str) -> Result<QueueDetails> {
        let resp = self
            .client
            .get_queue_attributes()
            .queue_url(url)
            .attribute_names(aws_sdk_sqs::types::QueueAttributeName::All)
            .send()
            .await?;

        let empty_map = std::collections::HashMap::new();
        let attributes = resp.attributes().unwrap_or(&empty_map);

        Ok(QueueDetails {
            arn: attributes
                .get(&aws_sdk_sqs::types::QueueAttributeName::QueueArn)
                .cloned(),
            created_timestamp: attributes
                .get(&aws_sdk_sqs::types::QueueAttributeName::CreatedTimestamp)
                .and_then(|v| v.parse::<i64>().ok()),
            last_modified_timestamp: attributes
                .get(&aws_sdk_sqs::types::QueueAttributeName::LastModifiedTimestamp)
                .and_then(|v| v.parse::<i64>().ok()),
            message_retention_period: attributes
                .get(&aws_sdk_sqs::types::QueueAttributeName::MessageRetentionPeriod)
                .and_then(|v| v.parse::<i32>().ok()),
            visibility_timeout: attributes
                .get(&aws_sdk_sqs::types::QueueAttributeName::VisibilityTimeout)
                .and_then(|v| v.parse::<i32>().ok()),
            maximum_message_size: attributes
                .get(&aws_sdk_sqs::types::QueueAttributeName::MaximumMessageSize)
                .and_then(|v| v.parse::<i32>().ok()),
            delay_seconds: attributes
                .get(&aws_sdk_sqs::types::QueueAttributeName::DelaySeconds)
                .and_then(|v| v.parse::<i32>().ok()),
        })
    }

    pub async fn purge_queue(&self, url: &str) -> Result<()> {
        self.client.purge_queue().queue_url(url).send().await?;
        Ok(())
    }
}
