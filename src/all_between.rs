use std::ops::Range;

pub trait AllBetween: Sized {
    fn all_between(self, other: Self) -> Range<Self>;
}

impl<T: PartialOrd> AllBetween for T {
    fn all_between(self, other: Self) -> Range<Self> {
        if self <= other {self..other} else {other..self}
    }
}
