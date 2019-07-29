#!/bin/sh

set -o errexit
set -o nounset

wget --output-file .codecov https://codecov.io/bash
chmod +x .codecov

