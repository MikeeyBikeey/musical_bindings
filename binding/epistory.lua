---@diagnostic disable: undefined-global
-- Inputs key presses for Epistory. Works best with a recorder and an Epistory mod that limits text to musical notes.

input_mode = 'keyboard'

function accepts(window)
    return window == 'Epistory'
end

local pressing_note = nil

function process()
    if power > 0.025 then
        if note ~= pressing_note then
            if pressing_note then
                keys_up(pressing_note)
            end
            keys_down(note)
            pressing_note = note
        end
    else
        if pressing_note then
            keys_up(pressing_note)
        end
        pressing_note = nil
    end
end
