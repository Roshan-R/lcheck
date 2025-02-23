use strum_macros::{Display, EnumString};

#[derive(Debug, EnumString, Display, PartialEq, Clone, Copy)]
pub enum SPDXLicense {
    #[strum(serialize = "MIT", serialize = "MIT License", to_string = "MIT")]
    MIT,
    #[strum(
        serialize = "Apache-2.0",
        serialize = "Apache 2.0",
        serialize = "Apache",
        serialize = "Apache License 2.0",
        to_string = "Apache-2.0"
    )]
    Apache2,
    #[strum(serialize = "Mozilla Public License 2.0 (MPL 2.0)")]
    MPL,
    #[strum(serialize = "GPL-3.0", to_string = "GPL-3.0-only")]
    GPL3,
    #[strum(serialize = "BSD-3-Clause", to_string = "BSD-3-Clause")]
    BSD3,
    #[strum(serialize = "Python Software Foundation License")]
    PSFL,
}
