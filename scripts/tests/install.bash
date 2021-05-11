#!/bin/sh

sudo apt-get install -y bash >&2 1>/dev/null
if [ $? -ne 0 ]; then
    echo "Failed to install bash" > 2
    exit 1;
fi

target="target/$TARGET/debug/install"

ret=$($target --install 2>&1)

src=$(echo "$ret" | tail -n 1 | head -n 1 | awk '{print $(NF)}')
if [ -z $src ]; then
    echo "can not find the source file" > 2
    exit 1
fi

echo $src
