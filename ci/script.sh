# This script takes care of testing your crate

set -ex

main() {
    cross build --target $TARGET
    # cross build --target $TARGET --release

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    cross test --target $TARGET
    # cross test --target $TARGET --release

    src=$(cross run --target $TARGET --bin install -- --install | tail -n 4 | head -1)
    cross run --target $TARGET --bin test -- $src
    cross run --target $TARGET --bin install -- --uninstall
    # cross run --target $TARGET --release --bin $CRATE_NAME

    # cross run --target $TARGET --release --bin $CRATE_NAME
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
