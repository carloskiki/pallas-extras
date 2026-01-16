pub struct Set<T>(Vec<T>);

impl<T> AsRef<[T]> for Set<T> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl<T: Ord + PartialEq> FromIterator<T> for Set<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut vec: Vec<T> = iter.into_iter().collect();
        vec.sort();
        vec.dedup();
        Set(vec)
    }
}

impl<T: Ord + PartialEq> Extend<T> for Set<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let mut vec: Vec<T> = iter.into_iter().collect();
        vec.sort();
        vec.dedup();
        self.0.extend(vec);
        self.0.sort();
        self.0.dedup();
    }
}
