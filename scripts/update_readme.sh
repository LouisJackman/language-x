#!/bin/sh

set -o errexit
set -o nounset

script=$(cat <<-'EOF'

    /^\$EXAMPLE_SOURCE$/ {
        getline

        print "```sylan"
        system("cat examples/readme_example.sy")
        print "```"
    }

    1;

EOF
)

awk "$script" scripts/README.md.tpl >README.md

