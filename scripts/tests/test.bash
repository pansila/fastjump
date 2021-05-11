#!/bin/bash
# set -x
echo $1
source $1

j -h

j
j -s

j --add $(pwd)
j -s

cd $HOME
j -s
