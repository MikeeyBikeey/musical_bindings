---@diagnostic disable: undefined-global
-- Writes the currently detected musical note from the microphone to a Windows 10 Notepad

-- `keyboard` mode because this script doesn't use emojis or special characters
input_mode = 'keyboard'

-- Only accepts windows that end with " - Notepad" in the title
function accepts(window)
    return window:find(' - Notepad$') ~= nil
end

local pressing_note = nil

function process()
    -- Makes sure the user is loud enough before pressing keys
    if power > 0.025 then
        if note ~= pressing_note then
            if pressing_note then
                keys_up(pressing_note) -- Un-presses previous note
            end
            keys_down(note) -- Presses current note
            pressing_note = note
        end
    else
        if pressing_note then
            keys_up(pressing_note) --Un-presses note because user is too quiet
        end
        pressing_note = nil
    end
end
