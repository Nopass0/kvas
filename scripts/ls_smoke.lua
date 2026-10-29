-- Smoke run of the built LAST SERVICE: start it, take the menu, start a new shift, walk to the
-- diner, open the front door, walk in, and screenshot each moment into
-- games/last_service/build/shots/. dm alias ls-smoke
--
-- Clicks and screenshots are relative to the game window, so any window size works.
local out = "games/last_service/build/shots"
fs.mkdir(out)

for _, pid in ipairs(proc.find("lastservice")) do
    proc.kill(pid)
end
-- Run from its folder; long brackets keep the backslashes.
dm_os.spawn([[cd /d games\last_service\build && .\lastservice.exe]])

local win = { found = false }
for _ = 1, 60 do
    dm_os.sleep(500)
    win = auto.find_window("LAST SERVICE")
    if win.found ~= false and win.w and win.w > 200 then
        break
    end
end
assert(win.found ~= false, "the game window did not open")
dm_os.sleep(5000)
auto.activate_window(win)
dm_os.sleep(800)

local function shot(name)
    win = auto.find_window("LAST SERVICE")
    auto.screenshot_region(out .. "/" .. name .. ".png", win.x, win.y, win.w, win.h)
    log.info("shot " .. name)
end

local function click(fx, fy)
    auto.click(math.floor(win.x + win.w * fx), math.floor(win.y + win.h * fy))
end

local function hold(key, ms)
    auto.key_down(key)
    dm_os.sleep(ms)
    auto.key_up(key)
end

shot("01_menu")
-- "New shift" is the first entry when there is no save; with a save, the second one plus "Yes".
click(0.14, 0.5)
dm_os.sleep(1200)
shot("02_after_click")
auto.key_press("space")
dm_os.sleep(2500)
shot("03_start")
hold("w", 2600)
dm_os.sleep(300)
shot("04_door")
auto.key_press("e")
dm_os.sleep(1500)
shot("05_door_open")
hold("w", 2000)
dm_os.sleep(500)
shot("06_inside")
for _, pid in ipairs(proc.find("lastservice")) do
    proc.kill(pid)
end
log.info("shots in " .. out)
