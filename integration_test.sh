#!/bin/bash

# Integration test for face authentication system
# Tests the complete workflow without camera

set -e  # Exit on error

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}"
echo "============================================================"
echo "  FACE AUTHENTICATION SYSTEM - INTEGRATION TEST"
echo "============================================================"
echo -e "${NC}"

# Test 1: Directory Creation
echo -e "${YELLOW}Test 1: Directory Setup${NC}"
echo "Creating required directories..."
mkdir -p generated source captured_images

if [ -d "generated" ] && [ -d "source" ]; then
    echo -e "${GREEN}✅ Directories created successfully${NC}"
else
    echo -e "${RED}❌ Failed to create directories${NC}"
    exit 1
fi
echo ""

# Test 2: Mock Data Generation
echo -e "${YELLOW}Test 2: Mock User Registration${NC}"
echo "Generating mock user data..."
./face_auth_env/bin/python test_mock_data.py > /dev/null 2>&1

GENERATED_COUNT=$(ls -1 generated/*.json 2>/dev/null | wc -l | xargs)
if [ "$GENERATED_COUNT" -ge 3 ]; then
    echo -e "${GREEN}✅ Generated $GENERATED_COUNT mock users${NC}"
    ls generated/*.json | sed 's/^/  - /'
else
    echo -e "${RED}❌ Failed to generate mock users${NC}"
    exit 1
fi
echo ""

# Test 3: Source Directory Verification
echo -e "${YELLOW}Test 3: Source Directory Population${NC}"
SOURCE_COUNT=$(ls -1 source/*.json 2>/dev/null | wc -l | xargs)
if [ "$SOURCE_COUNT" -ge 2 ]; then
    echo -e "${GREEN}✅ Source directory has $SOURCE_COUNT users${NC}"
    ls source/*.json | sed 's/^/  - /'
else
    echo -e "${RED}❌ Source directory not populated correctly${NC}"
    exit 1
fi
echo ""

# Test 4: File Structure Validation
echo -e "${YELLOW}Test 4: File Structure Validation${NC}"
echo "Validating JSON structure..."
./face_auth_env/bin/python test_auth_loading.py > /dev/null 2>&1

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ All files have valid structure${NC}"
else
    echo -e "${RED}❌ File structure validation failed${NC}"
    exit 1
fi
echo ""

# Test 5: User Management Workflow
echo -e "${YELLOW}Test 5: User Management Workflow${NC}"

# Test 5a: Add new user to source
echo "  Testing: Add user to source directory..."
if [ -f "generated/charlie.json" ]; then
    cp generated/charlie.json source/
    if [ -f "source/charlie.json" ]; then
        echo -e "  ${GREEN}✅ User added to source${NC}"
    else
        echo -e "  ${RED}❌ Failed to add user to source${NC}"
        exit 1
    fi
fi

# Test 5b: Verify user count increased
SOURCE_COUNT_AFTER=$(ls -1 source/*.json 2>/dev/null | wc -l | xargs)
if [ "$SOURCE_COUNT_AFTER" -gt "$SOURCE_COUNT" ]; then
    echo -e "  ${GREEN}✅ User count increased: $SOURCE_COUNT → $SOURCE_COUNT_AFTER${NC}"
else
    echo -e "  ${RED}❌ User count did not increase${NC}"
    exit 1
fi

# Test 5c: Remove user from source
echo "  Testing: Remove user from source directory..."
rm source/charlie.json
if [ ! -f "source/charlie.json" ]; then
    echo -e "  ${GREEN}✅ User removed from source${NC}"
else
    echo -e "  ${RED}❌ Failed to remove user from source${NC}"
    exit 1
fi
echo ""

# Test 6: Verify generated/ is unchanged
echo -e "${YELLOW}Test 6: Generated Directory Integrity${NC}"
GENERATED_COUNT_AFTER=$(ls -1 generated/*.json 2>/dev/null | wc -l | xargs)
if [ "$GENERATED_COUNT_AFTER" -eq "$GENERATED_COUNT" ]; then
    echo -e "${GREEN}✅ Generated directory unchanged: $GENERATED_COUNT files${NC}"
else
    echo -e "${RED}❌ Generated directory was modified unexpectedly${NC}"
    exit 1
fi
echo ""

# Test 7: File Content Validation
echo -e "${YELLOW}Test 7: File Content Validation${NC}"
echo "Checking file contents..."

for file in generated/*.json; do
    # Check if file has required fields
    if ! grep -q '"user_id"' "$file"; then
        echo -e "${RED}❌ Missing user_id in $file${NC}"
        exit 1
    fi

    if ! grep -q '"face_encodings"' "$file"; then
        echo -e "${RED}❌ Missing face_encodings in $file${NC}"
        exit 1
    fi

    if ! grep -q '"enrollment_date"' "$file"; then
        echo -e "${RED}❌ Missing enrollment_date in $file${NC}"
        exit 1
    fi
done

echo -e "${GREEN}✅ All files contain required fields${NC}"
echo ""

# Test 8: System Build Test
echo -e "${YELLOW}Test 8: Rust Application Build${NC}"
echo "Building Rust application..."
if cargo build --quiet 2>&1 | grep -q "error"; then
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
else
    echo -e "${GREEN}✅ Rust application builds successfully${NC}"
fi
echo ""

# Summary
echo -e "${BLUE}"
echo "============================================================"
echo "  TEST SUMMARY"
echo "============================================================"
echo -e "${NC}"
echo -e "${GREEN}✅ All integration tests passed!${NC}"
echo ""
echo "System Status:"
echo "  - Registered users (generated/): $GENERATED_COUNT"
echo "  - Authorized users (source/): $(ls -1 source/*.json 2>/dev/null | wc -l | xargs)"
echo "  - Rust build: Success"
echo ""
echo -e "${BLUE}============================================================"
echo "  NEXT STEPS"
echo "============================================================"
echo -e "${NC}"
echo "1. Test with real camera:"
echo "   ${YELLOW}cargo run${NC}"
echo ""
echo "2. Register real users:"
echo "   ${YELLOW}./face_auth_env/bin/python python_face_auth_simple.py --mode register --user YOUR_NAME --samples 3${NC}"
echo ""
echo "3. Authorize for authentication:"
echo "   ${YELLOW}cp generated/YOUR_NAME.json source/${NC}"
echo ""
echo "4. Test authentication:"
echo "   ${YELLOW}./face_auth_env/bin/python python_face_auth_simple.py --mode auth${NC}"
echo ""
echo "5. View documentation:"
echo "   ${YELLOW}cat WORKFLOW_GUIDE.md${NC}"
echo ""
echo -e "${GREEN}Integration test completed successfully!${NC}"
