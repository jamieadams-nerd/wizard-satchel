#!/usr/bin/zsh
#
# Scan the unicode_groups.txt file and generate
# unicode glyphs/symbols to standard out. 
#

begin_time=$(/usr/bin/date +%s%N)

master_file="unicode_groups.txt"

#
# Read the file line by line
#   We set IFS to a comma and use -r to handle 
#   backslashes/quotes literally
#
while IFS=',"' read -r start end block_name; do
    
    # Skip those commented out for pruning! 
    if [[ "$start" =~ ^# ]]; then
        echo "# --- Skipping Block: $block_name ($start to $end) ---"
        continue
    fi

    # Skip empty lines or headers if they exist
    # Skip lines begining with # 
    [[ -z "$start" ]] && continue

    echo
    echo "# --- Starting Block: $block_name ($start to $end) ---"
    for ((i=$start; i<=$end; i++)); do
    
        # Skip control characters
        if (( i <= 0x1F || i == 0x7F )); then
            echo "Skipping control character..."
            continue
        fi

        # 1. Store the value for Python (using a clean environment variable)
        # This replaces Zsh's "16#" prefix with the standard "0x"
        #
        raw_val=$i
        clean_hex="0x${raw_val#*#}"
    
        # 2. Get the Unicode name via Python
        # Python's int(val, 16) handles the hex string correctly now
        name=$(HEX=$clean_hex python3 -c "import unicodedata, os; h=os.environ['HEX']; print(unicodedata.name(chr(int(h, 16)), 'Unknown'))")
    
        # 3. Format the integer into an 8-digit padded hex string for the shell
        hex_padding=$(printf "%08X" $i)
        
        # 4. Display results
        echo -n "Processing: 0x$hex_padding "

        # 4a. Check for special invisible chaacters like special spaces and wrap them in quote
        q_needed=""
        if (( i == 0x202F || i == 0x00A0 )); then
            q_needed="'"
        fi
        if (( i >= 0x2000 && i <= 0x200A )); then
            q_needed="'"
        fi

        /usr/bin/printf "  Symbol: ${q_needed}\U${hex_padding}${q_needed}  Name: $name\n" 2>/dev/null || echo " [Rendering error]"
    done

done < "$master_file"

end_time=$(/usr/bin/date +%s%N)
elapsed_ns=$((end_time - begin_time))
elapsed_ms=$((elapsed_ns / 1000000))
echo
echo "Elapased: ${elapased_ms}"


