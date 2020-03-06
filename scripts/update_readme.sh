#!/bin/sh

set -o errexit
set -o nounset

script=$(cat <<-'EOF'

    /^\$EXAMPLE_SOURCE$/ {
        getline

        print "```sylan"
        system("cat examples/main.sy")
        print "```"
    }

    1;

EOF
)

awk "$script" scripts/README.md.tpl >README.md

