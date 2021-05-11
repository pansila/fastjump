#!/bin/bash

target="target/$TARGET/debug/install"

ret=$($target --install 2>&1)

src=$(echo "$ret" | tail -n 1 | head -n 1 | awk '{print $(NF)}')
if [ -z $src ]; then
    >&2 echo "can not find the source file"
    exit 1
fi

echo $src
