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

GlobalData = {}

function on_map_peeked(map)
    if not GlobalData.SPAWNERS then
        GlobalData.SPAWNERS = {}
    end
    local tiles = map:get_walkable_tiles()
    local monster_types = map:get_monster_types()
    
    -- shuffle the tiles to ensure randomness
    for i = #tiles, 2, -1 do
        local j = math.random(i)
        tiles[i], tiles[j] = tiles[j], tiles[i]
    end

    local spawners_count = math.max(map:get_tier(), 2)
    local spawners = {}

    for i = 1, spawners_count do
        -- select a random tile from the shuffled list
        local tile = tiles[i]
        local monster = map:add_monster(0, tile)
        GlobalData.SPAWNERS[monster:get_id()] = monster_types[i % #monster_types + 1]
    end
end