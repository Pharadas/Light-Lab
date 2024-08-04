use std::mem::transmute;

use math_vector::Vector;

#[derive(Debug, Clone, Copy)]
pub struct KeyValue {
    key: u32,
    value: u32
}

#[derive(Debug)]
pub struct GPUHashTable {
    list: Vec<KeyValue>,
    block_size: Vector<u32>
}

impl GPUHashTable {
    pub fn new(block_size: Vector<u32>) -> GPUHashTable {
        return GPUHashTable {
            list: vec![KeyValue {key: 0, value: 0}; 1000],
            block_size
        };
    }

    fn hash(&self, val: Vector<u32>) -> usize {
        // we reserve 0 as a free space
        return ((val.x + self.block_size.y * (val.y + self.block_size.z * val.z)) % 1000 as u32) as usize + 1;
    }

    fn resize(&mut self) {} // TODO

    pub fn insert(&mut self, key: Vector<u32>, val: u32)  {
        let mut current_index = self.hash(key);
        let original_hash = current_index;

        loop {
            if current_index == self.list.capacity() {
                self.resize();
                current_index = self.hash(key);
            }

            // we had already stored this exact value
            if self.list[current_index].key == original_hash as u32 && self.list[current_index].value == val as u32 {
                break;
            }

            // we found an available opening
            if self.list[current_index].key == 0 {
                self.list[current_index] = KeyValue {key: original_hash as u32, value: val as u32};
                break;
            }

            current_index += 1;
        }
    }

    pub unsafe fn opengl_compatible_list(&self) -> &[i32] {
        return transmute(self.list.as_slice());
    }
}
