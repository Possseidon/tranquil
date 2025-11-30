use std::fmt;

pub(crate) fn display_with<F: Fn(&mut fmt::Formatter) -> fmt::Result>(f: F) -> impl fmt::Display {
    struct DisplayWith<F> {
        f: F,
    }

    impl<F: Fn(&mut fmt::Formatter) -> fmt::Result> fmt::Display for DisplayWith<F> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            (self.f)(f)
        }
    }

    DisplayWith { f }
}

macro_rules! display {
    (move $($arg:tt)*) => { $crate::display::display_with(move |f| write!(f, $($arg)*)) };
    ($($arg:tt)*) => { $crate::display::display_with(|f| write!(f, $($arg)*)) };
}

pub(crate) use display;
