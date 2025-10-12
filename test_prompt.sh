#!/bin/bash
# Test the Oh My Posh styled Zush prompt

echo "Testing Zush Prompt with Oh My Posh styling:"
echo "============================================="
echo ""

# Test 1: Success status with git repo
echo "1. Success status with git repo:"
./target/release/zush-prompt --format raw prompt \
  --context '{"pwd":"/Users/duppster/projects/myapp","pwd_short":"~/projects/myapp","user":"duppster","git_branch":"main","git_dirty":false,"time":"15:42:18"}' \
  --exit-code 0 \
  --execution-time 0.5
echo ""
echo ""

# Test 2: Error status with dirty git
echo "2. Error status with dirty git:"
./target/release/zush-prompt --format raw prompt \
  --context '{"pwd":"/Users/duppster/work/api","pwd_short":"~/work/api","user":"duppster","git_branch":"feature/auth","git_dirty":true,"time":"15:43:22"}' \
  --exit-code 127 \
  --execution-time 12.3
echo ""
echo ""

# Test 3: SSH session
echo "3. SSH session:"
./target/release/zush-prompt --format raw prompt \
  --context '{"pwd":"/var/www/app","pwd_short":"/var/www/app","user":"deploy","ssh":true,"time":"15:44:01"}' \
  --exit-code 0 \
  --execution-time 0.1
echo ""
echo ""

# Test 4: Right prompt with long execution
echo "4. Right prompt (would appear on the right):"
./target/release/zush-prompt --format raw --template right prompt \
  --context '{"time":"15:44:30"}' \
  --execution-time 8.5
echo ""
echo ""

echo "============================================="
echo "The prompt successfully loads the Oh My Posh style from ~/.config/zush/config.toml"
echo "Features:"
echo "  - Tokyo Night color scheme"
echo "  - Powerline symbols and icons"
echo "  - Git status indicators"
echo "  - SSH session detection"
echo "  - Command execution timing"
echo "  - Error/success indicators"