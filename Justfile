all: bash fish zsh tcsh
all_windows: powershell cmd

install:
	#!/bin/bash --login
	$src=$(target/$TARGET/debug/install --install | tail -n 1 | head -n 1 | awk '{print $(NF)}')
	if [ $? -ne 0 ]; then
		exit 1
	fi
	export __SRC_FILE=$src

uninstall:
	#!/bin/sh
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

tests_bash:
	#!/usr/bin/env bash
	source {{env_var(__SRC_FILE)}}
	cd
	j
	j -s
	cd ..
	j -s

tests_fish:
	#!/usr/bin/env fish
	source {{env_var(__SRC_FILE)}}
	cd
	j
	j -s
	cd ..
	j -s

tests_zsh:
	#!/usr/bin/env zsh
	source {{env_var(__SRC_FILE)}}
	cd
	j
	j -s
	cd ..
	j -s

tests_tcsh:
	#!/usr/bin/env tcsh
	source {{env_var(__SRC_FILE)}}
	cd
	j
	j -s
	cd ..
	j -s

bash: install
	just tests-bash
	just uninstall

fish: install
	just tests-fish
	just uninstall

zsh: install
	just tests-zsh
	just uninstall

tcsh: install
	just tests-tcsh
	just uninstall

cmd: install_win
	just tests_cmd
	just uninstall_win

powershell: install_win
	just tests_powershell
	just uninstall_win
