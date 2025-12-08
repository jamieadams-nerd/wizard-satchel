#!/usr/bin/bash
#
# This script steamlines configuring your command git on a host to sign commits.
#

##########################################################################
# Obtain user name and email
##########################################################################

# Function to validate the user name
# Allows alphanumeric characters, spaces, hyphens, and single quotes
validate_name() {
    local name="$1"
    # Basic validation: must contain at least one character and only allowed characters
    if [[ "$name" =~ ^[a-zA-Z0-9\ \'\-]+$ ]]; then
        return 0 # Valid
    else
        echo "Error: Invalid character in name. Only alphanumeric, spaces, hyphens, and single quotes are allowed."
        return 1 # Invalid
    fi
}

# Function to validate the email address using a common regex pattern
validate_email() {
    local email="$1"
    # A common, reasonably strict email regex pattern
    local regex="^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"

    if [[ "$email" =~ $regex ]]; then
        return 0 # Valid
    else
        echo "Error: Invalid email format. Please use a standard email address format (e.g., user@example.com)."
        return 1 # Invalid
    fi
}

# Prompt for Name
while true; do
    read -p "Enter your full name: " user_name
    if validate_name "$user_name"; then
        break
    fi
done

# Prompt for Email
while true; do
    read -p "Enter your email address: " user_email
    if validate_email "$user_email"; then
        break
    fi
done

# Display the collected information
echo -e ":: Thank you. Input successful:"

git config --global user.name "$user_name"
git config --global user.email "$user_email"
git config --global init.defaultBranch "main"
echo ":: Set git config parameters user name and email."

##########################################################################
# Find the signing key for the currently set email for git configuration.
# Then set git configuration sigining to correctly.
# 
# 1. Get the email address Git is currently using
GIT_EMAIL=$(git config --get user.email)

if [ -z "$GIT_EMAIL" ]; then
    echo "Error: Git user.email is not configured." 
    echo "Please set your email using 'git config --global user.email \"you@example.com\"'." 
    exit 1
else
    echo ":: Finding key for $GIT_EMAIL..."
fi


##########################################################################
# Find the fingerprint matching that email address using a more 
# reliable awk pattern
#
function get_fingerprint() {
  local git_email=$1

  GPG_FINGERPRINT=$(gpg --list-secret-keys --with-colons --keyid-format LONG | awk -F: '
    /^fpr:/ {
        # When an FPR line is found, store the fingerprint value
        fingerprint = $10
    }
    /^uid:/ {
        # When a UID line is found, check if the email matches the Git email
        # $10 contains the full UID string, e.g., "Hubot <hubot@example.com>"
        if ($10 ~ /<'$git_email'>/) {
            # If it matches, print the stored fingerprint and exit successfully
            print fingerprint
            exit 0
        }
    }
')
echo "$GPG_FINGERPRINT"
}

##########################################################################
# Create a signing key
#
# gpg --full-generate-key
#
GPG_FINGERPRINT=$(get_fingerprint "$user_email")
if [ -z "$GPG_FINGERPRINT" ]; then
   gpg --quick-gen-key "$user_name <$user_email>" RSA4096 sign,encrypt
fi

GPG_FINGERPRINT=$(get_fingerprint "$user_email")
if [ -z "$GPG_FINGERPRINT" ]; then
    echo "Error: No GPG signing key found matching the email address: $GIT_EMAIL"
    echo "Make sure your GPG key's User ID matches your Git config." 
    exit 1
fi

# 3. Configure Git globally with the obtained GPG fingerprint
git config --global user.signingkey "$GPG_FINGERPRINT"

echo ":: Git global signing key set to: $GPG_FINGERPRINT"
echo "   -- (matching $GIT_EMAIL) --"
git config --global commit.gpgSign true

# Format options are: openpgp, x509, or ssh. openpgp is default
git config --global gpg.format openpgp

echo ":: To export your public key, use:"
echo "      gpg --armor --export $GPG_FINGERPRINT"
echo ":: Copy/paste the output into your GitHub."
