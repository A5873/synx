#!/usr/bin/env bash
# 
# Example of a valid shell script with proper error handling
# This file should pass validation with Synx
#

# Exit on error, undefined variables, and pipe failures
set -euo pipefail

# Define constants
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly CONFIG_FILE="${SCRIPT_DIR}/config.json"
readonly LOG_FILE="/tmp/backup-$(date +%Y%m%d).log"
readonly BACKUP_DIR="/backup"

# Usage information
usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS] [DIRECTORY]

Performs a backup of the specified directory or the current directory if none is provided.

Options:
  -h, --help         Show this help message and exit
  -v, --verbose      Enable verbose output
  -c, --config FILE  Use a custom config file
  --dry-run          Show what would be done without making changes

Example:
  $(basename "$0") --verbose /path/to/directory

EOF
    exit 1
}

# Initialize variables
verbose=0
dry_run=0
custom_config=""
backup_source="$PWD"

# Log function
log() {
    local level="$1"
    local message="$2"
    local timestamp
    timestamp="$(date +"%Y-%m-%d %H:%M:%S")"
    
    # Print to stdout if verbose or error level
    if [[ "$verbose" -eq 1 ]] || [[ "$level" == "ERROR" ]]; then
        echo "[$timestamp] [$level] $message"
    fi
    
    # Always log to file
    echo "[$timestamp] [$level] $message" >> "$LOG_FILE"
}

# Error handler
error_handler() {
    local line="$1"
    local cmd="$2"
    local code="$3"
    log "ERROR" "Command '$cmd' failed with exit code $code on line $line"
    exit "$code"
}

# Set error handler
trap 'error_handler ${LINENO} "$BASH_COMMAND" $?' ERR

# Check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Create directory if it doesn't exist
create_dir_if_not_exists() {
    local dir="$1"
    if [[ ! -d "$dir" ]]; then
        if [[ "$dry_run" -eq 1 ]]; then
            log "INFO" "Would create directory: $dir"
        else
            mkdir -p "$dir"
            log "INFO" "Created directory: $dir"
        fi
    fi
}

# Parse command line arguments
# shellcheck disable=SC2220  # We handle missing cases with *)
while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help)
            usage
            ;;
        -v|--verbose)
            verbose=1
            shift
            ;;
        --dry-run)
            dry_run=1
            shift
            ;;
        -c|--config)
            if [[ -z "$2" || "$2" == -* ]]; then
                log "ERROR" "The $1 option requires an argument"
                usage
            fi
            custom_config="$2"
            shift 2
            ;;
        -*)
            log "ERROR" "Unknown option: $1"
            usage
            ;;
        *)
            backup_source="$1"
            shift
            ;;
    esac
done

# Validate the source directory
if [[ ! -d "$backup_source" ]]; then
    log "ERROR" "Directory does not exist: $backup_source"
    exit 1
fi

# Set config file path
if [[ -n "$custom_config" ]]; then
    if [[ ! -f "$custom_config" ]]; then
        log "ERROR" "Config file does not exist: $custom_config"
        exit 1
    fi
    config="$custom_config"
else
    config="$CONFIG_FILE"
fi

# Check required commands
for cmd in rsync jq tar; do
    if ! command_exists "$cmd"; then
        log "ERROR" "Required command '$cmd' not found"
        exit 1
    fi
done

# Backup function
perform_backup() {
    local source_dir="$1"
    local target_dir="$2"
    local timestamp
    timestamp="$(date +"%Y%m%d_%H%M%S")"
    local backup_file="${target_dir}/backup_${timestamp}.tar.gz"
    
    log "INFO" "Starting backup of $source_dir to $backup_file"
    
    create_dir_if_not_exists "$target_dir"
    
    if [[ "$dry_run" -eq 1 ]]; then
        log "INFO" "Would backup $source_dir to $backup_file"
    else
        # Perform the backup with error checking
        if tar -czf "$backup_file" -C "$(dirname "$source_dir")" "$(basename "$source_dir")"; then
            log "INFO" "Backup completed successfully: $backup_file"
            log "INFO" "Backup size: $(du -h "$backup_file" | cut -f1)"
            return 0
        else
            log "ERROR" "Backup failed"
            return 1
        fi
    fi
}

# Main script execution
log "INFO" "Starting backup script"
log "INFO" "Source directory: $backup_source"
log "INFO" "Using config file: $config"

# Create backup directory
create_dir_if_not_exists "$BACKUP_DIR"

# Perform backup
if perform_backup "$backup_source" "$BACKUP_DIR"; then
    log "INFO" "Backup process completed successfully"
else
    log "ERROR" "Backup process failed"
    exit 1
fi

log "INFO" "Backup script completed"
exit 0

