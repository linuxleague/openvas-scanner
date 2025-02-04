// SPDX-FileCopyrightText: 2023 Greenbone AG
//
// SPDX-License-Identifier: GPL-2.0-or-later

use std::{fmt::Display, path::PathBuf};

use feed::VerifyError;
use nasl_interpreter::{InterpretError, LoadError};
use nasl_syntax::{SyntaxError, Token};
use storage::StorageError;

#[derive(Debug, Clone)]
pub enum CliErrorKind {
    WrongAction,

    PluginPathIsNotADir(PathBuf),
    Openvas {
        args: Option<String>,
        err_msg: String,
    },
    InterpretError(InterpretError),
    LoadError(LoadError),
    StorageError(StorageError),
    SyntaxError(SyntaxError),
    Corrupt(String),
}

impl CliErrorKind {
    pub fn as_token(&self) -> Option<&Token> {
        match self {
            CliErrorKind::InterpretError(e) => match &e.origin {
                Some(s) => s.as_token(),
                None => None,
            },
            CliErrorKind::SyntaxError(e) => e.as_token(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CliError {
    pub filename: String,
    pub kind: CliErrorKind,
}

impl Display for CliErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliErrorKind::WrongAction => write!(f, "wrong action."),
            CliErrorKind::PluginPathIsNotADir(e) => write!(f, "expected {e:?} to be a dir."),
            CliErrorKind::Openvas { args, err_msg } => write!(
                f,
                "openvas {} failed with: {err_msg}",
                args.clone().unwrap_or_default()
            ),
            CliErrorKind::InterpretError(e) => write!(f, "{e}"),
            CliErrorKind::LoadError(e) => write!(f, "{e}"),
            CliErrorKind::StorageError(e) => write!(f, "{e}"),
            CliErrorKind::SyntaxError(e) => write!(f, "{e}"),
            CliErrorKind::Corrupt(x) => write!(f, "Corrupt: {x}"),
        }
    }
}

impl Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}: {}",
            self.filename,
            self.kind
                .as_token()
                .map(|x| { format!(", line: {}, col: {}", x.position.0, x.position.1) })
                .unwrap_or_default(),
            self.kind
        )
    }
}

impl From<VerifyError> for CliError {
    fn from(value: VerifyError) -> Self {
        let filename = match &value {
            VerifyError::SumsFileCorrupt(e) => e.sum_file(),
            VerifyError::LoadError(_) => "",
            VerifyError::HashInvalid {
                expected: _,
                actual: _,
                key,
            } => key,
        };
        Self {
            filename: filename.to_string(),
            kind: CliErrorKind::Corrupt(value.to_string()),
        }
    }
}

impl From<LoadError> for CliErrorKind {
    fn from(value: LoadError) -> Self {
        Self::LoadError(value)
    }
}

impl From<InterpretError> for CliErrorKind {
    fn from(value: InterpretError) -> Self {
        Self::InterpretError(value)
    }
}

impl From<StorageError> for CliErrorKind {
    fn from(value: StorageError) -> Self {
        Self::StorageError(value)
    }
}

impl From<SyntaxError> for CliErrorKind {
    fn from(value: SyntaxError) -> Self {
        Self::SyntaxError(value)
    }
}

impl From<feed::UpdateError> for CliError {
    fn from(value: feed::UpdateError) -> Self {
        let kind = match value.kind {
            feed::UpdateErrorKind::InterpretError(e) => CliErrorKind::InterpretError(e),
            feed::UpdateErrorKind::SyntaxError(e) => CliErrorKind::SyntaxError(e),
            feed::UpdateErrorKind::StorageError(e) => CliErrorKind::StorageError(e),
            feed::UpdateErrorKind::LoadError(e) => CliErrorKind::Corrupt(load_error_to_string(&e)),
            feed::UpdateErrorKind::MissingExit(_) => {
                CliErrorKind::Corrupt("description run without exit.".to_string())
            }
            feed::UpdateErrorKind::VerifyError(e) => CliErrorKind::Corrupt(e.to_string()),
        };
        CliError {
            filename: value.key,
            kind,
        }
    }
}

fn load_error_to_string(le: &LoadError) -> String {
    match le {
        LoadError::Retry(f) => f,
        LoadError::NotFound(f) => f,
        LoadError::PermissionDenied(f) => f,
        LoadError::Dirty(f) => f,
    }
    .to_owned()
}
