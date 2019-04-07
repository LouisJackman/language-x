#!/bin/sh

set -o errexit
set -o nounset

curl -LSs "https://codecov.io/bash" >.codecov
chmod +x .codecov

