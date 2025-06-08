use std::path::PathBuf;
use std::sync::{Mutex, mpsc};
use std::time::{SystemTime, UNIX_EPOCH};
use std::thread;
use std::fmt;
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use log::{debug, info, warn, error};
use once_cell::sync::Lazy;
use blake3;

/// Global audit logger instance
static AUDIT_LOGGER: Lazy<Mutex<AuditLogger>> = Lazy::new(|| {
    Mutex::new(AuditLogger::new().expect("Failed to initialize audit logger"))
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Path to audit log file
    pub log_path: PathBuf,
    /// Maximum log file size in bytes
    pub max_log_size: u64,
    /// Number of log files to keep
    pub log_retention: u32,
    /// Whether to sign log entries
    pub sign_entries: bool,
    /// Events to audit
    pub audit_events: Vec<AuditEventType>,
    /// Minimum severity level to log
    pub min_severity: EventSeverity,
    /// Whether to enable real-time alerts for critical events
    pub enable_alerts: bool,
    /// Alert destination (e.g., "syslog", "email", "webhook")
    pub alert_destination: Option<String>,
    /// Alert configuration (JSON string)
    pub alert_config: Option<String>,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            log_path: PathBuf::from("/var/log/synx/audit.log"),
            max_log_size: 10 * 1024 * 1024, // 10MB
            log_retention: 5,
            sign_entries: true,
            audit_events: vec![
                AuditEventType::ToolExecution,
                AuditEventType::ConfigChange,
                AuditEventType::SecurityViolation,
                AuditEventType::FileAccess,
                AuditEventType::ValidationEvent,
                AuditEventType::ResourceEvent,
                AuditEventType::AuthorizationEvent,
                AuditEventType::ConfigurationEvent,
            ],
            min_severity: EventSeverity::Info,
            enable_alerts: false,
            alert_destination: None,
            alert_config: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditEventType {
    ToolExecution,
    ConfigChange,
    SecurityViolation,
    FileAccess,
    UserAction,
    SystemEvent,
    // New event types for comprehensive coverage
    ValidationEvent,
    ResourceEvent,
    AuthorizationEvent,
    ConfigurationEvent,
}

impl fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditEventType::ToolExecution => write!(f, "Tool Execution"),
            AuditEventType::ConfigChange => write!(f, "Config Change"),
            AuditEventType::SecurityViolation => write!(f, "Security Violation"),
            AuditEventType::FileAccess => write!(f, "File Access"),
            AuditEventType::UserAction => write!(f, "User Action"),
            AuditEventType::SystemEvent => write!(f, "System Event"),
            AuditEventType::ValidationEvent => write!(f, "Validation Event"),
            AuditEventType::ResourceEvent => write!(f, "Resource Event"),
            AuditEventType::AuthorizationEvent => write!(f, "Authorization Event"),
            AuditEventType::ConfigurationEvent => write!(f, "Configuration Event"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditEvent {
    /// Timestamp of the event
    timestamp: u64,
    /// Type of event
    event_type: AuditEventType,
    /// User who triggered the event
    user: String,
    /// Description of the event
    description: String,
    /// Additional context
    context: serde_json::Value,
    /// Cryptographic signature (if enabled)
    signature: Option<String>,
    /// Severity level of the event
    severity: EventSeverity,
    /// Correlation ID for event tracking
    correlation_id: Option<String>,
    /// Source component that generated the event
    source: String,
    /// Session or transaction ID
    session_id: Option<String>,
}

/// Severity levels for audit events
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl Default for EventSeverity {
    fn default() -> Self {
        EventSeverity::Info
    }
}

impl AuditEvent {
    pub fn new(
        event_type: AuditEventType,
        description: String,
        context: serde_json::Value,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        let user = get_current_user();
        
        Self {
            timestamp,
            event_type,
            user,
            description,
            context,
            signature: None,
            severity: EventSeverity::Info,
            correlation_id: None,
            source: "synx".to_string(),
            session_id: None,
        }
    }
    
    /// Create a new audit event with specified severity
    pub fn with_severity(
        event_type: AuditEventType,
        description: String,
        context: serde_json::Value,
        severity: EventSeverity,
    ) -> Self {
        let mut event = Self::new(event_type, description, context);
        event.severity = severity;
        event
    }
    
    /// Set the correlation ID for event tracking
    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
    
    /// Set the source component
    pub fn with_source(mut self, source: String) -> Self {
        self.source = source;
        self
    }
    
    /// Set the session ID
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Sign the event data
    fn sign(&mut self, key: &[u8]) {
        let event_data = serde_json::to_vec(&(
            self.timestamp,
            &self.event_type,
            &self.user,
            &self.description,
            &self.context
        )).unwrap_or_default();
        
        // Use regular hash if key is not the expected size
        let signature = if key.len() == blake3::KEY_LEN {
            // Convert slice to array reference
            let key_array: &[u8; blake3::KEY_LEN] = key.try_into().unwrap_or(&[0; blake3::KEY_LEN]);
            blake3::keyed_hash(key_array, &event_data)
        } else {
            // Fall back to regular hash if key size is incorrect
            blake3::hash(&event_data)
        };
        
        self.signature = Some(signature.to_hex().to_string());
    }

    /// Verify the event signature
    fn verify(&self, key: &[u8]) -> bool {
        if let Some(sig) = &self.signature {
            let event_data = serde_json::to_vec(&(
                self.timestamp,
                &self.event_type,
                &self.user,
                &self.description,
                &self.context
            )).unwrap_or_default();
            
            // Use regular hash if key is not the expected size
            let expected = if key.len() == blake3::KEY_LEN {
                // Convert slice to array reference
                let key_array: &[u8; blake3::KEY_LEN] = key.try_into().unwrap_or(&[0; blake3::KEY_LEN]);
                blake3::keyed_hash(key_array, &event_data)
            } else {
                // Fall back to regular hash if key size is incorrect
                blake3::hash(&event_data)
            };
            
            sig == &expected.to_hex().to_string()
        } else {
            false
        }
    }
}

struct AlertSender {
    enabled: bool,
    tx: Option<mpsc::Sender<AuditEvent>>,
}

pub struct AuditLogger {
    config: AuditConfig,
    signing_key: Vec<u8>,
    alert_sender: AlertSender,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new() -> Result<Self> {
        let config = AuditConfig::default();
        
        // Create log directory if it doesn't exist
        if let Some(parent) = config.log_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create audit log directory")?;
        }
        
        // Generate a random signing key
        let mut signing_key = vec![0u8; 32];
        getrandom::getrandom(&mut signing_key)
            .map_err(|e| anyhow::anyhow!("Failed to generate signing key: {}", e))?;
        
        // Set up alert channel if enabled
        let alert_sender = if config.enable_alerts {
            let (tx, rx) = mpsc::channel();
            
            // Start the alert handler thread
            let alert_config = config.alert_config.clone();
            let alert_destination = config.alert_destination.clone();
            
            thread::spawn(move || {
                alert_handler_loop(rx, alert_destination, alert_config);
            });
            
            AlertSender {
                enabled: true,
                tx: Some(tx),
            }
        } else {
            AlertSender {
                enabled: false,
                tx: None,
            }
        };
        
        Ok(Self {
            config,
            signing_key,
            alert_sender,
        })
    }

    /// Configure the audit logger
    pub fn configure(&mut self, config: AuditConfig) -> Result<()> {
        // Reconfigure alert system if alert settings changed
        if self.config.enable_alerts != config.enable_alerts ||
           self.config.alert_destination != config.alert_destination ||
           self.config.alert_config != config.alert_config {
            
            // If alerts are now enabled but weren't before
            if config.enable_alerts && !self.config.enable_alerts {
                let (tx, rx) = mpsc::channel();
                
                // Start the alert handler thread
                let alert_config = config.alert_config.clone();
                let alert_destination = config.alert_destination.clone();
                
                thread::spawn(move || {
                    alert_handler_loop(rx, alert_destination, alert_config);
                });
                
                self.alert_sender = AlertSender {
                    enabled: true,
                    tx: Some(tx),
                };
            } else if !config.enable_alerts && self.config.enable_alerts {
                // Disable alerts
                self.alert_sender = AlertSender {
                    enabled: false,
                    tx: None,
                };
            }
        }
        
        self.config = config;
        Ok(())
    }

    /// Log an audit event
    pub fn log_event(&self, event: &mut AuditEvent) -> Result<()> {
        // Check if we should audit this event type and severity
        if !self.config.audit_events.contains(&event.event_type) || 
           event.severity < self.config.min_severity {
            return Ok(());
        }
        
        // Sign the event if enabled
        if self.config.sign_entries {
            event.sign(&self.signing_key);
        }
        
        // Handle critical events with real-time alerts if enabled
        if self.alert_sender.enabled && 
          (event.severity == EventSeverity::Critical || event.severity == EventSeverity::Error) {
            if let Some(tx) = &self.alert_sender.tx {
                if let Err(e) = tx.send(event.clone()) {
                    warn!("Failed to send alert: {}", e);
                }
            }
        }
        
        // Serialize the event
        let event_json = serde_json::to_string(&event)
            .context("Failed to serialize audit event")?;
        
        // Rotate log file if needed
        self.rotate_logs_if_needed()?;
        
        // Write the event to the log file
        use std::fs::OpenOptions;
        use std::io::Write;
        
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.config.log_path)
            .context("Failed to open audit log file")?;
            
        writeln!(file, "{}", event_json)
            .context("Failed to write audit event")?;
        
        Ok(())
    }

    /// Rotate log files if the current one exceeds the size limit
    fn rotate_logs_if_needed(&self) -> Result<()> {
        let metadata = std::fs::metadata(&self.config.log_path)
            .context("Failed to get log file metadata")?;
            
        if metadata.len() > self.config.max_log_size {
            self.rotate_logs()?;
        }
        
        Ok(())
    }

    /// Rotate log files
    fn rotate_logs(&self) -> Result<()> {
        for i in (1..self.config.log_retention).rev() {
            let old_path = self.config.log_path.with_extension(format!("log.{}", i));
            let new_path = self.config.log_path.with_extension(format!("log.{}", i + 1));
            
            if old_path.exists() {
                std::fs::rename(&old_path, &new_path)
                    .context(format!("Failed to rotate log file: {:?}", old_path))?;
            }
        }
        
        let backup_path = self.config.log_path.with_extension("log.1");
        if self.config.log_path.exists() {
            std::fs::rename(&self.config.log_path, &backup_path)
                .context("Failed to rotate current log file")?;
        }
        
        Ok(())
    }

    /// Verify the integrity of a log file
    pub fn verify_log_file(&self, path: &PathBuf) -> Result<bool> {
        use std::io::{BufRead, BufReader};
        use std::fs::File;
        
        let file = File::open(path)
            .context("Failed to open log file for verification")?;
            
        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            let line = line.context("Failed to read log line")?;
            
            let event: AuditEvent = serde_json::from_str(&line)
                .context("Failed to parse audit event")?;
                
            if self.config.sign_entries {
                if !event.verify(&self.signing_key) {
                    return Ok(false);
                }
            }
        }
        
        Ok(true)
    }
}

/// Alert handler loop that processes events from the alert channel
fn alert_handler_loop(
    rx: mpsc::Receiver<AuditEvent>,
    destination: Option<String>,
    config: Option<String>,
) {
    debug!("Starting alert handler loop");
    
    let alert_config = match config.and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok()) {
        Some(cfg) => cfg,
        None => serde_json::json!({})
    };
    
    for event in rx {
        match &destination {
            Some(dest) if dest == "syslog" => {
                send_syslog_alert(&event);
            },
            Some(dest) if dest == "email" => {
                send_email_alert(&event, &alert_config);
            },
            Some(dest) if dest == "webhook" => {
                send_webhook_alert(&event, &alert_config);
            },
            _ => {
                // Default to console logging if no destination is specified
                let severity_str = match event.severity {
                    EventSeverity::Critical => "CRITICAL",
                    EventSeverity::Error => "ERROR",
                    _ => "ALERT",
                };
                
                // Just log the alert to stderr
                eprintln!("[{}] ALERT: {} - {}", 
                    severity_str, 
                    event.event_type, 
                    event.description
                );
            }
        }
    }
}

/// Send an alert to syslog
fn send_syslog_alert(event: &AuditEvent) {
    // In a real implementation, this would use a syslog crate
    warn!("SECURITY ALERT: {} - {}", event.event_type, event.description);
}

/// Send an email alert
fn send_email_alert(event: &AuditEvent, config: &serde_json::Value) {
    // Extract email configuration
    let recipient = config.get("email_recipient")
        .and_then(|v| v.as_str())
        .unwrap_or("admin@example.com");
        
    // In a real implementation, this would use an email crate
    info!("Would send email alert to {}: {} - {}", 
        recipient, 
        event.event_type, 
        event.description
    );
}

/// Send a webhook alert
fn send_webhook_alert(event: &AuditEvent, config: &serde_json::Value) {
    // Extract webhook configuration
    let url = config.get("webhook_url")
        .and_then(|v| v.as_str())
        .unwrap_or("https://example.com/webhook");
        
    // In a real implementation, this would use reqwest or similar
    info!("Would send webhook alert to {}: {:?}", url, event);
}

/// Get the current user name
pub(crate) fn get_current_user() -> String {
    #[cfg(unix)]
    {
        use std::env;
        env::var("USER").unwrap_or_else(|_| "unknown".to_string())
    }
    
    #[cfg(windows)]
    {
        use std::env;
        env::var("USERNAME").unwrap_or_else(|_| "unknown".to_string())
    }
}

/// Log a tool execution event
pub fn log_tool_execution(
    tool_name: &str,
    command: &str,
    result: &str,
) -> Result<()> {
    let mut event = AuditEvent::new(
        AuditEventType::ToolExecution,
        format!("Executed tool: {}", tool_name),
        serde_json::json!({
            "command": command,
            "result": result,
        }),
    );
    
    AUDIT_LOGGER.lock()
        .unwrap()
        .log_event(&mut event)
}

/// Log a configuration change event
pub fn log_config_change(
    component: &str,
    old_value: &str,
    new_value: &str,
) -> Result<()> {
    let mut event = AuditEvent::new(
        AuditEventType::ConfigChange,
        format!("Configuration changed: {}", component),
        serde_json::json!({
            "old_value": old_value,
            "new_value": new_value,
        }),
    );
    
    AUDIT_LOGGER.lock()
        .unwrap()
        .log_event(&mut event)
}

/// Log a security violation event
pub fn log_security_violation(
    violation_type: &str,
    details: &str,
) -> Result<()> {
    let mut event = AuditEvent::new(
        AuditEventType::SecurityViolation,
        format!("Security violation: {}", violation_type),
        serde_json::json!({
            "details": details,
        }),
    );
    
    AUDIT_LOGGER.lock()
        .unwrap()
        .log_event(&mut event)
}

/// Log a file access event
pub fn log_file_access(
    path: &PathBuf,
    operation: &str,
) -> Result<()> {
    let mut event = AuditEvent::new(
        AuditEventType::FileAccess,
        format!("File access: {}", operation),
        serde_json::json!({
            "path": path.to_string_lossy().to_string(),
        }),
    );
    
    AUDIT_LOGGER.lock()
        .unwrap()
        .log_event(&mut event)
}

/// Log a validation event
pub fn log_validation_event(
    validator: &str,
    file_path: &PathBuf,
    status: bool,
    details: Option<&str>,
    severity: EventSeverity,
) -> Result<()> {
    let mut event = AuditEvent::with_severity(
        AuditEventType::ValidationEvent,
        format!("Validation: {}", validator),
        serde_json::json!({
            "file_path": file_path.to_string_lossy().to_string(),
            "status": status,
            "details": details.unwrap_or(""),
        }),
        severity,
    );
    
    AUDIT_LOGGER.lock()
        .unwrap()
        .log_event(&mut event)
}

/// Log a resource usage event
pub fn log_resource_event(
    resource_type: &str,
    process_id: u32,
    threshold: u64,
    actual: u64,
    action_taken: &str,
) -> Result<()> {
    let severity = if actual > threshold * 2 {
        EventSeverity::Critical
    } else if actual > threshold {
        EventSeverity::Warning
    } else {
        EventSeverity::Info
    };
    
    let mut event = AuditEvent::with_severity(
        AuditEventType::ResourceEvent,
        format!("Resource usage: {}", resource_type),
        serde_json::json!({
            "process_id": process_id,
            "threshold": threshold,
            "actual": actual,
            "action_taken": action_taken,
        }),
        severity,
    ).with_correlation_id(format!("process_{}", process_id));
    
    AUDIT_LOGGER.lock()
        .unwrap()
        .log_event(&mut event)
}

/// Log an authorization event
pub fn log_authorization_event(
    subject: &str,
    resource: &str,
    action: &str,
    allowed: bool,
    reason: Option<&str>,
) -> Result<()> {
    // Use Critical severity for denied access, Info for allowed
    let severity = if allowed {
        EventSeverity::Info
    } else {
        EventSeverity::Error
    };
    
    let mut event = AuditEvent::with_severity(
        AuditEventType::AuthorizationEvent,
        format!("Authorization: {} {} {}", subject, if allowed { "allowed" } else { "denied" }, action),
        serde_json::json!({
            "subject": subject,
            "resource": resource,
            "action": action,
            "allowed": allowed,
            "reason": reason.unwrap_or(""),
        }),
        severity,
    );
    
    AUDIT_LOGGER.lock()
        .unwrap()
        .log_event(&mut event)
}

/// Log a configuration validation event
pub fn log_configuration_event(
    config_path: &PathBuf,
    validation_status: bool,
    changes: Option<serde_json::Value>,
    issues: Option<Vec<String>>,
) -> Result<()> {
    let severity = if validation_status {
        EventSeverity::Info
    } else {
        EventSeverity::Warning
    };
    
    let context = match (changes, issues) {
        (Some(c), Some(i)) => serde_json::json!({
            "config_path": config_path.to_string_lossy().to_string(),
            "changes": c,
            "issues": i,
        }),
        (Some(c), None) => serde_json::json!({
            "config_path": config_path.to_string_lossy().to_string(),
            "changes": c,
        }),
        (None, Some(i)) => serde_json::json!({
            "config_path": config_path.to_string_lossy().to_string(),
            "issues": i,
        }),
        (None, None) => serde_json::json!({
            "config_path": config_path.to_string_lossy().to_string(),
        }),
    };
    
    let mut event = AuditEvent::with_severity(
        AuditEventType::ConfigurationEvent,
        format!("Configuration validation: {}", if validation_status { "passed" } else { "failed" }),
        context,
        severity,
    );
    
    AUDIT_LOGGER.lock()
        .unwrap()
        .log_event(&mut event)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_audit_event_signing() {
        let key = vec![0u8; 32];
        let mut event = AuditEvent::new(
            AuditEventType::UserAction,
            "Test event".to_string(),
            serde_json::json!({}),
        );
        
        event.sign(&key);
        assert!(event.signature.is_some());
        assert!(event.verify(&key));
    }

    #[test]
    fn test_audit_logger() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        
        let mut config = AuditConfig::default();
        config.log_path = log_path.clone();
        
        let mut logger = AuditLogger::new().unwrap();
        logger.configure(config);
        
        let mut event = AuditEvent::new(
            AuditEventType::SystemEvent,
            "Test system event".to_string(),
            serde_json::json!({
                "test_key": "test_value"
            }),
        );
        
        assert!(logger.log_event(&mut event).is_ok());
        assert!(log_path.exists());
    }

    #[test]
    fn test_log_rotation() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        
        let mut config = AuditConfig::default();
        config.log_path = log_path.clone();
        config.max_log_size = 100; // Small size to trigger rotation
        
        let mut logger = AuditLogger::new().unwrap();
        logger.configure(config).unwrap();
        
        // Write multiple events to trigger rotation
        for i in 0..10 {
            let mut event = AuditEvent::new(
                AuditEventType::SystemEvent,
                format!("Test event {}", i),
                serde_json::json!({}),
            );
            assert!(logger.log_event(&mut event).is_ok());
        }
        
        // Check if rotation occurred
        assert!(log_path.with_extension("log.1").exists());
    }
    
    #[test]
    fn test_event_severity() {
        let mut event = AuditEvent::with_severity(
            AuditEventType::SecurityViolation,
            "Critical security breach".to_string(),
            serde_json::json!({}),
            EventSeverity::Critical,
        );
        
        assert_eq!(event.severity, EventSeverity::Critical);
        
        // Create logger with minimum severity set to Error
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        
        let mut config = AuditConfig::default();
        config.log_path = log_path.clone();
        config.min_severity = EventSeverity::Error;
        
        let mut logger = AuditLogger::new().unwrap();
        logger.configure(config).unwrap();
        
        // Critical event should be logged (above threshold)
        assert!(logger.log_event(&mut event).is_ok());
        
        // Info event should not be logged (below threshold)
        let mut info_event = AuditEvent::with_severity(
            AuditEventType::SystemEvent,
            "Regular info event".to_string(),
            serde_json::json!({}),
            EventSeverity::Info,
        );
        
        // This should succeed but not actually write to the log
        assert!(logger.log_event(&mut info_event).is_ok());
    }
    
    #[test]
    fn test_correlation_id() {
        let event = AuditEvent::new(
            AuditEventType::UserAction,
            "User action test".to_string(),
            serde_json::json!({}),
        ).with_correlation_id("test-correlation-123".to_string());
        
        assert_eq!(event.correlation_id, Some("test-correlation-123".to_string()));
    }
    
    #[test]
    fn test_new_event_types() {
        // Test ValidationEvent
        let mut event = AuditEvent::new(
            AuditEventType::ValidationEvent,
            "Validation test".to_string(),
            serde_json::json!({
                "file_path": "/test/path.rs",
                "status": true,
            }),
        );
        
        assert_eq!(event.event_type, AuditEventType::ValidationEvent);
        
        // Test ResourceEvent
        let mut event = AuditEvent::new(
            AuditEventType::ResourceEvent,
            "Resource test".to_string(),
            serde_json::json!({
                "resource_type": "memory",
                "process_id": 1234,
            }),
        );
        
        assert_eq!(event.event_type, AuditEventType::ResourceEvent);
    }
}
