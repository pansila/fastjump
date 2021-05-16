all: (sh 'bash') (sh 'fish') (sh 'zsh') (sh 'tcsh')
#all_windows: powershell cmd
all_windows: powershell

install_sh shell:
	#!/usr/bin/env {{shell}}
	export SHELL={{shell}}
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
	echo $src

uninstall_sh shell:
	#!/usr/bin/env {{shell}}
	export SHELL={{shell}}
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

tests_bash rcfile $SHELL="bash":
	#!/usr/bin/env bash
	source {{rcfile}}
	cd
	j
	j -s
	cd ..
	j -s

tests_fish rcfile $SHELL="fish":
	#!/usr/bin/env fish
	source {{rcfile}}
	cd
	j
	j -s
	cd ..
	j -s

tests_zsh rcfile $SHELL="zsh":
	#!/usr/bin/env zsh
	source {{rcfile}}
	cd
	j
	j -s
	cd ..
	j -s

tests_tcsh rcfile $SHELL="tcsh":
	#!/usr/bin/env tcsh
	source {{rcfile}}
	cd
	j
	j -s
	cd ..
	j -s

sh shell:
	#!/usr/bin/env {{shell}}
	export SHELL={{shell}}
	rcfile=$(just install_sh {{shell}})
	just tests_{{shell}} $rcfile
	just uninstall_sh {{shell}}

cmd: install_win
	just tests_cmd
	just uninstall_win

powershell: install_win
	just tests_powershell
	just uninstall_win
