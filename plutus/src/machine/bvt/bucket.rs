//! [`Bucket`]s and [`Chunk`]s.

use std::{cell::UnsafeCell, mem::MaybeUninit};

/// A [`Bucket`] that can hold up to [`SIZE`] elements.
///
///
/// Acts as a clone-on-write container for a bucket of elements.
#[derive(Debug, Clone, Copy)]
pub struct Bucket<'a, T: Copy, const SIZE: usize> {
    data: &'a UnsafeCell<Chunk<T, SIZE>>,
    perceived: u8,
}

impl<'a, T: Copy, const SIZE: usize> Bucket<'a, T, SIZE> {
    pub fn new(arena: &'a crate::Arena) -> Self {
        Bucket {
            data: arena.alloc(UnsafeCell::new(Chunk::default())),
            perceived: 0,
        }
    }

    // Gets a reference to the element at the given index.
    //
    // # Safety
    //
    // The index must be in bounds.
    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        debug_assert!(index < self.perceived as usize);
        // Safety: There are no concurrent mutable references to `data` in this function, so we can safely get a
        // reference to it.
        let data = unsafe { &*self.data.get() };
        // Safety: The index is within the bucket, so it must be within the bucket.
        unsafe { data.get_unchecked(index) }
    }

    pub fn len(&self) -> usize {
        self.perceived as usize
    }

    /// Pushes a value to the bucket, panicking if the bucket is full.
    pub fn push(&mut self, value: T, arena: &'a crate::Arena) {
        let slot = self.perceived as usize;
        self.perceived += 1;

        // Safety: Within this function there are no concurrent references to `data`, so we can
        // get a `&mut` to the content.
        let bucket = unsafe { &mut *self.data.get() };
        if slot == bucket.len as usize {
            // Our perceived length is up to date; we can push.
            bucket.push(value);
            return;
        }

        // Our perceived length is out of date; we must copy.
        let mut data = *bucket;
        data.data[slot].write(value);
        data.len = self.perceived;
        self.data = arena.alloc(UnsafeCell::new(data));
    }
}

impl<'a, T: Copy, const SIZE: usize> AsRef<[T]> for Bucket<'a, T, SIZE> {
    fn as_ref(&self) -> &[T] {
        // Safety: There are no concurrent mutable references to `data` in this function, so we can safely get a
        // reference to it.
        let data = unsafe { &*self.data.get() };
        // Safety: The perceived length is less than or equal to the actual length, so we can
        // safely create a slice of the perceived length.
        unsafe { std::slice::from_raw_parts(data.data.as_ptr() as *const T, self.perceived as usize) }
    }
}

/// An inline vector of up to [`SIZE`] elements.
#[derive(Debug, Clone, Copy)]
pub struct Chunk<T: Copy, const SIZE: usize> {
    data: [MaybeUninit<T>; SIZE],
    len: u8,
}

impl<T: Copy, const SIZE: usize> Chunk<T, SIZE> {
    /// Gets a reference to the element at the given index.
    ///
    /// # Safety
    ///
    /// The index must be less than the length of the chunk.
    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        debug_assert!(index < self.len as usize);
        // Safety: The index is within the chunk.
        unsafe { self.data[index].assume_init_ref() }
    }

    /// Gets a reference to the element at the given index.
    ///
    /// # Safety
    ///
    /// The index must be less than the length of the chunk.
    pub unsafe fn get_mut_unchecked(&mut self, index: usize) -> &mut T {
        debug_assert!(index < self.len as usize);
        // Safety: The index is within the chunk.
        unsafe { self.data[index].assume_init_mut() }
    }

    /// Pushes a value to the chunk. Panics if the chunk is full.
    pub fn push(&mut self, value: T) {
        self.data[self.len as usize].write(value);
        self.len += 1;
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }
}

impl<T: Copy, const SIZE: usize> Default for Chunk<T, SIZE> {
    fn default() -> Self {
        // Safety: Standard procedure to create an array of `uninit`.
        let data: [MaybeUninit<T>; SIZE] = unsafe { MaybeUninit::uninit().assume_init() };
        Chunk { data, len: 0 }
    }
}
