#!/bin/sh

test_linux() {
    sudo -h >/dev/null 2>&1

    if [ $? -eq 0 ]; then
        sudo apt-get install -y bash >/dev/null
    else
        apt-get install -y bash >/dev/null
    fi

    if [ $? -ne 0 ]; then
        >&2 echo "Failed to install bash"
        exit 1
    fi
}

test_macos() {
    echo 111
}

if [ $TRAVIS_OS_NAME = linux ]; then
    test_linux
else
    test_macos
fi

target="target/$TARGET/debug/install"

ret=$($target --install 2>&1)

src=$(echo "$ret" | tail -n 1 | head -n 1 | awk '{print $(NF)}')
if [ -z $src ]; then
    >&2 echo "can not find the source file"
    exit 1
fi

echo $src
