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

---@class Position
---@field x integer
---@field y integer
Position = {}

---@meta
---@param monster Monster
---@return boolean
function on_spawn(monster) end

---@param monster Monster
---@return boolean
function on_update(monster) end

---@param monster Monster
---@return boolean
function on_death(monster) end

---@param map Map
function on_map_peeked(map) end

---@param weapon Weapon
---@param player Player
---@param target Player|Monster
---@return integer
function on_get_attack_damage(weapon, player, target) end

---@return Map
function get_current_map() end

---@param id integer
---@return MonsterKind
function get_monster_kind_by_id(id) end