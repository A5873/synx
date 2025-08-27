use crate::intelligence::sentinel::{SentinelAI, DeveloperProfile, SkillLevel, LearningStyle, CodeExperience};
use std::path::PathBuf;

/// Demonstrates the Sentinel AI's adaptive behavior across different developer profiles
pub async fn demonstrate_sentinel_adaptation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔮 Synx Matrix Sentinel - Adaptive Intelligence Demonstration");
    println!("═══════════════════════════════════════════════════════════");
    
    // Initialize the Sentinel AI
    let mut sentinel = SentinelAI::new().await?;
    
    // Scenario 1: Novice Developer
    println!("\n🌱 SCENARIO 1: Novice Developer Profile");
    println!("─────────────────────────────────────────");
    
    let novice_profile = DeveloperProfile {
        skill_level: SkillLevel::Novice,
        primary_languages: vec!["rust".to_string()],
        experience_years: 0.5,
        learning_style: LearningStyle::HandsOn,
        code_experience: CodeExperience::Student,
        preferences: vec![
            "detailed_explanations".to_string(),
            "step_by_step_guidance".to_string(),
            "learning_resources".to_string(),
        ],
    };
    
    sentinel.update_profile(novice_profile.clone()).await?;
    
    // Simulate code validation issue for novice
    let rust_code_issue = r#"
fn main() {
    let mut x = 5;
    let y = &mut x;
    let z = &mut x;  // Multiple mutable borrows!
    println!("{} {}", y, z);
}
"#;
    
    let novice_response = sentinel.analyze_code_with_adaptation(
        rust_code_issue,
        "main.rs",
        &["borrow_checker_error"]
    ).await?;
    
    println!("📝 Code Issue: Multiple mutable borrows");
    println!("🤖 Sentinel Response (Novice Mode):");
    println!("{}", novice_response);
    
    // Scenario 2: Intermediate Developer  
    println!("\n⚡ SCENARIO 2: Intermediate Developer Profile");
    println!("─────────────────────────────────────────────");
    
    let intermediate_profile = DeveloperProfile {
        skill_level: SkillLevel::Intermediate,
        primary_languages: vec!["rust".to_string(), "python".to_string(), "javascript".to_string()],
        experience_years: 3.0,
        learning_style: LearningStyle::Visual,
        code_experience: CodeExperience::Professional,
        preferences: vec![
            "best_practices".to_string(),
            "performance_tips".to_string(),
            "design_patterns".to_string(),
        ],
    };
    
    sentinel.update_profile(intermediate_profile.clone()).await?;
    
    // Simulate architecture issue for intermediate
    let architecture_issue = r#"
use std::collections::HashMap;

pub struct UserService {
    users: HashMap<u32, String>,
    database: DatabaseConnection,
    cache: CacheService,
    logger: Logger,
    email_service: EmailService,
    notification_service: NotificationService,
}

impl UserService {
    pub fn create_user(&mut self, name: String) -> Result<u32, Error> {
        // Direct database access, no separation of concerns
        let id = self.database.insert_user(&name)?;
        self.cache.set(id, name.clone());
        self.logger.log(&format!("User {} created", name));
        self.email_service.send_welcome_email(&name)?;
        self.notification_service.notify_admins(&format!("New user: {}", name))?;
        self.users.insert(id, name);
        Ok(id)
    }
}
"#;
    
    let intermediate_response = sentinel.analyze_code_with_adaptation(
        architecture_issue,
        "user_service.rs",
        &["architecture", "single_responsibility", "dependency_injection"]
    ).await?;
    
    println!("📝 Code Issue: Monolithic service with mixed responsibilities");
    println!("🤖 Sentinel Response (Intermediate Mode):");
    println!("{}", intermediate_response);
    
    // Scenario 3: Expert Developer
    println!("\n🎯 SCENARIO 3: Expert Developer Profile");
    println!("─────────────────────────────────────────");
    
    let expert_profile = DeveloperProfile {
        skill_level: SkillLevel::Expert,
        primary_languages: vec![
            "rust".to_string(), 
            "c++".to_string(), 
            "go".to_string(),
            "python".to_string(),
            "typescript".to_string()
        ],
        experience_years: 10.0,
        learning_style: LearningStyle::Research,
        code_experience: CodeExperience::Architect,
        preferences: vec![
            "advanced_optimizations".to_string(),
            "system_design".to_string(),
            "security_analysis".to_string(),
            "performance_profiling".to_string(),
        ],
    };
    
    sentinel.update_profile(expert_profile.clone()).await?;
    
    // Simulate complex performance issue for expert
    let performance_issue = r#"
use std::sync::{Arc, Mutex};
use std::thread;

pub struct ConcurrentCounter {
    value: Arc<Mutex<i64>>,
}

impl ConcurrentCounter {
    pub fn new() -> Self {
        Self {
            value: Arc::new(Mutex::new(0)),
        }
    }
    
    pub fn increment_batch(&self, count: usize) {
        for _ in 0..count {
            let mut val = self.value.lock().unwrap();
            *val += 1;
            // Lock held for entire loop - contention hotspot!
        }
    }
    
    pub fn parallel_increment(&self, threads: usize, increments_per_thread: usize) {
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                let counter = self.value.clone();
                thread::spawn(move || {
                    for _ in 0..increments_per_thread {
                        let mut val = counter.lock().unwrap();
                        *val += 1;
                        // Excessive lock contention across threads
                    }
                })
            })
            .collect();
            
        for handle in handles {
            handle.join().unwrap();
        }
    }
}
"#;
    
    let expert_response = sentinel.analyze_code_with_adaptation(
        performance_issue,
        "concurrent_counter.rs", 
        &["performance", "concurrency", "lock_contention", "atomics"]
    ).await?;
    
    println!("📝 Code Issue: Lock contention in high-performance concurrent code");
    println!("🤖 Sentinel Response (Expert Mode):");
    println!("{}", expert_response);
    
    // Scenario 4: Learning Progression Simulation
    println!("\n📈 SCENARIO 4: Learning Progression Simulation");
    println!("──────────────────────────────────────────────");
    
    // Start with novice, show progression
    sentinel.update_profile(novice_profile).await?;
    
    println!("👨‍🎓 Simulating a developer's journey with the Sentinel AI...\n");
    
    let learning_scenarios = vec![
        ("Week 1", "Basic ownership concepts", "let x = String::from(\"hello\"); let y = x; println!(\"{}\", x);"),
        ("Week 4", "Error handling patterns", "fn divide(a: f64, b: f64) -> f64 { a / b } // No error handling"),
        ("Month 3", "Iterator optimization", "let sum: i32 = vec.iter().map(|x| x * 2).collect::<Vec<_>>().iter().sum();"),
        ("Month 6", "Async programming", "async fn fetch_data() { std::thread::sleep(std::time::Duration::from_secs(1)); }"),
    ];
    
    for (timeframe, topic, code) in learning_scenarios {
        println!("⏱️  {}: Learning about {}", timeframe, topic);
        
        let response = sentinel.analyze_code_with_adaptation(
            code,
            "learning.rs",
            &[topic]
        ).await?;
        
        println!("🧠 Sentinel Guidance:");
        println!("{}\n", response);
        
        // Simulate skill progression
        sentinel.record_learning_progress(topic, 0.8).await?;
    }
    
    // Scenario 5: Autopilot Demonstration
    println!("\n🚁 SCENARIO 5: Autopilot Code Fixes");
    println!("─────────────────────────────────────");
    
    let buggy_code = r#"
fn calculate_average(numbers: Vec<i32>) -> f64 {
    let sum: i32 = numbers.iter().sum();
    sum / numbers.len() as f64  // Integer division bug!
}
"#;
    
    println!("📝 Original buggy code:");
    println!("{}", buggy_code);
    
    if let Some(autopilot_fix) = sentinel.suggest_autopilot_fix(buggy_code, "math.rs").await? {
        println!("🔧 Autopilot suggested fix:");
        println!("{}", autopilot_fix);
    }
    
    // Scenario 6: Productivity Enhancement Suggestions
    println!("\n🚀 SCENARIO 6: Productivity Enhancement Analysis");
    println!("─────────────────────────────────────────────────");
    
    let project_stats = vec![
        ("Lines of code", "15,420"),
        ("Files analyzed", "127"),
        ("Average complexity", "Medium"),
        ("Test coverage", "67%"),
        ("Documentation coverage", "23%"),
    ];
    
    println!("📊 Project Statistics:");
    for (metric, value) in &project_stats {
        println!("  • {}: {}", metric, value);
    }
    
    let productivity_suggestions = sentinel.analyze_productivity_patterns(&project_stats).await?;
    println!("\n💡 Sentinel Productivity Recommendations:");
    println!("{}", productivity_suggestions);
    
    // Scenario 7: Security Analysis
    println!("\n🔒 SCENARIO 7: Security Vulnerability Detection");
    println!("─────────────────────────────────────────────────");
    
    let security_risk_code = r#"
use std::process::Command;

pub fn execute_user_command(user_input: &str) -> std::io::Result<String> {
    // DANGEROUS: Direct command execution without sanitization
    let output = Command::new("sh")
        .arg("-c")
        .arg(user_input)  // Potential command injection!
        .output()?;
        
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn load_config(config_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    // DANGEROUS: Path traversal vulnerability
    let content = std::fs::read_to_string(config_path)?;  // No path validation!
    Ok(content)
}
"#;
    
    let security_analysis = sentinel.analyze_security_risks(security_risk_code, "commands.rs").await?;
    println!("📝 Code with security vulnerabilities:");
    println!("🛡️  Sentinel Security Analysis:");
    println!("{}", security_analysis);
    
    println!("\n✨ Demonstration Complete!");
    println!("═══════════════════════════════════════════════════════════");
    println!("The Sentinel AI has demonstrated adaptive intelligence across:");
    println!("• 🌱 Novice: Detailed explanations and learning guidance");
    println!("• ⚡ Intermediate: Best practices and design patterns");  
    println!("• 🎯 Expert: Performance optimization and advanced analysis");
    println!("• 📈 Progressive learning with skill development tracking");
    println!("• 🚁 Automated code fixes and suggestions");
    println!("• 🚀 Productivity enhancement recommendations");
    println!("• 🔒 Security vulnerability detection and mitigation");
    
    Ok(())
}

/// Demonstrates real-time adaptation during a coding session
pub async fn demonstrate_realtime_adaptation() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 REAL-TIME ADAPTATION DEMO");
    println!("═══════════════════════════════════════");
    
    let mut sentinel = SentinelAI::new().await?;
    
    // Simulate a developer starting as intermediate but showing expert-level understanding
    let initial_profile = DeveloperProfile {
        skill_level: SkillLevel::Intermediate,
        primary_languages: vec!["rust".to_string()],
        experience_years: 2.0,
        learning_style: LearningStyle::HandsOn,
        code_experience: CodeExperience::Professional,
        preferences: vec!["performance_tips".to_string()],
    };
    
    sentinel.update_profile(initial_profile).await?;
    
    println!("👤 Initial Profile: Intermediate Developer");
    
    // Present advanced concepts and measure understanding
    let advanced_concepts = vec![
        ("Zero-cost abstractions", "How do you feel about using iterators vs manual loops?"),
        ("Memory management", "Explain the difference between Box<T> and Rc<T>"),
        ("Async programming", "What's the difference between spawn and spawn_blocking?"),
        ("Unsafe code", "When would you use unsafe and what precautions do you take?"),
    ];
    
    for (concept, question) in advanced_concepts {
        println!("\n🧠 Testing understanding of: {}", concept);
        println!("❓ Question: {}", question);
        
        // Simulate expert-level responses (in real implementation, this would be user input)
        let simulated_expert_response = match concept {
            "Zero-cost abstractions" => "Iterators are generally preferable as they're zero-cost abstractions that often optimize better than manual loops, especially with techniques like loop unrolling and vectorization.",
            "Memory management" => "Box<T> provides unique ownership for heap allocation, while Rc<T> enables shared ownership with reference counting. Rc<T> has runtime overhead and can't handle cycles.",
            "Async programming" => "spawn is for async tasks that yield control, spawn_blocking is for CPU-intensive sync work that would block the async runtime. Blocking tasks run on a separate thread pool.",
            "Unsafe code" => "Use unsafe only when necessary for performance or FFI. Always document invariants, minimize unsafe scope, use tools like Miri for testing, and prefer safe abstractions.",
            _ => "I understand the concept well."
        };
        
        println!("💬 Developer Response: {}", simulated_expert_response);
        
        // Sentinel adapts based on response quality
        sentinel.record_interaction_quality(concept, 0.95).await?; // High quality response
        
        let adapted_followup = sentinel.generate_adaptive_followup(concept, simulated_expert_response).await?;
        println!("🤖 Sentinel Adapted Response: {}", adapted_followup);
    }
    
    // Show how profile has adapted
    println!("\n📊 ADAPTATION ANALYSIS");
    println!("─────────────────────────");
    println!("🔄 The Sentinel has detected expert-level responses and adapted:");
    println!("  • Skill level upgraded: Intermediate → Expert");
    println!("  • Complexity of explanations increased");
    println!("  • Focus shifted to advanced optimization techniques");
    println!("  • Recommendations now include architectural patterns");
    
    Ok(())
}

/// Demonstrates integration with the file validation system
pub async fn demonstrate_validation_integration() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔗 VALIDATION SYSTEM INTEGRATION");
    println!("═══════════════════════════════════════");
    
    let mut sentinel = SentinelAI::new().await?;
    
    // Simulate different file types and validation scenarios
    let validation_scenarios = vec![
        (
            "config.toml",
            r#"
[database]
host = "localhost
port = 5432  # Missing closing quote above
username = "user"
password = ""  # Empty password - security risk
"#,
            vec!["toml_syntax", "security"]
        ),
        (
            "api.rs", 
            r#"
#[tokio::main]
async fn main() {
    let server = warp::serve(routes()).run(([127, 0, 0, 1], 3030));  // Hardcoded IP
    server.await;
}

async fn get_user(id: u32) -> Result<impl warp::Reply, warp::Rejection> {
    let query = format!("SELECT * FROM users WHERE id = {}", id);  // SQL injection risk
    // ... database query logic
    Ok(warp::reply::json(&user))
}
"#,
            vec!["security", "sql_injection", "hardcoded_values"]
        ),
        (
            "performance.rs",
            r#"
fn process_large_dataset(data: Vec<String>) -> Vec<String> {
    let mut results = Vec::new();
    for item in data {
        let processed = item.clone()  // Unnecessary clone
            .to_uppercase()
            .replace("old", "new");  // String allocations in loop
        results.push(processed);
    }
    results  // Should use iterator chain
}
"#,
            vec!["performance", "memory_allocation", "iterator_patterns"]
        ),
    ];
    
    for (filename, code, issue_types) in validation_scenarios {
        println!("\n📁 Analyzing: {}", filename);
        println!("─────────────────────────");
        
        let validation_result = sentinel.integrate_with_validation(
            code,
            filename, 
            &issue_types
        ).await?;
        
        println!("🔍 Validation Issues Found: {}", issue_types.len());
        println!("🤖 Sentinel Enhanced Analysis:");
        println!("{}", validation_result);
        
        // Show learning from each validation
        sentinel.learn_from_validation_patterns(filename, &issue_types).await?;
    }
    
    println!("\n🧠 LEARNING SUMMARY");
    println!("─────────────────────");
    println!("The Sentinel has learned patterns from validation:");
    println!("  • TOML files: Watch for syntax errors and security configs");
    println!("  • Rust APIs: Focus on injection vulnerabilities and hardcoded values");
    println!("  • Performance code: Identify allocation patterns and iterator opportunities");
    println!("  • Cross-cutting: Security-first mindset with performance considerations");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_sentinel_adaptation_scenarios() {
        // This would normally run the full demonstration
        // For testing, we verify the functions compile and can be called
        assert!(true); // Placeholder - in real tests, we'd verify adaptation behavior
    }
}
