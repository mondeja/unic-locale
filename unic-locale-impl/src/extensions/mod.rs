//! Unicode Extensions provide a mechanism to extend the `LanguageIdentifier` with
//! additional bits of information.
//!
//! There are four types of extensions:
//!
//!  * Unicode Extensions - marked as `u`.
//!  * Transform Extensions - marked as `t`.
//!  * Private Use Extensions - marked as `x`.
//!  * Other extensions - marked as any `a-z` except of `u`, `t` and `x`.
mod private;
mod transform;
mod unicode;

pub use private::PrivateExtensionList;
pub use transform::TransformExtensionList;
pub use unicode::UnicodeExtensionList;

use std::collections::BTreeMap;
use std::fmt::Write;
use std::iter::Peekable;
use std::str::FromStr;

use tinystr::TinyStr8;

use crate::parser::ParserError;

/// Defines the type of extension.
#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash, PartialOrd, Ord)]
pub enum ExtensionType {
    /// Transform Extension Type marked as `t`.
    Transform,
    /// Unicode Extension Type marked as `u`.
    Unicode,
    /// Private Extension Type marked as `x`.
    Private,
    /// Other Extension Type marked as `a-z` except of `t`, `u` and `x`.
    Other(char),
}

impl ExtensionType {
    pub fn from_byte(key: u8) -> Result<Self, ParserError> {
        let key = key.to_ascii_lowercase();
        match key {
            b'u' => Ok(ExtensionType::Unicode),
            b't' => Ok(ExtensionType::Transform),
            b'x' => Ok(ExtensionType::Private),
            sign if sign.is_ascii_alphanumeric() => Ok(ExtensionType::Other(char::from(sign))),
            _ => Err(ParserError::InvalidExtension),
        }
    }
}

impl std::fmt::Display for ExtensionType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let ch = match self {
            ExtensionType::Unicode => 'u',
            ExtensionType::Transform => 't',
            ExtensionType::Other(n) => *n,
            ExtensionType::Private => 'x',
        };
        f.write_char(ch)
    }
}

/// A map of extensions associated with a given `Locale.
#[derive(Debug, Default, PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub struct ExtensionsMap {
    pub unicode: UnicodeExtensionList,
    pub transform: TransformExtensionList,
    pub other: BTreeMap<char, Vec<TinyStr8>>,
    pub private: PrivateExtensionList,
}

impl ExtensionsMap {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ParserError> {
        let mut iterator = bytes.split(|c| *c == b'-' || *c == b'_').peekable();
        Self::try_from_iter(&mut iterator)
    }

    pub(crate) fn try_from_iter<'a>(
        iter: &mut Peekable<impl Iterator<Item = &'a [u8]>>,
    ) -> Result<Self, ParserError> {
        let mut result = ExtensionsMap::default();

        let mut st = iter.next();
        while let Some(subtag) = st {
            match subtag.first().map(|b| ExtensionType::from_byte(*b)) {
                Some(Ok(ExtensionType::Unicode)) => {
                    result.unicode = UnicodeExtensionList::try_from_iter(iter)?;
                }
                Some(Ok(ExtensionType::Transform)) => {
                    result.transform = TransformExtensionList::try_from_iter(iter)?;
                }
                Some(Ok(ExtensionType::Private)) => {
                    result.private = PrivateExtensionList::try_from_iter(iter)?;
                }
                None => {}
                _ => unimplemented!(),
            }

            st = iter.next();
        }

        Ok(result)
    }

    pub fn is_empty(&self) -> bool {
        self.unicode.is_empty() && self.transform.is_empty() && self.private.is_empty()
    }
}

impl FromStr for ExtensionsMap {
    type Err = ParserError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        Self::from_bytes(source.as_bytes())
    }
}

impl std::fmt::Display for ExtensionsMap {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Alphabetic by singleton (t, u, x)
        write!(f, "{}{}{}", self.transform, self.unicode, self.private)?;

        Ok(())
    }
}
