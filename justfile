default: test-miri

test-miri:
    cargo run

doc:
    RUSTDOCFLAGS="-Z unstable-options --generate-link-to-definition" cargo doc --workspace --all --no-deps --document-private-items

legacy-doc:
    #!/usr/bin/env bash
    set -euxo pipefail

    export RUSTDOCFLAGS="-Z unstable-options --generate-link-to-definition"

    pushd legacy
    for design in *; do
        pushd $design
        cargo doc --workspace --all --no-deps --document-private-items
        popd
    done
    popd

pages: doc legacy-doc
    #!/usr/bin/env bash
    set -euxo pipefail

    mkdir -p target/pages/legacy
    cp -r target/doc/* target/pages/

    pushd legacy
    for design in *; do
        cp -r "$design/target/doc/*" ../target/pages/legacy/
    done
    popd

    echo "<meta http-equiv=\"refresh\" content=\"0; url=design/index.html\">" > target/pages/index.html
