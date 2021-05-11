#!/bin/bash
# set -x
source $1

j -h

j
j -s

j --add $(pwd)
j -s

cd $HOME
j -s
