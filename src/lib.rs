pub trait SymmetricDifference: IntoIterator {
    fn diff<Rhs>(self, rhs: Rhs) -> SymDiffIter<Self::IntoIter, Rhs::IntoIter>
    where
        Self::Item: Eq + Ord,
        Rhs: IntoIterator<Item = Self::Item>;

    fn diff_internal<Rhs, FL, FR>(self, rhs: Rhs, fl: FL, fr: FR)
    where
        Self::Item: Eq + Ord,
        Rhs: IntoIterator<Item = Self::Item>,
        FL: FnMut(Self::Item),
        FR: FnMut(Self::Item);

    fn diff_internal_alt<Rhs, F>(self, rhs: Rhs, f: F)
    where
        Self::Item: Eq + Ord,
        Rhs: IntoIterator<Item = Self::Item>,
        F: FnMut(Tag<Self::Item>);
}

impl<T: IntoIterator> SymmetricDifference for T {
    fn diff<Rhs>(self, rhs: Rhs) -> SymDiffIter<Self::IntoIter, Rhs::IntoIter>
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

    fn diff_internal<Rhs, FL, FR>(self, rhs: Rhs, mut fl: FL, mut fr: FR)
    where
        Self::Item: Eq + Ord,
        Rhs: IntoIterator<Item = Self::Item>,
        FL: FnMut(Self::Item),
        FR: FnMut(Self::Item),
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
                    fl(item);
                    for item in left {
                        fl(item);
                    }
                    return;
                }

                (None, Some(item)) => {
                    fr(item);
                    for item in right {
                        fr(item);
                    }
                    return;
                }

                (Some(a), Some(b)) => match a.cmp(&b) {
                    Greater => {
                        fr(b);
                        curr_left = Some(a);
                        curr_right = right.next();
                    }

                    Less => {
                        fl(a);
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

    fn diff_internal_alt<Rhs, F>(self, rhs: Rhs, mut f: F)
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
        let set: HashSet<_> = LEFT.diff(RIGHT).map(Tag::unwrap).collect();
        let expected_diff: HashSet<_> = {
            let left: HashSet<_> = LEFT.into_iter().collect();
            let right: HashSet<_> = RIGHT.into_iter().collect();
            left.symmetric_difference(&right).map(|&x| x).collect()
        };

        assert_eq!(set, expected_diff);
    }

    #[test]
    fn diff_internal_works() {
        let mut left = Vec::new();
        let mut right = Vec::new();

        LEFT.diff_internal(
            RIGHT,
            |x| {
                left.push(x);
            },
            |x| {
                right.push(x);
            },
        );

        let set: HashSet<_> = left.into_iter().chain(right).collect();
        let expected_diff: HashSet<_> = {
            let left: HashSet<_> = LEFT.into_iter().collect();
            let right: HashSet<_> = RIGHT.into_iter().collect();
            left.symmetric_difference(&right).map(|&x| x).collect()
        };

        assert_eq!(set, expected_diff);
    }

    #[test]
    fn diff_internal_alt_works() {
        let mut set = HashSet::new();

        LEFT.diff_internal_alt(RIGHT, |x| {
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
