#!/bin/sh

set -o errexit
set -o nounset

wget --output-document .codecov https://codecov.io/bash
chmod +x .codecov

