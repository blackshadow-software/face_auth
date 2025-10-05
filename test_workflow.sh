#!/bin/bash

echo "=========================================="
echo "Face Authentication Test Workflow"
echo "=========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}Step 1: Checking directories${NC}"
mkdir -p generated source captured_images
echo "✓ Created/verified: generated/, source/, captured_images/"
echo ""

echo -e "${BLUE}Step 2: System Status${NC}"
echo "Generated users:"
ls -1 generated/*.json 2>/dev/null | wc -l | xargs echo "  Files in generated/:"
ls generated/*.json 2>/dev/null | sed 's/^/  - /' || echo "  (none)"
echo ""

echo "Source users (for authentication):"
ls -1 source/*.json 2>/dev/null | wc -l | xargs echo "  Files in source/:"
ls source/*.json 2>/dev/null | sed 's/^/  - /' || echo "  (none)"
echo ""

echo -e "${YELLOW}=========================================="
echo "Quick Commands:"
echo "=========================================="
echo ""
echo "Register a user:"
echo "  ./face_auth_env/bin/python python_face_auth_simple.py --mode register --user USERNAME --samples 3"
echo ""
echo "Copy user to source (enable auth):"
echo "  cp generated/USERNAME.json source/"
echo ""
echo "Test authentication:"
echo "  ./face_auth_env/bin/python python_face_auth_simple.py --mode auth --tolerance 0.6"
echo ""
echo "Run Rust application:"
echo "  cargo run"
echo ""
echo "Copy all users to source:"
echo "  cp generated/*.json source/"
echo -e "${NC}"
