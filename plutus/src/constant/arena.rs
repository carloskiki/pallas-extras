//! Arena and constant pool.

use std::cell::UnsafeCell;
use crate::{ConstantIndex, Data, constant::Constant};

/// Arena.
///
/// Maintains both the allocator for the lifetime of the program _and_ the constant pool.
#[derive(Default, Debug)]
pub struct Arena {
    /// The constant pool for a `Program`.
    constants: UnsafeCell<Vec<Constant<'static>>>,
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
    pub fn integer<'a>(&'a self, int: rug::Integer) -> &'a rug::Integer {
        let int = self.bump.alloc(int);
        // Safety: There are no other references to `self.integers` in this function.
        // Valid pointers are pushed to the drop list and unused until `Drop`.
        unsafe { (*self.integers.get()).push(int as *mut _) };
        int
    }

    pub fn integers<'a>(&'a self, integers: Vec<rug::Integer>) -> &'a [rug::Integer] {
        let integers = self.bump.alloc_slice_fill_iter(integers.into_iter());
        // Safety: As above.
        unsafe {
            (*self.integers.get()).extend(integers.iter_mut().map(|i| i as *mut _));
        }
        integers
    }

    pub fn data<'a>(&'a self, data: Data) -> &'a Data {
        let data = self.bump.alloc(data);
        // Safety: There are no other references to `self.data` in this function.
        // Valid pointers are pushed to the drop list and unused until `Drop`.
        unsafe { (*self.data.get()).push(data as *mut _) };
        data
    }

    pub fn datas<'a>(&'a self, data: Vec<Data>) -> &'a [Data] {
        let data = self.bump.alloc_slice_fill_iter(data.into_iter());
        // Safety: As above.
        unsafe {
            (*self.data.get()).extend(data.iter_mut().map(|d| d as *mut _));
        }
        data
    }

    pub fn pair_data<'a>(&'a self, pairs: Vec<(Data, Data)>) -> &'a [(Data, Data)] {
        let pairs = self.bump.alloc_slice_fill_iter(pairs.into_iter());
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
    pub fn alloc<'a, T: Copy>(&'a self, value: T) -> &'a T {
        self.bump.alloc(value)
    }

    /// Allocate a slice of `Copy` values to the arena.
    pub fn slice_fill<'a, T: Copy>(
        &'a self,
        iter: impl IntoIterator<Item = T, IntoIter: ExactSizeIterator>,
    ) -> &'a [T] {
        self.bump.alloc_slice_fill_iter(iter)
    }

    pub fn string<'a>(&'a self, string: &str) -> &'a str {
        self.bump.alloc_str(string)
    }

    /// Add a constant with members allocated to the arena to the constant pool, returning its
    /// index.
    ///
    /// # Panics
    ///
    /// Panics if the constant pool ends up with than `u32::MAX` constants.
    pub fn constant<F, E>(&self, f: F) -> Result<ConstantIndex, E>
    where
        F: FnOnce(&Self) -> Result<Constant<'_>, E>,
    {
        let constant = f(self)?;
        // Safety: There are no other references to `self.constants` in this function.
        // Valid constants are pushed to the list and unused until `Drop`.
        unsafe {
            let constants = &mut *self.constants.get();
            let index = constants.len();
            constants.push(std::mem::transmute::<Constant<'_>, Constant<'static>>(
                constant,
            ));
            Ok(ConstantIndex(index.try_into().expect("too many constants in constant pool")))
        }
    }

    /// Get a constant from the constant pool by index.
    ///
    /// # Panics
    ///
    /// Panics if the `index` is out of scope of the constant pool.
    pub fn get<'a>(&'a self, index: ConstantIndex) -> Constant<'a> {
        // Safety: the `index` is either out of scope (in which case this method panics) or points
        // to a valid constant that was pushed to the list by `constant`, and holds references to
        // this arena. The lifetime of the returned constant is correctly bounded. There are
        // also no other references to `self.constants` in this function.
        unsafe { std::mem::transmute::<Constant<'static>, Constant<'a>>((*self.constants.get())[index.0 as usize]) }
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        for int in &*self.integers.get_mut() {
            // Safety: `Bump` guarantees that the pointers are valid for reads and dropping,
            // properly aligned, and nonnull. Having a unique reference to the arena guarantees
            // that there are no outstanding references to the values, so the pointed values can't
            // be accessed while we're dropping them, and they are valid for writes by
            // `drop_in_place`.
            unsafe { std::ptr::drop_in_place(*int) };
        }
        for data in &*self.data.get_mut() {
            // Safety: As above.
            unsafe { std::ptr::drop_in_place(*data) };
        }
    }
}
