# the login $SHELL isn't always the one used
# NOTE: problems might occur if /bin/sh is symlinked to /bin/bash
if [ -n "${BASH}" ]; then
    shell="bash"
elif [ -n "${ZSH_NAME}" ]; then
    shell="zsh"
elif [ -n "${__fish_datadir}" ]; then
    shell="fish"
elif [ -n "${version}" ]; then
    shell="tcsh"
else
    shell=$(echo ${SHELL} | awk -F/ '{ print $NF }')
fi

# prevent circular loop for sh shells
if [ "${shell}" = "sh" ]; then
    return 0

# check local install
elif [ -s ~/.fastjump/share/fastjump/fastjump.${shell} ]; then
    source ~/.fastjump/share/fastjump/fastjump.${shell}

# check global install
elif [ -s /usr/local/share/fastjump/fastjump.${shell} ]; then
    source /usr/local/share/fastjump/fastjump.${shell}
fi
