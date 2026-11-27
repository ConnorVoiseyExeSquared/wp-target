list:
    @just --list

[group('build')]
[doc('Static release binary build')]
build-release: 
    cargo zigbuild --release --target x86_64-unknown-linux-musl
