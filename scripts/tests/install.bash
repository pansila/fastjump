# TODO: shell is not set in the cross container
if [ ! -v $SHELL ]; then
    echo "Warning: env var SHELL is not set" >&2
    export SHELL=/bin/bash
fi

# TODO: Mac OS has no var TARGET
if [ ! -v $TARGET ]; then
    echo "Warning: env var TARGET is not set" >&2
    export TARGET=x86_64-apple-darwin
fi

target="target/$TARGET/debug/install"

ret=$($target --install 2>&1)
if [ $? -ne 0 ]; then
    exit 1
fi

src=$(echo "$ret" | tail -n 1 | head -n 1 | awk '{print $(NF)}')
if [ -z $src ]; then
    echo "Error: can not find the source file"
    exit 1
fi

echo $src
