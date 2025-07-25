use std::path::{Path, PathBuf};
use std::process::{Command, Stdio, Child};
use std::time::Duration;
use anyhow::{Result, anyhow, Context};
use regex::Regex;
use log::debug;
use serde::{Serialize, Deserialize};

// Platform-specific imports
#[cfg(target_os = "linux")]
use rlimit::{Resource, setrlimit};

#[cfg(unix)]
use nix::sys::signal::{kill, Signal};
#[cfg(unix)]
use nix::unistd::Pid;


#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::{
    CreateJobObjectW, SetInformationJobObject, AssignProcessToJobObject,
    JOBOBJECT_BASIC_LIMIT_INFORMATION, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JOB_OBJECT_LIMIT_PROCESS_MEMORY, JOB_OBJECT_LIMIT_PROCESS_TIME,
};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Maximum execution time in seconds
    pub timeout: u64,
    /// Maximum memory usage in megabytes
    pub memory_limit: u64,
    /// Maximum CPU usage percentage
    pub cpu_limit: u32,
    /// Whether to allow network access
    pub allow_network: bool,
    /// Allowed directories for file access
    pub allowed_paths: Vec<PathBuf>,
    /// Additional security restrictions
    pub restrictions: SecurityRestrictions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRestrictions {
    /// Whether to allow shell expansions
    pub allow_shell_expansion: bool,
    /// Whether to allow file writes
    pub allow_file_writes: bool,
    /// Whether to allow subprocess creation
    pub allow_subprocesses: bool,
    /// Whether to allow environment modifications
    pub allow_env_modifications: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            timeout: 30,
            memory_limit: 512,
            cpu_limit: 50,
            allow_network: false,
            allowed_paths: vec![],
            restrictions: SecurityRestrictions {
                allow_shell_expansion: false,
                allow_file_writes: false,
                allow_subprocesses: false,
                allow_env_modifications: false,
            },
        }
    }
}

/// A secure command builder that enforces security restrictions
pub struct SecureCommand {
    program: PathBuf,
    args: Vec<String>,
    config: SecurityConfig,
    current_dir: Option<PathBuf>,
    env_vars: Vec<(String, String)>,
}

impl SecureCommand {
    /// Create a new secure command with the given program path
    pub fn new<P: AsRef<Path>>(program: P) -> Result<Self> {
        let program_path = validate_program_path(program.as_ref())?;
        
        Ok(Self {
            program: program_path,
            args: Vec::new(),
            config: SecurityConfig::default(),
            current_dir: None,
            env_vars: Vec::new(),
        })
    }

    /// Set security configuration
    pub fn with_config(mut self, config: SecurityConfig) -> Self {
        self.config = config;
        self
    }

    /// Add an argument to the command
    pub fn arg<S: AsRef<str>>(mut self, arg: S) -> Result<Self> {
        let safe_arg = sanitize_argument(arg.as_ref())?;
        self.args.push(safe_arg);
        Ok(self)
    }

    /// Add multiple arguments to the command
    pub fn args<I, S>(mut self, args: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for arg in args {
            self = self.arg(arg)?;
        }
        Ok(self)
    }

    /// Set the working directory for the command
    pub fn current_dir<P: AsRef<Path>>(mut self, dir: P) -> Result<Self> {
        let safe_path = validate_path(dir.as_ref(), &self.config.allowed_paths)?;
        self.current_dir = Some(safe_path);
        Ok(self)
    }

    /// Add an environment variable
    pub fn env<K, V>(mut self, key: K, val: V) -> Result<Self>
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        if !self.config.restrictions.allow_env_modifications {
            return Err(anyhow!("Environment modifications are not allowed"));
        }

        let safe_key = sanitize_env_key(key.as_ref())?;
        let safe_val = sanitize_env_value(val.as_ref())?;
        self.env_vars.push((safe_key, safe_val));
        Ok(self)
    }

    /// Execute the command and return its output
    pub fn output(self) -> Result<std::process::Output> {
        let mut command = self.build_command()?;
        
        // Apply platform-specific resource limitations before execution
        self.apply_resource_limits()?;

        // Execute with timeout
        let child = command.spawn()?;
        
        // Apply platform-specific process constraints after spawning
        self.apply_process_constraints(&child)?;
        
        // Run with timeout mechanism (works on all platforms)
        self.run_with_timeout(child)
    }
    
    /// Apply resource limits in a platform-specific way
    fn apply_resource_limits(&self) -> Result<()> {
        // Linux-specific resource limits using rlimit
        #[cfg(target_os = "linux")]
        {
            // Set memory limit
            if let Ok(Resource::AS) = Resource::from_str("AS") {
                let memory_bytes = self.config.memory_limit * 1024 * 1024;
                setrlimit(Resource::AS, memory_bytes, memory_bytes)?;
            }
            
            // Set CPU limit
            if let Ok(Resource::CPU) = Resource::from_str("CPU") {
                setrlimit(Resource::CPU, self.config.cpu_limit as u64, self.config.cpu_limit as u64)?;
            }
        }
        
        // macOS-specific resource limits
        #[cfg(target_os = "macos")]
        {
            // macOS doesn't have direct rlimit equivalents for all resources
            // We'll rely on sandbox profiles for more fine-grained control
            debug!("Using macOS sandbox for resource limitations");
        }
        
        // Windows-specific resource limits using job objects
        #[cfg(target_os = "windows")]
        {
            // Windows resource limits are applied during apply_process_constraints
            // after the process is spawned
            debug!("Windows resource limits will be applied via job objects");
        }
        
        Ok(())
    }
    
    /// Apply additional constraints to an already spawned process
    fn apply_process_constraints(&self, child: &Child) -> Result<()> {
        // Windows-specific process constraints using job objects
        #[cfg(target_os = "windows")]
        {
            unsafe {
                // Create a job object for the process
                let job_handle = CreateJobObjectW(std::ptr::null_mut(), std::ptr::null());
                if job_handle != 0 {
                    // Set memory and CPU limits
                    let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
                    info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_PROCESS_MEMORY | JOB_OBJECT_LIMIT_PROCESS_TIME;
                    
                    // Convert MB to bytes for memory limit
                    info.ProcessMemoryLimit = (self.config.memory_limit * 1024 * 1024) as usize;
                    
                    // Convert seconds to 100-nanosecond intervals for CPU time
                    info.BasicLimitInformation.PerProcessUserTimeLimit = ((self.config.cpu_limit as u64) * 10000000) as i64;
                    
                    // Apply the limits to the job
                    let result = SetInformationJobObject(
                        job_handle, 
                        9, // JobObjectExtendedLimitInformation
                        &info as *const _ as *const std::ffi::c_void,
                        std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32
                    );
                    
                    if result == 0 {
                        warn!("Failed to set job object information");
                    }
                    
                    // Assign the process to the job
                    let process_handle = child.id() as isize;
                    if process_handle != 0 {
                        let result = AssignProcessToJobObject(job_handle, process_handle);
                        if result == 0 {
                            warn!("Failed to assign process to job object");
                        }
                    }
                } else {
                    warn!("Failed to create job object for process constraints");
                }
            }
        }
        
        // macOS-specific process constraints
        #[cfg(target_os = "macos")]
        {
            // On macOS, we would ideally apply sandbox profiles here
            // However, since that requires elevated privileges, we use a fallback approach
            debug!("Using basic process constraints on macOS");
        }
        
        Ok(())
    }

    /// Build the underlying command with all security measures applied
    fn build_command(&self) -> Result<Command> {
        let mut command = Command::new(&self.program);
        
        // Set up basic command parameters (platform-agnostic)
        command.args(&self.args)
               .stdin(Stdio::null())
               .stdout(Stdio::piped())
               .stderr(Stdio::piped());

        // Set working directory if specified
        if let Some(dir) = &self.current_dir {
            command.current_dir(dir);
        }

        // Add environment variables
        for (key, val) in &self.env_vars {
            command.env(key, val);
        }

        // Apply platform-specific security restrictions
        
        // Linux: Apply seccomp filters via pre_exec
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::process::CommandExt;
            
            // Only apply seccomp filters if seccomp is enabled at compile time
            #[cfg(feature = "seccomp")]
            {
                // Clone the config to move it into the closure
                let config = self.config.clone();
                
                unsafe {
                    command.pre_exec(move || {
                        // Set up seccomp filters
                        if let Err(e) = setup_seccomp_filters(&config) {
                            error!("Failed to set up seccomp filters: {}", e);
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "Failed to set up security filters"
                            ));
                        }
                        
                        Ok(())
                    });
                }
            }
            
            #[cfg(not(feature = "seccomp"))]
            {
                debug!("Seccomp filtering disabled at compile time");
            }
        }
        
        // macOS: Apply sandbox profile if enabled
        #[cfg(target_os = "macos")]
        {
            
            
            // Only apply sandbox if the feature is enabled
            #[cfg(feature = "macos-security")]
            {
                // Clone the config to move it into the closure
                let config = self.config.clone();
                
                unsafe {
                    command.pre_exec(move || {
                        // Set up macOS sandbox profile
                        if let Err(e) = setup_macos_sandbox(&config) {
                            error!("Failed to set up macOS sandbox: {}", e);
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "Failed to set up security sandbox"
                            ));
                        }
                        
                        Ok(())
                    });
                }
            }
            
            #[cfg(not(feature = "macos-security"))]
            {
                debug!("macOS sandbox disabled at compile time");
            }
        }
        
        // Windows: Security measures are applied after process creation

        Ok(command)
    }

    /// Run the command with a timeout (platform-agnostic implementation)
    fn run_with_timeout(self, child: Child) -> Result<std::process::Output> {
        let timeout = Duration::from_secs(self.config.timeout);
        
        // Start timeout thread
        let (tx, rx) = std::sync::mpsc::channel();
        let pid = child.id();
        
        std::thread::spawn(move || {
            std::thread::sleep(timeout);
            let _ = tx.send(());
            
            // Terminate process in a platform-specific way
            terminate_process(pid);
        });

        // Wait for either completion or timeout
        match rx.recv_timeout(timeout) {
            Ok(_) => {
                // Timeout occurred
                Err(anyhow!("Command execution timed out after {} seconds", self.config.timeout))
            }
            Err(_) => {
                // Command completed before timeout
                child.wait_with_output()
                    .context("Failed to wait for command output")
            }
        }
    }
}

/// Validate and sanitize a program path
fn validate_program_path(path: &Path) -> Result<PathBuf> {
    let canonical = path.canonicalize()
        .context("Failed to canonicalize program path")?;
        
    // Check if the program exists and is executable
    if !canonical.exists() {
        return Err(anyhow!("Program does not exist: {}", path.display()));
    }
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = canonical.metadata()?;
        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(anyhow!("Program is not executable: {}", path.display()));
        }
    }

    Ok(canonical)
}

/// Validate and sanitize a path against allowed paths
fn validate_path(path: &Path, allowed_paths: &[PathBuf]) -> Result<PathBuf> {
    let canonical = path.canonicalize()
        .context("Failed to canonicalize path")?;
        
    // Check if the path is within allowed directories
    if !allowed_paths.is_empty() {
        let is_allowed = allowed_paths.iter().any(|allowed| {
            canonical.starts_with(allowed)
        });
        
        if !is_allowed {
            return Err(anyhow!("Path is not in allowed directories: {}", path.display()));
        }
    }

    Ok(canonical)
}

/// Sanitize a command argument
fn sanitize_argument(arg: &str) -> Result<String> {
    // Define patterns for potentially dangerous characters
    let dangerous_patterns = Regex::new(r#"[;&|`$<>]"#)?;
    
    if dangerous_patterns.is_match(arg) {
        return Err(anyhow!("Argument contains dangerous characters: {}", arg));
    }

    Ok(arg.to_string())
}

/// Sanitize an environment variable key
fn sanitize_env_key(key: &str) -> Result<String> {
    // Environment variables should be ASCII and not contain =
    if !key.chars().all(|c| c.is_ascii() && c != '=') {
        return Err(anyhow!("Invalid environment variable name: {}", key));
    }

    Ok(key.to_string())
}

/// Sanitize an environment variable value
fn sanitize_env_value(value: &str) -> Result<String> {
    // Basic sanitation for env values
    let dangerous_patterns = Regex::new(r#"[;&|`$<>]"#)?;
    
    if dangerous_patterns.is_match(value) {
        return Err(anyhow!("Environment value contains dangerous characters: {}", value));
    }

    Ok(value.to_string())
}

/// Platform-agnostic function to terminate a process
fn terminate_process(pid: u32) {
    #[cfg(unix)]
    {
        let pid = Pid::from_raw(pid as i32);
        let _ = kill(pid, Signal::SIGTERM);
        
        // Give it a moment to terminate gracefully before sending SIGKILL
        std::thread::sleep(Duration::from_millis(100));
        let _ = kill(pid, Signal::SIGKILL);
    }

    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Threading::{OpenProcess, TerminateProcess};
        use windows_sys::Win32::Foundation::HANDLE;
        
        unsafe {
            let process_handle = OpenProcess(0x0001, 0, pid); // PROCESS_TERMINATE access right
            if process_handle != 0 {
                let _ = TerminateProcess(process_handle, 1);
                let _ = windows_sys::Win32::Foundation::CloseHandle(process_handle);
            }
        }
    }
}

/// Setup macOS sandbox for process isolation
#[cfg(all(target_os = "macos", feature = "macos-security"))]
fn setup_macos_sandbox(config: &SecurityConfig) -> Result<()> {
    // On macOS, we can use the sandbox_init function to apply predefined profiles
    // or custom sandbox profiles to restrict process capabilities.
    
    // Basic sandbox profile that denies everything and then allows specific operations
    let mut profile = String::from("(version 1)\n(deny default)\n");
    
    // Allow basic process execution
    profile.push_str("(allow process-exec)\n");
    profile.push_str("(allow process-fork)\n");
    
    // Allow reading from standard file descriptors
    profile.push_str("(allow file-read-data (literal \"/dev/null\"))\n");
    profile.push_str("(allow file-write-data (literal \"/dev/stdout\"))\n");
    profile.push_str("(allow file-write-data (literal \"/dev/stderr\"))\n");
    
    // Add allowed paths
    for path in &config.allowed_paths {
        let path_str = path.to_string_lossy();
        profile.push_str(&format!("(allow file-read* (subpath \"{}\"))\n", path_str));
        
        if config.restrictions.allow_file_writes {
            profile.push_str(&format!("(allow file-write* (subpath \"{}\"))\n", path_str));
        }
    }
    
    // Allow network if enabled
    if config.allow_network {
        profile.push_str("(allow network-outbound)\n");
        profile.push_str("(allow network-inbound)\n");
    }
    
    // Allow subprocess creation if enabled
    if config.restrictions.allow_subprocesses {
        profile.push_str("(allow process-fork)\n");
        profile.push_str("(allow process-exec)\n");
    }
    
    // Clone the profile string for logging before converting it to CString
    let profile_for_log = profile.clone();
    
    // Convert to C string for sandbox_init
    let profile_c_string = CString::new(profile)?;
    
    // macOS sandbox requires linking to libsystem_sandbox.dylib
    // For simplicity in this example, we're using a safe wrapper approach
    // In a real implementation, you would use a proper sandbox crate or FFI bindings
    
    debug!("macOS sandbox profile created. In a real implementation, would call sandbox_init()");
    debug!("Profile content: {}", profile_for_log);
    
    // Simulate sandbox initialization for compatibility
    // In a real implementation, you'd use:
    // 
    // extern "C" {
    //     fn sandbox_init(profile: *const libc::c_char, flags: u64, error: *mut *mut libc::c_char) -> i32;
    // }
    // 
    // let mut error: *mut libc::c_char = std::ptr::null_mut();
    // let result = unsafe { sandbox_init(profile_c_string.as_ptr(), 0, &mut error) };
    // if result != 0 {
    //     let error_string = unsafe { CStr::from_ptr(error).to_string_lossy().into_owned() };
    //     return Err(anyhow!("Failed to initialize sandbox: {}", error_string));
    // }
    
    // For now, we'll just provide a stub implementation and log the profile
    warn!("macOS sandbox implementation is a stub - security restrictions may not be fully applied");
    
    Ok(())
}

#[cfg(target_os = "linux")]
fn setup_seccomp_filters(config: &SecurityConfig) -> Result<()> {
    #[cfg(feature = "seccomp")]
    {
        use seccomp_rs::{SeccompFilter, Action, Comparison, Syscall};
        
        let mut filter = SeccompFilter::new(Action::Allow)?;

        // Block network access if not allowed
        if !config.allow_network {
            filter.add_rule(Syscall::socket, Action::Errno(1))?;
            filter.add_rule(Syscall::connect, Action::Errno(1))?;
            filter.add_rule(Syscall::accept, Action::Errno(1))?;
        }

        // Block file writes if not allowed
        if !config.restrictions.allow_file_writes {
            filter.add_rule(Syscall::open, Action::Errno(1))?;
            filter.add_rule(Syscall::creat, Action::Errno(1))?;
            filter.add_rule(Syscall::rename, Action::Errno(1))?;
            filter.add_rule(Syscall::unlink, Action::Errno(1))?;
        }

        // Block subprocess creation if not allowed
        if !config.restrictions.allow_subprocesses {
            filter.add_rule(Syscall::fork, Action::Errno(1))?;
            filter.add_rule(Syscall::vfork, Action::Errno(1))?;
            filter.add_rule(Syscall::clone, Action::Errno(1))?;
        }

        filter.load()?;
    }
    
    #[cfg(not(feature = "seccomp"))]
    {
        debug!("Seccomp filtering disabled at compile time");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_secure_command_basic() {
        let cmd = SecureCommand::new("echo").unwrap()
            .arg("hello").unwrap()
            .output().unwrap();
        
        assert!(cmd.status.success());
        assert_eq!(String::from_utf8_lossy(&cmd.stdout).trim(), "hello");
    }

    #[test]
    fn test_secure_command_timeout() {
        let config = SecurityConfig {
            timeout: 1,
            ..Default::default()
        };

        let cmd = SecureCommand::new("sleep").unwrap()
            .with_config(config)
            .arg("5").unwrap()
            .output();
            
        assert!(cmd.is_err());
    }

    #[test]
    fn test_path_validation() {
        let temp_dir = TempDir::new().unwrap();
        let allowed_paths = vec![temp_dir.path().to_path_buf()];
        
        // Test allowed path
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test").unwrap();
        
        assert!(validate_path(&test_file, &allowed_paths).is_ok());

        // Test disallowed path
        let outside_file = PathBuf::from("/tmp/test.txt");
        assert!(validate_path(&outside_file, &allowed_paths).is_err());
    }

    #[test]
    fn test_argument_sanitization() {
        // Test safe argument
        assert!(sanitize_argument("safe_argument").is_ok());

        // Test dangerous arguments
        assert!(sanitize_argument("dangerous;rm").is_err());
        assert!(sanitize_argument("unsafe|pipe").is_err());
        assert!(sanitize_argument("bad`tick`").is_err());
    }

    #[test]
    fn test_env_sanitization() {
        // Test safe environment variables
        assert!(sanitize_env_key("SAFE_KEY").is_ok());
        assert!(sanitize_env_value("safe_value").is_ok());

        // Test dangerous environment variables
        assert!(sanitize_env_key("UNSAFE=KEY").is_err());
        assert!(sanitize_env_value("unsafe;value").is_err());
    }
}
