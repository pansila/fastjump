#!/bin/sh

sudo apt-get install -y bash

target="target/$TARGET/debug/install"

src=$($target --install | tail -n 1)
if [ -z $src ]; then
    exit 1
fi

echo $src
