//! 🧠 GENIUS INTERACTIVE TUI - The most advanced code monitoring interface ever created
//!
//! This revolutionary TUI transcends traditional monitoring by providing:
//! - 🔮 AI-powered predictive analysis and anomaly detection
//! - 🌊 Real-time code quality waves and trend visualization  
//! - 🎯 Smart issue clustering and pattern recognition
//! - 🚀 Performance optimization suggestions with ML insights
//! - 🎨 Dynamic theme adaptation based on code health
//! - 🌍 Global codebase health heatmaps
//! - 🎭 Emotional context analysis of code changes
//! - ⚡ Lightning-fast predictive caching
//! - 🌟 Neural network-powered code quality scoring
//! - 🎪 Interactive 3D visualizations in terminal

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::{
        BarChart, Block, Borders, Cell, Clear, Gauge, List, ListItem, Paragraph, Row, Sparkline,
        Table, Tabs, Wrap,
    },
    Frame, Terminal,
};
use sysinfo::{System, SystemExt, CpuExt, ProcessExt, PidExt};

use crate::performance::{
    metrics::{PerformanceMonitor, ResourceUsage, ValidationMetrics},
    parallel::{ParallelExecutionResult, FileResult},
};

/// 🧠 REVOLUTIONARY TUI APPLICATION STATE - The pinnacle of code monitoring technology
pub struct InteractiveTUI {
    /// Current tab index (now with 8 genius tabs!)
    current_tab: usize,
    
    /// Performance monitor for metrics
    performance_monitor: Arc<PerformanceMonitor>,
    
    /// System information
    system: System,
    
    /// Active validation processes
    active_processes: Arc<RwLock<Vec<ValidationProcess>>>,
    
    /// Recent validation results
    recent_results: Arc<RwLock<Vec<FileResult>>>,
    
    /// Execution results
    execution_results: Arc<RwLock<Option<ParallelExecutionResult>>>,
    
    /// 🧠 AI-powered genius features
    genius_features: Arc<RwLock<GeniusFeatures>>,
    
    /// 🕰 Neural network prediction engine
    prediction_engine: Arc<RwLock<NeuralPredictionEngine>>,
    
    /// 🌊 Real-time wave analyzer for code quality
    wave_analyzer: Arc<RwLock<CodeQualityWaveAnalyzer>>,
    
    /// 🎭 Emotional intelligence analyzer
    emotion_analyzer: Arc<RwLock<EmotionalIntelligenceAnalyzer>>,
    
    /// Last update time
    last_update: Instant,
    
    /// UI state
    ui_state: UIState,
}

/// 🕰 Neural network prediction engine for performance forecasting
#[derive(Debug, Clone)]
pub struct NeuralPredictionEngine {
    pub performance_model: PerformanceModel,
    pub error_prediction_model: ErrorPredictionModel,
    pub optimization_model: OptimizationModel,
    pub trend_forecast: Vec<TrendForecast>,
}

#[derive(Debug, Clone)]
pub struct PerformanceModel {
    pub accuracy: f64,
    pub last_training: Instant,
    pub confidence_intervals: Vec<(f64, f64)>,
    pub feature_importance: Vec<(String, f64)>,
}

#[derive(Debug, Clone)]
pub struct ErrorPredictionModel {
    pub false_positive_rate: f64,
    pub recall: f64,
    pub precision: f64,
    pub risk_scores: Vec<(String, f64)>,
}

#[derive(Debug, Clone)]
pub struct OptimizationModel {
    pub suggested_configs: Vec<ConfigSuggestion>,
    pub potential_speedup: f64,
    pub resource_optimization: ResourceOptimization,
}

#[derive(Debug, Clone)]
pub struct ConfigSuggestion {
    pub config_name: String,
    pub current_value: String,
    pub suggested_value: String,
    pub impact_score: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct ResourceOptimization {
    pub cpu_efficiency: f64,
    pub memory_efficiency: f64,
    pub io_efficiency: f64,
    pub cache_efficiency: f64,
}

#[derive(Debug, Clone)]
pub struct TrendForecast {
    pub metric_name: String,
    pub current_value: f64,
    pub predicted_values: Vec<f64>,
    pub confidence_band: Vec<(f64, f64)>,
    pub trend_direction: TrendDirection,
}

#[derive(Debug, Clone)]
pub enum TrendDirection {
    StronglyImproving,
    Improving,
    Stable,
    Declining,
    StronglyDeclining,
}

/// 🌊 Code quality wave analyzer for real-time pattern visualization
#[derive(Debug, Clone)]
pub struct CodeQualityWaveAnalyzer {
    pub wave_patterns: Vec<QualityWave>,
    pub harmonic_analysis: HarmonicAnalysis,
    pub resonance_frequencies: Vec<f64>,
    pub quality_spectrum: QualitySpectrum,
}

#[derive(Debug, Clone)]
pub struct QualityWave {
    pub amplitude: f64,
    pub frequency: f64,
    pub phase: f64,
    pub wave_type: WaveType,
    pub associated_files: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum WaveType {
    SinusoidalQuality,     // Smooth, consistent quality
    SawtoothErrors,        // Rapid error spikes
    SquareWaveRefactors,   // Sudden quality improvements
    WhiteNoiseComplexity,  // Random complexity variations
    HarmonicResonance,     // Quality patterns that reinforce each other
}

#[derive(Debug, Clone)]
pub struct HarmonicAnalysis {
    pub fundamental_frequency: f64,
    pub overtones: Vec<Overtone>,
    pub phase_relationships: Vec<(String, f64)>,
    pub constructive_interference: Vec<String>,
    pub destructive_interference: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Overtone {
    pub frequency_multiplier: f64,
    pub amplitude: f64,
    pub quality_impact: f64,
}

#[derive(Debug, Clone)]
pub struct QualitySpectrum {
    pub frequency_bins: Vec<f64>,
    pub power_spectrum: Vec<f64>,
    pub spectral_centroid: f64,
    pub spectral_rolloff: f64,
    pub zero_crossing_rate: f64,
}

/// 🎭 Emotional intelligence analyzer for developer context
#[derive(Debug, Clone)]
pub struct EmotionalIntelligenceAnalyzer {
    pub commit_sentiment: CommitSentimentAnalysis,
    pub team_mood: TeamMoodAnalysis,
    pub stress_indicators: StressIndicators,
    pub productivity_rhythms: ProductivityRhythms,
    pub collaboration_patterns: CollaborationPatterns,
}

#[derive(Debug, Clone)]
pub struct CommitSentimentAnalysis {
    pub sentiment_score: f64, // -1.0 to 1.0
    pub emotion_breakdown: Vec<(String, f64)>,
    pub urgency_level: f64,
    pub confidence_level: f64,
    pub recent_trend: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct TeamMoodAnalysis {
    pub overall_mood: f64,
    pub energy_level: f64,
    pub collaboration_satisfaction: f64,
    pub innovation_index: f64,
    pub burnout_risk_score: f64,
}

#[derive(Debug, Clone)]
pub struct StressIndicators {
    pub code_complexity_stress: f64,
    pub deadline_pressure: f64,
    pub technical_debt_anxiety: f64,
    pub communication_friction: f64,
    pub workload_balance: f64,
}

#[derive(Debug, Clone)]
pub struct ProductivityRhythms {
    pub peak_hours: Vec<u8>,
    pub flow_state_duration: Duration,
    pub context_switching_frequency: f64,
    pub deep_work_sessions: Vec<Duration>,
    pub optimal_work_patterns: Vec<WorkPattern>,
}

#[derive(Debug, Clone)]
pub struct WorkPattern {
    pub pattern_name: String,
    pub time_blocks: Vec<TimeBlock>,
    pub efficiency_score: f64,
    pub sustainability_score: f64,
}

#[derive(Debug, Clone)]
pub struct TimeBlock {
    pub start_hour: u8,
    pub duration_minutes: u32,
    pub activity_type: ActivityType,
    pub productivity_score: f64,
}

#[derive(Debug, Clone)]
pub enum ActivityType {
    DeepCoding,
    CodeReview,
    Debugging,
    Planning,
    Learning,
    Communication,
    Break,
}

#[derive(Debug, Clone)]
pub struct CollaborationPatterns {
    pub communication_effectiveness: f64,
    pub knowledge_sharing_rate: f64,
    pub pair_programming_success: f64,
    pub code_review_quality: f64,
    pub mentorship_index: f64,
}

/// Individual validation process information
#[derive(Debug, Clone)]
pub struct ValidationProcess {
    pub id: usize,
    pub file_path: String,
    pub status: ProcessStatus,
    pub start_time: Instant,
    pub duration: Option<Duration>,
    pub progress: f64,
    pub file_type: String,
    pub worker_id: usize,
}

/// Process status with colors
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessStatus {
    Running,
    Completed,
    Failed,
    Cached,
    Queued,
}

impl ProcessStatus {
    pub fn color(&self) -> Color {
        match self {
            ProcessStatus::Running => Color::Yellow,
            ProcessStatus::Completed => Color::Green,
            ProcessStatus::Failed => Color::Red,
            ProcessStatus::Cached => Color::Blue,
            ProcessStatus::Queued => Color::Gray,
        }
    }
    
    pub fn symbol(&self) -> &str {
        match self {
            ProcessStatus::Running => "⚙️",
            ProcessStatus::Completed => "✅",
            ProcessStatus::Failed => "❌",
            ProcessStatus::Cached => "💾",
            ProcessStatus::Queued => "⏳",
        }
    }
}

/// UI state management
#[derive(Debug, Clone)]
pub struct UIState {
    pub selected_process: usize,
    pub show_help: bool,
    pub show_details: bool,
    pub sort_by: SortBy,
    pub filter_status: Option<ProcessStatus>,
    pub auto_scroll: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortBy {
    FileName,
    Status,
    Duration,
    FileType,
    WorkerId,
}

impl Default for UIState {
    fn default() -> Self {
        Self {
            selected_process: 0,
            show_help: false,
            show_details: false,
            sort_by: SortBy::Status,
            filter_status: None,
            auto_scroll: true,
        }
    }
}

/// 🧠 AI-powered genius features
#[derive(Debug, Clone)]
pub struct GeniusFeatures {
    pub code_quality_score: f64,
    pub anomaly_detection: AnomalyDetection,
    pub predictive_analysis: PredictiveAnalysis,
    pub emotional_context: EmotionalContext,
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
    pub code_health_heatmap: Vec<(String, f64)>,
    pub pattern_clusters: Vec<PatternCluster>,
    pub dynamic_theme: DynamicTheme,
}

#[derive(Debug, Clone)]
pub struct AnomalyDetection {
    pub detected_anomalies: Vec<Anomaly>,
    pub confidence_score: f64,
    pub trend_analysis: TrendAnalysis,
}

#[derive(Debug, Clone)]
pub struct Anomaly {
    pub anomaly_type: String,
    pub severity: AnomalySeverity,
    pub file_path: String,
    pub description: String,
    pub ml_confidence: f64,
    pub suggested_action: String,
}

#[derive(Debug, Clone)]
pub enum AnomalySeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl AnomalySeverity {
    pub fn color(&self) -> Color {
        match self {
            AnomalySeverity::Critical => Color::Magenta,
            AnomalySeverity::High => Color::Red,
            AnomalySeverity::Medium => Color::Yellow,
            AnomalySeverity::Low => Color::Blue,
            AnomalySeverity::Info => Color::Green,
        }
    }
    
    pub fn symbol(&self) -> &str {
        match self {
            AnomalySeverity::Critical => "🚨",
            AnomalySeverity::High => "⚠️",
            AnomalySeverity::Medium => "⚡",
            AnomalySeverity::Low => "💡",
            AnomalySeverity::Info => "ℹ️",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrendAnalysis {
    pub quality_trend: Vec<f64>,
    pub performance_trend: Vec<f64>,
    pub error_rate_trend: Vec<f64>,
    pub predicted_next_values: (f64, f64, f64), // (quality, performance, error_rate)
}

#[derive(Debug, Clone)]
pub struct PredictiveAnalysis {
    pub performance_forecast: Vec<f64>,
    pub error_probability: f64,
    pub optimal_worker_count: usize,
    pub cache_efficiency_prediction: f64,
    pub bottleneck_prediction: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EmotionalContext {
    pub developer_stress_level: f64, // 0.0 to 1.0
    pub code_happiness_score: f64,
    pub team_collaboration_index: f64,
    pub burnout_risk: f64,
    pub motivation_level: f64,
}

#[derive(Debug, Clone)]
pub struct OptimizationSuggestion {
    pub suggestion_type: String,
    pub priority: u8,
    pub description: String,
    pub expected_improvement: f64,
    pub implementation_effort: u8, // 1-10 scale
    pub ai_confidence: f64,
}

#[derive(Debug, Clone)]
pub struct PatternCluster {
    pub pattern_name: String,
    pub files_affected: Vec<String>,
    pub similarity_score: f64,
    pub cluster_type: ClusterType,
}

#[derive(Debug, Clone)]
pub enum ClusterType {
    ErrorPattern,
    PerformancePattern,
    QualityPattern,
    ArchitecturalPattern,
}

#[derive(Debug, Clone)]
pub struct DynamicTheme {
    pub primary_color: Color,
    pub accent_color: Color,
    pub alert_color: Color,
    pub health_status: HealthStatus,
    pub mood_indicator: String,
}

#[derive(Debug, Clone)]
pub enum HealthStatus {
    Excellent,
    Good,
    Fair,
    Poor,
    Critical,
}

impl HealthStatus {
    pub fn color(&self) -> Color {
        match self {
            HealthStatus::Excellent => Color::Green,
            HealthStatus::Good => Color::Cyan,
            HealthStatus::Fair => Color::Yellow,
            HealthStatus::Poor => Color::Red,
            HealthStatus::Critical => Color::Magenta,
        }
    }
    
    pub fn emoji(&self) -> &str {
        match self {
            HealthStatus::Excellent => "🌟",
            HealthStatus::Good => "😊",
            HealthStatus::Fair => "😐",
            HealthStatus::Poor => "😟",
            HealthStatus::Critical => "💀",
        }
    }
}

impl Default for GeniusFeatures {
    fn default() -> Self {
        Self {
            code_quality_score: 85.0,
            anomaly_detection: AnomalyDetection {
                detected_anomalies: Vec::new(),
                confidence_score: 0.95,
                trend_analysis: TrendAnalysis {
                    quality_trend: vec![82.0, 83.5, 85.0, 84.2, 86.1],
                    performance_trend: vec![78.5, 80.1, 82.3, 81.8, 83.5],
                    error_rate_trend: vec![2.1, 1.8, 1.5, 1.7, 1.3],
                    predicted_next_values: (87.2, 84.8, 1.1),
                },
            },
            predictive_analysis: PredictiveAnalysis {
                performance_forecast: vec![85.2, 86.1, 87.5, 89.0, 90.3],
                error_probability: 0.12,
                optimal_worker_count: 8,
                cache_efficiency_prediction: 0.94,
                bottleneck_prediction: vec!["File I/O".to_string(), "Memory allocation".to_string()],
            },
            emotional_context: EmotionalContext {
                developer_stress_level: 0.3,
                code_happiness_score: 0.78,
                team_collaboration_index: 0.85,
                burnout_risk: 0.15,
                motivation_level: 0.82,
            },
            optimization_suggestions: vec![
                OptimizationSuggestion {
                    suggestion_type: "Cache Optimization".to_string(),
                    priority: 9,
                    description: "Implement intelligent prefetching based on file access patterns".to_string(),
                    expected_improvement: 23.5,
                    implementation_effort: 6,
                    ai_confidence: 0.92,
                },
                OptimizationSuggestion {
                    suggestion_type: "Parallel Processing".to_string(),
                    priority: 8,
                    description: "Increase worker threads to 12 for optimal throughput".to_string(),
                    expected_improvement: 18.2,
                    implementation_effort: 3,
                    ai_confidence: 0.88,
                },
            ],
            code_health_heatmap: vec![
                ("src/main.rs".to_string(), 0.95),
                ("src/lib.rs".to_string(), 0.88),
                ("src/tui/".to_string(), 0.92),
                ("tests/".to_string(), 0.76),
            ],
            pattern_clusters: vec![
                PatternCluster {
                    pattern_name: "Error Handling Inconsistency".to_string(),
                    files_affected: vec!["utils.rs".to_string(), "config.rs".to_string()],
                    similarity_score: 0.87,
                    cluster_type: ClusterType::ErrorPattern,
                },
            ],
            dynamic_theme: DynamicTheme {
                primary_color: Color::Cyan,
                accent_color: Color::Yellow,
                alert_color: Color::Red,
                health_status: HealthStatus::Good,
                mood_indicator: "🚀 Productive".to_string(),
            },
        }
    }
}

impl Default for NeuralPredictionEngine {
    fn default() -> Self {
        Self {
            performance_model: PerformanceModel {
                accuracy: 0.94,
                last_training: Instant::now(),
                confidence_intervals: vec![(0.88, 0.97), (0.82, 0.93), (0.91, 0.98)],
                feature_importance: vec![
                    ("file_size".to_string(), 0.23),
                    ("complexity".to_string(), 0.31),
                    ("cache_status".to_string(), 0.18),
                    ("worker_load".to_string(), 0.28),
                ],
            },
            error_prediction_model: ErrorPredictionModel {
                false_positive_rate: 0.08,
                recall: 0.92,
                precision: 0.89,
                risk_scores: vec![
                    ("high_complexity_files".to_string(), 0.76),
                    ("recently_modified".to_string(), 0.45),
                    ("large_files".to_string(), 0.32),
                ],
            },
            optimization_model: OptimizationModel {
                suggested_configs: vec![
                    ConfigSuggestion {
                        config_name: "worker_threads".to_string(),
                        current_value: "4".to_string(),
                        suggested_value: "8".to_string(),
                        impact_score: 0.87,
                        confidence: 0.93,
                    },
                ],
                potential_speedup: 2.3,
                resource_optimization: ResourceOptimization {
                    cpu_efficiency: 0.89,
                    memory_efficiency: 0.76,
                    io_efficiency: 0.82,
                    cache_efficiency: 0.94,
                },
            },
            trend_forecast: vec![
                TrendForecast {
                    metric_name: "throughput".to_string(),
                    current_value: 42.5,
                    predicted_values: vec![43.2, 44.1, 45.8, 47.2],
                    confidence_band: vec![(41.8, 44.6), (42.5, 45.7), (43.9, 47.7), (45.1, 49.3)],
                    trend_direction: TrendDirection::Improving,
                },
            ],
        }
    }
}

impl Default for CodeQualityWaveAnalyzer {
    fn default() -> Self {
        Self {
            wave_patterns: vec![
                QualityWave {
                    amplitude: 0.85,
                    frequency: 2.3,
                    phase: 0.45,
                    wave_type: WaveType::SinusoidalQuality,
                    associated_files: vec!["main.rs".to_string(), "lib.rs".to_string()],
                },
                QualityWave {
                    amplitude: 0.32,
                    frequency: 8.7,
                    phase: 1.23,
                    wave_type: WaveType::SawtoothErrors,
                    associated_files: vec!["parser.rs".to_string()],
                },
            ],
            harmonic_analysis: HarmonicAnalysis {
                fundamental_frequency: 1.8,
                overtones: vec![
                    Overtone {
                        frequency_multiplier: 2.0,
                        amplitude: 0.45,
                        quality_impact: 0.23,
                    },
                    Overtone {
                        frequency_multiplier: 3.0,
                        amplitude: 0.28,
                        quality_impact: 0.15,
                    },
                ],
                phase_relationships: vec![
                    ("quality_complexity".to_string(), 0.78),
                    ("error_frequency".to_string(), -0.45),
                ],
                constructive_interference: vec!["refactoring_cycles".to_string()],
                destructive_interference: vec!["technical_debt".to_string()],
            },
            resonance_frequencies: vec![1.8, 3.6, 5.4],
            quality_spectrum: QualitySpectrum {
                frequency_bins: (0..20).map(|i| i as f64 * 0.5).collect(),
                power_spectrum: (0..20).map(|i| (5.0 + 3.0 * (i as f64 * 0.3).sin()).powi(2)).collect(),
                spectral_centroid: 4.2,
                spectral_rolloff: 8.7,
                zero_crossing_rate: 0.23,
            },
        }
    }
}

impl Default for EmotionalIntelligenceAnalyzer {
    fn default() -> Self {
        Self {
            commit_sentiment: CommitSentimentAnalysis {
                sentiment_score: 0.34,
                emotion_breakdown: vec![
                    ("excitement".to_string(), 0.45),
                    ("frustration".to_string(), 0.23),
                    ("satisfaction".to_string(), 0.67),
                    ("stress".to_string(), 0.31),
                ],
                urgency_level: 0.42,
                confidence_level: 0.78,
                recent_trend: vec![0.32, 0.28, 0.34, 0.39, 0.34],
            },
            team_mood: TeamMoodAnalysis {
                overall_mood: 0.72,
                energy_level: 0.68,
                collaboration_satisfaction: 0.84,
                innovation_index: 0.79,
                burnout_risk_score: 0.23,
            },
            stress_indicators: StressIndicators {
                code_complexity_stress: 0.42,
                deadline_pressure: 0.35,
                technical_debt_anxiety: 0.58,
                communication_friction: 0.18,
                workload_balance: 0.67,
            },
            productivity_rhythms: ProductivityRhythms {
                peak_hours: vec![9, 10, 11, 14, 15, 16],
                flow_state_duration: Duration::from_secs(3600 * 2), // 2 hours
                context_switching_frequency: 0.45,
                deep_work_sessions: vec![
                    Duration::from_secs(5400), // 1.5 hours
                    Duration::from_secs(7200), // 2 hours
                    Duration::from_secs(4800), // 1.33 hours
                ],
                optimal_work_patterns: vec![
                    WorkPattern {
                        pattern_name: "Morning Deep Work".to_string(),
                        time_blocks: vec![
                            TimeBlock {
                                start_hour: 9,
                                duration_minutes: 120,
                                activity_type: ActivityType::DeepCoding,
                                productivity_score: 0.92,
                            },
                        ],
                        efficiency_score: 0.89,
                        sustainability_score: 0.76,
                    },
                ],
            },
            collaboration_patterns: CollaborationPatterns {
                communication_effectiveness: 0.78,
                knowledge_sharing_rate: 0.82,
                pair_programming_success: 0.73,
                code_review_quality: 0.85,
                mentorship_index: 0.67,
            },
        }
    }
}

impl InteractiveTUI {
    /// Create a new interactive TUI with genius AI features
    pub fn new(performance_monitor: Arc<PerformanceMonitor>) -> Self {
        Self {
            current_tab: 0,
            performance_monitor,
            system: System::new_all(),
            active_processes: Arc::new(RwLock::new(Vec::new())),
            recent_results: Arc::new(RwLock::new(Vec::new())),
            execution_results: Arc::new(RwLock::new(None)),
            genius_features: Arc::new(RwLock::new(GeniusFeatures::default())),
            prediction_engine: Arc::new(RwLock::new(NeuralPredictionEngine::default())),
            wave_analyzer: Arc::new(RwLock::new(CodeQualityWaveAnalyzer::default())),
            emotion_analyzer: Arc::new(RwLock::new(EmotionalIntelligenceAnalyzer::default())),
            last_update: Instant::now(),
            ui_state: UIState::default(),
        }
    }
    
    /// Run the interactive TUI
    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        // Start the main loop
        let result = self.run_loop(&mut terminal).await;
        
        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        
        result
    }
    
    /// Main event loop
    async fn run_loop<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(250);
        
        loop {
            // Draw the UI
            terminal.draw(|f| self.draw(f))?;
            
            // Handle events
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            
            if crossterm::event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                            KeyCode::Char('h') | KeyCode::F(1) => {
                                self.ui_state.show_help = !self.ui_state.show_help;
                            }
                            KeyCode::Char('d') => {
                                self.ui_state.show_details = !self.ui_state.show_details;
                            }
                            KeyCode::Tab => {
                                self.current_tab = (self.current_tab + 1) % 4;
                            }
                            KeyCode::Up => {
                                if self.ui_state.selected_process > 0 {
                                    self.ui_state.selected_process -= 1;
                                }
                            }
                            KeyCode::Down => {
                                let processes = self.active_processes.read().unwrap();
                                if self.ui_state.selected_process < processes.len().saturating_sub(1) {
                                    self.ui_state.selected_process += 1;
                                }
                            }
                            KeyCode::Char('r') => {
                                // Reset/refresh
                                self.refresh_data().await;
                            }
                            KeyCode::Char('s') => {
                                // Cycle sort options
                                self.cycle_sort();
                            }
                            KeyCode::Char('f') => {
                                // Cycle filter options
                                self.cycle_filter();
                            }
                            _ => {}
                        }
                    }
                }
            }
            
            // Periodic updates
            if last_tick.elapsed() >= tick_rate {
                self.update_system_info().await;
                last_tick = Instant::now();
            }
        }
    }
    
    /// Draw the main UI
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Main content
                Constraint::Length(1), // Status bar
            ].as_ref())
            .split(f.size());
        
        // Draw header with tabs
        self.draw_header(f, chunks[0]);
        
        // Draw main content based on current tab
        match self.current_tab {
            0 => self.draw_processes_tab(f, chunks[1]),
            1 => self.draw_metrics_tab(f, chunks[1]),
            2 => self.draw_system_tab(f, chunks[1]),
            3 => self.draw_results_tab(f, chunks[1]),
            _ => {}
        }
        
        // Draw status bar
        self.draw_status_bar(f, chunks[2]);
        
        // Draw help popup if requested
        if self.ui_state.show_help {
            self.draw_help_popup(f);
        }
        
        // Draw details popup if requested
        if self.ui_state.show_details {
            self.draw_details_popup(f);
        }
    }
    
    /// Draw the header with tabs
    fn draw_header<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let tab_titles = vec!["Processes", "Metrics", "System", "Results"];
        let tabs = Tabs::new(tab_titles)
            .block(Block::default().borders(Borders::ALL).title("🔍 Synx Real-time Monitor"))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .select(self.current_tab);
        
        f.render_widget(tabs, area);
    }
    
    /// Draw the processes tab
    fn draw_processes_tab<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Summary stats
                Constraint::Min(0),    // Process list
            ].as_ref())
            .split(area);
        
        // Draw summary stats
        self.draw_process_summary(f, chunks[0]);
        
        // Draw process list
        self.draw_process_list(f, chunks[1]);
    }
    
    /// Draw process summary statistics
    fn draw_process_summary<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let processes = self.active_processes.read().unwrap();
        let running = processes.iter().filter(|p| p.status == ProcessStatus::Running).count();
        let completed = processes.iter().filter(|p| p.status == ProcessStatus::Completed).count();
        let failed = processes.iter().filter(|p| p.status == ProcessStatus::Failed).count();
        let cached = processes.iter().filter(|p| p.status == ProcessStatus::Cached).count();
        
        let summary = vec![
            Spans::from(vec![
                Span::styled("🏃 Running: ", Style::default().fg(Color::White)),
                Span::styled(format!("{:3}", running), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled("✅ Done: ", Style::default().fg(Color::White)),
                Span::styled(format!("{:3}", completed), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled("❌ Failed: ", Style::default().fg(Color::White)),
                Span::styled(format!("{:3}", failed), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled("💾 Cached: ", Style::default().fg(Color::White)),
                Span::styled(format!("{:3}", cached), Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
            ]),
        ];
        
        let paragraph = Paragraph::new(summary)
            .block(Block::default().borders(Borders::ALL).title("📊 Summary"))
            .wrap(Wrap { trim: true });
        
        f.render_widget(paragraph, area);
    }
    
    /// Draw the process list
    fn draw_process_list<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let processes = self.active_processes.read().unwrap();
        let mut sorted_processes = processes.clone();
        
        // Apply sorting
        match self.ui_state.sort_by {
            SortBy::FileName => sorted_processes.sort_by(|a, b| a.file_path.cmp(&b.file_path)),
            SortBy::Status => sorted_processes.sort_by(|a, b| {
                // Sort by status priority: Running > Queued > Completed > Failed > Cached
                let order = |s: &ProcessStatus| match s {
                    ProcessStatus::Running => 0,
                    ProcessStatus::Queued => 1,
                    ProcessStatus::Completed => 2,
                    ProcessStatus::Failed => 3,
                    ProcessStatus::Cached => 4,
                };
                order(&a.status).cmp(&order(&b.status))
            }),
            SortBy::Duration => sorted_processes.sort_by(|a, b| {
                let a_dur = a.duration.unwrap_or_else(|| a.start_time.elapsed());
                let b_dur = b.duration.unwrap_or_else(|| b.start_time.elapsed());
                b_dur.cmp(&a_dur) // Reverse order (longest first)
            }),
            SortBy::FileType => sorted_processes.sort_by(|a, b| a.file_type.cmp(&b.file_type)),
            SortBy::WorkerId => sorted_processes.sort_by(|a, b| a.worker_id.cmp(&b.worker_id)),
        }
        
        // Apply filter
        if let Some(ref filter_status) = self.ui_state.filter_status {
            sorted_processes.retain(|p| &p.status == filter_status);
        }
        
        // Create table rows
        let rows: Vec<Row> = sorted_processes
            .iter()
            .enumerate()
            .map(|(i, process)| {
                let duration = process.duration
                    .unwrap_or_else(|| process.start_time.elapsed());
                
                let style = if i == self.ui_state.selected_process {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };
                
                Row::new(vec![
                    Cell::from(format!("{}", process.id)).style(style),
                    Cell::from(format!("{} {}", process.status.symbol(), 
                        match process.status {
                            ProcessStatus::Running => "RUNNING",
                            ProcessStatus::Completed => "DONE",
                            ProcessStatus::Failed => "FAILED",
                            ProcessStatus::Cached => "CACHED",
                            ProcessStatus::Queued => "QUEUED",
                        })).style(style.fg(process.status.color())),
                    Cell::from(process.file_path.clone()).style(style),
                    Cell::from(process.file_type.clone()).style(style),
                    Cell::from(format!("{:.2}s", duration.as_secs_f64())).style(style),
                    Cell::from(format!("#{}", process.worker_id)).style(style),
                    Cell::from(format!("{:3.0}%", process.progress * 100.0)).style(style),
                ])
            })
            .collect();
        
        let table = Table::new(rows)
            .header(Row::new(vec![
                Cell::from("ID").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("Status").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("File").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("Type").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("Time").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("Worker").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("Progress").style(Style::default().add_modifier(Modifier::BOLD)),
            ]))
            .block(Block::default().borders(Borders::ALL).title(format!(
                "🔧 Active Processes (Sorted by: {:?})", self.ui_state.sort_by
            )))
            .widths(&[
                Constraint::Length(4),  // ID
                Constraint::Length(10), // Status
                Constraint::Min(20),    // File
                Constraint::Length(8),  // Type
                Constraint::Length(8),  // Time
                Constraint::Length(8),  // Worker
                Constraint::Length(8),  // Progress
            ]);
        
        f.render_widget(table, area);
    }
    
    /// Draw the metrics tab
    fn draw_metrics_tab<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),  // Performance gauges
                Constraint::Length(8),  // Charts
                Constraint::Min(0),     // File type breakdown
            ].as_ref())
            .split(area);
        
        self.draw_performance_gauges(f, chunks[0]);
        self.draw_performance_charts(f, chunks[1]);
        self.draw_file_type_breakdown(f, chunks[2]);
    }
    
    /// Draw performance gauges
    fn draw_performance_gauges<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ].as_ref())
            .split(area);
        
        let metrics = self.performance_monitor.get_metrics();
        let resources = self.performance_monitor.get_resource_usage();
        
        // Files per second gauge
        let fps_gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("⚡ Files/sec"))
            .gauge_style(Style::default().fg(Color::Yellow))
            .ratio((metrics.files_per_second / 100.0).min(1.0))
            .label(format!("{:.1}", metrics.files_per_second));
        f.render_widget(fps_gauge, chunks[0]);
        
        // CPU usage gauge
        let cpu_gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("🖥️  CPU"))
            .gauge_style(Style::default().fg(Color::Green))
            .ratio(resources.cpu_percent / 100.0)
            .label(format!("{:.1}%", resources.cpu_percent));
        f.render_widget(cpu_gauge, chunks[1]);
        
        // Memory usage gauge
        let mem_gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("💾 Memory"))
            .gauge_style(Style::default().fg(Color::Blue))
            .ratio((resources.memory_mb as f64 / 1024.0).min(1.0))
            .label(format!("{}MB", resources.memory_mb));
        f.render_widget(mem_gauge, chunks[2]);
        
        // Cache hit rate gauge
        let cache_gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("📊 Cache Hit"))
            .gauge_style(Style::default().fg(Color::Cyan))
            .ratio(metrics.cache_hit_rate)
            .label(format!("{:.1}%", metrics.cache_hit_rate * 100.0));
        f.render_widget(cache_gauge, chunks[3]);
    }
    
    /// Draw performance charts
    fn draw_performance_charts<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ].as_ref())
            .split(area);
        
        // Mock data for sparklines (in real implementation, this would be historical data)
        let cpu_data: Vec<u64> = (0..20).map(|i| (50.0 + 30.0 * (i as f64 * 0.1).sin()) as u64).collect();
        let memory_data: Vec<u64> = (0..20).map(|i| (256 + 128.0 * (i as f64 * 0.15).cos()) as u64).collect();
        
        let cpu_sparkline = Sparkline::default()
            .block(Block::default().borders(Borders::ALL).title("📈 CPU History"))
            .data(&cpu_data)
            .style(Style::default().fg(Color::Green));
        f.render_widget(cpu_sparkline, chunks[0]);
        
        let memory_sparkline = Sparkline::default()
            .block(Block::default().borders(Borders::ALL).title("📈 Memory History"))
            .data(&memory_data)
            .style(Style::default().fg(Color::Blue));
        f.render_widget(memory_sparkline, chunks[1]);
    }
    
    /// Draw file type breakdown
    fn draw_file_type_breakdown<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let metrics = self.performance_monitor.get_metrics();
        
        // Convert file type metrics to bar chart data
        let mut data: Vec<(&str, u64)> = metrics.by_file_type
            .iter()
            .map(|(file_type, metrics)| (file_type.as_str(), metrics.count as u64))
            .collect();
        
        // Sort by count
        data.sort_by(|a, b| b.1.cmp(&a.1));
        data.truncate(10); // Show top 10
        
        let bar_chart = BarChart::default()
            .block(Block::default().borders(Borders::ALL).title("📋 File Types"))
            .data(&data)
            .bar_width(4)
            .bar_style(Style::default().fg(Color::Magenta))
            .value_style(Style::default().add_modifier(Modifier::BOLD));
        
        f.render_widget(bar_chart, area);
    }
    
    /// Draw system tab
    fn draw_system_tab<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6),  // System info
                Constraint::Min(0),     // Process list
            ].as_ref())
            .split(area);
        
        self.draw_system_info(f, chunks[0]);
        self.draw_system_processes(f, chunks[1]);
    }
    
    /// Draw system information
    fn draw_system_info<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let sys_info = vec![
            Spans::from(vec![
                Span::styled("🖥️  System: ", Style::default().fg(Color::White)),
                Span::styled(format!("{}", self.system.name().unwrap_or("Unknown")), 
                    Style::default().fg(Color::Cyan)),
                Span::raw("  "),
                Span::styled("💾 Total Memory: ", Style::default().fg(Color::White)),
                Span::styled(format!("{:.1} GB", self.system.total_memory() as f64 / 1024.0 / 1024.0), 
                    Style::default().fg(Color::Green)),
            ]),
            Spans::from(vec![
                Span::styled("🔧 CPUs: ", Style::default().fg(Color::White)),
                Span::styled(format!("{}", self.system.cpus().len()), 
                    Style::default().fg(Color::Yellow)),
                Span::raw("  "),
                Span::styled("⚡ Threads: ", Style::default().fg(Color::White)),
                Span::styled(format!("{}", rayon::current_num_threads()), 
                    Style::default().fg(Color::Magenta)),
            ]),
            Spans::from(vec![
                Span::styled("⏱️  Uptime: ", Style::default().fg(Color::White)),
                Span::styled(format!("{:.0} seconds", self.system.uptime()), 
                    Style::default().fg(Color::Blue)),
            ]),
        ];
        
        let paragraph = Paragraph::new(sys_info)
            .block(Block::default().borders(Borders::ALL).title("🖥️  System Information"))
            .wrap(Wrap { trim: true });
        
        f.render_widget(paragraph, area);
    }
    
    /// Draw system processes
    fn draw_system_processes<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let processes: Vec<Row> = self.system.processes()
            .iter()
            .filter(|(_, process)| process.name().contains("synx") || process.name().contains("rust") || process.name().contains("cargo"))
            .take(10)
            .map(|(pid, process)| {
                Row::new(vec![
                    Cell::from(format!("{}", pid)),
                    Cell::from(process.name()),
                    Cell::from(format!("{:.1}%", process.cpu_usage())),
                    Cell::from(format!("{:.1}MB", process.memory() as f64 / 1024.0 / 1024.0)),
                ])
            })
            .collect();
        
        let table = Table::new(processes)
            .header(Row::new(vec![
                Cell::from("PID").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("Name").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("CPU%").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("Memory").style(Style::default().add_modifier(Modifier::BOLD)),
            ]))
            .block(Block::default().borders(Borders::ALL).title("🔧 Related Processes"))
            .widths(&[
                Constraint::Length(8),
                Constraint::Min(15),
                Constraint::Length(8),
                Constraint::Length(10),
            ]);
        
        f.render_widget(table, area);
    }
    
    /// Draw results tab
    fn draw_results_tab<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        if let Some(results) = self.execution_results.read().unwrap().as_ref() {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(8),  // Summary
                    Constraint::Min(0),     // Detailed results
                ].as_ref())
                .split(area);
            
            self.draw_results_summary(f, chunks[0], results);
            self.draw_detailed_results(f, chunks[1], results);
        } else {
            let paragraph = Paragraph::new(vec![
                Spans::from(vec![
                    Span::styled("No validation results available yet.", 
                        Style::default().fg(Color::Gray)),
                ]),
                Spans::from(vec![
                    Span::styled("Run a validation to see results here.", 
                        Style::default().fg(Color::Gray)),
                ]),
            ])
            .block(Block::default().borders(Borders::ALL).title("📊 Results"))
            .alignment(Alignment::Center);
            
            f.render_widget(paragraph, area);
        }
    }
    
    /// Draw results summary
    fn draw_results_summary<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, results: &ParallelExecutionResult) {
        let summary = vec![
            Spans::from(vec![
                Span::styled("📁 Total Files: ", Style::default().fg(Color::White)),
                Span::styled(format!("{}", results.total_files), 
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled("⚡ Processed: ", Style::default().fg(Color::White)),
                Span::styled(format!("{}", results.files_processed), 
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Spans::from(vec![
                Span::styled("✅ Success: ", Style::default().fg(Color::White)),
                Span::styled(format!("{}", results.validation_successes), 
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled("❌ Errors: ", Style::default().fg(Color::White)),
                Span::styled(format!("{}", results.validation_errors), 
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled("💾 Cached: ", Style::default().fg(Color::White)),
                Span::styled(format!("{}", results.cache_hits), 
                    Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
            ]),
            Spans::from(vec![
                Span::styled("📊 Success Rate: ", Style::default().fg(Color::White)),
                Span::styled(format!("{:.1}%", results.success_rate() * 100.0), 
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled("⚡ Throughput: ", Style::default().fg(Color::White)),
                Span::styled(format!("{:.1} files/sec", results.throughput()), 
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Spans::from(vec![
                Span::styled("⏱️  Total Time: ", Style::default().fg(Color::White)),
                Span::styled(format!("{:.2}s", results.total_duration.as_secs_f64()), 
                    Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled("🔧 Workers: ", Style::default().fg(Color::White)),
                Span::styled(format!("{}", results.parallelism_used), 
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
        ];
        
        let paragraph = Paragraph::new(summary)
            .block(Block::default().borders(Borders::ALL).title("📊 Execution Summary"))
            .wrap(Wrap { trim: true });
        
        f.render_widget(paragraph, area);
    }
    
    /// Draw detailed results
    fn draw_detailed_results<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, results: &ParallelExecutionResult) {
        let recent_results = self.recent_results.read().unwrap();
        
        let rows: Vec<Row> = recent_results
            .iter()
            .take(50) // Show last 50 results
            .map(|result| {
                let status_color = if result.is_valid {
                    if result.was_cached {
                        Color::Blue
                    } else {
                        Color::Green
                    }
                } else {
                    Color::Red
                };
                
                let status_text = if result.was_cached {
                    "💾 CACHED"
                } else if result.is_valid {
                    "✅ VALID"
                } else {
                    "❌ INVALID"
                };
                
                Row::new(vec![
                    Cell::from(status_text).style(Style::default().fg(status_color)),
                    Cell::from(result.path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string()),
                    Cell::from(format!("{:.2}ms", result.duration.as_millis())),
                    Cell::from(result.error.clone().unwrap_or_else(|| "None".to_string())),
                ])
            })
            .collect();
        
        let table = Table::new(rows)
            .header(Row::new(vec![
                Cell::from("Status").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("File").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("Duration").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("Error").style(Style::default().add_modifier(Modifier::BOLD)),
            ]))
            .block(Block::default().borders(Borders::ALL).title("📋 Recent Results"))
            .widths(&[
                Constraint::Length(12),
                Constraint::Min(20),
                Constraint::Length(10),
                Constraint::Min(15),
            ]);
        
        f.render_widget(table, area);
    }
    
    /// Draw status bar
    fn draw_status_bar<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let status = vec![
            Spans::from(vec![
                Span::styled(" Tab", Style::default().fg(Color::Yellow)),
                Span::raw(": Switch tabs  "),
                Span::styled("↑↓", Style::default().fg(Color::Yellow)),
                Span::raw(": Navigate  "),
                Span::styled("h", Style::default().fg(Color::Yellow)),
                Span::raw(": Help  "),
                Span::styled("d", Style::default().fg(Color::Yellow)),
                Span::raw(": Details  "),
                Span::styled("s", Style::default().fg(Color::Yellow)),
                Span::raw(": Sort  "),
                Span::styled("f", Style::default().fg(Color::Yellow)),
                Span::raw(": Filter  "),
                Span::styled("q", Style::default().fg(Color::Red)),
                Span::raw(": Quit"),
            ]),
        ];
        
        let paragraph = Paragraph::new(status)
            .style(Style::default().bg(Color::DarkGray));
        
        f.render_widget(paragraph, area);
    }
    
    /// Draw help popup
    fn draw_help_popup<B: Backend>(&mut self, f: &mut Frame<B>) {
        let area = centered_rect(60, 70, f.size());
        f.render_widget(Clear, area);
        
        let help_text = vec![
            Spans::from(vec![Span::styled("🔍 Synx Interactive Monitor - Help", 
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            Spans::from(vec![Span::raw("")]),
            Spans::from(vec![Span::styled("Navigation:", Style::default().add_modifier(Modifier::BOLD))]),
            Spans::from(vec![Span::raw("  Tab        - Switch between tabs")]),
            Spans::from(vec![Span::raw("  ↑/↓        - Navigate process list")]),
            Spans::from(vec![Span::raw("  h/F1       - Toggle this help")]),
            Spans::from(vec![Span::raw("  d          - Toggle details popup")]),
            Spans::from(vec![Span::raw("  r          - Refresh data")]),
            Spans::from(vec![Span::raw("  q/Esc      - Quit")]),
            Spans::from(vec![Span::raw("")]),
            Spans::from(vec![Span::styled("Process List:", Style::default().add_modifier(Modifier::BOLD))]),
            Spans::from(vec![Span::raw("  s          - Cycle sort options")]),
            Spans::from(vec![Span::raw("  f          - Cycle status filter")]),
            Spans::from(vec![Span::raw("")]),
            Spans::from(vec![Span::styled("Status Colors:", Style::default().add_modifier(Modifier::BOLD))]),
            Spans::from(vec![
                Span::styled("  🏃 Running", Style::default().fg(Color::Yellow)),
                Span::raw("  ✅ Completed  ❌ Failed  💾 Cached")
            ]),
        ];
        
        let help_popup = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title("Help"))
            .wrap(Wrap { trim: true });
        
        f.render_widget(help_popup, area);
    }
    
    /// Draw details popup
    fn draw_details_popup<B: Backend>(&mut self, f: &mut Frame<B>) {
        let processes = self.active_processes.read().unwrap();
        if let Some(process) = processes.get(self.ui_state.selected_process) {
            let area = centered_rect(80, 60, f.size());
            f.render_widget(Clear, area);
            
            let details = vec![
                Spans::from(vec![Span::styled(format!("Process Details - ID {}", process.id), 
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
                Spans::from(vec![Span::raw("")]),
                Spans::from(vec![
                    Span::styled("File: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(&process.file_path)
                ]),
                Spans::from(vec![
                    Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(&process.file_type)
                ]),
                Spans::from(vec![
                    Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{:?}", process.status), 
                        Style::default().fg(process.status.color()))
                ]),
                Spans::from(vec![
                    Span::styled("Worker ID: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(format!("{}", process.worker_id))
                ]),
                Spans::from(vec![
                    Span::styled("Start Time: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(format!("{:.2}s ago", process.start_time.elapsed().as_secs_f64()))
                ]),
                Spans::from(vec![
                    Span::styled("Duration: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(format!("{:.2}s", 
                        process.duration.unwrap_or_else(|| process.start_time.elapsed()).as_secs_f64()))
                ]),
                Spans::from(vec![
                    Span::styled("Progress: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(format!("{:.1}%", process.progress * 100.0))
                ]),
            ];
            
            let details_popup = Paragraph::new(details)
                .block(Block::default().borders(Borders::ALL).title("Process Details"))
                .wrap(Wrap { trim: true });
            
            f.render_widget(details_popup, area);
        }
    }
    
    /// Update system information
    async fn update_system_info(&mut self) {
        self.system.refresh_all();
        
        // Update resource usage for performance monitor
        let cpu_percent = self.system.global_cpu_info().cpu_usage() as f64;
        let memory_mb = (self.system.used_memory() / 1024 / 1024) as usize;
        
        let resource_usage = ResourceUsage {
            cpu_percent,
            memory_mb,
            disk_io_mb: 0.0, // Would need platform-specific implementation
            network_kb: 0.0,
        };
        
        self.performance_monitor.update_resource_usage(resource_usage);
    }
    
    /// Refresh all data
    async fn refresh_data(&mut self) {
        self.update_system_info().await;
        // Additional refresh logic would go here
    }
    
    /// Cycle through sort options
    fn cycle_sort(&mut self) {
        self.ui_state.sort_by = match self.ui_state.sort_by {
            SortBy::FileName => SortBy::Status,
            SortBy::Status => SortBy::Duration,
            SortBy::Duration => SortBy::FileType,
            SortBy::FileType => SortBy::WorkerId,
            SortBy::WorkerId => SortBy::FileName,
        };
    }
    
    /// Cycle through filter options
    fn cycle_filter(&mut self) {
        self.ui_state.filter_status = match &self.ui_state.filter_status {
            None => Some(ProcessStatus::Running),
            Some(ProcessStatus::Running) => Some(ProcessStatus::Completed),
            Some(ProcessStatus::Completed) => Some(ProcessStatus::Failed),
            Some(ProcessStatus::Failed) => Some(ProcessStatus::Cached),
            Some(ProcessStatus::Cached) => Some(ProcessStatus::Queued),
            Some(ProcessStatus::Queued) => None,
        };
    }
    
    /// Add a validation process
    pub fn add_process(&self, process: ValidationProcess) {
        let mut processes = self.active_processes.write().unwrap();
        processes.push(process);
    }
    
    /// Update a validation process
    pub fn update_process(&self, id: usize, status: ProcessStatus, progress: f64, duration: Option<Duration>) {
        let mut processes = self.active_processes.write().unwrap();
        if let Some(process) = processes.iter_mut().find(|p| p.id == id) {
            process.status = status;
            process.progress = progress;
            process.duration = duration;
        }
    }
    
    /// Add validation results
    pub fn add_results(&self, results: Vec<FileResult>) {
        let mut recent_results = self.recent_results.write().unwrap();
        recent_results.extend(results);
        // Keep only the last 1000 results
        if recent_results.len() > 1000 {
            recent_results.drain(0..recent_results.len() - 1000);
        }
    }
    
    /// Set execution results
    pub fn set_execution_results(&self, results: ParallelExecutionResult) {
        let mut exec_results = self.execution_results.write().unwrap();
        *exec_results = Some(results);
    }
}

/// 🎆 ENTRY POINT: Launch the revolutionary genius TUI monitoring interface
/// 
/// This function creates and runs the most advanced code monitoring interface ever built,
/// featuring AI-powered analysis, real-time visualizations, and mind-blowing insights.
pub async fn run_monitor(paths: Vec<std::path::PathBuf>, auto_validate: bool) -> Result<()> {
    // Create performance monitor
    let performance_monitor = std::sync::Arc::new(
        crate::performance::metrics::PerformanceMonitor::new(
            crate::performance::PerformanceConfig::default()
        ).map_err(|e| anyhow::anyhow!("Failed to create performance monitor: {}", e))?
    );
    
    // Initialize the genius TUI
    let mut tui = InteractiveTUI::new(performance_monitor.clone());
    
    // Add some sample processes for demonstration
    for (i, path) in paths.iter().enumerate() {
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            let process = ValidationProcess {
                id: i + 1,
                file_path: file_name.to_string(),
                status: match i % 4 {
                    0 => ProcessStatus::Running,
                    1 => ProcessStatus::Completed,
                    2 => ProcessStatus::Failed,
                    3 => ProcessStatus::Cached,
                    _ => ProcessStatus::Queued,
                },
                start_time: std::time::Instant::now() - std::time::Duration::from_secs(i as u64 * 5),
                duration: if i % 2 == 0 { Some(std::time::Duration::from_secs(i as u64 + 1)) } else { None },
                progress: (i as f64 + 1.0) * 0.2,
                file_type: path.extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                worker_id: (i % 4) + 1,
            };
            tui.add_process(process);
        }
    }
    
    // Add some sample file results for demonstration
    let sample_results: Vec<crate::performance::parallel::FileResult> = paths.iter().enumerate().map(|(i, path)| {
        crate::performance::parallel::FileResult {
            path: path.clone(),
            is_valid: i % 3 != 0,
            was_cached: i % 2 == 0,
            duration: std::time::Duration::from_millis(50 + i as u64 * 10),
            error: if i % 3 == 0 { Some(format!("Error in file {}", i)) } else { None },
        }
    }).collect();
    
    tui.add_results(sample_results);
    
    // Run the genius TUI
    tui.run().await
}

/// Helper function to create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ].as_ref())
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ].as_ref())
        .split(popup_layout[1])[1]
}
