use std::hash::{Hash, Hasher};
use std::path::Path;

use utils::HashValue;

/// A `Location` describes where the source data for a resource is located. It
/// usually contains a Path that can be resolved to an URI. `Location`s are also
/// used for sharing. If 2 `Location`s are completely identical, they identify
/// the same resource.
#[derive(Debug, Clone, Copy)]
pub struct Location<'a> {
    code: Signature,
    location: &'a Path,
}

impl<'a> Location<'a> {
    pub fn unique<P>(location: &'a P) -> Self
    where
        P: ?Sized + AsRef<Path>,
    {
        Location {
            location: location.as_ref(),
            code: Signature::Unique,
        }
    }

    pub fn shared<P>(location: &'a P) -> Self
    where
        P: ?Sized + AsRef<Path>,
    {
        Location {
            location: location.as_ref(),
            code: Signature::Shared,
        }
    }

    pub fn token<P>(code: u8, location: &'a P) -> Self
    where
        P: ?Sized + AsRef<Path>,
    {
        Location {
            location: location.as_ref(),
            code: Signature::TokenShared(code),
        }
    }

    /// Returns true if this location is shared.
    #[inline]
    pub fn is_shared(&self) -> bool {
        self.code.is_shared()
    }

    /// Gets the uniform resource identifier.
    #[inline]
    pub fn uri(&self) -> &Path {
        self.location
    }

    /// Gets hash object of `Location`.
    #[inline]
    pub fn hash(&self) -> LocationAtom {
        LocationAtom {
            code: self.code,
            location: self.location.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum Signature {
    Unique,
    Shared,
    TokenShared(u8),
}

impl Signature {
    #[inline]
    pub fn is_shared(&self) -> bool {
        if let Signature::Unique = *self {
            false
        } else {
            true
        }
    }
}

impl<'a> PartialEq for Location<'a> {
    fn eq(&self, other: &Self) -> bool {
        if self.code.is_shared() {
            self.code == other.code && self.location == other.location
        } else {
            false
        }
    }
}

impl<'a> Eq for Location<'a> {}

impl<'a> Hash for Location<'a> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        if self.code.is_shared() {
            self.code.hash(state);
            self.location.hash(state);
        } else {
            panic!("Trying to hash unique location.");
        }
    }
}

impl<'a> Default for Location<'a> {
    fn default() -> Self {
        Location {
            code: Signature::Unique,
            location: Path::new(""),
        }
    }
}

/// Hash object of `Location`.
#[derive(Debug, Clone, Copy)]
pub struct LocationAtom {
    code: Signature,
    location: HashValue<Path>,
}

impl<'a> From<Location<'a>> for LocationAtom {
    fn from(v: Location) -> Self {
        LocationAtom {
            code: v.code,
            location: v.location.into(),
        }
    }
}

impl LocationAtom {
    /// Returns true if this location is shared.
    #[inline]
    pub fn is_shared(&self) -> bool {
        self.code.is_shared()
    }
}

impl PartialEq for LocationAtom {
    fn eq(&self, other: &Self) -> bool {
        if self.code.is_shared() {
            self.code == other.code && self.location == other.location
        } else {
            false
        }
    }
}

impl Eq for LocationAtom {}

impl Hash for LocationAtom {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        if self.code.is_shared() {
            self.code.hash(state);
            self.location.hash(state);
        } else {
            panic!("Trying to hash unique location.");
        }
    }
}

impl Default for LocationAtom {
    fn default() -> Self {
        Location::default().into()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn basic() {
        let l1 = Location::unique("1");
        let l2 = Location::unique("1");
        assert!(l1 != l2);

        let l1 = Location::token(0, "1");
        let l2 = Location::token(1, "1");
        assert!(l1 != l2);

        let l1 = Location::token(0, "1");
        let l2 = Location::token(0, "1");
        assert!(l1 == l2);

        let l1 = Location::shared("1");
        let l2 = Location::shared("1");
        assert!(l1 == l2);
    }

    #[test]
    fn shared_container() {
        let l1 = Location::token(0, "1").hash();
        let l2 = Location::token(0, "2").hash();

        let mut map = HashSet::new();
        assert_eq!(map.insert(l1), true);
        assert_eq!(map.contains(&l1), true);
        assert_eq!(map.contains(&l2), false);

        assert_eq!(map.insert(l2), true);
        assert_eq!(map.contains(&l1), true);
        assert_eq!(map.contains(&l2), true);

        assert_eq!(map.insert(l1), false);
    }
}
