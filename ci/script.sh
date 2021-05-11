# This script takes care of testing your crate

set -ex

build_stage() {
    cross build --target $TARGET
    # cross build --target $TARGET --release
}

unit_test_stage() {
    cross test --target $TARGET
    # cross test --target $TARGET --release
}

integration_test_stage() {
    src=$(cross run --target $TARGET --bin install -- --install | tail -n 1)
    if [ -z $src ]; then
        exit 1
    fi

    cross run --target $TARGET --bin test -- $src

    cross run --target $TARGET --bin install -- --uninstall
}

main() {
    build_stage

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    unit_test_stage

    integration_test_stage
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
