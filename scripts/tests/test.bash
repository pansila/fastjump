#!/bin/bash
# set -x

source $1
if [ $? -ne 0 ]; then
    exit 1
fi

j -h

j
j -s

j --add $(pwd)
j -s

cd $HOME
j -s
