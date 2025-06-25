// SPDX-License-Identifier: MIT
//
// Copyright (c) 2025 Alexandre Severino
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use rand::{Rng, thread_rng};

use crate::{
    creature::Creature,
    lua_interface::{LuaInterfaceRc, LuaScripted},
    maps::map::{Map, MapRef},
    player::Player,
    position::Position,
    tile::{NO_CREATURE, PLAYER_CREATURE_ID},
};

fn do_damage(
    player: &mut Player,
    map_ref: &MapRef,
    target_id: u32,
    damage: i32,
    lua_interface: &LuaInterfaceRc,
) {
    let mut map = map_ref.borrow_mut();
    let target: &mut dyn Creature = if target_id == PLAYER_CREATURE_ID as u32 {
        player as &mut dyn Creature
    } else {
        map.monsters
            .get_mut(&target_id)
            .expect("Target creature not found") as &mut dyn Creature
    };

    // Scope to auto-drop the first lock before the second
    {
        target.add_health(-damage);
        println!("{} takes {} damage!", target.name(), damage);

        if target.get_health().0 <= 0 {
            let target_pos = target.pos();
            let target_name = target.name().to_string();
            map.generated_map.tiles[target_pos].creature = NO_CREATURE;
            println!("{} has been defeated!", target_name);
        } else {
            println!("{} has {} HP left.", target.name(), target.get_health().0);
            return;
        }
    } // <-- drops write lock here

    // Now safe to lock again
    if target_id != PLAYER_CREATURE_ID as u32 {
        {
            let mut monster = map
                .monsters
                .get_mut(&target_id)
                .unwrap_or_else(|| panic!("Error on do_damage: no monster."))
                .clone();
            drop(map);
            if monster.kind.is_scripted() {
                // let r = lua_interface.borrow_mut().on_death(&mut monster);
                // // Re-lock the map to remove the monster
                // let mut map = map_ref.borrow_mut();
                // // update the monster in the map from Lua code
                // *map.monsters
                //     .get_mut(&target_id)
                //     .expect("Target creature not found") = monster;
                // if let Err(e) = r {
                //     eprintln!("Error calling Lua on_death: {}", e);
                // }
            }
        }

        {
            let mut map = map_ref.borrow_mut();
            map.monsters.remove(&target_id);
        }
    }
}

pub(crate) fn do_melee_combat(
    player: &mut Player,
    map_ref: &mut MapRef,
    _attacker_pos: Position,
    target_pos: Position,
    lua_interface: &LuaInterfaceRc,
) {
    let damage = {
        if player.equipment.weapon.is_some() {
            // Temporarily take the weapon out to avoid aliasing
            let mut weapon = player.equipment.weapon.take().unwrap();

            let mut damage: u32 = 0;

            let (target_id, mut monster) = {
                let map = map_ref.borrow_mut();
                let target_id = map.generated_map.tiles[target_pos].creature;
                (
                    target_id,
                    map.monsters
                        .get(&target_id)
                        .unwrap_or_else(|| panic!("Error on do_melee_combat: no monster."))
                        .clone(),
                )
            };

            if weapon.is_scripted() {
                // let lua_result = lua_interface.borrow_mut().on_get_attack_damage(
                //     &mut weapon,
                //     player,
                //     &mut monster,
                // );

                // update the monster in the map from Lua code
                *map_ref
                    .borrow_mut()
                    .monsters
                    .get_mut(&target_id)
                    .expect("Target creature not found") = monster;

                // match lua_result {
                //     Ok(lua_damage) => {
                //         damage = lua_damage as u32;
                //         println!("Damage from Lua script: {}", damage);
                //     }
                //     Err(e) => {
                //         eprintln!("Error calling Lua on_get_attack_damage: {}", e);
                //     }
                // }
            } else {
                for &d in weapon.attack_dice.iter() {
                    let mut rng = thread_rng();
                    let roll = rng.gen_range(1..=d);
                    damage += roll + weapon.base_holdable.modifier as u32;
                }
            }

            // Put the weapon back
            player.equipment.weapon = Some(weapon);

            damage
        } else {
            1u32
        }
    };

    let creature_id = map_ref.borrow().generated_map.tiles[target_pos].creature;
    if creature_id >= 0 {
        do_damage(
            player,
            map_ref,
            creature_id as u32,
            damage as i32,
            lua_interface,
        );
    }
}

pub(crate) fn do_spell_combat(
    player: &mut Player,
    map_ref: &MapRef,
    _attacker_pos: Position,
    target_pos: Position,
    spell_index: usize,
    lua_interface: &LuaInterfaceRc,
) {
    let map = map_ref.borrow_mut();
    if !map.is_tile_walkable(target_pos) {
        println!("Target position is not walkable for spell casting.");
        return;
    }

    let spell = player
        .spells
        .get_mut(spell_index)
        .expect("Selected spell index out of bounds");

    let damage = spell.spell_type.basepower as i32;

    let mut target_positions: Vec<Position> = Vec::new();
    let mut target_creatures: Vec<u32> = Vec::new();

    map.spell_fov_cache.area.iter().for_each(|&pos| {
        target_positions.push(pos);
        let creature_id = map.generated_map.tiles[pos].creature;
        if creature_id >= 0 {
            target_creatures.push(creature_id as u32);
        }
    });

    drop(map);
    for target_creature in target_creatures {
        do_damage(player, &map_ref, target_creature, damage, lua_interface);
    }

    // let target = self.monsters.get_mut(target_creature as usize)
    //     .expect("Target creature not found");
    // target.hp -= damage;
    // println!("{} takes {} damage!", target.name(), damage);

    // if target.hp <= 0 {
    //     self.tiles[target_pos].creature = NO_CREATURE; // Remove monster from tile
    //     println!("{} has been defeated!", target.name());
    //     // Optionally, remove the monster from the list
    //     // self.monsters.remove(target_creature as usize);
    // } else {
    //     println!("{} has {} HP left.", target.name(), target.hp);
    // }
}
