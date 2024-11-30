use dobutsu_shogi_core::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone)]
pub struct DobutsuShogiService {
    buffer: Box<[u8]>,
    cursor: usize,
}

#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct BufferAddress(usize);

#[wasm_bindgen]
impl DobutsuShogiService {
    pub fn new(state_capacity: usize) -> DobutsuShogiService {
        DobutsuShogiService {
            buffer: vec![0; state_capacity * 8].into_boxed_slice(),
            cursor: 0,
        }
    }

    pub fn buffer(&self) -> *const u8 {
        self.buffer.as_ptr()
    }

    pub fn cursor(&self) -> BufferAddress {
        BufferAddress(self.cursor)
    }

    pub fn clear(&mut self) {
        self.cursor = 0;
    }

    pub fn write_initial_state(&mut self) -> BufferAddress {
        self.write_state(State::initial())
    }

    pub fn write_children(&mut self, parent_address: BufferAddress) -> BufferAddress {
        let len_address = BufferAddress(self.cursor);
        self.cursor += 8;
        let parent = self.read_state(parent_address);
        let mut len = 0u64;
        parent.visit_children(|child| {
            self.write_state(child);
            len += 1;
        });
        self.buffer[len_address.0..len_address.0 + 8].copy_from_slice(&len.to_le_bytes());
        len_address
    }
}

impl DobutsuShogiService {
    fn write_state(&mut self, state: State) -> BufferAddress {
        if self.cursor >= self.buffer.len() {
            panic!(
                "Dobutsu shogi service buffer overflowed limit of {}.",
                self.buffer.len()
            );
        }

        let location = self.cursor;
        self.buffer[self.cursor..8].copy_from_slice(&state.0.to_le_bytes());
        self.cursor += 8;
        BufferAddress(location)
    }

    fn read_state(&self, address: BufferAddress) -> State {
        State(u64::from_le_bytes(
            self.buffer[address.0..address.0 + 8].try_into().unwrap(),
        ))
    }
}
