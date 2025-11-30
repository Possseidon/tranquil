use std::fmt;

pub trait MarkdownList: Sized + IntoIterator {
    fn markdown_list(self) -> DisplayMarkdownList<Self::IntoIter> {
        DisplayMarkdownList {
            iter: self.into_iter(),
            nesting: 0,
            style: Default::default(),
        }
    }
}

impl<T: IntoIterator> MarkdownList for T {}

#[derive(Default)]
pub enum MarkdownListStyle {
    #[default]
    Unordered,
    Ordered,
}

/// [`fmt::Display`]s an [`Iterator`] as a markdown list.
pub struct DisplayMarkdownList<I> {
    iter: I,
    nesting: usize,
    style: MarkdownListStyle,
}

impl<I: Clone + Iterator<Item: fmt::Display>> fmt::Display for DisplayMarkdownList<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nesting = |f: &mut fmt::Formatter| -> fmt::Result {
            for _ in 0..self.nesting {
                write!(f, "  ")?;
            }
            Ok(())
        };

        let mut iter = self.iter.clone();
        match self.style {
            MarkdownListStyle::Unordered => {
                if let Some(line) = iter.next() {
                    nesting(f)?;
                    write!(f, "- {line}")?;
                    for line in iter {
                        writeln!(f)?;
                        nesting(f)?;
                        write!(f, "- {line}")?;
                    }
                }
            }
            MarkdownListStyle::Ordered => {
                if let Some(line) = iter.next() {
                    nesting(f)?;
                    write!(f, "1. {line}")?;
                    for (index, line) in iter.enumerate() {
                        writeln!(f)?;
                        nesting(f)?;
                        write!(f, "{}. {line}", index + 2)?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl<I> DisplayMarkdownList<I> {
    pub fn style(mut self, style: MarkdownListStyle) -> Self {
        self.style = style;
        self
    }

    pub fn nesting(mut self, nesting: usize) -> Self {
        self.nesting = nesting;
        self
    }
}

#[test]
fn test() {
    let x = (1..=5)
        .map(|x| format!("numbers:\n{}", (1..=x).markdown_list().nesting(1)))
        .markdown_list();
    panic!("{x}");
}
