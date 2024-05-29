#!/bin/bash

# echo everything out.
set -x

# echo out all environment variables
env

echo "Current directory: $(pwd)"
ls -aln /github/workspace

# Copy the workspace to the /app/workspace directory
mkdir -p /app/workspace
cp -r $(pwd)/.git /app/workspace/.git
cp -r $(pwd)/.semver.json /app/workspace/.semver.json

chown -R 1:1 /app/workspace
ls -aln /app/workspace

# Change to the workspace directory
cd /app/workspace
ls -aln /app

# Run the application
/app/flexvers --skip-non-formatted