export FASTJUMP_SOURCED=1

# set user installation paths
if [[ -d ~/.fastjump/bin ]]; then
    export PATH=~/.fastjump/bin:"${PATH}"
fi


# set error file location
if [[ "$(uname)" == "Darwin" ]]; then
    export FASTJUMP_ERROR_PATH=~/Library/fastjump/errors.log
elif [[ -n "${XDG_DATA_HOME}" ]]; then
    export FASTJUMP_ERROR_PATH="${XDG_DATA_HOME}/fastjump/errors.log"
else
    export FASTJUMP_ERROR_PATH=~/.local/share/fastjump/errors.log
fi

if [[ ! -d "$(dirname ${FASTJUMP_ERROR_PATH})" ]]; then
    mkdir -p "$(dirname ${FASTJUMP_ERROR_PATH})"
fi


# enable tab completion
_fastjump() {
        local cur
        cur=${COMP_WORDS[*]:1}
        comps=$(fastjump --complete $cur)
        while read i; do
            COMPREPLY=("${COMPREPLY[@]}" "${i}")
        done <<EOF
        $comps
EOF
}
complete -F _fastjump j


# change pwd hook
fastjump_add_to_database() {
    if [[ -f "${FASTJUMP_ERROR_PATH}" ]]; then
        (fastjump --add "$(pwd)" >/dev/null 2>>${FASTJUMP_ERROR_PATH} &) &>/dev/null
    else
        (fastjump --add "$(pwd)" >/dev/null &) &>/dev/null
    fi
}

while IFS= read -r line; do
    IFS=' '
    read -ra fields <<<$line
    if [[ ${fields[1]} =~ ^cd$ ]]; then
        if [[ -f "${FASTJUMP_ERROR_PATH}" ]]; then
            (fastjump --add ${fields[2]} >/dev/null 2>>${FASTJUMP_ERROR_PATH} &) &>/dev/null
        else
            (fastjump --add ${fields[2]} >/dev/null &) &>/dev/null
        fi
    fi
done <<< "$(history)"

case $PROMPT_COMMAND in
    *fastjump*)
        ;;
    *)
        PROMPT_COMMAND="${PROMPT_COMMAND:+$(echo "${PROMPT_COMMAND}" | awk '{gsub(/; *$/,"")}1') ; }fastjump_add_to_database"
        ;;
esac


# default fastjump command
j() {
    if [[ ${1} == -* ]] && [[ ${1} != "--" ]]; then
        fastjump ${@}
        return
    fi

    output="$(fastjump ${@})"
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

    output="$(fastjump ${@})"
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
                echo "Unknown operating system: ${OSTYPE}." 1>&2
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
