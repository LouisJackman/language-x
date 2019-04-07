#!/bin/sh

set -o errexit
set -o nounset

target_debug=${1:?}

# For each test program, find the test executable and report its coverage.
for file in $(find "$target_debug" -maxdepth 1 -name 'sylan-*' -executable -type f)
do

    # Generate the coverage report
    mkdir -p "target/cov/$(basename "$file")"
    /usr/bin/kcov \
        --exclude-pattern=/.cargo,/usr/lib \
        --verify "target/cov/$(basename "$file")" \
        "$file"

    if [ "$?" == 0 ]
    then
        echo succeeded coverage report for file "$file"
    else
        echo failed kcov
        exit 1
    fi
done

./.codecov -t "$CODECOV_TOKEN"

