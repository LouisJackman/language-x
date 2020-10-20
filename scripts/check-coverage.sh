#!/bin/bash

set -o errexit
set -o nounset
set -o xtrace

target_debug=${1:?}

file=$(find "$target_debug" -maxdepth 1 -name 'sylan-*' -executable -type f)

# Generate the coverage report
mkdir -p "cov/$(basename "$file")"
/usr/local/bin/kcov \
    --exclude-pattern=/.cargo,/usr/lib \
    --verify "cov/$(basename "$file")" \
    "$file"

if [ "$?" -eq 0 ]
then
    echo succeeded coverage report for file "$file"
else
    echo failed kcov
    exit 1
fi

mv /cov/sylan-*/sylan-*.*/cobertura.xml /opt/coverage-results

coverage=$(jq -r '.percent_covered' /cov/sylan-*/sylan-*.*/coverage.json)
echo Coverage: "$coverage"

