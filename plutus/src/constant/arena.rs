//! Arena storing constants for the lifetime of the program.

use crate::Data;
use std::cell::UnsafeCell;

/// Arena used by the evaluator.
///
/// Maintains the allocator and all memory used for the lifetime of the program.
#[derive(Default, Debug)]
pub struct Arena {
    /// The main allocator.
    ///
    /// This only deallocates memory when the arena is dropped. We track types that need drop
    /// (`rug::Integer` and `Data`) separately to not leak memory.
    bump: bumpalo::Bump,
    /// Drop list for integers.
    integers: UnsafeCell<Vec<*mut rug::Integer>>,
    /// Drop list for data.
    data: UnsafeCell<Vec<*mut Data>>,
}

impl Arena {
    pub(crate) fn integer(&self, int: rug::Integer) -> &rug::Integer {
        let int = self.bump.alloc(int);
        // Safety: There are no other references to `self.integers` in this function.
        // Valid pointers are pushed to the drop list and unused until `Drop`.
        unsafe { (*self.integers.get()).push(int as *mut _) };
        int
    }

    pub(crate) fn integers(&self, integers: Vec<rug::Integer>) -> &[rug::Integer] {
        let integers = self.bump.alloc_slice_fill_iter(integers);
        // Safety: As above.
        unsafe {
            (*self.integers.get()).extend(integers.iter_mut().map(|i| i as *mut _));
        }
        integers
    }

    pub(crate) fn data(&self, data: Data) -> &Data {
        let data = self.bump.alloc(data);
        // Safety: There are no other references to `self.data` in this function.
        // Valid pointers are pushed to the drop list and unused until `Drop`.
        unsafe { (*self.data.get()).push(data as *mut _) };
        data
    }

    pub(crate) fn datas(&self, data: Vec<Data>) -> &[Data] {
        let data = self.bump.alloc_slice_fill_iter(data);
        // Safety: As above.
        unsafe {
            (*self.data.get()).extend(data.iter_mut().map(|d| d as *mut _));
        }
        data
    }

    pub(crate) fn pair_data(&self, pair: (Data, Data)) -> &(Data, Data) {
        let pair = self.bump.alloc(pair);
        // Safety: There are no other references to `self.data` in this function.
        // Valid pointers are pushed to the drop list and unused until `Drop`.
        unsafe {
            (*self.data.get()).extend([&mut pair.0 as *mut _, &mut pair.1 as *mut _]);
        }
        pair
    }

    pub(crate) fn pair_datas(&self, pairs: Vec<(Data, Data)>) -> &[(Data, Data)] {
        let pairs = self.bump.alloc_slice_fill_iter(pairs);
        // Safety: As above.
        unsafe {
            (*self.data.get()).extend(
                pairs
                    .iter_mut()
                    .flat_map(|(k, v)| [k as *mut _, v as *mut _]),
            );
        }
        pairs
    }

    /// Allocate to the arena, only for types that implement `Copy` (thus don't need drop
    /// tracking).
    pub(crate) fn alloc<T: Copy>(&self, value: T) -> &T {
        self.bump.alloc(value)
    }

    /// Allocate a slice of `Copy` values to the arena.
    pub(crate) fn slice_fill<T: Copy>(
        &self,
        iter: impl IntoIterator<Item = T, IntoIter: ExactSizeIterator>,
    ) -> &[T] {
        self.bump.alloc_slice_fill_iter(iter)
    }

    pub(crate) fn string<'a>(&'a self, string: &str) -> &'a str {
        self.bump.alloc_str(string)
    }

    /// Clear the arena, dropping all values contained in it.
    ///
    /// This does not deallocate memory. To do so, the arena must be dropped.
    pub fn clear(&mut self) {
        for int in &*self.integers.get_mut() {
            // Safety: `Bump` guarantees that the pointers are valid for reads and dropping,
            // properly aligned, and nonnull. Having a unique reference to the arena guarantees
            // that there are no outstanding references to the values, so the pointed values can't
            // be accessed while or after we're dropping them, and they are valid for writes by
            // `drop_in_place`.
            unsafe { std::ptr::drop_in_place(*int) };
        }
        for data in &*self.data.get_mut() {
            // Safety: As above.
            unsafe { std::ptr::drop_in_place(*data) };
        }
        self.integers.get_mut().clear();
        self.data.get_mut().clear();
        self.bump.reset();
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        self.clear();
    }
}
