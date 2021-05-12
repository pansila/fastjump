# This script takes care of testing your crate

set -ex

build_stage() {
    cross build --target $TARGET
    # cross build --target $TARGET --release
}

unit_test_stage() {
    cross test --target $TARGET -- --skip integration_tests
    # cross test --target $TARGET --release
}

integration_test_stage() {
    # only x86 code is runnable in the traivs
    if [ ! $TARGET = x86_64-unknown-linux-gnu ] && [ ! $TARGET = i686-unknown-linux-gnu ]; then
        return
    fi

    cross test integration_tests --target $TARGET -- --nocapture
    # cross test --target $TARGET --release
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
