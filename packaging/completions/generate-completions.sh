#!/bin/sh
# Resolve script directory and change to it
cd "$(dirname "$0")"

echo "Generating shell completion scripts..."
# Typically: ../../target/release/rstart --generate-completions <shell>
