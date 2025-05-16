#!/bin/bash
# This is an invalid shell script with common errors
# It should fail validation with Synx

# Missing error handling (no set -e or equivalent)

# Using undefined variables without checks
echo "Home directory: $HOME_DIR"  # Should be $HOME

# Command substitution with deprecated backticks
user=`whoami`

# Using == instead of = in test
if [ "$user" == "root" ]
then
    echo "Running as root"
fi

# Missing quotes around variables
echo Your username is $user

# Unquoted path expansion
rm -rf /path/to/directory/*

# Using cd without checking if it succeeded
cd /nonexistent/directory
echo "Current directory: $(pwd)"  # This won't run if cd fails

# Using sudo in a script without checking if it's available
sudo apt-get update

# Using echo for error messages
echo "Error: something went wrong" >&2

# Redirecting to /dev/null without redirecting stderr
command > /dev/null

# Using eval unsafely
eval "echo $USER_INPUT"  # Potential command injection

# Not checking if commands exist
rsync -avz source/ destination/

# Using [ instead of [[ for complex conditions
if [ "$var1" = "value" -a "$var2" = "value" ]; then
    echo "Both conditions are true"
fi

# Using non-POSIX extensions without checking shell type
function my_function {
    echo "This is a function"
}

# Using ls in scripts
files=`ls /tmp`

# Hard-coded paths without configuration
backup_dir=/home/user/backups

# Using cat unnecessarily (useless use of cat)
cat file.txt | grep "pattern"

# Missing double quotes for patterns
grep $pattern file.txt

# Command injection vulnerability
cmd="ls -la $user_input"
$cmd

# Not checking exit status
rm -rf important_directory
# work with the directory without checking if rm succeeded

# Using deprecated and inconsistent flags
grep -r --include="*.txt" pattern /dir

# Unsafe temp file creation
temp_file=/tmp/script_temp_$RANDOM
echo "data" > $temp_file

# Not cleaning up temp files
# Missing trap for script exit

# Missing shebang line for functions
my_other_function() {
  local x=5
  echo $x;  # Semicolon not needed
}

# Missing exit at end

