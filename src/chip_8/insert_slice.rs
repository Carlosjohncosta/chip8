pub trait InsertSlice {
    type Item;
    fn insert_slice(&mut self, slice: &[Self::Item]);
}

impl<T> InsertSlice for [T]
where
    T: Clone,
{
    type Item = T;

    fn insert_slice(&mut self, slice: &[T]) {
        for (x, y) in self.iter_mut().zip(slice) {
            *x = y.clone();
        }
    }
}
