// Vendored version of https://crates.io/crates/debouncing, without alloc

// MIT License
//
// Copyright (c) 2024 Florian Finkernagel]
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

// use no_std_compat::prelude::v1::*;

pub struct Debouncer<'a> {
    patterns: &'a mut [u8],
}

#[repr(u8)]
#[derive(PartialEq)]
pub enum DebounceResult {
    NoChange,
    Pressed,
    Released
}

impl<'a> Debouncer<'a> {
    pub fn new(slice: &'a mut [u8]) -> Debouncer<'a> {
        Debouncer{
            patterns: slice
        }
    }

    pub fn update(&mut self, key_no: usize, pressed: bool) -> DebounceResult
    {
        let next: u8 = if pressed {1} else {0};
        self.patterns[key_no] = self.patterns[key_no] << 1 | next;
        let mut result = DebounceResult::NoChange;
        //debounce following hackadays ultimate debouncing schema
        let mask: u8 = 0b11000111;
        let seen = self.patterns[key_no] & mask;
        if seen == 0b00000111 {
            result = DebounceResult::Pressed;
            self.patterns[key_no] = 0b1111111;
        }
        else if seen == 0b11000000 {
            result = DebounceResult::Released;
            self.patterns[key_no] = 0b0000000;
        }

        return result;
    }
}

#[cfg(test)]
mod tests {
    use crate::debouncing::{DebounceResult, Debouncer};
    #[test]
    fn test_it_works() {
        let mut storage = [1];
        let mut db = Debouncer::new(&mut storage);
        //activate
        assert!(db.update(0, true) == DebounceResult::NoChange);
        assert!(db.update(0, true) == DebounceResult::NoChange);
        assert!(db.update(0, true) == DebounceResult::Pressed);
        //deactivate
        assert!(db.update(0, false) == DebounceResult::NoChange);
        assert!(db.update(0, false) == DebounceResult::NoChange);
        assert!(db.update(0, false) == DebounceResult::Released);

        //let's do noise.
        assert!(db.update(0, true) == DebounceResult::NoChange);
        assert!(db.update(0, false) == DebounceResult::NoChange);
        assert!(db.update(0, false) == DebounceResult::NoChange);
        assert!(db.update(0, false) == DebounceResult::NoChange);
        assert!(db.update(0, false) == DebounceResult::NoChange);
        assert!(db.update(0, false) == DebounceResult::NoChange);
        assert!(db.update(0, false) == DebounceResult::NoChange);

        assert!(db.update(0, true) == DebounceResult::NoChange);
        assert!(db.update(0, true) == DebounceResult::NoChange);
        assert!(db.update(0, true) == DebounceResult::Pressed);
        assert!(db.update(0, true) == DebounceResult::NoChange);
        assert!(db.update(0, false) == DebounceResult::NoChange);
        assert!(db.update(0, false) == DebounceResult::NoChange);
        assert!(db.update(0, true) == DebounceResult::NoChange);
        assert!(db.update(0, true) == DebounceResult::NoChange);
        assert!(db.update(0, true) == DebounceResult::NoChange);


    }
}