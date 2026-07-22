default: watch-doc

watch-doc:
    fd -e rs -e toml -e patch | entr -c -c just doc

doc *FLAGS: build
    cargo doc --workspace --all --no-deps --document-private-items {{ FLAGS }}

legacy-doc: build
    #!/usr/bin/env bash
    set -euxo pipefail

    pushd legacy
    for design in *; do
        pushd $design
        cargo doc --workspace --all --no-deps --document-private-items
        popd
    done
    popd

test: build
    cargo test --workspace --all-targets
    cargo test --workspace --doc

miri: build
    cargo miri test --workspace --all-targets
    cargo miri test --workspace --doc

pages: doc legacy-doc build
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

build: E-struct_of_arrays

E-struct_of_arrays:
    cd examples/struct_of_arrays \
        && cargo expand expansion \
           | head --lines=-1 \
           | tail --lines=+9 \
           | sed 's/::core::primitive:://g' \
           | sed 's/::core::marker::Sized/Sized/g' \
           | sed 's/::core::default::Default/Default/g' \
           | sed 's/::design::ops::place::borrowck:://g' \
           | sed 's/::design::ops::place:://g' \
           | sed 's/::design::place:://g' \
           | sed 's/___Field/F/g' \
           | sed 's/___Handle/H/g' \
           | sed 's/<\([A-Z]\) as [A-Za-z_]\+>::/\1::/g' \
           | sed 's/^ *\(fn\|impl\|unsafe impl\|pub\|struct\)/\n\1/g' \
           | rustfmt \
           | sed 's/^/ /' \
           > src/derive_expansion.rs
