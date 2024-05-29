#! /bin/bash

# echo everything out.
set -x

# echo out all environment variables
env

# Copy the workspace to the /app directory
cp -r . /app/workspace

# Change to the workspace directory
cd /app/workspace

# Run the application
/app/flexvers --skip-non-formatted