#!/bin/sh

set -o errexit
set -o nounset

report=$(find target/debug -maxdepth 1 -name 'sylan-*' -a ! -name '*.d')

for file in $report
do
    mkdir -p "target/cov/$(basename $file)"
    kcov --exclude-pattern=/.cargo,/usr/lib --verify "target/cov/$(basename $file)" "$file"
done

wget -O - -q "https://codecov.io/bash" >.codecov
chmod +x .codecov
./.codecov -t "$CODECOV_TOKEN"

