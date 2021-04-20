use std::fmt::{Display, Formatter};
use std::str::FromStr;

use binread::io::{Read, Seek, SeekFrom};
use binread::{BinRead, BinResult, ReadOptions};

use crate::string::parse_string;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Identifier {
    namespace: String,
    path: String,
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum IdentifierError {
    InvalidNamespaceCharacter,
    InvalidPathCharacter,
}

impl Identifier {
    pub fn try_new(mut namespace: &str, path: &str) -> Result<Identifier, IdentifierError> {
        if namespace.contains(|c| !matches!(c, '_' | '-' | 'a'..='z' | '0'..='9'| '.' )) {
            return Err(IdentifierError::InvalidNamespaceCharacter);
        }
        if path.contains(|c| !matches!(c, '_' | '-' | 'a'..='z' | '0'..='9'| '.' | '/' )) {
            return Err(IdentifierError::InvalidPathCharacter);
        }
        if namespace.is_empty() {
            namespace = "minecraft";
        }
        Ok(Identifier {
            namespace: namespace.to_string(),
            path: path.to_string(),
        })
    }
}

impl FromStr for Identifier {
    type Err = IdentifierError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut namespace = "";
        let mut path = value;

        if let Some(index) = value.find(':') {
            let (ns, p) = value.split_at(index);
            // Note: path has the `:` right now, skip it
            namespace = ns;
            path = &p[1..];
        }

        Self::try_new(namespace, path)
    }
}

impl BinRead for Identifier {
    type Args = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _args: Self::Args,
    ) -> BinResult<Self> {
        parse_string(reader, options, (None,))?
            .parse()
            .or_else(|e| {
                let pos = reader.seek(SeekFrom::Current(0))?;
                Err(binread::Error::Custom {
                    pos,
                    err: Box::new(e),
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_new() {
        assert_eq!(
            Ok(Identifier {
                namespace: "minecraft".to_string(),
                path: "air".to_string(),
            }),
            Identifier::try_new("minecraft", "air")
        );
        assert_eq!(
            Ok(Identifier {
                namespace: "minecraft".to_string(),
                path: "air".to_string(),
            }),
            Identifier::try_new("", "air")
        );
        assert_eq!(
            Ok(Identifier {
                namespace: "mojang".to_string(),
                path: "air".to_string(),
            }),
            Identifier::try_new("mojang", "air")
        );
    }

    #[test]
    fn try_new_failures() {
        assert_eq!(
            Err(IdentifierError::InvalidNamespaceCharacter),
            Identifier::try_new("&&&&", "air")
        );
        // Make sure slashes only work in path
        assert_eq!(
            Err(IdentifierError::InvalidNamespaceCharacter),
            Identifier::try_new("////", "air")
        );
        assert_eq!(
            Err(IdentifierError::InvalidPathCharacter),
            Identifier::try_new("minecraft", "&&&&")
        );
    }

    #[test]
    fn try_from_str() {
        assert_eq!(
            Ok(Identifier {
                namespace: "minecraft".to_string(),
                path: "air".to_string(),
            }),
            "minecraft:air".parse::<Identifier>()
        );
        assert_eq!(
            Ok(Identifier {
                namespace: "minecraft".to_string(),
                path: "air".to_string(),
            }),
            ":air".parse::<Identifier>()
        );
        assert_eq!(
            Ok(Identifier {
                namespace: "mojang".to_string(),
                path: "air".to_string(),
            }),
            "mojang:air".parse::<Identifier>()
        );
    }

    #[test]
    fn try_from_str_failures() {
        assert_eq!(
            Err(IdentifierError::InvalidNamespaceCharacter),
            "&&&&:air".parse::<Identifier>()
        );
        // Make sure slashes only work in path
        assert_eq!(
            Err(IdentifierError::InvalidNamespaceCharacter),
            "////:air".parse::<Identifier>()
        );
        assert_eq!(
            Err(IdentifierError::InvalidPathCharacter),
            "minecraft:&&&&".parse::<Identifier>()
        );
    }
}
