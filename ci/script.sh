# This script takes care of testing your crate

set -ex

build_stage() {
    cross build --target $TARGET
    # cross build --target $TARGET --release
}

test_stage() {
    cross test --target $TARGET --skip 
    # cross test --target $TARGET --release
}

main() {
    build_stage

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    test_stage
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
