#!/bin/zsh
#
#
#
#
console_center_text() {
    local text="$1"

    # Get terminal width or default to 80
    columns=$(tput cols || echo 80)

    # Calculate the number of leading spaces needed
    padding=$(( (${#text} + columns) / 2 ))

    # Use printf with the calculated width to right-align the text in a larger field
    printf "%*s\\n" "$padding" "$text"
}

console_right_text() {
    local text="$1"

    # Get terminal width or default to 80
    COLUMNS=$(tput cols || echo 80)

    # Calculate the number of leading spaces needed

    # Use printf with the calculated width to right-align the text in a larger field
    printf "%*s\\n" $COLUMNS "$text"
}

horiz_multichars() {
    local char="$1"
    local count="$2"

    for i in {1..$count}; do printf "$char"; done 
}

light_box() {
    echo "Light Box"
    echo -n "\u250C"        # upper left cornder
    echo -n "\u2500\u2500\u2500\u2500"  # top horizontal bar
    echo "\u2510"           # upper right coner

    echo "\u2502    \u2502" # Vertical bars

    echo -n "\u2514"        # Lower left cornder
    echo -n "\u2500\u2500\u2500\u2500"  # Bottom horizontal bar
    echo "\u251B"           # loweer right cornder
}

# Courners are rounded
rounded_box() {
    local box_width="$1"
    local msg="$2"

   # Top of box 
    echo -n "\u256D"
    horiz_multichars "\u2501" $((box_width - 2 ))
    echo  "\u256E" # upper rightcorner

    # Now horizontally, center text inside of box
    local inner_width=$(( box_width - 2 )) 
    local msg_len=${#msg}

    local left_padding=$(( (inner_width - msg_len) / 2 ))
    local right_padding=$(( inner_width - msg_len - left_padding ))

    printf "\u2502%*s%s%*s\u2502\n" \
        "$left_padding" "" \
        "$msg" \
        "$right_padding" ""


    # Now print bottom of box.
    echo -n "\u2570"
    horiz_multichars "\u2501" $((box_width - 2 ))
    echo "\u256F"

}

#
# Call the heavy_box() function with the following arguments:
# heavy_box <width> <msg>
#
heavy_box() {
    local box_width="$1"
    local msg="$2"

   # Top of box 
    echo -n "\u250F"
    horiz_multichars "\u2501" $((box_width - 2 ))
    echo  "\u2513" # upper rightcorner

    # Now horizontally, center text inside of box
    local inner_width=$(( box_width - 2 )) 
    local msg_len=${#msg}

    local left_padding=$(( (inner_width - msg_len) / 2 ))
    local right_padding=$(( inner_width - msg_len - left_padding ))

    printf "\u2503%*s%s%*s\u2503\n" \
        "$left_padding" "" \
        "$msg" \
        "$right_padding" ""


    # Now print bottom of box.
    echo -n "\u2517"
    horiz_multichars "\u2501" $((box_width - 2 ))
    echo "\u251B"

}


double_box() {
    local box_width="$1"
    local msg="$2"

   # Top of box 
    echo -n "\u2554"
    horiz_multichars "\u2550" $((box_width - 2 ))
    echo  "\u2557" # upper rightcorner

    # Now horizontally, center text inside of box
    local inner_width=$(( box_width - 2 )) 
    local msg_len=${#msg}

    local left_padding=$(( (inner_width - msg_len) / 2 ))
    local right_padding=$(( inner_width - msg_len - left_padding ))

    printf "\u2551%*s%s%*s\u2551\n" \
        "$left_padding" "" \
        "$msg" \
        "$right_padding" ""


    # Now print bottom of box.
    echo -n "\u255A"
    horiz_multichars "\u2550" $((box_width - 2 ))
    echo "\u255D"

}

gradient_bar() {
    local width="$1"
    local msg="$2"
    local padding="  " # Initial left padding
    local full_str="${padding}${msg}"
    
    # Fill the rest of the string with spaces to reach total width
    while [ ${#full_str} -lt $width ]; do
        full_str="${full_str} "
    done

    # Grayscale ramp (ANSI 256-colors)
    # 255 is White, 232 is almost Black
    # We want to stay solid for a while, then fade
    local start_fade_at=$(( width * 7/10 )) # Start fading at 70% of the width

    for (( i=0; i<$width; i++ )); do
        local char="${full_str:$i:1}"
        local color=255 # Default solid white

        if [ $i -ge $start_fade_at ]; then
            # Calculate a decreasing color code
            # Maps the remaining distance to the 255-236 range
            local steps_into_fade=$(( i - start_fade_at ))
            local fade_range=$(( width - start_fade_at ))
            # 255 down to 238 (18 steps of grey)
            color=$(( 255 - (steps_into_fade * 18 / fade_range) ))
        fi

        # \e[48;5;${color}m  <- Sets Background color
        # \e[38;5;232m       <- Sets Text color to Black for contrast
        echo -ne "\e[48;5;${color}m\e[38;5;232m${char}"
    done

    echo -e "\e[0m" # Reset colors
}

gradient_bar_rgb() {
    local width="$1"
    local msg="  $2  "
    local full_str="$msg"
    
    # Fill remaining width with spaces
    while [ ${#full_str} -lt $width ]; do
        full_str="${full_str} "
    done

    # RGB for Bright Green: (150, 255, 150) - A nice "Terminal Green"
    # RGB for Dark Green:   (0, 50, 0)
    local r=150; local g=255; local b=150
    local start_fade_at=$(( width * 6/10 )) # Fade starts at 60% width

    for (( i=0; i<$width; i++ )); do
        local char="${full_str:$i:1}"
        
        if [ $i -ge $start_fade_at ]; then
            # Calculate how far we are into the fade (0.0 to 1.0)
            local steps_in=$(( i - start_fade_at ))
            local total_steps=$(( width - start_fade_at ))
            
            # Decrease R, G, and B proportionally
            # We use $(( ... )) for integer math
            r=$(( 150 - (150 * steps_in / total_steps) ))
            g=$(( 255 - (205 * steps_in / total_steps) )) # Ends at 50
            b=$(( 150 - (150 * steps_in / total_steps) ))
        fi

        # \e[48;2;R;G;Bm  <- Sets Background to RGB
        # \e[38;2;0;0;0m  <- Sets Text to Black
        echo -ne "\e[48;2;${r};${g};${b}m\e[38;2;0;0;0m${char}"
    done

    echo -e "\e[0m" # Reset
}

# Example:
#gradient_bar_rgb 60 "SECURE KERNEL LOADED"

# Example usage:
#gradient_bar 60 "SYSTEM DIAGNOSTICS - VERSION 2.0.4"
#gradient_bar 60 "ESTABLISHING SECURE CONNECTION..."


gradient_bar_green() {
    local width="$1"
    local msg="$2"
    local padding="  "
    local full_str="${padding}${msg}"
    
    # Fill remaining width with spaces
    while [ ${#full_str} -lt $width ]; do
        full_str="${full_str} "
    done

    # The Green Palette (Brightest to Darkest)
    # These are specific ANSI 256-color IDs
    local green_ramp=(82 82 76 70 64 46 40 34 28 22)
    local ramp_size=${#green_ramp[@]}
    
    local start_fade_at=$(( width - ramp_size ))

    for (( i=0; i<$width; i++ )); do
        local char="${full_str:$i:1}"
        local color=82 # Solid bright green
        local text_color=16 # Black text for high contrast

        if [ $i -ge $start_fade_at ]; then
            local steps_into_fade=$(( i - start_fade_at ))
            color=${green_ramp[$steps_into_fade]}
            
            # Optional: If the green gets too dark, make the text white
            [[ $color -lt 34 ]] && text_color=255
        fi

        # Set Background ($color) and Foreground ($text_color)
        echo -ne "\e[48;5;${color}m\e[38;5;${text_color}m${char}"
    done

    echo -e "\e[0m" # Reset
}

# Try it out:
#gradient_bar_green 60 "INITIALIZING GREEN CORE..."

gradient_custom_bar() {
    local width="$1"
    local msg="  $2  "
    local full_str="$msg"
    
    # Fill remaining width with spaces
    while [ ${#full_str} -lt $width ]; do
        full_str="${full_str} "
    done

    # Cyberpunk Gold: 255, 190, 0 → 40, 30, 0
    # Deep Space Blue: 100, 150, 255 → 0, 10, 40
    # Alert Red: 255, 80, 80 → 50, 0, 0
    # Sexy green: 150, 255, 150 -> 0, 40, 0

    # Start Color (Bright Green): 150, 255, 150
    # End Color (Deep Forest): 0, 40, 0
    local r_start=150; local g_start=255; local b_start=150
    local r_end=0;     local g_end=40;    local b_end=0

    local start_fade_at=$(( width * 6/10 )) 
    local fade_width=$(( width - start_fade_at ))

    for (( i=0; i<$width; i++ )); do
        local char="${full_str:$i:1}"
        local r=$r_start; local g=$g_start; local b=$b_start

        if [ $i -ge $start_fade_at ]; then
            local n=$(( i - start_fade_at ))
            # Calculate the decay
            r=$(( r_start - ( (r_start - r_end) * n / fade_width ) ))
            g=$(( g_start - ( (g_start - g_end) * n / fade_width ) ))
            b=$(( b_start - ( (b_start - b_end) * n / fade_width ) ))
        fi

        # Check for TrueColor support
        if [[ $COLORTERM == "truecolor" || $COLORTERM == "24bit" ]]; then
            echo -ne "\e[48;2;${r};${g};${b}m\e[38;2;0;0;0m${char}"
        else
            # Fallback to standard green if RGB isn't supported
            echo -ne "\e[42m\e[30m${char}"
        fi
    done
    echo -e "\e[0m"
}




heavy_box 50 "Controlled Unclassified Information"

console_center_text "THIS IS A VERY BIG TITLE"
console_right_text "Right side THIS IS A VERY BIG TITLE"

double_box 50 "Controlled Unclassified Information"

rounded_box 50 "Controlled Unclassified Information"

# Black and white grayscale
gradient_bar 50 "Some message"

gradient_bar_green 60 "INITIALIZING GREEN CORE..."
echo
gradient_bar_rgb 60 "SECURE KERNEL LOADED"
echo
gradient_custom_bar 80 " "
gradient_custom_bar 80 "UNLOCKING TRANSLATIONS NOW..."
gradient_custom_bar 80 " "
echo




