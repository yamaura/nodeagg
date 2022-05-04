pub struct ConcatIterator<'a, T> {
    pub itrs: Vec<Box<dyn Iterator<Item = T> + 'a>>,
}

impl<T> Iterator for ConcatIterator<'_, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        let mut index: usize = 0;
        loop {
            match self.itrs.get_mut(index) {
                None => break None,
                Some(e) => match e.next() {
                    None => index += 1,
                    Some(v) => break Some(v),
                },
            }
        }
    }
}

pub type NodeaggIterator<'a> = itertools::Unique<ConcatIterator<'a, String>>;
