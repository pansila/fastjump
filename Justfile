all: bash fish zsh tcsh
all_windows: powershell cmd

install_bash $SHELL="bash":
	#!/bin/bash
	msg=$(target/$TARGET/debug/install --install 2>&1)
	if [ $? -ne 0 ]; then
		echo "$msg"
		echo "Error: failed to run install"
		exit 1
	fi
	echo "$msg"
	src=$(echo "$msg" | tail -n 1 | head -n 1 | awk '{print $(NF)}')
	if [ -z $src ]; then
		echo "Error: can not find source file"
		exit 1
	fi
	export __SRC_FILE=$src

install_fish $SHELL="fish":
	#!/bin/fish
	msg=$(target/$TARGET/debug/install --install 2>&1)
	if [ $? -ne 0 ]; then
		echo "$msg"
		echo "Error: failed to run install"
		exit 1
	fi
	echo "$msg"
	src=$(echo "$msg" | tail -n 1 | head -n 1 | awk '{print $(NF)}')
	if [ -z $src ]; then
		echo "Error: can not find source file"
		exit 1
	fi
	export __SRC_FILE=$src

install_zsh $SHELL="zsh":
	#!/bin/zsh
	msg=$(target/$TARGET/debug/install --install 2>&1)
	if [ $? -ne 0 ]; then
		echo "$msg"
		echo "Error: failed to run install"
		exit 1
	fi
	echo "$msg"
	src=$(echo "$msg" | tail -n 1 | head -n 1 | awk '{print $(NF)}')
	if [ -z $src ]; then
		echo "Error: can not find source file"
		exit 1
	fi
	export __SRC_FILE=$src

install_tcsh $SHELL="tcsh":
	#!/bin/tcsh
	msg=$(target/$TARGET/debug/install --install 2>&1)
	if [ $? -ne 0 ]; then
		echo "$msg"
		echo "Error: failed to run install"
		exit 1
	fi
	echo "$msg"
	src=$(echo "$msg" | tail -n 1 | head -n 1 | awk '{print $(NF)}')
	if [ -z $src ]; then
		echo "Error: can not find source file"
		exit 1
	fi
	export __SRC_FILE=$src

uninstall_bash $SHELL="bash":
	#!/bin/bash
	target/$TARGET/debug/install --uninstall

uninstall_fish $SHELL="fish":
	#!/bin/fish
	target/$TARGET/debug/install --uninstall

uninstall_zsh $SHELL="zsh":
	#!/bin/zsh
	target/$TARGET/debug/install --uninstall

uninstall_tcsh $SHELL="tcsh":
	#!/bin/tcsh
	target/$TARGET/debug/install --uninstall

install_win:
	#!powershell
	target\\{{env_var_or_default("TARGET", "")}}\\debug\\install --install

uninstall_win:
	#!powershell
	target\\{{env_var_or_default("TARGET", "")}}\\debug\\install --uninstall

tests_cmd:
	#!cmd /c
	j
	j -s
	cd
	j -s
	cd target
	j -s
	j --add debug
	j -s

tests_powershell:
	#!powershell
	j
	j -s
	cd
	j -s
	cd target
	j -s
	j --add debug
	j -s

tests_bash $SHELL="bash":
	#!/usr/bin/env bash
	source {{env_var("__SRC_FILE")}}
	cd
	j
	j -s
	cd ..
	j -s

tests_fish $SHELL="fish":
	#!/usr/bin/env fish
	source {{env_var("__SRC_FILE")}}
	cd
	j
	j -s
	cd ..
	j -s

tests_zsh $SHELL="zsh":
	#!/usr/bin/env zsh
	source {{env_var("__SRC_FILE")}}
	cd
	j
	j -s
	cd ..
	j -s

tests_tcsh $SHELL="tcsh":
	#!/usr/bin/env tcsh
	source {{env_var("__SRC_FILE")}}
	cd
	j
	j -s
	cd ..
	j -s

bash: install_bash
	just tests_bash
	just uninstall_bash

fish: install_fish
	just tests_fish
	just uninstall_fish

zsh: install_zsh
	just tests_zsh
	just uninstall_zsh

tcsh: install_tcsh
	just tests_tcsh
	just uninstall_tcsh

cmd: install_win
	just tests_cmd
	just uninstall_win

powershell: install_win
	just tests_powershell
	just uninstall_win
