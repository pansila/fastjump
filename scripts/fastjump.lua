local FASTJUMP_DIR = debug.getinfo(1, "S").source:match[[^@?(.*[\/])[^\/]-$]] .. "..\\FastJump"
local FASTJUMP_BIN_DIR = FASTJUMP_DIR .. "\\bin"
local FASTJUMP_BIN = (FASTJUMP_BIN_DIR or clink.get_env("LOCALAPPDATA") .. "\\fastjump\\bin") .. "\\fastjump"

function fastjump_add_to_database()
  os.execute("\"" .. FASTJUMP_BIN .. "\"" .. " --add " .. "\"" .. clink.get_cwd() .. "\"" .. " 2> " .. clink.get_env("TEMP") .. "\\fastjump_error.txt")
end

clink.prompt.register_filter(fastjump_add_to_database, 99)

function fastjump_completion(word)
  for line in io.popen("\"" .. FASTJUMP_BIN .. "\"" ..  " --complete " .. word):lines() do
    clink.add_match(line)
  end
  return {}
end

local fastjump_parser = clink.arg.new_parser()
fastjump_parser:set_arguments({ fastjump_completion })

clink.arg.register_parser("j", fastjump_parser)
