//! A bitmapped vector trie for the machine's environment.
//!
//! This vector is implemented with a left leaning tree having bucket size of 8 (3 bits per level).

mod bucket;
pub use bucket::{Bucket, Chunk};

const BITS: usize = 2;
const SIZE: usize = 1 << BITS;
const MASK: usize = (1 << BITS) - 1;

/// Get the shift required to get the index of the root's child that contains the element at the
/// given index.
fn shift(index: usize) -> usize {
    (usize::BITS as usize - index.leading_zeros() as usize).saturating_sub(1) / BITS * BITS
}

/// A node in the tree.
#[derive(Debug, Clone, Copy)]
enum Node<'a, T: Copy> {
    /// A branch node.
    Branch(&'a Chunk<Node<'a, T>, SIZE>),
    /// A leaf node.
    ///
    /// Instead of cotaining a chunk of elements. It contains a chunk of "tails".
    /// Every time the tail is full, it is pushed into the tree as a child to a leaf node.
    Leaf(Bucket<'a, Bucket<'a, T, SIZE>, SIZE>),
}

impl<'a, T: Copy> Node<'a, T> {
    fn push(&mut self, tail: Bucket<'a, T, SIZE>, mut index: usize, arena: &'a crate::Arena) {
        let shift = shift(index);
        let b = index >> shift;
        index &= !(MASK << shift);

        match self {
            Node::Leaf(bucket) if bucket.len() != SIZE => {
                bucket.push(tail, arena);
            }
            // The rest of the index being zero means we need to push a new child.
            // `b == 1` means we are at a power of SIZE. The node is full, we need to grow.
            //
            // This tansforms node `N -> Branch([N, Leaf(tail)])`.
            Node::Leaf(_) | Node::Branch(_) if index == 0 && b == 1 => {
                let mut new_chunk = Chunk::default();
                new_chunk.push(*self);
                let mut leaf_bucket = Bucket::new(arena);
                leaf_bucket.push(tail, arena);
                new_chunk.push(Node::Leaf(leaf_bucket));
                *self = Node::Branch(arena.alloc(new_chunk));
            }
            Node::Branch(chunk) => {
                let mut new_chunk = **chunk;
                // let chunk = Rc::make_mut(chunk);
                if index != 0 {
                    // Safety: We know that `b` is a valid index within the bucket because the tree is
                    // valid.
                    let child = unsafe { new_chunk.get_mut_unchecked(b) };
                    child.push(tail, index, arena);
                } else {
                    let mut leaf_bucket = Bucket::new(arena);
                    leaf_bucket.push(tail, arena);
                    new_chunk.push(Node::Leaf(leaf_bucket));
                }
                *chunk = arena.alloc(new_chunk);
            }
            Node::Leaf(_) => unreachable!(),
        }
    }
}

/// A bitmapped vector trie.
#[derive(Debug, Clone, Copy)]
pub struct Vector<'a, T: Copy> {
    /// The root node of the tree.
    root: Node<'a, T>,
    /// The number of full chunks in the root.
    size: usize,
    /// The tail chunk.
    tail: Bucket<'a, T, SIZE>,
}

impl<'a, T: Copy> Vector<'a, T> {
    pub fn new(arena: &'a crate::Arena) -> Self {
        Vector {
            root: Node::Leaf(Bucket::new(arena)),
            size: 0,
            tail: Bucket::new(arena),
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        let tree_size = self.size * SIZE;
        if index >= tree_size + self.tail.len() {
            return None;
        } else if index >= tree_size {
            // Safety: The tail contains the element.
            return Some(unsafe { self.tail.get_unchecked(index - tree_size) });
        }

        let mut max_index = tree_size - 1;
        let mut node = &self.root;
        loop {
            match node {
                Node::Branch(chunk) => {
                    let shift = shift(max_index);
                    let b = (index >> shift) & MASK;
                    if b != chunk.len() - 1 {
                        // We are not in the last child, so the subtree is full.
                        max_index = max_index.next_power_of_two() - 1;
                    }
                    max_index &= !(MASK << shift);

                    // Safety: The index is valid since the tree is well formed.
                    node = unsafe { chunk.get_unchecked(b) };
                }
                Node::Leaf(bucket) => {
                    // Safety: The index is less than the length of the vec. This index is
                    // therefore valid since the tree structure is correct.
                    let bucket = unsafe { bucket.get_unchecked((index >> BITS) & MASK) };
                    // Safety: Same as above.
                    return Some(unsafe { bucket.get_unchecked(index & MASK) });
                }
            }
        }
    }

    pub fn push(&mut self, v: T, arena: &'a crate::Arena) {
        // Need to push the tail into the tree.
        if self.tail.len() == SIZE {
            let tail = std::mem::replace(&mut self.tail, Bucket::new(arena));
            self.root.push(tail, self.size, arena);
            self.size += 1;
        }
        self.tail.push(v, arena);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    impl<'a, T: Copy> Vector<'a, T> {
        fn len(&self) -> usize {
            self.size * SIZE + self.tail.len()
        }
    }

    #[test]
    fn len() {
        let arena = crate::Arena::default();
        let mut vector: Vector<i32> = Vector::new(&arena);
        assert_eq!(vector.len(), 0);

        for i in 0..1000 {
            vector.push(i, &arena);
            assert_eq!(vector.len(), (i + 1) as usize);
        }
    }

    #[test]
    fn push() {
        let limit = 32 * 32 * 32 + 1;
        let arena = crate::Arena::default();
        let mut vector: Vector<i32> = Vector::new(&arena);

        for i in 0..limit {
            assert_eq!(vector.len(), i as usize);
            vector.push(-i, &arena);
            assert_eq!(vector.get(i as usize), Some(&-i));
        }

        assert_eq!(vector.len(), limit as usize);
    }

    #[test]
    fn get() {
        let limit = 32 * 32 * 32 + 1;
        let arena = crate::Arena::default();
        let mut vector = Vector::new(&arena);

        for i in 0..(limit - 1) {
            vector.push(i + 1, &arena);
        }
        vector.push(limit, &arena);

        assert_eq!(vector.get(0), Some(&1));
        assert_eq!(vector.get(2020), Some(&2021));
        assert_eq!(vector.get(limit - 1), Some(&limit));
        assert_eq!(vector.get(limit), None);
    }

    #[test]
    fn copy() {
        let arena = crate::Arena::default();
        let mut vector = Vector::new(&arena);
        vector.push(1, &arena);
        vector.push(2, &arena);
        let mut copied = vector;
        assert_eq!(vector.len(), copied.len());
        assert_eq!(vector.get(0), copied.get(0));
        assert_eq!(vector.get(1), copied.get(1));
        assert_eq!(vector.get(2), copied.get(2));
        copied.push(3, &arena);
        assert_eq!(vector.len(), 2);
        assert_eq!(copied.len(), 3);
    }

    #[test]
    fn complex() {
        let arena = crate::Arena::default();
        let mut a = Vector::new(&arena);
        a.push(10, &arena);
        a.push(20, &arena);
        let mut b = a;
        let c = b;
        a.push(30, &arena);
        assert_eq!(a.len(), 3);
        assert_eq!(b.len(), 2);
        b.push(40, &arena);
        assert_eq!(a.get(2), Some(&30));
        assert_eq!(b.get(2), Some(&40));
        assert_eq!(c.len(), 2);
        assert_eq!(c.get(0), Some(&10));

        let mut d = a;
        for i in 0..2000 {
            a.push(i, &arena);
        }
        assert_eq!(d.len(), 3);
        d.push(50, &arena);
        assert_eq!(d.get(3), Some(&50));
        assert_eq!(a.len(), 2003);
        assert_eq!(a.get(2002), Some(&1999));
        assert_eq!(a.get(5), Some(&2));
        assert_eq!(b.get(5), None);
        assert_eq!(c.get(5), None);
        assert_eq!(d.get(5), None);
    }
}
