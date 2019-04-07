#!/bin/sh

set -o errexit
set -o nounset

if ! [ -f ./kcov-build/usr/local/bin/kcov ]
then

    # Download and unpack the kcov program.
    wget https://github.com/SimonKagstrom/kcov/archive/master.tar.gz
    tar xzf master.tar.gz

    (
        # Build it.
        cd kcov-master
        mkdir build
        cd build
        cmake ..
        make

        # Install it.
        make install DESTDIR=../../kcov-build
    )

    # Clean it up.
    rm -rf kcov-master
fi

# For each test program, find the test executable and report its coverage.
for file in $(find target/debug -maxdepth 1 -name 'sylan-*' -executable -type f)
do

    # Generate the coverage report
    mkdir -p "target/cov/$(basename $file)"
    ./kcov-build/usr/local/bin/kcov \
        --exclude-pattern=/.cargo,/usr/lib \
        --verify "target/cov/$(basename $file)" \
        "$file"

    if [ "$?" == 0 ]
    then
        echo succeeded coverage report for file "$file"
    else
        echo failed kcov
        exit 1
    fi
done

if ! [ -f ./.codecov ]
then
    wget -O - -q "https://codecov.io/bash" >.codecov
    chmod +x .codecov
fi
./.codecov -t "$CODECOV_TOKEN"

