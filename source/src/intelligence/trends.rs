use std::collections::HashMap;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use super::{HistoricalSnapshot, TrendDirection, TrendSummary};

/// Trend analysis for tracking code quality over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub time_period: String,
    pub quality_trend: TrendDirection,
    pub complexity_trend: TrendDirection,
    pub error_trend: TrendDirection,
    pub productivity_trend: TrendDirection,
    pub trend_confidence: f64,
    pub key_insights: Vec<String>,
}

/// Tracker for analyzing trends in code metrics
pub struct TrendTracker {
    // Configuration for trend analysis
    min_data_points: usize,
    trend_window_days: i64,
    significance_threshold: f64,
}

impl TrendTracker {
    pub fn new() -> Self {
        Self {
            min_data_points: 3,
            trend_window_days: 30,
            significance_threshold: 0.1, // 10% change considered significant
        }
    }
    
    /// Analyze trends from historical data
    pub fn analyze_trends(&self, historical_data: &[HistoricalSnapshot]) -> TrendSummary {
        if historical_data.len() < self.min_data_points {
            return TrendSummary {
                quality_trend: TrendDirection::Stable,
                complexity_trend: TrendDirection::Stable,
                error_trend: TrendDirection::Stable,
                productivity_trend: TrendDirection::Stable,
            };
        }
        
        // Sort by timestamp
        let mut sorted_data = historical_data.to_vec();
        sorted_data.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        let quality_trend = self.analyze_quality_trend(&sorted_data);
        let complexity_trend = self.analyze_complexity_trend(&sorted_data);
        let error_trend = self.analyze_error_trend(&sorted_data);
        let productivity_trend = self.analyze_productivity_trend(&sorted_data);
        
        TrendSummary {
            quality_trend,
            complexity_trend,
            error_trend,
            productivity_trend,
        }
    }
    
    /// Get current trend analysis with detailed insights
    pub fn get_current_trends(&self) -> TrendAnalysis {
        // This would typically use stored historical data
        // For now, return a placeholder
        TrendAnalysis {
            time_period: "Last 30 days".to_string(),
            quality_trend: TrendDirection::Stable,
            complexity_trend: TrendDirection::Stable,
            error_trend: TrendDirection::Improving,
            productivity_trend: TrendDirection::Improving,
            trend_confidence: 0.75,
            key_insights: vec![
                "Code quality has remained stable over the past month".to_string(),
                "Error rates are decreasing, indicating improved code reliability".to_string(),
                "Team productivity is trending upward".to_string(),
            ],
        }
    }
    
    /// Record a new data point for trend analysis
    pub fn record_snapshot(&mut self, _snapshot: HistoricalSnapshot) -> Result<()> {
        // In a real implementation, this would store the snapshot
        // to a database or persistent storage
        Ok(())
    }
    
    /// Analyze quality trend from historical data
    fn analyze_quality_trend(&self, data: &[HistoricalSnapshot]) -> TrendDirection {
        if data.len() < 2 {
            return TrendDirection::Stable;
        }
        
        let quality_scores: Vec<f64> = data.iter()
            .map(|snapshot| snapshot.quality_summary.overall_score)
            .collect();
        
        self.calculate_trend_direction(&quality_scores)
    }
    
    /// Analyze complexity trend from historical data
    fn analyze_complexity_trend(&self, data: &[HistoricalSnapshot]) -> TrendDirection {
        if data.len() < 2 {
            return TrendDirection::Stable;
        }
        
        // Use file count as a proxy for complexity growth
        let complexity_indicators: Vec<f64> = data.iter()
            .map(|snapshot| snapshot.file_count as f64)
            .collect();
        
        // Invert the direction since increasing file count might indicate growing complexity
        match self.calculate_trend_direction(&complexity_indicators) {
            TrendDirection::Improving => TrendDirection::Declining,
            TrendDirection::Declining => TrendDirection::Improving,
            other => other,
        }
    }
    
    /// Analyze error trend from historical data
    fn analyze_error_trend(&self, data: &[HistoricalSnapshot]) -> TrendDirection {
        if data.len() < 2 {
            return TrendDirection::Stable;
        }
        
        // This would analyze error frequency over time
        // For now, use reliability as an inverse indicator
        let reliability_scores: Vec<f64> = data.iter()
            .map(|snapshot| snapshot.quality_summary.reliability)
            .collect();
        
        self.calculate_trend_direction(&reliability_scores)
    }
    
    /// Analyze productivity trend from historical data
    fn analyze_productivity_trend(&self, data: &[HistoricalSnapshot]) -> TrendDirection {
        if data.len() < 2 {
            return TrendDirection::Stable;
        }
        
        // Use maintainability as a productivity indicator
        let productivity_scores: Vec<f64> = data.iter()
            .map(|snapshot| snapshot.quality_summary.maintainability)
            .collect();
        
        self.calculate_trend_direction(&productivity_scores)
    }
    
    /// Calculate trend direction from a series of values
    fn calculate_trend_direction(&self, values: &[f64]) -> TrendDirection {
        if values.len() < 2 {
            return TrendDirection::Stable;
        }
        
        // Calculate linear regression slope
        let n = values.len() as f64;
        let x_values: Vec<f64> = (0..values.len()).map(|i| i as f64).collect();
        
        let sum_x: f64 = x_values.iter().sum();
        let sum_y: f64 = values.iter().sum();
        let sum_xy: f64 = x_values.iter().zip(values.iter()).map(|(x, y)| x * y).sum();
        let sum_x_squared: f64 = x_values.iter().map(|x| x * x).sum();
        
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x_squared - sum_x * sum_x);
        
        // Calculate variance to determine if trend is significant
        let mean_y = sum_y / n;
        let variance = values.iter().map(|y| (y - mean_y).powi(2)).sum::<f64>() / n;
        let std_dev = variance.sqrt();
        
        // Determine significance
        let relative_slope = if std_dev > 0.0 { slope.abs() / std_dev } else { 0.0 };
        
        if relative_slope < self.significance_threshold {
            return TrendDirection::Stable;
        }
        
        // Check for volatility (high variance relative to trend)
        if std_dev > slope.abs() * 2.0 {
            return TrendDirection::Volatile;
        }
        
        if slope > self.significance_threshold {
            TrendDirection::Improving
        } else if slope < -self.significance_threshold {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        }
    }
    
    /// Generate trend insights based on analysis
    pub fn generate_insights(&self, trend_summary: &TrendSummary) -> Vec<String> {
        let mut insights = Vec::new();
        
        match trend_summary.quality_trend {
            TrendDirection::Improving => {
                insights.push("Code quality is improving over time - keep up the good work!".to_string());
            }
            TrendDirection::Declining => {
                insights.push("Code quality is declining - consider focusing on refactoring and code reviews".to_string());
            }
            TrendDirection::Volatile => {
                insights.push("Code quality is volatile - aim for more consistent development practices".to_string());
            }
            TrendDirection::Stable => {
                insights.push("Code quality is stable - maintaining current standards".to_string());
            }
        }
        
        match trend_summary.complexity_trend {
            TrendDirection::Improving => {
                insights.push("Code complexity is being well-managed and improving".to_string());
            }
            TrendDirection::Declining => {
                insights.push("Code complexity is increasing - consider breaking down complex functions".to_string());
            }
            TrendDirection::Volatile => {
                insights.push("Code complexity varies significantly - establish consistent complexity guidelines".to_string());
            }
            TrendDirection::Stable => {
                insights.push("Code complexity is stable and under control".to_string());
            }
        }
        
        match trend_summary.error_trend {
            TrendDirection::Improving => {
                insights.push("Error rates are decreasing - testing and quality practices are working".to_string());
            }
            TrendDirection::Declining => {
                insights.push("Error rates are increasing - review testing strategies and code review processes".to_string());
            }
            TrendDirection::Volatile => {
                insights.push("Error patterns are inconsistent - establish more systematic quality controls".to_string());
            }
            TrendDirection::Stable => {
                insights.push("Error rates are stable - maintaining current quality levels".to_string());
            }
        }
        
        match trend_summary.productivity_trend {
            TrendDirection::Improving => {
                insights.push("Team productivity is increasing - good development velocity".to_string());
            }
            TrendDirection::Declining => {
                insights.push("Productivity may be declining - investigate potential blockers or technical debt".to_string());
            }
            TrendDirection::Volatile => {
                insights.push("Productivity varies significantly - consider workflow optimization".to_string());
            }
            TrendDirection::Stable => {
                insights.push("Team productivity is steady and predictable".to_string());
            }
        }
        
        insights
    }
    
    /// Predict future trends based on current data
    pub fn predict_future_trend(
        &self, 
        historical_data: &[HistoricalSnapshot], 
        days_ahead: i64
    ) -> Result<HashMap<String, f64>> {
        let mut predictions = HashMap::new();
        
        if historical_data.len() < self.min_data_points {
            return Ok(predictions);
        }
        
        // Simple linear extrapolation for predictions
        let quality_scores: Vec<f64> = historical_data.iter()
            .map(|s| s.quality_summary.overall_score)
            .collect();
        
        if let Some(predicted_quality) = self.extrapolate_trend(&quality_scores, days_ahead) {
            predictions.insert("quality_score".to_string(), predicted_quality);
        }
        
        let reliability_scores: Vec<f64> = historical_data.iter()
            .map(|s| s.quality_summary.reliability)
            .collect();
        
        if let Some(predicted_reliability) = self.extrapolate_trend(&reliability_scores, days_ahead) {
            predictions.insert("reliability_score".to_string(), predicted_reliability);
        }
        
        Ok(predictions)
    }
    
    /// Extrapolate trend into the future
    fn extrapolate_trend(&self, values: &[f64], days_ahead: i64) -> Option<f64> {
        if values.len() < 2 {
            return None;
        }
        
        let n = values.len() as f64;
        let x_values: Vec<f64> = (0..values.len()).map(|i| i as f64).collect();
        
        let sum_x: f64 = x_values.iter().sum();
        let sum_y: f64 = values.iter().sum();
        let sum_xy: f64 = x_values.iter().zip(values.iter()).map(|(x, y)| x * y).sum();
        let sum_x_squared: f64 = x_values.iter().map(|x| x * x).sum();
        
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x_squared - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;
        
        // Project forward
        let future_x = values.len() as f64 + (days_ahead as f64 / 7.0); // Convert days to data points (assuming weekly snapshots)
        let prediction = slope * future_x + intercept;
        
        // Clamp predictions to reasonable ranges
        Some(prediction.max(0.0).min(100.0))
    }
}

impl Default for TrendTracker {
    fn default() -> Self {
        Self::new()
    }
}
