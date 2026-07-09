doc:
    RUSTDOCFLAGS="-Z unstable-options --generate-link-to-definition" cargo doc --workspace --all --no-deps --document-private-items
