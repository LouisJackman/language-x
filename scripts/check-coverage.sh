#!/bin/bash

set -o errexit
set -o nounset

target_debug=${1:?}

file=$(find "$target_debug" -maxdepth 1 -name 'sylan-*' -executable -type f)

# Generate the coverage report
mkdir -p "target/cov/$(basename "$file")"
/usr/local/bin/kcov \
    --exclude-pattern=/.cargo,/usr/lib \
    --verify "target/cov/$(basename "$file")" \
    "$file"

if [ "$?" -eq 0 ]
then
    echo succeeded coverage report for file "$file"
else
    echo failed kcov
    exit 1
fi

mv /target/cov/sylan-*/sylan-*.*/cobertura.xml /opt/coverage-results

