#!/bin/sh

mkdir -p .esp/ .esp/espup

# Install espup if missing
if [ ! -x ".esp/bin/bin/espup" ]; then
    cargo install --locked --root .esp/bin espup
fi

# Run espup install if export file missing
if [ ! -f ".esp/espup/export-esp.sh" ]; then
    .esp/bin/bin/espup install --export-file .esp/espup/export-esp.sh
fi

# Install missing cargo tools
for tool in ldproxy espflash cargo-espflash; do
    if [ ! -x ".esp/bin/bin/$tool" ]; then
        cargo install --locked --root .esp/bin "$tool"
    fi
done

# Check for direnv install, ask to run direnv allow if present
if ! command -v direnv >/dev/null 2>&1; then
    echo "Warning: direnv is not installed. Install it to automatically load environment variables."
    echo "See https://direnv.net/ for installation instructions."
else
    printf "Run 'direnv allow' now? [y/N] "
    read -r answer
    case $answer in
        [yY]) direnv allow ;;
        *) echo "Run 'direnv allow' when ready." ;;
    esac
fi
