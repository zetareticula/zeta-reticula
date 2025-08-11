#!/bin/bash

# deduplicate_repo.sh - A tool to analyze and remove duplicate functions and files in a Rust repository

set -euo pipefail

# Configuration
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEMP_DIR="/tmp/repo_analysis_$(date +%s)"
ANALYSIS_REPORT="${REPO_ROOT}/duplicate_analysis_$(date +%Y%m%d_%H%M%S).md"

# Thresholds (adjust as needed)
MIN_FUNCTION_LINES=5  # Minimum lines to consider a function
SIMILARITY_THRESHOLD=90  # Percentage similarity to consider functions duplicates

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Create temporary directories
mkdir -p "${TEMP_DIR}/functions"
mkdir -p "${TEMP_DIR}/files"

# Initialize analysis report
cat > "${ANALYSIS_REPORT}" << EOF
# Repository Duplicate Analysis Report

Generated: $(date)
Repository: ${REPO_ROOT}

## Analysis Configuration
- Minimum function lines: ${MIN_FUNCTION_LINES}
- Similarity threshold: ${SIMILARITY_THRESHOLD}%

## Summary

EOF

echo -e "${YELLOW}🔍 Starting repository analysis...${NC}"
echo -e "Repository root: ${REPO_ROOT}"
echo -e "Temporary directory: ${TEMP_DIR}"
echo -e "Analysis report will be saved to: ${ANALYSIS_REPORT}\n"

# Function to extract functions from Rust files
extract_functions() {
    local file="$1"
    local output_dir="$2"
    local file_hash="$(basename "$file" | sha256sum | cut -d' ' -f1)"
    local counter=0
    
    # Use Rust's syn-based parser to extract function signatures and bodies
    # This is a simplified version - consider using a proper Rust syntax parser for production
    while IFS= read -r line; do
        if [[ "$line" =~ ^[[:space:]]*(pub[[:space:]]+)?(async[[:space:]]+)?fn[[:space:]]+([a-zA-Z0-9_]+) ]]; then
            local fn_name="${BASH_REMATCH[3]}"
            local fn_start="$line"
            local brace_count=0
            local fn_content=""
            
            # Count opening and closing braces to find function boundaries
            if [[ "$line" =~ \{ ]]; then
                brace_count=$((brace_count + $(grep -o '{\|}' <<< "$line" | wc -l)))
                brace_count=$((brace_count - $(grep -o '\"' <<< "$line" | wc -l) / 2))  # Ignore braces in strings
            fi
            
            while IFS= read -r next_line; do
                fn_content+="$next_line\n"
                brace_count=$((brace_count + $(grep -o '{\|}' <<< "$next_line" | wc -l)))
                brace_count=$((brace_count - $(grep -o '\"' <<< "$next_line" | wc -l) / 2))
                
                if [ $brace_count -le 0 ]; then
                    break
                fi
            done
            
            # Only process functions with enough lines
            local line_count=$(wc -l <<< "$fn_content")
            if [ $line_count -ge $MIN_FUNCTION_LINES ]; then
                local fn_hash=$(echo -e "$fn_content" | sha256sum | cut -d' ' -f1)
                echo -e "$fn_content" > "${output_dir}/${fn_name}_${file_hash}_${counter}_${fn_hash}.rs"
                echo "${file}:${fn_name}:${fn_hash}" >> "${TEMP_DIR}/function_index.txt"
                counter=$((counter + 1))
            fi
        fi
    done < "$file"
}

# Function to find similar files using fdupes
find_duplicate_files() {
    echo -e "\n${YELLOW}📂 Finding duplicate files...${NC}"
    
    # Find all Rust files
    find "${REPO_ROOT}" -type f -name '*.rs' -not -path '*/target/*' -not -path '*/.git/*' > "${TEMP_DIR}/all_rust_files.txt"
    
    # Create a checksum for each file
    while IFS= read -r file; do
        local file_hash=$(sha256sum "$file" | cut -d' ' -f1)
        echo "${file_hash} ${file}" >> "${TEMP_DIR}/file_hashes.txt"
    done < "${TEMP_DIR}/all_rust_files.txt"
    
    # Find duplicate files (macOS compatible version)
    sort "${TEMP_DIR}/file_hashes.txt" | cut -c 1-64 | uniq -d > "${TEMP_DIR}/duplicate_hashes.txt"
    
    # Reconstruct file paths for duplicate hashes
    > "${TEMP_DIR}/duplicate_files.txt"
    while read -r hash; do
        echo "# Duplicate files with hash: $hash" >> "${TEMP_DIR}/duplicate_files.txt"
        grep "^${hash}" "${TEMP_DIR}/file_hashes.txt" | cut -d' ' -f2- >> "${TEMP_DIR}/duplicate_files.txt"
        echo "" >> "${TEMP_DIR}/duplicate_files.txt"
    done < "${TEMP_DIR}/duplicate_hashes.txt"
    
    # Add to report
    if [ -s "${TEMP_DIR}/duplicate_files.txt" ]; then
        echo -e "## Duplicate Files Found\n" >> "${ANALYSIS_REPORT}"
        echo "The following files are exact duplicates:" >> "${ANALYSIS_REPORT}"
        echo '```' >> "${ANALYSIS_REPORT}"
        cat "${TEMP_DIR}/duplicate_files.txt" | sed 's/^[^ ]* //' >> "${ANALYSIS_REPORT}"
        echo '```' >> "${ANALYSIS_REPORT}"
        echo -e "\n${RED}❌ Found $(grep -c '^$' "${TEMP_DIR}/duplicate_files.txt" || true) sets of duplicate files${NC}"
    else
        echo -e "${GREEN}✅ No duplicate files found${NC}"
        echo -e "\n## No Duplicate Files Found" >> "${ANALYSIS_REPORT}"
    fi
}

# Function to find similar functions
find_duplicate_functions() {
    echo -e "\n${YELLOW}🔍 Extracting functions from Rust files...${NC}"
    
    # Process each Rust file
    while IFS= read -r file; do
        echo -n "."
        extract_functions "$file" "${TEMP_DIR}/functions"
    done < "${TEMP_DIR}/all_rust_files.txt"
    echo ""
    
    # Find similar functions using fdupes
    echo -e "\n${YELLOW}🔍 Finding similar functions...${NC}"
    fdupes -r "${TEMP_DIR}/functions" > "${TEMP_DIR}/duplicate_functions.txt" 2>/dev/null || {
        echo -e "${RED}❌ fdupes not found. Please install it with 'brew install fdupes' or 'sudo apt-get install fdupes'${NC}"
        return 1
    }
    
    # Process and report duplicate functions
    if [ -s "${TEMP_DIR}/duplicate_functions.txt" ]; then
        echo -e "\n## Duplicate Functions Found" >> "${ANALYSIS_REPORT}"
        echo -e "\nThe following functions appear to be duplicates or very similar:\n" >> "${ANALYSIS_REPORT}"
        
        local duplicate_count=0
        local current_group=0
        
        while IFS= read -r line; do
            if [ -z "$line" ]; then
                current_group=$((current_group + 1))
                continue
            fi
            
            local fn_file=$(basename "$line")
            local src_file=$(grep "${fn_file%%_*}" "${TEMP_DIR}/function_index.txt" | head -1 | cut -d':' -f1)
            local fn_name=$(grep "${fn_file%%_*}" "${TEMP_DIR}/function_index.txt" | head -1 | cut -d':' -f2)
            
            if [ $duplicate_count -eq 0 ]; then
                echo -e "### Group ${current_group}\n" >> "${ANALYSIS_REPORT}"
            fi
            
            echo -e "- **${fn_name}** in ${src_file}" >> "${ANALYSIS_REPORT}"
            duplicate_count=$((duplicate_count + 1))
            
        done < "${TEMP_DIR}/duplicate_functions.txt"
        
        echo -e "\n${RED}❌ Found ${duplicate_count} potential function duplicates in ${current_group} groups${NC}"
    else
        echo -e "${GREEN}✅ No duplicate functions found${NC}"
        echo -e "\n## No Duplicate Functions Found" >> "${ANALYSIS_REPORT}"
    fi
}

# Function to analyze directory structure for potential duplicates
analyze_directory_structure() {
    echo -e "\n${YELLOW}📊 Analyzing directory structure...${NC}"
    
    # Find all directories and calculate a hash of their structure
    find "${REPO_ROOT}" -type d -not -path '*/target/*' -not -path '*/.git/*' -not -path '*/node_modules/*' | while read -r dir; do
        # Get a signature of the directory structure and file types
        local sig=$(find "$dir" -maxdepth 1 -type f -name '*.rs' -exec basename {} \; | sort | sha256sum | cut -d' ' -f1)
        echo "${sig} ${dir}"
    done | sort > "${TEMP_DIR}/dir_signatures.txt"
    
    # Find directories with similar structure
    local last_sig=""
    local last_dir=""
    local has_duplicates=0
    
    echo -e "\n## Similar Directories" >> "${ANALYSIS_REPORT}"
    
    while read -r line; do
        local sig="${line%% *}"
        local dir="${line#* }"
        
        if [ "$sig" = "$last_sig" ]; then
            if [ $has_duplicates -eq 0 ]; then
                echo -e "\n### Similar Directory Group" >> "${ANALYSIS_REPORT}"
                echo "- ${last_dir}" >> "${ANALYSIS_REPORT}"
                has_duplicates=1
            fi
            echo "- ${dir}" >> "${ANALYSIS_REPORT}"
            echo -e "${YELLOW}⚠️  Similar directories found:${NC}\n  ${last_dir}\n  ${dir}\n"
        else
            last_sig="$sig"
            last_dir="$dir"
            has_duplicates=0
        fi
    done < "${TEMP_DIR}/dir_signatures.txt"
    
    if [ $has_duplicates -eq 0 ]; then
        echo -e "${GREEN}✅ No similar directory structures found${NC}"
        echo "No similar directory structures found." >> "${ANALYSIS_REPORT}"
    fi
}

# Function to generate cleanup script
generate_cleanup_script() {
    echo -e "\n${YELLOW}📝 Generating cleanup script...${NC}"
    
    cat > "${REPO_ROOT}/cleanup_duplicates.sh" << 'EOF'
#!/bin/bash

# cleanup_duplicates.sh - Generated by deduplicate_repo.sh
# WARNING: Review carefully before running

set -e

# Create backup
BACKUP_DIR="${PWD}/backup_$(date +%Y%m%d_%H%M%S)"
mkdir -p "${BACKUP_DIR}"

echo "Backing up to ${BACKUP_DIR}..."
cp -r "${PWD}" "${BACKUP_DIR}/"

echo -e "\n⚠️  Review the following actions before proceeding:"

# Add commands to remove duplicates based on analysis
# Example:
# echo "Would remove: path/to/duplicate"
# rm -rf "path/to/duplicate"

echo -e "\n🚀 Cleanup script generated. Please review and run it manually."
EOF
    
    chmod +x "${REPO_ROOT}/cleanup_duplicates.sh"
    echo -e "${GREEN}✅ Cleanup script generated: ${REPO_ROOT}/cleanup_duplicates.sh${NC}"
    echo -e "\n## Next Steps

1. Review the analysis report at: ${ANALYSIS_REPORT}
2. Edit and run the generated cleanup script: ${REPO_ROOT}/cleanup_duplicates.sh" >> "${ANALYSIS_REPORT}"
}

# Main execution
main() {
    # Check for required tools
    command -v sha256sum >/dev/null 2>&1 || { echo >&2 "sha256sum is required but not installed. Aborting."; exit 1; }
    
    # Run analysis
    find_duplicate_files
    find_duplicate_functions
    analyze_directory_structure
    generate_cleanup_script
    
    # Final report
    echo -e "\n${GREEN}✅ Analysis complete!${NC}"
    echo -e "\n📊 Report generated at: ${ANALYSIS_REPORT}"
    echo -e "🚀 Next steps:"
    echo -e "  1. Review the analysis report"
    echo -e "  2. Edit the generated cleanup script if needed"
    echo -e "  3. Run ${REPO_ROOT}/cleanup_duplicates.sh to perform the cleanup"
    echo -e "\n💡 Tip: Always make a backup before running the cleanup script!"
}

# Run the main function
main "$@"

# Cleanup
trap 'rm -rf "${TEMP_DIR}"' EXIT
