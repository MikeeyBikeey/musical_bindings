---@diagnostic disable: undefined-global
-- Simply makes the player character walk left and right in Terraria depending on pitch.

input_mode = 'keyboard'

function accepts(window)
    return window:find('^Terraria:') ~= nil
end

local pressing_a = false
local pressing_d = false

function process()
    if power > 0.025 and pitch > 600.0 then
        if not pressing_d then
            keys_down('D')
            pressing_d = true
        end
    else
        if pressing_d then
            keys_up('D')
            pressing_d = false
        end
    end

    if power > 0.025 and pitch < 600.0 then
        if not pressing_a then
            keys_down('A')
            pressing_a = true
        end
    else
        if pressing_a then
            keys_up('A')
            pressing_a = false
        end
    end
end
