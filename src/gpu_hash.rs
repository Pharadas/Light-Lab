use std::u32;

use nalgebra::Vector3;
use web_sys::console;

#[derive(Debug, Clone, Copy)]
pub struct KeyValue {
    key: u32,
    value: u32,
    pub next: u32
}

#[derive(Debug, Clone)]
pub struct GPUHashTable {
    pub buckets: Vec<u32>,
    pub objects: Vec<KeyValue>,
    objects_left: Vec<usize>,
    block_size: Vector3<u32>,
}

impl GPUHashTable {
    pub fn new(block_size: Vector3<u32>) -> GPUHashTable {
        return GPUHashTable {
            buckets: vec![u32::MAX; 1000], // capacity hard coded for now
            objects: vec![KeyValue {key: 0, value: 0, next: u32::MAX}; 1000], // capacity hard coded for now
            objects_left: (0..1000).collect(),
            block_size,
        };
    }

    fn hash(&self, val: Vector3<u32>) -> u32 {
        // we reserve 0 as a free space
        return val.x + self.block_size.y * (val.y + self.block_size.z * val.z);
    }

    pub fn insert(&mut self, key: Vector3<u32>, val: u32)  {
        // console::log_1(&self.objects_last.into());

        // if self.objects_last == 1000 {
        //     console::log_1(&format!("{:?}", self.objects).into());
        //     console::log_1(&format!("{:?}", self.buckets).into());
        // }

        let original_hash = self.hash(key);
        let index = (original_hash % 1000) as usize;
        console::log_1(&format!("creating object and putting it at index: {:?}", index).into());

        let mut current_object: KeyValue;

        if self.objects_left.is_empty() {
            console::log_1(&format!("{:?}", self.objects).into());
            console::log_1(&format!("{:?}", self.buckets).into());
            // TODO: throw error
        }

        // this bucket hasnt been used, just add the item at any available position
        if self.buckets[index] == u32::MAX {
            let last_index_available = self.objects_left.pop().unwrap(); // we can safely unwrap as
                                                                         // we already now it won't be empty
            self.buckets[index] = last_index_available as u32;
            self.objects[last_index_available] = KeyValue {key: original_hash, value: val, next: u32::MAX};
            return;
        }

        // if we reach this part of the code, then this bucket isn't empty, let's find the last item of
        // the bucket by following it as a linked list
        current_object = self.objects[self.buckets[index] as usize];
        let mut last_next = self.buckets[index];

        while current_object.next != u32::MAX {
            if current_object.value == val && current_object.key == original_hash { // in case we already stored this same value
                console::log_1(&format!("key {:?} with value {:?} was already stored", current_object.key, current_object.value).into());
                return;
            }

            last_next = current_object.next;
            current_object = self.objects[current_object.next as usize];
        }

        // the current object has no next, so let's add it
        let last_index_available = self.objects_left.pop().unwrap();

        self.objects[last_next as usize].next = last_index_available as u32;
        self.objects[last_index_available] = KeyValue {key: original_hash, value: val, next: u32::MAX};

        console::log_2(&format!("{:?}", key).into(), &format!("{:?}", original_hash).into());
    }

    pub fn remove(&mut self, key: Vector3<u32>, val: u32) -> Result<(), String> {
        let original_hash = self.hash(key);
        let bucket_index = (original_hash % 1000) as usize;

        // if we reach this part of the code, then this bucket isn't empty, let's find the last item of
        // the bucket by following it as a linked list
        if self.buckets[bucket_index] == u32::MAX {
            return Err(format!("Item with key {:?} couldn't be found", key).to_string());
        }

        let mut current_object = self.objects[self.buckets[bucket_index] as usize];
        console::log_1(&format!("Removing item: {:?} with key: {:?} and buckets index: {:?}", current_object, key, bucket_index).into());

        let mut last_index = u32::MAX;
        let mut current_index = self.buckets[bucket_index];

        while current_index as u32 != u32::MAX {
            current_object = self.objects[current_index as usize];

            // TODO: might be good to rewrite this to something less absolutely terrible
            // should just be a simple linked list item removal with just some added things
            if current_object.value == val && current_object.key == original_hash {
                // uhh uhhh muh cache hits (shut up nerd)
                // items are liberated backwards, could change that to
                // optimize for cache hits
                self.objects_left.push(current_index as usize); // "liberate" this index

                // the simplest case, there's only one item left and
                // it's the one we want to remove
                if last_index == u32::MAX && current_object.next == u32::MAX {
                    console::log_1(&format!("Object was the only item in linked list, removing key {:?} from buckets", bucket_index).into());
                    self.objects[current_index as usize] = KeyValue {key: 0, value: 0, next: u32::MAX};
                    self.buckets[bucket_index] = u32::MAX;
                    return Ok(());

                // this is the last but not the first item
                } else if current_object.next == u32::MAX {
                    console::log_1(&format!("Object was last in linked list").into());
                    self.objects[last_index as usize].next = u32::MAX;
                    self.objects[current_index as usize] = KeyValue {key: 0, value: 0, next: u32::MAX};
                    return Ok(());

                // this is the first item
                } else if last_index == u32::MAX {
                    console::log_1(&format!("Object was first item in linked list").into());
                    self.objects[current_index as usize] = KeyValue {key: 0, value: 0, next: u32::MAX};
                    // update first item
                    self.buckets[bucket_index] = current_object.next;
                    last_index = u32::MAX;

                } else {
                    console::log_1(&format!("Object wasn't unique in linked list").into());
                    self.objects[last_index as usize].next = current_object.next;
                    self.objects[current_index as usize] = KeyValue {key: 0, value: 0, next: u32::MAX};
                    last_index = current_index;
                }
            }

            current_index = current_object.next;
        }
        return Ok(());
    }

    // terrible stuff
    pub fn opengl_compatible_objects_list(&self, list_to_fill: &mut [u32]) {
        for (i, val) in self.objects.iter().enumerate() {
            list_to_fill[i * 3] = val.key;
            list_to_fill[(i * 3) + 1] = val.value;
            list_to_fill[(i * 3) + 2] = val.next;
        }
    }
}
