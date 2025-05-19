use std::collections::HashMap;
use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};
use super::{ToolRequirement, ToolCapabilities, PlatformReqs, PackageManager};

lazy_static! {
    pub static ref BUILTIN_REQUIREMENTS: HashMap<String, ToolRequirement> = {
        let mut map = HashMap::new();
        
        // Rust tools
        map.insert("rustc".to_string(), ToolRequirement {
            name: "rustc".to_string(),
            description: "Rust compiler".to_string(),
            package_name: "rustc".to_string(),
            version_req: Some(">=1.70.0".to_string()),
            version_cmd: vec!["--version".to_string()],
            version_pattern: r"rustc (\d+\.\d+\.\d+)".to_string(),
            required: true,
            alternatives: vec![],
            capabilities: ToolCapabilities {
                features: vec!["compile".to_string(), "check".to_string()],
                optional_features: vec!["clippy".to_string()],
                dependencies: vec!["cargo".to_string()],
            },
            platform_reqs: create_rust_platform_reqs(),
            install_instructions: "Visit https://rustup.rs for installation instructions".to_string(),
            info_url: "https://www.rust-lang.org".to_string\(\),
        });

        // C/C++ tools
        map.insert("gcc".to_string(), ToolRequirement {
            name: "gcc".to_string(),
            description: "GNU C Compiler".to_string(),
            package_name: "gcc".to_string(),
            version_req: Some(">=9.0.0".to_string()),
            version_cmd: vec!["--version".to_string()],
            version_pattern: r"gcc \(.*\) (\d+\.\d+\.\d+)".to_string(),
            required: true,
            alternatives: vec!["clang".to_string()],
            capabilities: ToolCapabilities {
                features: vec!["compile".to_string(), "link".to_string()],
                optional_features: vec!["sanitize".to_string()],
                dependencies: vec!["make".to_string()],
            },
            platform_reqs: create_gcc_platform_reqs(),
            install_instructions: "Install using your system's package manager".to_string(),
            info_url: "https://gcc.gnu.org".to_string\(\),
        });

        // Python tools
        map.insert("python3".to_string(), ToolRequirement {
            name: "python3".to_string(),
            description: "Python interpreter".to_string(),
            package_name: "python3".to_string(),
            version_req: Some(">=3.8.0".to_string()),
            version_cmd: vec!["--version".to_string()],
            version_pattern: r"Python (\d+\.\d+\.\d+)".to_string(),
            required: true,
            alternatives: vec![],
            capabilities: ToolCapabilities {
                features: vec!["interpret".to_string(), "pip".to_string()],
                optional_features: vec!["venv".to_string()],
                dependencies: vec![],
            },
            platform_reqs: create_python_platform_reqs(),
            install_instructions: "Install using your system's package manager".to_string(),
            info_url: "https://www.python.org".to_string\(\),
        });

        map.insert("mypy".to_string(), ToolRequirement {
            name: "mypy".to_string(),
            description: "Python static type checker".to_string(),
            package_name: "mypy".to_string(),
            version_req: Some(">=1.0.0".to_string()),
            version_cmd: vec!["--version".to_string()],
            version_pattern: r"mypy (\d+\.\d+\.\d+)".to_string(),
            required: true,
            alternatives: vec!["pyright".to_string()],
            capabilities: ToolCapabilities {
                features: vec!["type-check".to_string()],
                optional_features: vec!["plugins".to_string()],
                dependencies: vec!["python3".to_string()],
            },
            platform_reqs: create_python_tool_platform_reqs(),
            install_instructions: "Install using pip: pip install mypy".to_string(),
            info_url: "https://mypy.readthedocs.io".to_string\(\),
        });

        // JavaScript/TypeScript tools
        map.insert("node".to_string(), ToolRequirement {
            name: "node".to_string(),
            description: "Node.js JavaScript runtime".to_string(),
            package_name: "nodejs".to_string(),
            version_req: Some(">=18.0.0".to_string()),
            version_cmd: vec!["--version".to_string()],
            version_pattern: r"v(\d+\.\d+\.\d+)".to_string(),
            required: true,
            alternatives: vec!["deno".to_string()],
            capabilities: ToolCapabilities {
                features: vec!["execute".to_string(), "npm".to_string()],
                optional_features: vec!["yarn".to_string(), "pnpm".to_string()],
                dependencies: vec![],
            },
            platform_reqs: create_node_platform_reqs(),
            install_instructions: "Visit https://nodejs.org for installation instructions".to_string(),
            info_url: "https://nodejs.org".to_string\(\),
        });

        map.insert("eslint".to_string(), ToolRequirement {
            name: "eslint".to_string(),
            description: "JavaScript linter".to_string(),
            package_name: "eslint".to_string(),
            version_req: Some(">=8.0.0".to_string()),
            version_cmd: vec!["--version".to_string()],
            version_pattern: r"v(\d+\.\d+\.\d+)".to_string(),
            required: true,
            alternatives: vec!["standardjs".to_string()],
            capabilities: ToolCapabilities {
                features: vec!["lint".to_string(), "fix".to_string()],
                optional_features: vec!["typescript".to_string()],
                dependencies: vec!["node".to_string()],
            },
            platform_reqs: create_node_tool_platform_reqs(),
            install_instructions: "Install using npm: npm install -g eslint".to_string(),
            info_url: "https://eslint.org".to_string\(\),
        });

        // Other tools...
        // Add more tool requirements as needed

        map
    };
}

fn create_rust_platform_reqs() -> HashMap<String, PlatformReqs> {
    let mut platform_reqs = HashMap::new();
    
    // Linux requirements
    let mut linux_reqs = PlatformReqs {
        min_os_version: None,
        required_libs: vec!["gcc".to_string(), "libc6-dev".to_string()],
        required_env: HashMap::new(),
        package_info: HashMap::new(),
    };
    
    // Add package managers for Linux
    linux_reqs.package_info.insert("apt".to_string(), PackageManager {
        name: "apt".to_string(),
        install_cmd: "apt-get install -y".to_string(),
        update_cmd: Some("apt-get update".to_string()),
        packages: {
            let mut map = HashMap::new();
            map.insert("default".to_string(), "rustc".to_string());
            map
        },
    });

    platform_reqs.insert("linux".to_string(), linux_reqs);
    
    // Add other platforms as needed
    platform_reqs
}

fn create_gcc_platform_reqs() -> HashMap<String, PlatformReqs> {
    let mut platform_reqs = HashMap::new();
    
    let mut linux_reqs = PlatformReqs {
        min_os_version: None,
        required_libs: vec!["libc6-dev".to_string()],
        required_env: HashMap::new(),
        package_info: HashMap::new(),
    };
    
    linux_reqs.package_info.insert("apt".to_string(), PackageManager {
        name: "apt".to_string(),
        install_cmd: "apt-get install -y".to_string(),
        update_cmd: Some("apt-get update".to_string()),
        packages: {
            let mut map = HashMap::new();
            map.insert("default".to_string(), "gcc".to_string());
            map.insert("c++".to_string(), "g++".to_string());
            map
        },
    });

    platform_reqs.insert("linux".to_string(), linux_reqs);
    platform_reqs
}

fn create_python_platform_reqs() -> HashMap<String, PlatformReqs> {
    let mut platform_reqs = HashMap::new();
    
    let mut linux_reqs = PlatformReqs {
        min_os_version: None,
        required_libs: vec![],
        required_env: HashMap::new(),
        package_info: HashMap::new(),
    };
    
    linux_reqs.package_info.insert("apt".to_string(), PackageManager {
        name: "apt".to_string(),
        install_cmd: "apt-get install -y".to_string(),
        update_cmd: Some("apt-get update".to_string()),
        packages: {
            let mut map = HashMap::new();
            map.insert("default".to_string(), "python3".to_string());
            map.insert("dev".to_string(), "python3-dev".to_string());
            map
        },
    });

    platform_reqs.insert("linux".to_string(), linux_reqs);
    platform_reqs
}

fn create_python_tool_platform_reqs() -> HashMap<String, PlatformReqs> {
    let mut platform_reqs = HashMap::new();
    
    let linux_reqs = PlatformReqs {
        min_os_version: None,
        required_libs: vec![],
        required_env: HashMap::new(),
        package_info: HashMap::new(),
    };

    platform_reqs.insert("linux".to_string(), linux_reqs);
    platform_reqs
}

fn create_node_platform_reqs() -> HashMap<String, PlatformReqs> {
    let mut platform_reqs = HashMap::new();
    
    let mut linux_reqs = PlatformReqs {
        min_os_version: None,
        required_libs: vec![],
        required_env: HashMap::new(),
        package_info: HashMap::new(),
    };
    
    linux_reqs.package_info.insert("apt".to_string(), PackageManager {
        name: "apt".to_string(),
        install_cmd: "apt-get install -y".to_string(),
        update_cmd: Some("apt-get update".to_string()),
        packages: {
            let mut map = HashMap::new();
            map.insert("default".to_string(), "nodejs".to_string());
            map.insert("npm".to_string(), "npm".to_string());
            map
        },
    });

    platform_reqs.insert("linux".to_string(), linux_reqs);
    platform_reqs
}

fn create_node_tool_platform_reqs() -> HashMap<String, PlatformReqs> {
    let mut platform_reqs = HashMap::new();
    
    let linux_reqs = PlatformReqs {
        min_os_version: None,
        required_libs: vec![],
        required_env: HashMap::new(),
        package_info: HashMap::new(),
    };

    platform_reqs.insert("linux".to_string(), linux_reqs);
    platform_reqs
}

// Additional tool requirements
lazy_static! {
    pub static ref TOOL_CATEGORIES: HashMap<String, Vec<String>> = {
        let mut map = HashMap::new();
        
        // Language-specific tools
        map.insert("rust".to_string(), vec![
            "rustc".to_string(),
            "cargo".to_string(),
            "clippy".to_string(),
            "rustfmt".to_string()
        ]);
        
        map.insert("c".to_string(), vec![
            "gcc".to_string(),
            "gdb".to_string(),
            "make".to_string(),
            "valgrind".to_string()
        ]);
        
        map.insert("cpp".to_string(), vec![
            "g++".to_string(),
            "cmake".to_string(),
            "ninja".to_string()
        ]);
        
        map.insert("python".to_string(), vec![
            "python3".to_string(),
            "mypy".to_string(),
            "pylint".to_string(),
            "black".to_string(),
            "pytest".to_string()
        ]);
        
        map.insert("javascript".to_string(), vec![
            "node".to_string(),
            "npm".to_string(),
            "eslint".to_string(),
            "prettier".to_string()
        ]);
        
        map.insert("go".to_string(), vec![
            "go".to_string(),
            "golangci-lint".to_string(),
            "gofmt".to_string()
        ]);

        map.insert("java".to_string(), vec![
            "java".to_string(),
            "javac".to_string(),
            "maven".to_string(),
            "checkstyle".to_string()
        ]);

        // Build tools
        map.insert("build".to_string(), vec![
            "make".to_string(),
            "cmake".to_string(),
            "ninja".to_string(),
            "bazel".to_string()
        ]);

        // Code quality tools
        map.insert("quality".to_string(), vec![
            "clang-format".to_string(),
            "clang-tidy".to_string(),
            "cppcheck".to_string(),
            "shellcheck".to_string(),
            "hadolint".to_string()
        ]);

        // Version control tools
        map.insert("vcs".to_string(), vec![
            "git".to_string(),
            "git-lfs".to_string(),
            "svn".to_string()
        ]);

        map
    };
}

// Platform-specific configurations
fn create_windows_platform_reqs() -> HashMap<String, PlatformReqs> {
    let mut platform_reqs = HashMap::new();
    
    let mut windows_reqs = PlatformReqs {
        min_os_version: Some("10.0".to_string()),
        required_libs: vec![],
        required_env: HashMap::new(),
        package_info: HashMap::new(),
    };
    
    // Add Chocolatey package manager
    windows_reqs.package_info.insert("choco".to_string(), PackageManager {
        name: "choco".to_string(),
        install_cmd: "choco install -y".to_string(),
        update_cmd: Some("choco upgrade all -y".to_string()),
        packages: HashMap::new(),
    });

    // Add Scoop package manager
    windows_reqs.package_info.insert("scoop".to_string(), PackageManager {
        name: "scoop".to_string(),
        install_cmd: "scoop install".to_string(),
        update_cmd: Some("scoop update *".to_string()),
        packages: HashMap::new(),
    });

    platform_reqs.insert("windows".to_string(), windows_reqs);
    platform_reqs
}

fn create_macos_platform_reqs() -> HashMap<String, PlatformReqs> {
    let mut platform_reqs = HashMap::new();
    
    let mut macos_reqs = PlatformReqs {
        min_os_version: Some("10.15".to_string()),
        required_libs: vec![],
        required_env: HashMap::new(),
        package_info: HashMap::new(),
    };
    
    // Add Homebrew package manager
    macos_reqs.package_info.insert("brew".to_string(), PackageManager {
        name: "brew".to_string(),
        install_cmd: "brew install".to_string(),
        update_cmd: Some("brew update".to_string()),
        packages: HashMap::new(),
    });

    // Add MacPorts package manager
    macos_reqs.package_info.insert("port".to_string(), PackageManager {
        name: "port".to_string(),
        install_cmd: "port install".to_string(),
        update_cmd: Some("port selfupdate".to_string()),
        packages: HashMap::new(),
    });

    platform_reqs.insert("macos".to_string(), macos_reqs);
    platform_reqs
}

// Helper function to create package info for multiple package managers
fn create_multi_platform_package_info() -> HashMap<String, PackageManager> {
    let mut package_info = HashMap::new();
    
    // Linux package managers
    package_info.insert("apt".to_string(), PackageManager {
        name: "apt".to_string(),
        install_cmd: "apt-get install -y".to_string(),
        update_cmd: Some("apt-get update".to_string()),
        packages: HashMap::new(),
    });

    package_info.insert("dnf".to_string(), PackageManager {
        name: "dnf".to_string(),
        install_cmd: "dnf install -y".to_string(),
        update_cmd: Some("dnf check-update".to_string()),
        packages: HashMap::new(),
    });

    package_info.insert("pacman".to_string(), PackageManager {
        name: "pacman".to_string(),
        install_cmd: "pacman -S --noconfirm".to_string(),
        update_cmd: Some("pacman -Sy".to_string()),
        packages: HashMap::new(),
    });

    // Windows package managers
    package_info.insert("choco".to_string(), PackageManager {
        name: "choco".to_string(),
        install_cmd: "choco install -y".to_string(),
        update_cmd: Some("choco upgrade all -y".to_string()),
        packages: HashMap::new(),
    });

    package_info.insert("scoop".to_string(), PackageManager {
        name: "scoop".to_string(),
        install_cmd: "scoop install".to_string(),
        update_cmd: Some("scoop update *".to_string()),
        packages: HashMap::new(),
    });

    // macOS package managers
    package_info.insert("brew".to_string(), PackageManager {
        name: "brew".to_string(),
        install_cmd: "brew install".to_string(),
        update_cmd: Some("brew update".to_string()),
        packages: HashMap::new(),
    });

    package_info.insert("port".to_string(), PackageManager {
        name: "port".to_string(),
        install_cmd: "port install".to_string(),
        update_cmd: Some("port selfupdate".to_string()),
        packages: HashMap::new(),
    });

    package_info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_categories() {
        let categories = TOOL_CATEGORIES.get("rust").unwrap();
        assert!(categories.contains(&"rustc".to_string()));
        assert!(categories.contains(&"cargo".to_string()));
        assert!(categories.contains(&"clippy".to_string()));
    }

    #[test]
    fn test_package_managers() {
        let package_info = create_multi_platform_package_info();
        
        // Test Linux package managers
        assert!(package_info.contains_key("apt"));
        assert!(package_info.contains_key("dnf"));
        assert!(package_info.contains_key("pacman"));
        
        // Test Windows package managers
        assert!(package_info.contains_key("choco"));
        assert!(package_info.contains_key("scoop"));
        
        // Test macOS package managers
        assert!(package_info.contains_key("brew"));
        assert!(package_info.contains_key("port"));
    }

    #[test]
    fn test_platform_requirements() {
        let windows_reqs = create_windows_platform_reqs();
        let macos_reqs = create_macos_platform_reqs();
        
        assert!(windows_reqs.contains_key("windows"));
        assert!(macos_reqs.contains_key("macos"));
    }
}
