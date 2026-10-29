-- Starts the built LAST SERVICE (games/last_service/build) from its own folder.
-- dm alias ls-run
local exe = "games/last_service/build/lastservice.exe"
if not fs.exists(exe) then
    log.error("no build: run `dm alias ls-build` first")
    return
end
-- Run from its folder (it finds game.pak next to itself); long brackets keep the backslashes.
dm_os.spawn([[cd /d games\last_service\build && .\lastservice.exe]])
log.info("LAST SERVICE started")
