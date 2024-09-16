use ::core::fmt::Debug;
use ::std::collections::HashMap;

pub mod alloc;
pub mod condition;
pub mod core;
#[cfg(feature = "reqwest")]
pub mod reqwest;
#[cfg(feature = "std")]
pub mod std;
#[cfg(feature = "tokio")]
pub mod tokio;

// TODO: Impl for ranges.
// TODO: Impl for dir?
// TODO: Impl for option?
pub trait HasLength {
    fn is_empty(&self) -> bool {
        self.length() == 0
    }

    fn is_not_empty(&self) -> bool {
        !self.is_empty()
    }

    fn length(&self) -> usize;

    fn type_name_hint(&self) -> Option<&'static str> {
        None
    }
}

impl HasLength for &str {
    fn is_empty(&self) -> bool {
        str::is_empty(self)
    }

    fn length(&self) -> usize {
        str::len(self)
    }
}

impl HasLength for String {
    fn is_empty(&self) -> bool {
        String::is_empty(self)
    }

    fn length(&self) -> usize {
        String::len(self)
    }
}

impl HasLength for &String {
    fn is_empty(&self) -> bool {
        String::is_empty(self)
    }

    fn length(&self) -> usize {
        String::len(self)
    }
}

impl<T> HasLength for &[T] {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<T, const S: usize> HasLength for [T; S] {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<T> HasLength for Vec<T> {
    fn is_empty(&self) -> bool {
        Vec::is_empty(self)
    }

    fn length(&self) -> usize {
        Vec::len(self)
    }
}

impl<T> HasLength for &Vec<T> {
    fn is_empty(&self) -> bool {
        Vec::is_empty(self)
    }

    fn length(&self) -> usize {
        Vec::len(self)
    }
}

impl<K: Debug, V: Debug> HasLength for HashMap<K, V> {
    fn is_empty(&self) -> bool {
        HashMap::is_empty(self)
    }

    fn length(&self) -> usize {
        HashMap::len(self)
    }

    fn type_name_hint(&self) -> Option<&'static str> {
        Some("HashMap")
    }
}

impl<K: Debug, V: Debug> HasLength for &HashMap<K, V> {
    fn is_empty(&self) -> bool {
        HashMap::is_empty(self)
    }

    fn length(&self) -> usize {
        HashMap::len(self)
    }

    fn type_name_hint(&self) -> Option<&'static str> {
        Some("&HashMap")
    }
}
