A while back, I wrote an article about [internal and external iteration](https://jmarcher.io/internal-vs-external-iteration/). My examples were effective but artificial. I have since run across a real world use case where internal iteration yields not only superior ergonomics but also superior performance when compared with external iteration.

# The use case

The symmetric difference of two sets is the set of values from each set not found in the other set. Did that make sense? Let me try again...

Say you have two lists `A` and `B`. The symmetric difference of `A` and `B` is all the items from `A` not found in `B` along with all the items found in `B` but not in `A`. Better? Anyway...

> Warning! The Symmetric Difference programming challenge is on this same subject, so don't read this article if you don't want a solution to the challenge.

...Let's call this "ordered symmetric difference" a subset of that, where the two sets are sorted ahead of time and we know that each value is larger than the last. This permits us to make some potentially drastic optimizations to the process, but the code involved is also fairly complicated. To provide a baseline, let's first discuss the naive implementation.

## Naive, hashset-based implementation

In this case, we mean "naive" not because this is stupid but because the implementation below assumes nothing about the nature of the sets in question.

The code in C# is straightforward with one annoying caveat we will discuss in a moment:

```csharp
var a = new HashSet<int> { 1, 2, 4, 5 };
var b = new HashSet<int> { 1, 3, 4, 5 };

a.SymmetricExceptWith(b);
Console.WriteLine(a.Sum());
```

The Rust version is similar, allowing for some ergonomic differences in the implementation:

```rust
let a = set(&[1, 2, 4, 5]);
let b = set(&[1, 3, 4, 5]);

let sum: i32 = a.symmetric_difference(&b).map(|&&x| x).sum();
println!("{}", sum);
```

The big difference between these two is that the Rust implementation borrows the two hashsets and returns references to the values contained therein. In contrast, the C# version modifies the left hand side (`a`) in place, removing values contained in `b` and adding values from `b` not found in `a`.

Rust is generally faster than C#, but not by as *much* in this case, because the hash routine employed by C# for these integers is significantly faster (relatively speaking) than the one used by Rust. More on that later.

## Not-so-naive external iteration

There is going to be a lot of code in this section. You have been warned.

tl;dr: iterate over both the left and right collections; assume an item in one collection is unique when the other collection has advanced to an item strictly greater than the first; emit unique items; when items are equal, discard both.

That's easy to explain in English, but it's a little more complicated to explain in Computerese.

### In CSharp

```csharp
public static IEnumerable<Tag<T>> Difference<T>(this IEnumerable<T> collection, IEnumerable<T> rhs)
    where T : IComparable<T>
{
    var iterLeft = collection.GetEnumerator();
    var iterRight = rhs.GetEnumerator();

    var hasLeft = iterLeft.MoveNext();
    var hasRight = iterRight.MoveNext();

    while (hasLeft || hasRight)
    {
        var left = iterLeft.Current;
        var right = iterRight.Current;

        if (!hasLeft && !hasRight)
        {
            yield break;
        }

        if (hasLeft && !hasRight)
        {
            yield return Tag.Left(left);
            while (iterLeft.MoveNext())
            {
                yield return Tag.Left(iterLeft.Current);
            }
            yield break;
        }

        if (hasRight && !hasLeft)
        {
            yield return Tag.Right(right);
            while (iterRight.MoveNext())
            {
                yield return Tag.Right(iterRight.Current);
            }
            yield break;
        }

        var comparison = left.CompareTo(right);
        if (comparison < 0)
        {
            yield return Tag.Left(left);
            hasRight = iterLeft.MoveNext();
        }
        else if (comparison > 0)
        {
            yield return Tag.Right(right);
            hasRight = iterRight.MoveNext();
        }
        else
        {
            hasLeft = iterLeft.MoveNext();
            hasRight = iterRight.MoveNext();
        }
    }
}
```

I believe this to be correct. I have a test suite. It is not exhaustive. If it's wrong in some way, feel free to let me know.

You'll probably notice right off the bat that this is an iterator implemented in the idiomatic C# way: it's built using the yield keyword. It is possible to implement this via the ordinar iterator protocol, which results in code similar to what I've written in Rust.

<code here>

### In Rust

This is a two-parter in Rust; here we have the constructor--which is basically a member of a trait...

```
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
```

...and here we have the actual SymDiffIter implementation:

```
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
```

## Not-so-naive internal iteration

In both Rust and C#, this code runs faster than the alternative.