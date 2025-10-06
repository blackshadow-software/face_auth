#!/bin/bash

echo "🧪 Simple Plug and Play Verification"
echo "======================================"
echo ""

# Step 1: Check if venv exists
echo "1️⃣  Virtual Environment Check"
if [ -f "./face_auth_env/bin/python" ]; then
    echo "   ✅ EXISTS at ./face_auth_env/bin/python"
    PYTHON_VER=$(./face_auth_env/bin/python --version 2>&1)
    echo "   📦 Version: $PYTHON_VER"
else
    echo "   ❌ NOT FOUND (will be created on first run)"
fi
echo ""

# Step 2: Check dependencies
echo "2️⃣  Dependencies Check"
if [ -f "./face_auth_env/bin/python" ]; then
    if ./face_auth_env/bin/python -c "import face_recognition" 2>/dev/null; then
        FR_VER=$(./face_auth_env/bin/python -c "import face_recognition; print(face_recognition.__version__)" 2>&1)
        echo "   ✅ face_recognition: $FR_VER"
    else
        echo "   ❌ face_recognition: NOT INSTALLED"
    fi

    if ./face_auth_env/bin/python -c "import cv2" 2>/dev/null; then
        CV_VER=$(./face_auth_env/bin/python -c "import cv2; print(cv2.__version__)" 2>&1)
        echo "   ✅ opencv-python: $CV_VER"
    else
        echo "   ❌ opencv-python: NOT INSTALLED"
    fi

    if ./face_auth_env/bin/python -c "import numpy" 2>/dev/null; then
        echo "   ✅ numpy: INSTALLED"
    else
        echo "   ❌ numpy: NOT INSTALLED"
    fi
else
    echo "   ⏭️  SKIPPED (no venv yet)"
fi
echo ""

# Step 3: Check binary
echo "3️⃣  Binary Check"
if [ -f "./target/release/face_auth" ]; then
    SIZE=$(du -h ./target/release/face_auth | cut -f1)
    echo "   ✅ EXISTS at ./target/release/face_auth"
    echo "   📏 Size: $SIZE"
else
    echo "   ❌ NOT BUILT (run: cargo build --release)"
fi
echo ""

# Step 4: Summary
echo "════════════════════════════════════════"
echo "  SUMMARY"
echo "════════════════════════════════════════"

ALL_GOOD=true

if [ ! -f "./target/release/face_auth" ]; then
    echo "⚠️  Need to build: cargo build --release"
    ALL_GOOD=false
fi

if [ ! -f "./face_auth_env/bin/python" ]; then
    echo "ℹ️  Virtual environment will be created automatically on first run"
fi

if [ -f "./face_auth_env/bin/python" ]; then
    if ! ./face_auth_env/bin/python -c "import face_recognition" 2>/dev/null; then
        echo "⚠️  Dependencies incomplete (will auto-install on first run)"
    fi
fi

if [ "$ALL_GOOD" = true ] && [ -f "./face_auth_env/bin/python" ]; then
    if ./face_auth_env/bin/python -c "import face_recognition, cv2" 2>/dev/null; then
        echo ""
        echo "✅ READY TO USE!"
        echo ""
        echo "Run: ./target/release/face_auth"
    fi
else
    echo ""
    echo "🚀 PLUG AND PLAY MODE:"
    echo "   Just run: ./target/release/face_auth"
    echo "   Everything will auto-setup!"
fi
echo ""
