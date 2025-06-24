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

use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, RwLock},
};

use crate::{position::Position, ui::point_f::PointF};

pub trait Creature {
    fn name(&self) -> &str;
    fn pos(&self) -> Position;
    fn set_pos(&mut self, pos: Position);
    fn draw(&self, offset: PointF);

    fn add_health(&mut self, amount: i32);

    fn get_health(&self) -> (u32, u32);

    fn is_player(&self) -> bool {
        false
    }
    fn is_monster(&self) -> bool {
        false
    }
    fn as_any(&self) -> &dyn std::any::Any;
}

// pub type CreatureRef = Arc<RwLock<dyn Creature>>;

// pub fn downcast_rc_creature<T: 'static>(rc: &Rc<dyn Creature>) -> Option<Rc<T>> {
//     if rc.as_any().is::<T>() {
//         // SAFETY: we just checked type, so we can clone Rc and transmute its type
//         let raw = Rc::as_ptr(rc) as *const T;
//         let cloned = unsafe { Rc::from_raw(raw) };
//         let result = Rc::clone(&cloned);
//         std::mem::forget(cloned); // avoid dropping original
//         Some(result)
//     } else {
//         None
//     }
// }
