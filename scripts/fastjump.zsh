export FASTJUMP_SOURCED=1

# set user installation paths
if [[ -d ~/.fastjump/bin ]]; then
    path=(~/.fastjump/bin ${path})
fi
if [[ -d ~/.fastjump/functions ]]; then
    fpath=(~/.fastjump/functions ${fpath})
fi


# set homebrew installation paths
if command -v brew &>/dev/null; then
  local brew_prefix=${BREW_PREFIX:-$(brew --prefix)}
  if [[ -d "${brew_prefix}/share/zsh/site-functions" ]]; then
    fpath=("${brew_prefix}/share/zsh/site-functions" ${fpath})
  fi
fi


# set error file location
if [[ "$(uname)" == "Darwin" ]]; then
    export FASTJUMP_ERROR_PATH=~/Library/fastjump/errors.log
elif [[ -n "${XDG_DATA_HOME}" ]]; then
    export FASTJUMP_ERROR_PATH="${XDG_DATA_HOME}/fastjump/errors.log"
else
    export FASTJUMP_ERROR_PATH=~/.local/share/fastjump/errors.log
fi

if [[ ! -d ${FASTJUMP_ERROR_PATH:h} ]]; then
    mkdir -p ${FASTJUMP_ERROR_PATH:h}
fi


# change pwd hook
fastjump_chpwd() {
    if [[ -f "${FASTJUMP_ERROR_PATH}" ]]; then
        fastjump --add "$(pwd)" >/dev/null 2>>${FASTJUMP_ERROR_PATH} &!
    else
        fastjump --add "$(pwd)" >/dev/null &!
    fi
}

typeset -gaU chpwd_functions
chpwd_functions+=fastjump_chpwd


# default fastjump command
j() {
    if [[ ${1} == -* ]] && [[ ${1} != "--" ]]; then
        fastjump ${@}
        return
    fi

    setopt localoptions noautonamedirs
    local output="$(fastjump ${@})"
    if [[ -d "${output}" ]]; then
        if [ -t 1 ]; then  # if stdout is a terminal, use colors
                echo -e "\\033[31m${output}\\033[0m"
        else
                echo -e "${output}"
        fi
        cd "${output}"
    else
        echo "fastjump: directory '${@}' not found"
        echo "\n${output}\n"
        echo "Try \`fastjump --help\` for more information."
        false
    fi
}


# jump to child directory (subdirectory of current path)
jc() {
    if [[ ${1} == -* ]] && [[ ${1} != "--" ]]; then
        fastjump ${@}
        return
    else
        j $(pwd) ${@}
    fi
}


# open fastjump results in file browser
jo() {
    if [[ ${1} == -* ]] && [[ ${1} != "--" ]]; then
        fastjump ${@}
        return
    fi

    setopt localoptions noautonamedirs
    local output="$(fastjump ${@})"
    if [[ -d "${output}" ]]; then
        case ${OSTYPE} in
            linux*)
                xdg-open "${output}"
                ;;
            darwin*)
                open "${output}"
                ;;
            cygwin)
                cygstart "" $(cygpath -w -a ${output})
                ;;
            *)
                echo "Unknown operating system: ${OSTYPE}" 1>&2
                ;;
        esac
    else
        echo "fastjump: directory '${@}' not found"
        echo "\n${output}\n"
        echo "Try \`fastjump --help\` for more information."
        false
    fi
}


# open fastjump results (child directory) in file browser
jco() {
    if [[ ${1} == -* ]] && [[ ${1} != "--" ]]; then
        fastjump ${@}
        return
    else
        jo $(pwd) ${@}
    fi
}
