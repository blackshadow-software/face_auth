#!/bin/bash

echo "🔐 Face Authentication System - Testing Guide"
echo "============================================="
echo

echo "Current working directory: $(pwd)"
echo "Available files:"
ls -la | grep -E "(face_auth|python_face_auth|setup_python)"
echo

echo "🎯 Testing Options:"
echo "1. Test Rust Implementation (Fast, 66% accuracy)"
echo "2. Setup Python Environment for High Accuracy"
echo "3. Test Python Implementation (99% accuracy)"
echo "4. Test Hybrid Interface"
echo "5. Check Python Environment Status"
echo

read -p "Choose test option (1-5): " choice

case $choice in
    1)
        echo
        echo "🦀 Testing Rust Implementation..."
        echo "This will open the Rust face authentication system."
        echo "Choose option 1 to register, then option 2 to authenticate."
        echo
        read -p "Press ENTER to continue..."
        ./target/release/face_auth
        ;;
    2)
        echo
        echo "🐍 Setting up Python environment..."
        echo "This will install face_recognition and other dependencies."
        echo
        ./setup_python_env.sh
        ;;
    3)
        echo
        echo "🐍 Testing Python Implementation..."
        if [ ! -d "face_auth_env" ]; then
            echo "❌ Python environment not found. Run option 2 first."
            exit 1
        fi

        echo "Activating Python environment..."
        source face_auth_env/bin/activate

        echo
        echo "📝 Registering user 'test' with 3 samples..."
        python3 python_face_auth.py --mode register --user test --samples 3

        echo
        echo "🔍 Testing authentication..."
        python3 python_face_auth.py --mode auth
        ;;
    4)
        echo
        echo "🔗 Testing Hybrid Interface..."
        echo "This opens the Rust interface with Python backend options."
        echo "Choose option 3 (Python Register) or 4 (Python Auth) for high accuracy."
        echo
        read -p "Press ENTER to continue..."
        ./target/release/face_auth
        ;;
    5)
        echo
        echo "🔍 Checking Python Environment..."
        if [ -d "face_auth_env" ]; then
            echo "✅ Python environment exists"
            source face_auth_env/bin/activate
            python3 -c "
try:
    import face_recognition
    import cv2
    print('✅ face_recognition library: INSTALLED')
    print('✅ OpenCV library: INSTALLED')
    print('✅ Python environment: READY')
    print('🎉 High-accuracy face authentication available!')
except ImportError as e:
    print(f'❌ Missing dependency: {e}')
    print('💡 Run: ./setup_python_env.sh')
"
        else
            echo "❌ Python environment not found"
            echo "💡 Run option 2 to set up the environment"
        fi
        ;;
    *)
        echo "❌ Invalid choice"
        ;;
esac

echo
echo "📚 Quick Reference:"
echo "• Rust (Fast): ./target/release/face_auth → options 1,2"
echo "• Python (Accurate): python3 python_face_auth.py --mode register/auth"
echo "• Hybrid: ./target/release/face_auth → options 3,4"
echo "• Setup Python: ./setup_python_env.sh"