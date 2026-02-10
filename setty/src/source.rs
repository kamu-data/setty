use std::{borrow::Cow, marker::PhantomData, path::PathBuf};

use crate::Value;
use crate::errors::ReadError;
use crate::format::Format;

/////////////////////////////////////////////////////////////////////////////////////////

/// A source of configuration data.
///
/// Implementations return an optional `serde_json::Value` and provide a
/// short `name()` for diagnostics.
pub trait Source {
    /// Human-readable source name (used in error messages and docs).
    fn name(&self) -> std::borrow::Cow<'static, str>;

    /// Load the source and return `Ok(Some(value))` if present, `Ok(None)`
    /// if the source is absent, or `Err(ReadError)` on error.
    fn load(&self) -> Result<Option<Value>, ReadError>;
}

/////////////////////////////////////////////////////////////////////////////////////////

/// Allows providing [`Value`] as a [`Source`]
impl Source for Value {
    fn name(&self) -> std::borrow::Cow<'static, str> {
        "<raw value>".into()
    }

    fn load(&self) -> Result<Option<Value>, ReadError> {
        // TODO: Avoid cloning
        Ok(Some(self.clone()))
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

/// Allows passing a raw string of data as a [`Source`]
pub struct RawData<Fmt> {
    val: String,
    _p: PhantomData<Fmt>,
}

impl<Fmt> Clone for RawData<Fmt> {
    fn clone(&self) -> Self {
        Self {
            val: self.val.clone(),
            _p: PhantomData,
        }
    }
}

impl<Fmt> RawData<Fmt>
where
    Fmt: Format,
{
    pub fn new(val: impl Into<String>) -> Self {
        Self {
            val: val.into(),
            _p: PhantomData,
        }
    }
}

impl<Fmt> Source for RawData<Fmt>
where
    Fmt: Format,
{
    fn name(&self) -> std::borrow::Cow<'static, str> {
        format!("<raw {} data>", Fmt::name()).into()
    }

    fn load(&self) -> Result<Option<Value>, ReadError> {
        let v = Fmt::deserialize(&self.val).map_err(|e| ReadError::Serde(e.into()))?;
        Ok(Some(v))
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

/// File-backed [`Source`] that reads configuration from a file on disk.
///
/// By default `File::new(path)` is required (missing file causes an IO error).
/// Use `required(false)` to make the file optional.
pub struct File<Fmt> {
    path: PathBuf,
    required: bool,
    _p: PhantomData<Fmt>,
}

impl<Fmt> Clone for File<Fmt> {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            required: self.required,
            _p: PhantomData,
        }
    }
}

impl<Fmt> File<Fmt>
where
    Fmt: Format,
{
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            required: true,
            _p: PhantomData,
        }
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
}

impl<Fmt> Source for File<Fmt>
where
    Fmt: Format,
{
    fn name(&self) -> std::borrow::Cow<'static, str> {
        self.path.display().to_string().into()
    }

    fn load(&self) -> Result<Option<Value>, ReadError> {
        if !self.required && !self.path.is_file() {
            return Ok(None);
        }

        // TODO: Use reader
        // TODO: Carry file name info
        let s = std::fs::read_to_string(&self.path)?;
        let v = Fmt::deserialize(&s).map_err(|e| ReadError::Serde(e.into()))?;
        Ok(Some(v))
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

/// Environment variable based [`Source`].
///
/// `Env` looks for variables starting with `prefix` and splits the remainder
/// of the variable name by `separator` to produce nested keys in the
/// resulting JSON object.
pub struct Env<Fmt> {
    prefix: String,
    separator: Cow<'static, str>,
    _p: PhantomData<Fmt>,
}

impl<Fmt> Clone for Env<Fmt> {
    fn clone(&self) -> Self {
        Self {
            prefix: self.prefix.clone(),
            separator: self.separator.clone(),
            _p: PhantomData,
        }
    }
}

impl<Fmt> Env<Fmt>
where
    Fmt: Format,
{
    pub fn new(prefix: impl Into<String>, separator: impl Into<Cow<'static, str>>) -> Self {
        Self {
            prefix: prefix.into(),
            separator: separator.into(),
            _p: PhantomData,
        }
    }
}

impl<Fmt> Source for Env<Fmt>
where
    Fmt: Format,
{
    fn name(&self) -> std::borrow::Cow<'static, str> {
        format!("Env {}*{}**", self.prefix, self.separator).into()
    }

    fn load(&self) -> Result<Option<Value>, ReadError> {
        let mut ret = Value::Object(Default::default());

        for (name, value) in std::env::vars() {
            let Some(suffix) = name.strip_prefix(&self.prefix) else {
                continue;
            };

            let value = Fmt::deserialize(&value).map_err(|e| ReadError::Serde(e.into()))?;

            let mut current = &mut ret;
            let mut segments = suffix.split(self.separator.as_ref()).peekable();

            while let Some(seg) = segments.next() {
                if segments.peek().is_some() {
                    current = &mut current[seg];
                } else {
                    current[seg] = value;
                    break;
                }
            }
        }

        Ok(Some(ret))
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
