#!/bin/bash

# Copy the workspace to the /app/workspace directory
mkdir -p /app/workspace
cp -r $(pwd)/.git /app/workspace/.git
cp -r $(pwd)/.semver.json /app/workspace/.semver.json

chown -R $(id -u):$(id -g) /app/workspace

# Change to the workspace directory
cd /app/workspace

# Construct Args
args=""

# Handling the variables:
# INPUT_SKIP-NON-FORMATTED
# INPUT_KEEP-ROOT-VERSION-UP-TO-DATE
# INPUT_FORCE-RELEASE
# INPUT_FORCE-PRE-RELEASE
# Check if the --skip-non-formatted flag environment variable is true
if [ "${INPUT_SKIP-NON-FORMATTED}" = "true" ]; then
    echo "Skipping non-formatted"
    args="--skip-non-formatted"
fi
if [ "${INPUT_KEEP-ROOT-VERSION-UP-TO-DATE}" = "true" ]; then
    echo "Keeping root version up to date"
#   args="$args --keep-root-version-up-to-date"
fi
if [ "${INPUT_FORCE-RELEASE}" = "true" ]; then
    echo "Forcing release"
#   args="$args --force-release"
fi
if [ "${INPUT_FORCE-PRE-RELEASE}" = "true" ]; then
    echo "Forcing pre-release"
#   args="$args --force-pre-release"
fi

# Run the application
/app/flexvers $args