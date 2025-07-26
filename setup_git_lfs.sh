#!/bin/bash

# Git LFS and Repository Optimization Script for zeta-reticula
# This script configures Git LFS and optimizes the repository for better compression

set -e

echo "🚀 Starting Git LFS and Repository Optimization..."

# Check if Git LFS is installed
if ! command -v git-lfs &> /dev/null; then
    echo "❌ Git LFS is not installed. Please install it first:"
    echo "   - macOS: brew install git-lfs"
    echo "   - Linux: sudo apt-get install git-lfs"
    echo "   - Windows: winget install --id GitHub.GitLFS"
    exit 1
fi

# Initialize Git LFS if not already initialized
if ! git lfs env | grep -q "git config filter.lfs"; then
    echo "🔧 Initializing Git LFS..."
    git lfs install
else
    echo "✅ Git LFS already initialized"
fi

# Configure Git for better compression
echo "🔧 Configuring Git for optimal compression..."
git config --global core.compression 9
git config --global core.loosecompression 9
git config --global pack.depth 50
git config --global pack.windowMemory 1g

echo "📝 Creating .gitattributes with LFS rules..."

# Create or update .gitattributes with LFS rules
cat > .gitattributes << 'EOL'
# Text files - use Git's built-in diff and merge
*.rs text diff=rust
*.toml text
*.md text
*.yaml text
*.yml text
*.json text
*.html text
*.css text
*.js text
*.ts text
*.sh text eol=lf
*.py text

# Binary files - use Git LFS
*.bin filter=lfs diff=lfs merge=lfs -text
*.pt filter=lfs diff=lfs merge=lfs -text
*.pth filter=lfs diff=lfs merge=lfs -text
*.h5 filter=lfs diff=lfs merge=lfs -text
*.onnx filter=lfs diff=lfs merge=lfs -text
*.gguf filter=lfs diff=lfs merge=lfs -text
*.safetensors filter=lfs diff=lfs merge=lfs -text
*.zip filter=lfs diff=lfs merge=lfs -text
*.tar.gz filter=lfs diff=lfs merge=lfs -text
*.tar filter=lfs diff=lfs merge=lfs -text
*.7z filter=lfs diff=lfs merge=lfs -text
*.dat filter=lfs diff=lfs merge=lfs -text
*.pkl filter=lfs diff=lfs merge=lfs -text
*.npz filter=lfs diff=lfs merge=lfs -text
*.npy filter=lfs diff=lfs merge=lfs -text
*.tflite filter=lfs diff=lfs merge=lfs -text
*.pb filter=lfs diff=lfs merge=lfs -text
*.pbtxt filter=lfs diff=lfs merge=lfs -text
*.ot filter=lfs diff=lfs merge=lfs -text
*.msgpack filter=lfs diff=lfs merge=lfs -text
*.weights filter=lfs diff=lfs merge=lfs -text
*.index filter=lfs diff=lfs merge=lfs -text
*.data-00000-of-00001 filter=lfs diff=lfs merge=lfs -text
*.ckpt filter=lfs diff=lfs merge=lfs -text

# Exclude unnecessary files
/target/ export-ignore
*.log export-ignore
*.tmp export-ignore
*.bak export-ignore
*.swp export-ignore
*.swo export-ignore
.DS_Store export-ignore
.idea/ export-ignore
.vscode/ export-ignore
__pycache__/ export-ignore
*.py[cod] export-ignore
*$py.class export-ignore
EOL

echo "📊 Checking for large files in history..."
LARGE_FILES=$(find . -type f -size +10M -not -path "./.git/*" -not -path "./target/*" -not -path "./venv/*")

if [ -n "$LARGE_FILES" ]; then
    echo "⚠️  Found large files that should be in LFS:"
    echo "$LARGE_FILES" | while read -r file; do
        size=$(du -h "$file" | cut -f1)
        echo "   - $file ($size)"
    done
    
    echo -e "\n🚀 To migrate existing files to LFS, run the following commands:"
    echo "   git lfs migrate import --include="*.{bin,pt,pth,h5,onnx,gguf,safetensors,zip,tar.gz,tar,7z,dat,pkl,npz,npy,tflite,pb,pbtxt,ot,msgpack,weights,index,data-00000-of-00001,ckpt}" --everything"
    echo "   git push origin --force"
else
    echo "✅ No large files found that need to be migrated to LFS"
fi

# Optimize repository
echo "🔄 Optimizing repository..."

echo "🧹 Running git maintenance..."
git reflog expire --expire=now --all
git gc --prune=now --aggressive
git repack -adf --window=250 --depth=250

# Create a pre-commit hook to prevent large files
echo "🔧 Setting up pre-commit hook to prevent large files..."
mkdir -p .git/hooks
cat > .git/hooks/pre-commit << 'EOL'
#!/bin/bash

# Prevent large files from being committed
MAX_FILE_SIZE=10485760  # 10MB
LARGE_FILES=$(find . -type f -size +${MAX_FILE_SIZE}c -not -path "./.git/*" -not -path "./target/*" -not -path "./venv/*")

if [ -n "$LARGE_FILES" ]; then
    echo "❌ Error: The following files exceed 10MB and should be tracked with Git LFS:"
    echo "$LARGE_FILES" | while read -r file; do
        size=$(du -h "$file" | cut -f1)
        echo "   - $file ($size)"
    done
    echo "\nPlease add these files to Git LFS using:"
    echo "   git lfs track '*.ext'  # Replace with the appropriate extension"
    echo "   git add .gitattributes"
    echo "   git add file.ext"
    exit 1
fi
exit 0
EOL

chmod +x .git/hooks/pre-commit

echo "✅ Git LFS and repository optimization complete!"
echo "\n📝 Next steps:"
echo "1. Review the changes with 'git status'"
echo "2. Add and commit the .gitattributes file:"
echo "   git add .gitattributes"
echo "   git commit -m 'Configure Git LFS and optimize repository'"
echo "3. If you have existing large files, migrate them to LFS:"
echo "   git lfs migrate import --include="*.{bin,pt,pth,h5,onnx,gguf}" --everything"
echo "4. Push your changes:"
echo "   git push origin main"
echo "\n💡 Tip: Run 'git lfs ls-files' to see files being tracked by Git LFS"
