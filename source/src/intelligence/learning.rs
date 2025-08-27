use std::collections::HashMap;
use std::path::Path;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use chrono::{DateTime, Utc};
use super::ErrorRecord;

/// Adaptive learning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveConfig {
    pub learning_rate: f64,
    pub retention_policy: RetentionPolicy,
}

impl Default for AdaptiveConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.1,
            retention_policy: RetentionPolicy::KeepMostRecent(30), // Keep 30 most recent patterns
        }
    }
}

/// Retention policy for learning data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetentionPolicy {
    KeepAll,
    KeepMostRecent(usize),
    TimeBased(i64), // Retain data for a certain number of days
}

/// Learning engine for adaptive improvements
pub struct LearningEngine {
    config: AdaptiveConfig,
    learning_data: HashMap<String, LearningEntry>,
}

/// A learning entry for tracking patterns and improvements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEntry {
    pub pattern: String,
    pub occurrences: usize,
    pub last_seen: DateTime<Utc>,
    pub effectiveness: f64, // How effective past remedies were
}

impl LearningEngine {
    pub fn new() -> Self {
        Self {
            config: AdaptiveConfig::default(),
            learning_data: HashMap::new(),
        }
    }

    /// Learn from a reported error, updating internal patterns
    pub fn learn_from_error(&mut self, error: &ErrorRecord) {
        let pattern_key = error.error_type.clone();

        // Update or create learning entry
        let entry = self.learning_data.entry(pattern_key.clone()).or_insert(LearningEntry {
            pattern: pattern_key.clone(),
            occurrences: 0,
            last_seen: Utc::now(),
            effectiveness: 0.0,
        });

        entry.occurrences += 1;
        entry.last_seen = Utc::now();

        // Optionally update effectiveness based on resolution
        if error.fixed {
            entry.effectiveness += 1.0; // Simplified assumption
        }

        // Apply learning rate
        entry.effectiveness *= self.config.learning_rate;
    }

    /// Generate adaptive config based on learning data
    pub fn generate_adaptive_config(&self, _path: &Path) -> Result<AdaptiveConfig> {
        // In a real implementation, this would query a persistent store
        // For now, return the current config
        Ok(self.config.clone())
    }

    /// Cleanup learning data based on retention policy
    pub fn cleanup_learning_data(&mut self) {
        match self.config.retention_policy {
            RetentionPolicy::KeepAll => return,
            RetentionPolicy::KeepMostRecent(limit) => {
                let mut entries: Vec<_> = self.learning_data.values().cloned().collect();
                entries.sort_by_key(|e| e.last_seen);
                // Keep only the most recent entries
                while entries.len() > limit {
                    if let Some(oldest) = entries.first() {
                        self.learning_data.remove(&oldest.pattern);
                        entries.remove(0);
                    }
                }
            }
            RetentionPolicy::TimeBased(days) => {
                let cutoff = Utc::now() - chrono::Duration::days(days);
                self.learning_data.retain(|_, v| v.last_seen > cutoff);
            }
        }
    }

    /// Retrieve all learning entries for inspection
    pub fn get_learning_entries(&self) -> Vec<LearningEntry> {
        self.learning_data.values().cloned().collect()
    }
}

impl Default for LearningEngine {
    fn default() -> Self {
        Self::new()
    }
}
