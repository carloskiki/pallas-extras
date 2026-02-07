//! A bitmapped vector tree for the machine's environment.
//!
//! This vector is implemented with a left leaning tree having bucket size of 8 (3 bits per level).

use std::{hint::unreachable_unchecked, rc::Rc};

mod bucket;
use bucket::{Bucket, Chunk};

#[derive(Debug)]
enum Node<T> {
    Branch(Rc<Chunk<Node<T>>>),
    Leaf(Bucket<Bucket<T>>),
}

impl<T> Node<T> {
    /// Grow the node into a branch.
    ///
    /// `N -> Branch([N])`.
    fn grow(&mut self) -> &mut Chunk<Node<T>> {
        let old_self = std::mem::replace(self, Node::Branch(Rc::new(Chunk::default())));
        let Node::Branch(new_chunk) = self else {
            // Safety: We created a Branch node just above.
            unsafe { unreachable_unchecked() }
        };
        // Safety: We just created the bucket, so we have unique access to it.
        let new_chunk = unsafe { Rc::get_mut(new_chunk).unwrap_unchecked() };
        new_chunk.push(old_self);
        new_chunk
    }
}

impl<T> Clone for Node<T> {
    fn clone(&self) -> Self {
        match self {
            Node::Branch(a) => Node::Branch(Rc::clone(a)),
            Node::Leaf(a) => Node::Leaf(a.clone()),
        }
    }
}

#[derive(Debug)]
pub struct Vector<T> {
    // The root node of the tree.
    root: Node<T>,
    // The number of full chunks in the root.
    size: usize,
    // The tail chunk.
    tail: Bucket<T>,
}

impl<T> Vector<T> {
    pub fn get(&self, index: usize) -> Option<&T> {
        const MASK: usize = (1 << bucket::BITS) - 1;

        let tree_size = self.size * bucket::SIZE;
        if index >= tree_size + self.tail.len() {
            return None;
        } else if index >= tree_size {
            // Safety: The tail contains the element.
            return Some(unsafe { self.tail.get(index - tree_size) });
        }

        let mut node = &self.root;
        let mut shift = (usize::BITS - (tree_size - 1).leading_zeros()).saturating_sub(1) as usize
            / bucket::BITS
            * bucket::BITS;
        loop {
            match node {
                Node::Branch(chunk) => {
                    let b = (index >> shift) & MASK;
                    shift -= bucket::BITS;
                    // Safety: The index is valid since the tree is well formed.
                    node = unsafe { chunk.get(b) };
                }
                Node::Leaf(bucket) => {
                    // Safety: The index is less than the length of the vec. This index is
                    // therefore valid since the tree structure is correct.
                    let bucket = unsafe { bucket.get((index >> bucket::BITS) & MASK) };
                    // Safety: Same as above.
                    return Some(unsafe { bucket.get(index & MASK) });
                }
            }
        }
    }
}

impl<T: Clone> Vector<T> {
    pub fn push(&mut self, v: T) {
        // Need to push the tail into the tree.
        if self.tail.len() == bucket::SIZE {
            let mut index = self.size;
            let mut node = &mut self.root;
            let tail = std::mem::take(&mut self.tail);

            loop {
                let b = bucket::index(&mut index);
                match node {
                    Node::Leaf(bucket) if bucket.len() != bucket::SIZE => {
                        bucket.push(tail);
                    }
                    // The rest of the index being zero means we need to push a new child.
                    // `b == 1` means we are at a power of 8. The node is full, we need to grow.
                    Node::Leaf(_) | Node::Branch(_) if index == 0 && b == 1 => {
                        let chunk = node.grow();
                        let mut bucket = Bucket::default();
                        bucket.push(tail);
                        chunk.push(Node::Leaf(bucket));
                    }
                    Node::Branch(chunk) => {
                        let chunk = Rc::make_mut(chunk);
                        if index != 0 {
                            // Safety: We know that `b` is a valid index within the bucket because the tree is
                            // valid.
                            node = unsafe { chunk.get_mut(b) };
                            continue;
                        }

                        let mut bucket = Bucket::default();
                        bucket.push(tail);
                        chunk.push(Node::Leaf(bucket));
                    }
                    Node::Leaf(_) => unreachable!(),
                }
                break;
            }
            self.size += 1;
        }
        self.tail.push(v);
    }
}

impl<T> Clone for Vector<T> {
    fn clone(&self) -> Self {
        Vector {
            root: self.root.clone(),
            tail: self.tail.clone(),
            size: self.size,
        }
    }
}

impl<T> Default for Vector<T> {
    fn default() -> Vector<T> {
        Vector {
            root: Node::Leaf(Bucket::default()),
            tail: Bucket::default(),
            size: 0,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    impl<T> Vector<T> {
        fn len(&self) -> usize {
            self.size * bucket::SIZE + self.tail.len()
        }
    }

    #[test]
    fn len() {
        let mut vector: Vector<i32> = Vector::default();
        assert_eq!(vector.len(), 0);

        for i in 0..1000 {
            vector.push(i);
            assert_eq!(vector.len(), (i + 1) as usize);
        }
    }

    #[test]
    fn push() {
        let limit = 32 * 32 * 32 + 1;
        let mut vector: Vector<i32> = Vector::default();

        for i in 0..limit {
            assert_eq!(vector.len(), i as usize);
            vector.push(-i);
            assert_eq!(vector.get(i as usize), Some(&-i));
        }

        assert_eq!(vector.len(), limit as usize);
    }

    #[test]
    fn get() {
        let limit = 32 * 32 * 32 + 1;
        let mut vector = Vector::default();

        for i in 0..(limit - 1) {
            vector.push(i + 1);
        }
        vector.push(limit);

        assert_eq!(vector.get(0), Some(&1));
        assert_eq!(vector.get(2020), Some(&2021));
        assert_eq!(vector.get(limit - 1), Some(&limit));
        assert_eq!(vector.get(limit), None);
    }

    #[test]
    fn clone() {
        let mut vector = Vector::default();
        vector.push(1);
        vector.push(2);
        let clone = vector.clone();
        assert_eq!(vector.len(), clone.len());
        assert_eq!(vector.get(0), clone.get(0));
        assert_eq!(vector.get(1), clone.get(1));
        assert_eq!(vector.get(2), clone.get(2));
    }

    #[test]
    fn complex() {
        let mut a = Vector::default();
        a.push(10);
        a.push(20);
        let mut b = a.clone();
        let c = b.clone();
        a.push(30);
        assert_eq!(a.len(), 3);
        assert_eq!(b.len(), 2);
        b.push(40);
        assert_eq!(a.get(2), Some(&30));
        assert_eq!(b.get(2), Some(&40));
        assert_eq!(c.len(), 2);
        assert_eq!(c.get(0), Some(&10));

        let mut d = a.clone();
        for i in 0..2000 {
            a.push(i);
        }
        assert_eq!(d.len(), 3);
        d.push(50);
        assert_eq!(d.get(3), Some(&50));
        assert_eq!(a.len(), 2003);
        assert_eq!(a.get(2002), Some(&1999));
        assert_eq!(a.get(5), Some(&2));
        assert_eq!(b.get(5), None);
        assert_eq!(c.get(5), None);
        assert_eq!(d.get(5), None);
    }
}
