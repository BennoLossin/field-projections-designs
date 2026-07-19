default: watch-doc

watch-doc:
    fd -e rs -e toml -e patch | entr -c -c just doc

doc *FLAGS:
    cargo doc --workspace --all --no-deps --document-private-items {{FLAGS}}

legacy-doc:
    #!/usr/bin/env bash
    set -euxo pipefail

    pushd legacy
    for design in *; do
        pushd $design
        cargo doc --workspace --all --no-deps --document-private-items
        popd
    done
    popd

test:
    cargo test --workspace --all-targets
    # BROKEN:
    # cargo test --workspace --doc

miri:
    cargo miri test --workspace --all-targets
    # BROKEN:
    # cargo test --workspace --doc

pages: doc legacy-doc
    #!/usr/bin/env bash
    set -euxo pipefail

    mkdir -p target/pages/legacy
    cp -r target/doc target/pages/current

    pushd legacy
    for design in *; do
        cp -r "$design/target/doc" "../target/pages/legacy/$design"
    done
    popd

    cp .github/workflows/overview-head.template.html target/pages/index.html

    LINKS_HTML=""
    # ls -r handles the reverse alphanumeric sorting perfectly for numbered prefixes
    for design in $(ls -r legacy/); do
        crate=$(echo "_$design" | tr '-' '_')
        echo "<li class=\"legacy-design\"><span class=\"badge\">legacy</span><a href=\"legacy/$design/$crate/index.html\">$design</a></li>" \
            >> target/pages/index.html
    done

    cat .github/workflows/overview-tail.template.html >> target/pages/index.html
