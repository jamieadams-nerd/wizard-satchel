#!/usr/bin/env zsh
#
# Author: Jamie L. Adams
# Date:   Jan 2026
#
# fancy_strings.zsh
# ------------------
#
# PURPOSE
#   This script transforms ordinary ASCII text into visually distinct
#   Unicode variants using a configurable character mapping table.
#
#   For example:
#
#       "Hello 123!"  →  "𝐇𝐞𝐥𝐥𝐨 🯱🯲🯳❗"
#
#   Characters are replaced using Unicode codepoint ranges chosen for
#   stylistic or visualization purposes (bold math letters, segmented
#   digits, etc.).
#
#
# HOW IT WORKS
#
#   1) A global associative array CHARMAP stores mappings:
#
#          original character → Unicode replacement
#
#      Example:
#
#          "1" → U+1FBF1
#          "A" → U+1D400
#
#   2) Initialization functions populate CHARMAP:
#
#        • init_digits()      → maps '0'–'9'
#        • init_uppercase()   → maps 'A'–'Z'
#        • init_lowercase()   → maps 'a'–'z'
#        • init_punctuation() → maps selected punctuation
#
#      Most mappings are contiguous Unicode ranges, so values are
#      computed using:
#
#          mapped = BASE + offset
#
#   3) map_chars() processes input text one character at a time.
#
#        • If a character exists in CHARMAP, its mapped Unicode
#          replacement is emitted.
#        • Otherwise, the original character is preserved.
#
#      This guarantees unmapped text passes through unchanged.
#
#
# UNICODE OUTPUT
#
#   Unicode characters are emitted using:
#
#       printf "%b" "\\UXXXXXXXX"
#
#   where XXXXXXXX is an 8-digit hexadecimal codepoint.
#
#   This avoids printing raw escape sequences and ensures proper
#   character output on UTF-8 terminals.
#
#
# CONFIGURABLE RANGES
#
#   The script defines starting points for mapping blocks:
#
#       digit_start  → Unicode digit style block
#       chars_start  → Unicode uppercase alphabet block
#       lower_start  → Unicode lowercase alphabet block
#
#       Some fun blocks of code points to try. If you want 
#       all uppercase, just set lower case start to the same
#       starting codepoint as the upper. 
#
#       Set *_start variables below accordingly:
#
#       digit_start → 0x1FBF1  SEGMENTED DIGIT ONE
#                   → 0x02780  NUMBERS Enclosed in circle
#                   → 0x0278A  NUMBERS in filled circles 
#                   → 0x1D7CE  Math Bold
#                   → 0x1D7D8  Math Double struck
#                   → 0x1D7E2  Math Sans Serif
#                   → 0x1D7DE  Math Sans Serif Bold
#                   → 0x1D7F6  Math Monospace
#
#       chars_start → 0x1D400  MATHEMATICAL BOLD CAPITAL A
#                   → 0x1FBF1
#                   → 0x1D434  Math italic
#                   → 0x1D468  Math Bold italic
#                   → 0x1D5A0  
#                   → 0x1D5D4  
#
#       lower_start → 0x1D41A  Parent 0x1D400
#                   → 0x1D44E  Parent 0x1D434
#                   → 0x1D482  Parent 0x1D468
#                   → 0x1D5BA  Parent 0x1D5A0
#                   → 0x1D5EE  Parent 0x1D5D4
#
#
#   These can be replaced with any contiguous Unicode block that
#   matches ASCII ordering.
#
#
# LIMITATIONS
#
#   • Only characters present in CHARMAP are transformed.
#   • Some Unicode characters may not render in all terminals due
#     to font limitations.
#   • Combining characters and complex scripts are not handled here.
#
#
# TYPICAL USES
#
#   • Terminal UI decoration
#   • Encoding experiments
#   • Unicode exploration tools
#   • Visual transformations in shell scripts
#
#
# AUTHOR NOTE
#
#   Designed for experimentation and Unicode visualization rather
#   than strict typography or internationalization.
#



export LANG=${LANG:-en_US.UTF-8}

# ----------------------------
# Character mapping table
# ----------------------------
typeset -A CHARMAP

# Unicode Sets to use. Give the starting codepoint

#digit_start=0x1D7F6    
digit_start=0x02780    
chars_start=0x1D400    # A-Z Math Bold
lower_start=0x1D41A    # a-z Math Bold


unicode_char() {
    # emit Unicode char from codepoint
    printf "%b" "\\U$(printf '%08X' "$1")"
}

# ----------------------------
# Digits 0–9 → digit_start block
# ----------------------------
init_digits() {
    local d code
    for d in {0..9}; do
        #code=$((0x1FBF0 + d))
        code=$((digit_start + d))
        CHARMAP[$d]=$(unicode_char $code)
    done
}

# ----------------------------
# Letters A–Z → chars_start block
# ----------------------------
init_uppercase() {
    local i ascii code letter
    for ((i=0; i<26; i++)); do
        ascii=$((65 + i))
        code=$((chars_start + i))

        # convert ASCII code to character
        letter=$(printf "\\$(printf '%03o' "$ascii")")

        CHARMAP[$letter]=$(unicode_char $code)
    done
}

# ----------------------------
# Lowercase a–z → lower_start block
# ----------------------------
init_lowercase() {
    local i ascii code letter
    for ((i=0; i<26; i++)); do
        ascii=$((97 + i))
        code=$((lower_start + i))

        letter=$(printf "\\$(printf '%03o' "$ascii")")

        CHARMAP[$letter]=$(unicode_char $code)
    done
}

# ----------------------------
# Example punctuation mappings
# ----------------------------
init_punctuation() {
    CHARMAP["!"]="❗"
    CHARMAP["?"]="❓"
    CHARMAP["."]="•"
    CHARMAP[","]="‚"
    CHARMAP["-"]="−"
}

# ----------------------------
# Apply mapping
# ----------------------------
map_chars() {
    local input="$1"
    local out=""
    local c

    for ((i=1; i<=${#input}; i++)); do
        c="${input[i]}"

        if [[ -v "CHARMAP[$c]" ]]; then
            out+="${CHARMAP[$c]}"
        else
            out+="$c"
        fi
    done

    print -r -- "$out"
}

# ----------------------------
# Initialize tables
# ----------------------------
init_digits
init_uppercase
init_lowercase
init_punctuation

# ----------------------------
# Demo usage
# ----------------------------
map_chars "$*"

