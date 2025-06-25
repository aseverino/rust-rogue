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

SPAWNERS = {}

function on_map_peeked(map)
    -- SPAWNERS = {}
    -- local monster_types = map:get_monster_types()
    -- for _, monster_type in ipairs(monster_types) do
    --     SPAWNERS[monster_type] = 0
    -- end

    local tiles = map:get_walkable_tiles()
    -- select two random tiles, but careful not to select the same tile twice
    local tile1 = tiles[math.random(#tiles)]
    local tile2
    print('oi2')
    repeat
        print('oi3')
        tile2 = tiles[math.random(#tiles)]
    until tile1 ~= tile2
    print('oi4')

    print('adding monsters')

    for k, v in pairs(tile1) do
        print('tile1: ' .. k .. ' = ' .. v)
    end

    print(map)
    print(map.add_monster)
    map:add_monster(0, tile1)
    map:add_monster(0, tile2)
end