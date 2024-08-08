use math_vector::Vector;
use web_sys::console;

#[derive(Debug, Clone, Copy)]
pub struct KeyValue {
    key: u32,
    value: u32,
    next: u32
}

#[derive(Debug)]
pub struct GPUHashTable {
    pub buckets: Vec<u32>,
    objects: Vec<KeyValue>,
    objects_last: u32,
    block_size: Vector<u32>
}

impl GPUHashTable {
    pub fn new(block_size: Vector<u32>) -> GPUHashTable {
        return GPUHashTable {
            buckets: vec![u32::MAX; 1000], // capacity hard coded for now
            objects: vec![KeyValue {key: 0, value: 0, next: u32::MAX}; 1000], // capacity hard coded for now
            objects_last: 0,
            block_size
        };
    }

    fn hash(&self, val: Vector<u32>) -> u32 {
        // we reserve 0 as a free space
        return val.x + self.block_size.y * (val.y + self.block_size.z * val.z);
    }

    fn resize(&mut self) {} // TODO

    pub fn insert(&mut self, key: Vector<u32>, val: u32)  {
        // console::log_1(&self.objects_last.into());

        if self.objects_last == 1000 {
            console::log_1(&format!("{:?}", self.objects).into());
            console::log_1(&format!("{:?}", self.buckets).into());
        }

        let original_hash = self.hash(key);
        let index = (original_hash % 1000) as usize;
        let mut current_object: KeyValue;

        if self.buckets[index] == u32::MAX { // this bucket hasnt been used, just add the item at
                                             // the end of self.objects
            self.buckets[index] = self.objects_last;
            self.objects[self.objects_last as usize] = KeyValue {key: original_hash, value: val, next: u32::MAX};
            self.objects_last += 1;
            return;
        }

        // if we reach this part of the code, then this bucket isn't empty, let's find the last item of
        // the bucket by following it as a linked list
        current_object = self.objects[self.buckets[index] as usize];
        let mut last_next = self.buckets[index];

        while current_object.next != u32::MAX {
            if current_object.value == val && current_object.key == original_hash { // in case we already stored this same value
                console::log_1(&format!("key {:?} was already stored", current_object.key).into());
                return;
            }

            last_next = current_object.next;
            current_object = self.objects[current_object.next as usize];
        }

        // the current object has no next, so let's add it
        self.objects[last_next as usize].next = self.objects_last;
        self.objects[self.objects_last as usize] = KeyValue {key: original_hash, value: val, next: u32::MAX};
        self.objects_last += 1;

        console::log_2(&format!("{:?}", key).into(), &format!("{:?}", original_hash).into());
    }

    // terrible stuff
    pub fn opengl_compatible_objects_list(&self, list_to_fill: &mut [u32]) {
        for (i, key_val) in self.objects.iter().enumerate() {
            list_to_fill[i * 3] = key_val.key;
            list_to_fill[(i * 3) + 1] = key_val.value;
            list_to_fill[(i * 3) + 2] = key_val.next;
        }
    }
}
