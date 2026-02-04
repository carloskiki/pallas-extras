use std::{cell::UnsafeCell, mem::MaybeUninit, rc::Rc};

pub const BITS: usize = 3;
pub const SIZE: usize = 1 << BITS;


pub fn index(index: &mut usize) -> usize {
    let shift =
        (usize::BITS as usize - index.leading_zeros() as usize).saturating_sub(1) / BITS * BITS;
    let ret = (*index >> shift) & ((1 << BITS) - 1);
    *index &= (1 << shift) - 1;
    ret
}

#[derive(Debug)]
pub struct Bucket<T> {
    data: Rc<UnsafeCell<Chunk<T>>>,
    perceived: u8,
}

impl<T> Bucket<T> {
    // Gets a reference to the element at the given index.
    //
    // # Safety
    //
    // The index must be less than the length of the chunk.
    pub unsafe fn get(&self, index: usize) -> &T {
        debug_assert!(index < self.perceived as usize);
        // Safety: There are no concurrent mutable references to `data` in this function, so we can safely get a
        // reference to it.
        let data = unsafe { &*self.data.get() };
        // Safety: The index is within the chunk, so it must be within the bucket.
        unsafe { data.get(index) }
    }

    pub fn len(&self) -> usize {
        self.perceived as usize
    }
}

impl<T: Clone> Bucket<T> {
    /// Pushes a value to the chunk, panicking if the chunk is full.
    pub fn push(&mut self, value: T) {
        let slot = self.perceived as usize;
        self.perceived += 1;

        if let Some(cell) = Rc::get_mut(&mut self.data) {
            let bucket = cell.get_mut();
            bucket.data[slot].write(value);
            // Increase the length if self.perceived is up to date.
            bucket.len = bucket.len.max(self.perceived);
            return;
        }

        // Safety: Within this function there are no concurrent references to `data`, so we can
        // get a `&mut` to the content.
        let bucket = unsafe { &mut *self.data.get() };
        if slot == bucket.len as usize {
            // Our perceived length is up to date; we can push.
            bucket.push(value);
            return;
        }

        // Our perceived length is out of date; we must clone.
        //
        // Safety: Standard procedure to create an array of `uninit`.
        let mut data: [MaybeUninit<T>; SIZE] = unsafe { MaybeUninit::uninit().assume_init() };
        (0..slot).for_each(|i| {
            // Safety: We know that values up to `len` are valid and initialized. We know that `slot <=
            // len`. Thus, we can safely copy up to `slot`.
            let val = unsafe { bucket.data[i].assume_init_ref() };
            data[i].write(val.clone());
        });
        data[slot].write(value);
        self.data = Rc::new(UnsafeCell::new(Chunk {
            data,
            len: self.perceived,
        }));
    }
}

impl<T> Default for Bucket<T> {
    fn default() -> Self {
        Bucket {
            data: Rc::new(UnsafeCell::new(Chunk::default())),
            perceived: 0,
        }
    }
}

impl<T> Clone for Bucket<T> {
    fn clone(&self) -> Self {
        Bucket {
            data: Rc::clone(&self.data),
            perceived: self.perceived,
        }
    }
}

// A bucket that can hold up to `SIZE` elements.
#[derive(Debug)]
pub struct Chunk<T> {
    data: [MaybeUninit<T>; SIZE],
    len: u8,
}

impl<T> Chunk<T> {
    /// Gets a reference to the element at the given index.
    ///
    /// # Safety
    ///
    /// The index must be less than the length of the bucket.
    pub unsafe fn get(&self, index: usize) -> &T {
        debug_assert!(index < self.len as usize);
        // Safety: The index is within the bucket.
        unsafe { self.data[index].assume_init_ref() }
    }

    /// Gets a reference to the element at the given index.
    ///
    /// # Safety
    ///
    /// The index must be less than the length of the bucket.
    pub unsafe fn get_mut(&mut self, index: usize) -> &mut T {
        debug_assert!(index < self.len as usize);
        // Safety: The index is within the bucket.
        unsafe { self.data[index].assume_init_mut() }
    }

    /// Pushes a value to the bucket. Panics if the bucket is full.
    pub fn push(&mut self, value: T) {
        self.data[self.len as usize].write(value);
        self.len += 1;
    }
}

impl<T: Clone> Clone for Chunk<T> {
    fn clone(&self) -> Self {
        // Safety: Standard procedure to create an array of `uninit`.
        let mut new_data: [MaybeUninit<T>; SIZE] = unsafe { MaybeUninit::uninit().assume_init() };
        (0..self.len as usize).for_each(|i| {
            // Safety: We know that values up to `len` are valid and initialized.
            let val = unsafe { self.data[i].assume_init_ref() };
            new_data[i].write(val.clone());
        });
        Chunk {
            data: new_data,
            len: self.len,
        }
    }
}

impl<T> Default for Chunk<T> {
    fn default() -> Self {
        // Safety: Standard procedure to create an array of `uninit`.
        let data: [MaybeUninit<T>; SIZE] = unsafe { MaybeUninit::uninit().assume_init() };
        Chunk { data, len: 0 }
    }
}

impl<T> Drop for Chunk<T> {
    fn drop(&mut self) {
        // Safety: We only drop the initialized and valid elements.
        unsafe { self.data[..self.len as usize].assume_init_drop() };
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore]
    fn index() {
        let mut index = 0b_00100_00011_00010_00001;
        assert_eq!(super::index(&mut index), 0b00100);
        assert_eq!(super::index(&mut index), 0b00011);
        assert_eq!(super::index(&mut index), 0b00010);
        assert_eq!(super::index(&mut index), 0b00001);
        assert_eq!(index, 0);
    }
}
