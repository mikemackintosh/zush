#!/usr/bin/env bash

# Helper script for creating conventional commits
# Usage: ./scripts/commit.sh

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}╔══════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║${NC}           ${MAGENTA}Conventional Commit Helper${NC}               ${CYAN}║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════════════════╝${NC}"
echo ""

# Check if there are staged changes
if ! git diff --cached --quiet; then
    echo -e "${GREEN}✓${NC} Found staged changes"
else
    echo -e "${RED}✗${NC} No staged changes found"
    echo -e "${YELLOW}Run 'git add' to stage your changes first${NC}"
    exit 1
fi

echo ""
echo -e "${BLUE}Select commit type:${NC}"
echo "1) feat     - New feature (minor version bump)"
echo "2) fix      - Bug fix (patch version bump)"
echo "3) docs     - Documentation only"
echo "4) style    - Code style/formatting"
echo "5) refactor - Code refactoring (patch bump)"
echo "6) perf     - Performance improvement (patch bump)"
echo "7) test     - Adding tests"
echo "8) build    - Build system changes"
echo "9) ci       - CI configuration"
echo "10) chore   - Maintenance tasks"
echo ""

read -p "Enter type (1-10): " type_choice

case $type_choice in
    1) type="feat" ;;
    2) type="fix" ;;
    3) type="docs" ;;
    4) type="style" ;;
    5) type="refactor" ;;
    6) type="perf" ;;
    7) type="test" ;;
    8) type="build" ;;
    9) type="ci" ;;
    10) type="chore" ;;
    *)
        echo -e "${RED}Invalid choice${NC}"
        exit 1
        ;;
esac

echo ""
read -p "Enter scope (e.g., themes, template, git) [optional]: " scope

echo ""
read -p "Enter short description: " subject

if [ -z "$subject" ]; then
    echo -e "${RED}Description is required${NC}"
    exit 1
fi

echo ""
read -p "Enter detailed description [optional, press Enter to skip]: " body

echo ""
read -p "Is this a breaking change? (y/N): " is_breaking

breaking_suffix=""
breaking_footer=""
if [[ "$is_breaking" =~ ^[Yy]$ ]]; then
    breaking_suffix="!"
    read -p "Describe the breaking change: " breaking_description
    breaking_footer="\n\nBREAKING CHANGE: $breaking_description"
fi

# Build commit message
commit_msg="$type"
if [ -n "$scope" ]; then
    commit_msg="$commit_msg($scope)"
fi
commit_msg="$commit_msg$breaking_suffix: $subject"

if [ -n "$body" ]; then
    commit_msg="$commit_msg\n\n$body"
fi

if [ -n "$breaking_footer" ]; then
    commit_msg="$commit_msg$breaking_footer"
fi

echo ""
echo -e "${CYAN}══════════════════════════════════════════════════════${NC}"
echo -e "${YELLOW}Commit message:${NC}"
echo -e "${CYAN}══════════════════════════════════════════════════════${NC}"
echo -e "$commit_msg"
echo -e "${CYAN}══════════════════════════════════════════════════════${NC}"
echo ""

read -p "Create this commit? (Y/n): " confirm

if [[ ! "$confirm" =~ ^[Nn]$ ]]; then
    echo -e "$commit_msg" | git commit -F -
    echo ""
    echo -e "${GREEN}✓ Commit created successfully!${NC}"

    # Show what will happen on push to main
    if [[ "$type" == "feat" ]]; then
        echo -e "${BLUE}ℹ${NC} This will trigger a ${YELLOW}minor${NC} version bump (e.g., 0.1.0 → 0.2.0)"
    elif [[ "$type" == "fix" || "$type" == "refactor" || "$type" == "perf" ]]; then
        echo -e "${BLUE}ℹ${NC} This will trigger a ${YELLOW}patch${NC} version bump (e.g., 0.1.0 → 0.1.1)"
    else
        echo -e "${BLUE}ℹ${NC} This will ${YELLOW}not${NC} trigger a version bump"
    fi

    if [[ -n "$breaking_footer" ]]; then
        echo -e "${BLUE}ℹ${NC} Breaking change detected: This will trigger a ${RED}major${NC} version bump (e.g., 0.1.0 → 1.0.0)"
    fi
else
    echo -e "${YELLOW}Commit cancelled${NC}"
    exit 0
fi
