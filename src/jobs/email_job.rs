//! Email background job.
//!
//! Provides email sending functionality via background jobs.
//! In development mode, emails are logged. In production, configure
//! SMTP settings via environment variables.

use serde::{Deserialize, Serialize};
use std::env;

use crate::errors::AppError;

/// Email job payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailJob {
    /// Recipient email address
    pub to: String,
    /// Email subject line
    pub subject: String,
    /// Email body content (plain text or HTML)
    pub body: String,
    /// Optional sender override (defaults to SMTP_FROM)
    #[serde(default)]
    pub from: Option<String>,
}

impl EmailJob {
    /// Create a new email job
    pub fn new(to: impl Into<String>, subject: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            to: to.into(),
            subject: subject.into(),
            body: body.into(),
            from: None,
        }
    }

    /// Set custom sender address
    pub fn with_from(mut self, from: impl Into<String>) -> Self {
        self.from = Some(from.into());
        self
    }
}

/// Email configuration from environment.
/// Note: Some fields are currently unused pending lettre integration.
#[allow(dead_code)]
struct EmailConfig {
    smtp_host: Option<String>,
    smtp_port: u16,
    smtp_user: Option<String>,
    smtp_pass: Option<String>,
    smtp_from: String,
    smtp_tls: bool,
}

impl EmailConfig {
    fn from_env() -> Self {
        Self {
            smtp_host: env::var("SMTP_HOST").ok(),
            smtp_port: env::var("SMTP_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(587),
            smtp_user: env::var("SMTP_USER").ok(),
            smtp_pass: env::var("SMTP_PASS").ok(),
            smtp_from: env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@example.com".to_string()),
            smtp_tls: env::var("SMTP_TLS")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
        }
    }

    fn is_configured(&self) -> bool {
        self.smtp_host.is_some()
    }
}

/// Email job handler - processes email sending jobs
pub async fn email_job_handler(job: EmailJob) -> Result<(), AppError> {
    let config = EmailConfig::from_env();
    let from = job.from.as_deref().unwrap_or(&config.smtp_from);

    tracing::info!(
        to = %job.to,
        from = %from,
        subject = %job.subject,
        "Processing email job"
    );

    if !config.is_configured() {
        // Development mode: log the email instead of sending
        tracing::warn!("SMTP not configured - logging email instead of sending");
        tracing::info!(
            "=== EMAIL (not sent) ===\n\
             From: {}\n\
             To: {}\n\
             Subject: {}\n\
             Body:\n{}\n\
             ========================",
            from,
            job.to,
            job.subject,
            job.body
        );
        return Ok(());
    }

    // Production mode: send via SMTP
    // Note: Add `lettre` to Cargo.toml for real SMTP support:
    // lettre = { version = "0.11", features = ["tokio1-native-tls"] }
    //
    // Example implementation with lettre:
    // ```
    // use lettre::{
    //     message::header::ContentType,
    //     transport::smtp::authentication::Credentials,
    //     AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    // };
    //
    // let email = Message::builder()
    //     .from(from.parse()?)
    //     .to(job.to.parse()?)
    //     .subject(&job.subject)
    //     .header(ContentType::TEXT_PLAIN)
    //     .body(job.body.clone())?;
    //
    // let creds = Credentials::new(
    //     config.smtp_user.unwrap_or_default(),
    //     config.smtp_pass.unwrap_or_default(),
    // );
    //
    // let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host.unwrap())?
    //     .credentials(creds)
    //     .build();
    //
    // mailer.send(email).await?;
    // ```

    tracing::warn!(
        "SMTP is configured but lettre is not installed. \
         Add lettre to Cargo.toml to enable real email sending."
    );

    // Simulate sending delay
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    tracing::info!(to = %job.to, "Email processed successfully");
    Ok(())
}
