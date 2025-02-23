use strum_macros::{Display, EnumString};

#[derive(Debug, Clone)]
pub struct PackageLicense {
    pub name: String,
    pub license: Option<SPDXLicense>,
}

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
    #[strum(
        serialize = "Python Software Foundation License",
        to_string = "Python-2.0"
    )]
    PSFL,
}

pub fn is_compatibile(
    license_a: &Option<SPDXLicense>,
    license_b: &Option<SPDXLicense>,
    matrix: &serde_json::Value,
) -> bool {
    let (Some(license_a), Some(license_b)) = (license_a, license_b) else {
        return false;
    };

    let result = matrix
        .get(license_a.to_string())
        .and_then(|matrix_a| matrix_a.get(license_b.to_string()))
        .and_then(|value| value.as_str());

    match result {
        Some("Yes") | Some("Same") => true,
        Some(other) => {
            eprintln!("Unexpected value in matrix: {}", other);
            false
        }
        None => {
            eprintln!(
                "Entry not found in matrix for {} and {}",
                license_a.to_string(),
                license_b.to_string()
            );
            false
        }
    }
}
