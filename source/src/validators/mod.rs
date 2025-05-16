use anyhow::{Result, anyhow};
use std::path::Path;
use std::process::Command;
use std::str;
use tempfile;

pub struct ValidationOptions {
    pub strict: bool,
    pub verbose: bool,
}

pub fn validate_file(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let file_type = detect_file_type(file_path)?;
    let validator = get_validator_for_type(&file_type);
    validator(file_path, options)
}

fn detect_file_type(file_path: &Path) -> Result<String> {
    if let Some(ext) = file_path.extension() {
        if let Some(ext_str) = ext.to_str() {
            return Ok(ext_str.to_lowercase());
        }
    }
    let mime = tree_magic_mini::from_filepath(file_path)
        .ok_or_else(|| anyhow!("Failed to detect file type"))?;
    Ok(mime.split("/").last().unwrap_or("unknown").to_string())
}

fn get_validator_for_type(file_type: &str) -> fn(&Path, &ValidationOptions) -> Result<bool> {
    match file_type {
        "rs" => validate_rust,
        "cpp" | "cxx" | "cc" => validate_cpp,
        "c" => validate_c,
        "cs" => validate_csharp,
        "py" | "python" => validate_python,
        "js" | "javascript" => validate_javascript,
        "java" => validate_java,
        "go" => validate_go,
        "ts" | "tsx" => validate_typescript,
        "json" => validate_json,
        "yaml" | "yml" => validate_yaml,
        "html" | "htm" => validate_html,
        "css" => validate_css,
        "sh" | "bash" => validate_shell,
        "dockerfile" => validate_dockerfile,
        _ => validate_unknown,
    }
}

fn validate_rust(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let temp_dir = tempfile::Builder::new()
                    .prefix("synx-rust-check")
                    .tempdir()?;
    let output_path = temp_dir.path().join("output");

    let mut cmd = Command::new("rustc");
    cmd.arg("--edition=2021")
       .arg("--crate-type=lib")
       .arg("--error-format=short")
       .arg("-A").arg("dead_code")
       .arg("-o").arg(&output_path)
       .arg(file_path);

    if options.strict {
        cmd.arg("-D").arg("warnings");
    }

    let output = cmd.output()?;
    let success = output.status.success();
    
    if !success && options.verbose {
        eprintln!("Rust validation errors:");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    Ok(success)
}

fn validate_cpp(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = Command::new("g++");
    cmd.arg("-fsyntax-only")
       .arg("-pedantic")
       .arg("-Wall");

    if options.strict {
        cmd.arg("-Werror");
    }

    cmd.arg(file_path);
    let output = cmd.output()?;
    let success = output.status.success();

    if !success && options.verbose {
        eprintln!("C++ validation errors:");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    Ok(success)
}

fn validate_java(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = Command::new("javac");
    cmd.arg("-Werror")
       .arg("-Xlint:all")
       .arg(file_path);

    let output = cmd.output()?;
    let success = output.status.success();
    
    if !success && options.verbose {
        eprintln!("Java validation errors:");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    if success && options.strict {
        let checkstyle = Command::new("checkstyle")
            .arg("-c")
            .arg("/google_checks.xml")
            .arg(file_path)
            .output();

        if let Ok(output) = checkstyle {
            if !output.status.success() && options.verbose {
                eprintln!("Java style errors:");
                eprintln!("{}", String::from_utf8_lossy(&output.stdout));
                return Ok(false);
            }
        }
    }

    Ok(success)
}

fn validate_go(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let dir = file_path.parent().unwrap_or(Path::new("."));
    
    let mut cmd = Command::new("go");
    cmd.current_dir(dir)
       .arg("vet")
       .arg(file_path);

    let output = cmd.output()?;
    if !output.status.success() {
        if options.verbose {
            eprintln!("Go validation errors:");
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
        return Ok(false);
    }

    if options.strict {
        let gofmt = Command::new("gofmt")
            .arg("-l")
            .arg(file_path)
            .output()?;

        if !gofmt.stdout.is_empty() && options.verbose {
            eprintln!("Go formatting errors detected");
            return Ok(false);
        }

        let lint = Command::new("golangci-lint")
            .current_dir(dir)
            .arg("run")
            .arg("--no-config")
            .arg("--disable-all")
            .arg("--enable=govet,golint,gofmt,goimports")
            .arg(file_path)
            .output();

        if let Ok(output) = lint {
            if !output.status.success() && options.verbose {
                eprintln!("Go lint errors:");
                eprintln!("{}", String::from_utf8_lossy(&output.stdout));
                return Ok(false);
            }
        }
    }

    Ok(true)
}

fn validate_typescript(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let tsc_check = Command::new("tsc").arg("--version").output();
    if tsc_check.is_err() {
        if options.verbose {
            eprintln!("TypeScript validation requires tsc to be installed");
            eprintln!("Install with: npm install -g typescript");
        }
        return Ok(false);
    }

    let mut cmd = Command::new("tsc");
    cmd.arg("--noEmit")
       .arg("--pretty")
       .arg("--strict");

    if options.strict {
        cmd.arg("--noImplicitAny")
           .arg("--noImplicitThis")
           .arg("--alwaysStrict")
           .arg("--strictNullChecks")
           .arg("--strictFunctionTypes")
           .arg("--strictPropertyInitialization");
    }

    cmd.arg(file_path);
    let output = cmd.output()?;
    let success = output.status.success();

    if !success && options.verbose {
        eprintln!("TypeScript validation errors:");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    if success && options.strict {
        let eslint = Command::new("eslint")
            .arg("--parser")
            .arg("@typescript-eslint/parser")
            .arg("--plugin")
            .arg("@typescript-eslint")
            .arg(file_path)
            .output();

        if let Ok(output) = eslint {
            if !output.status.success() && options.verbose {
                eprintln!("TypeScript lint errors:");
                eprintln!("{}", String::from_utf8_lossy(&output.stdout));
                return Ok(false);
            }
        }
    }

    Ok(success)
}

fn validate_json(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let output = Command::new("jq")
        .arg(".")
        .arg(file_path)
        .output()?;

    let success = output.status.success();
    if !success && options.verbose {
        eprintln!("JSON validation errors:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(success)
}

fn validate_yaml(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = Command::new("yamllint");
    
    if options.strict {
        cmd.arg("-d").arg("{extends: default, rules: {line-length: disable}}");
    } else {
        cmd.arg("-d").arg("{extends: relaxed, rules: {line-length: disable}}");
    }
    
    cmd.arg(file_path);
    let output = cmd.output()?;

    let success = output.status.success();
    if options.verbose && !success {
        eprintln!("YAML validation errors:");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
        if !output.stdout.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stdout));
        }
    }

    Ok(success)
}

fn validate_html(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = Command::new("tidy");
    cmd.arg("-q")
       .arg("-e")
       .arg("--show-warnings").arg(if options.strict { "yes" } else { "no" })
       .arg("--show-errors").arg("1")
       .arg(file_path);

    let output = cmd.output()?;
    let success = output.status.success();
    
    if !success && options.verbose {
        eprintln!("HTML validation errors:");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    Ok(success)
}

fn validate_css(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let output = Command::new("csslint")
        .arg("--format=compact")
        .arg(file_path)
        .output()?;

    let success = output.status.success();
    if !success && options.verbose {
        eprintln!("CSS validation errors:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(success)
}

fn validate_shell(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = Command::new("shellcheck");
    
    if !options.strict {
        cmd.arg("--severity=error");
    }
    
    cmd.arg(file_path);
    let output = cmd.output()?;

    let success = output.status.success();
    if !success && options.verbose {
        eprintln!("Shell script errors:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(success)
}

fn validate_dockerfile(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    let mut cmd = Command::new("hadolint");
    
    if !options.strict {
        cmd.arg("--failure-threshold=error");
    }
    
    cmd.arg(file_path);
    let output = cmd.output()?;

    let success = output.status.success();
    if !success && options.verbose {
        eprintln!("Dockerfile validation errors:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(success)
}

fn validate_unknown(file_path: &Path, options: &ValidationOptions) -> Result<bool> {
    if options.verbose {
        eprintln!("No validator available for file: {}", file_path.display());
    }
    Ok(!options.strict)
}
