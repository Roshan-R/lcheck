use anyhow::Error;
use itertools::Itertools;
use pyproject_toml::PyProjectToml;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::string::ToString;
use strum_macros::{Display, EnumString};

#[derive(Debug, EnumString, Display, PartialEq, Clone, Copy)]
enum SPDXLicense {
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

#[derive(Serialize, Deserialize, Debug)]
struct PyPi {
    info: PyPiInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PyPiInfo {
    classifiers: Vec<String>,
    license: Option<String>,
}

impl PyPi {
    fn license(&self) -> Option<SPDXLicense> {
        let info = self.info.clone();
        let mut license = String::new();

        if let Some(lic) = info.license {
            // When the license is the whole text file and not just the license name
            if lic.len() > 100 {
                license = lic.split_whitespace().take(2).collect::<Vec<_>>().join(" ");
            }
            license = lic
        } else {
            for classifier in info.classifiers {
                let re = Regex::new(r"License :: OSI Approved :: (.*)$").unwrap();
                if let Some(caps) = re.captures(classifier.as_str()) {
                    license = caps.get(1).unwrap().as_str().to_string();
                    break;
                }
            }
        }

        return match license.parse::<SPDXLicense>() {
            Ok(license) => Some(license),
            Err(_) => None,
        };
    }
}
#[derive(Debug, Clone)]
struct PackageLicense {
    name: String,
    license: Option<SPDXLicense>,
}

async fn get_license_from_pypi(package_name: String, client: &Client) -> PackageLicense {
    let url = format!("https://pypi.org/pypi/{}/json", package_name);
    println!("Getting license information for {}", package_name);

    let resp = client.get(url).send().await.unwrap();
    let metadata: PyPi = resp.json().await.unwrap();
    PackageLicense {
        name: String::from(package_name),
        license: metadata.license(),
    }
}

fn is_compatibile(
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

#[tokio::main]
async fn main() -> Result<(), Error> {
    let json_buf = include_str!("../data/osadl-matrix.json");
    let matrix: serde_json::Value = serde_json::from_str(json_buf)?;

    let client = Client::new();
    let buf = fs::read_to_string("pyproject.toml")?;
    let pyproject: PyProjectToml = toml::from_str(buf.as_str())?;

    let mut tasks = Vec::new();
    for dep in pyproject.project.unwrap().dependencies.unwrap() {
        let name = dep.name.clone().as_ref().to_string();

        let client = client.clone();
        tasks.push(tokio::spawn(async move {
            get_license_from_pypi(name, &client).await
        }));
    }

    let mut incomplete_deps = Vec::new();
    let mut packages = Vec::new();

    for task in tasks {
        let package = task.await.unwrap();
        if package.license.is_none() {
            incomplete_deps.push(package)
        } else {
            packages.push(package);
        }
    }

    dbg!(&packages);

    let incompatible: Vec<_> = packages
        .iter()
        .combinations(2)
        .filter(|comb| {
            let package_a = &comb[0].license;
            let package_b = &comb[1].license;
            !is_compatibile(package_a, package_b, &matrix)
        })
        .collect();

    dbg!(incompatible);

    Ok(())
}
