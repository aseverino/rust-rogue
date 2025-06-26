-- SPDX-License-Identifier: MIT
--
-- Copyright (c) 2025 Alexandre Severino
--
-- Permission is hereby granted, free of charge, to any person obtaining a copy
-- of this software and associated documentation files (the "Software"), to deal
-- in the Software without restriction, including without limitation the rights
-- to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
-- copies of the Software, and to permit persons to whom the Software is
-- furnished to do so, subject to the following conditions:
--
-- The above copyright notice and this permission notice shall be included in
-- all copies or substantial portions of the Software.
--
-- THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
-- IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
-- FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
-- AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
-- LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
-- OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
-- SOFTWARE.

local turn_count = 1
local spawner_type = nil

function on_spawn(monster)
    if not GlobalData.SPAWNERS then
        return false
    end
    
    spawner_type = GlobalData.SPAWNERS[monster:get_id()]
    if not spawner_type then
        return false
    end

    return true
end

function on_update(monster)
    if not spawner_type then return false end

    if turn_count % 3 == 0 then
        local pos = monster:get_position()
        local map = get_current_map()

        pos = map:get_random_adjacent_walkable_position(pos)

        map:add_monster(spawner_type, pos)
    end

    turn_count = turn_count + 1
end