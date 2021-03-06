pub trait SymmetricDifference: IntoIterator {
    fn difference<Rhs>(self, rhs: Rhs) -> SymDiffIter<Self::IntoIter, Rhs::IntoIter>
    where
        Self::Item: Eq + Ord,
        Rhs: IntoIterator<Item = Self::Item>;

    fn iter_difference<Rhs, F>(self, rhs: Rhs, f: F)
    where
        Self::Item: Eq + Ord,
        Rhs: IntoIterator<Item = Self::Item>,
        F: FnMut(Tag<Self::Item>);
}

impl<T: IntoIterator> SymmetricDifference for T {
    fn difference<Rhs>(self, rhs: Rhs) -> SymDiffIter<Self::IntoIter, Rhs::IntoIter>
    where
        Self::Item: Eq + Ord,
        Rhs: IntoIterator<Item = Self::Item>,
    {
        SymDiffIter {
            left: self.into_iter(),
            right: rhs.into_iter(),
            rem: None,
        }
    }

    fn iter_difference<Rhs, F>(self, rhs: Rhs, mut f: F)
    where
        Self::Item: Eq + Ord,
        Rhs: IntoIterator<Item = Self::Item>,
        F: FnMut(Tag<Self::Item>),
    {
        use std::cmp::Ordering::*;

        let mut left = self.into_iter();
        let mut right = rhs.into_iter();

        let mut curr_left = left.next();
        let mut curr_right = right.next();

        loop {
            match (curr_left.take(), curr_right.take()) {
                (None, None) => return,

                (Some(item), None) => {
                    f(Tag::Left(item));
                    for item in left {
                        f(Tag::Left(item));
                    }
                    return;
                }

                (None, Some(item)) => {
                    f(Tag::Right(item));
                    for item in right {
                        f(Tag::Right(item));
                    }
                    return;
                }

                (Some(a), Some(b)) => match a.cmp(&b) {
                    Greater => {
                        f(Tag::Right(b));
                        curr_left = Some(a);
                        curr_right = right.next();
                    }

                    Less => {
                        f(Tag::Left(a));
                        curr_left = left.next();
                        curr_right = Some(b);
                    }

                    Equal => {
                        curr_left = left.next();
                        curr_right = right.next();
                    }
                },
            }
        }
    }
}

#[derive(Debug)]
pub enum Tag<T> {
    Left(T),
    Right(T),
}

impl<T> Tag<T> {
    pub fn unwrap(self) -> T {
        match self {
            Tag::Left(x) | Tag::Right(x) => x,
        }
    }

    pub fn value(&self) -> &T {
        match self {
            Tag::Left(x) | Tag::Right(x) => x,
        }
    }

    pub fn is_left(&self) -> bool {
        match self {
            Tag::Left(_) => true,
            _ => false,
        }
    }

    pub fn is_right(&self) -> bool {
        match self {
            Tag::Right(_) => true,
            _ => false,
        }
    }
}

pub struct SymDiffIter<Left, Right>
where
    Left: Iterator,
    Right: Iterator,
{
    left: Left,
    right: Right,
    rem: Option<Tag<Left::Item>>,
}

impl<Left, Right> Iterator for SymDiffIter<Left, Right>
where
    Left: Iterator,
    Left::Item: Eq + Ord,
    Right: Iterator<Item = Left::Item>,
{
    type Item = Tag<Left::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        use std::cmp::Ordering::*;

        let (mut left, mut right) = match self.rem.take() {
            None => (self.left.next(), self.right.next()),
            Some(Tag::Left(rem)) => (Some(rem), self.right.next()),
            Some(Tag::Right(rem)) => (self.left.next(), Some(rem)),
        };

        loop {
            match (left.take(), right.take()) {
                (Some(left), None) => return Some(Tag::Left(left)),
                (None, Some(right)) => return Some(Tag::Right(right)),
                (Some(left), Some(right)) => match left.cmp(&right) {
                    Greater => {
                        self.rem = Some(Tag::Left(left));
                        return Some(Tag::Right(right));
                    }

                    Less => {
                        self.rem = Some(Tag::Right(right));
                        return Some(Tag::Left(left));
                    }

                    _ => (),
                },

                _ => return None,
            }

            left = self.left.next();
            right = self.right.next();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    static LEFT: &[i32] = &[1, 2, 4, 5, 6, 8, 13, 15, 16];
    static RIGHT: &[i32] = &[2, 3, 4, 5, 6, 7, 8];

    #[test]
    fn diff_works() {
        let set: HashSet<_> = LEFT.difference(RIGHT).map(Tag::unwrap).collect();
        let expected_diff: HashSet<_> = {
            let left: HashSet<_> = LEFT.into_iter().collect();
            let right: HashSet<_> = RIGHT.into_iter().collect();
            left.symmetric_difference(&right).map(|&x| x).collect()
        };

        assert_eq!(set, expected_diff);
    }

    #[test]
    fn iter_difference_works() {
        let mut set = HashSet::new();

        LEFT.iter_difference(RIGHT, |x| {
            set.insert(x.unwrap());
        });

        let expected_diff: HashSet<_> = {
            let left: HashSet<_> = LEFT.into_iter().collect();
            let right: HashSet<_> = RIGHT.into_iter().collect();
            left.symmetric_difference(&right).map(|&x| x).collect()
        };

        assert_eq!(set, expected_diff);
    }
}
