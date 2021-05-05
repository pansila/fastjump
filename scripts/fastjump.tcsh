# set user installation paths
if (-d ~/.fastjump/bin) then
    set path = (~/.fastjump/bin path)
endif

# prepend fastjump to cwdcmd (run after every change of working directory)
if (`alias cwdcmd` !~ *fastjump*) then
    alias cwdcmd 'fastjump --add $cwd >/dev/null;' `alias cwdcmd`
endif

#default fastjump command
alias j 'cd `fastjump -- \!:1`'
