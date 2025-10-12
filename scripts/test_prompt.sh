#!/bin/bash

# Test the Zush prompt in different scenarios

echo "========================================="
echo "Zush Prompt Test Suite"
echo "========================================="
echo ""

PROMPT_BIN="./target/release/zush-prompt"

echo "1. Basic prompt (user in directory):"
$PROMPT_BIN --format raw prompt --context '{"user":"alice","pwd":"~/projects","exit_code":0}'
echo ""

echo "2. Prompt with git branch:"
$PROMPT_BIN --format raw prompt --context '{"user":"alice","pwd":"~/projects","git_branch":"main","exit_code":0}'
echo ""

echo "3. Prompt with error status:"
$PROMPT_BIN --format raw prompt --context '{"user":"alice","pwd":"~/projects","git_branch":"feature","exit_code":1}'
echo ""

echo "4. SSH session prompt:"
$PROMPT_BIN --format raw prompt --context '{"user":"alice","host":"server","pwd":"/var/www","ssh":"true","exit_code":0}'
echo ""

echo "5. Root user prompt:"
$PROMPT_BIN --format raw prompt --context '{"user":"root","pwd":"/etc","exit_code":0}'
echo ""

echo "6. Prompt with jobs:"
$PROMPT_BIN --format raw prompt --context '{"user":"alice","pwd":"~/work","jobs":"3","exit_code":0}'
echo ""

echo "7. Transient prompt:"
$PROMPT_BIN --format raw --template transient prompt --context '{"time":"14:23:05"}'
echo ""

echo "8. Right prompt with execution time:"
$PROMPT_BIN --format raw --template right prompt --context '{"time":"14:23:05","execution_time":3.5}'
echo ""

echo "========================================="
echo "Configuration example:"
echo "========================================="
$PROMPT_BIN config | head -20