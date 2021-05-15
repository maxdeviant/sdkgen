use std::convert::TryFrom;
use std::fmt;

#[derive(Debug, Clone)]
pub struct NonEmptyString(String);

impl From<NonEmptyString> for String {
    fn from(value: NonEmptyString) -> String {
        value.0
    }
}

impl TryFrom<String> for NonEmptyString {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err("String is empty.");
        }

        if value.trim().is_empty() {
            return Err("String consists only of whitespace characters.");
        }

        Ok(NonEmptyString(value))
    }
}

impl fmt::Display for NonEmptyString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
