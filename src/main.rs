use anyhow::Error;
use itertools::Itertools;
use pyproject_toml::PyProjectToml;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use strum_macros::{Display, EnumString};

#[derive(Debug, EnumString, Display, PartialEq)]
enum SPDXLicense {
    #[strum(serialize = "MIT", serialize = "MIT License")]
    MIT,
    #[strum(
        serialize = "Apache-2.0",
        serialize = "Apache 2.0",
        serialize = "Apache",
        serialize = "Apache License 2.0"
    )]
    Apache2,
    #[strum(serialize = "Mozilla Public License 2.0 (MPL 2.0)")]
    MPL,
    #[strum(serialize = "GPL-3.0")]
    GPL3,
    #[strum(serialize = "BSD-3-Clause")]
    BSD3,
    #[strum(serialize = "Python Software Foundation License")]
    PSFL, // Add more licenses as needed
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
                license = String::from_iter(lic.split_whitespace().take(2));
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
#[derive(Debug)]
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

fn is_compatibile(packages: &Vec<&PackageLicense>) -> bool {
    dbg!(packages);
    false
}

#[tokio::main]
async fn main() -> Result<(), Error> {
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
    let mut incompatible = Vec::new();

    for task in tasks {
        let package = task.await.unwrap();
        if package.license.is_none() {
            incomplete_deps.push(package)
        } else {
            packages.push(package);
        }
    }

    for comb in packages.iter().combinations(2) {
        if !is_compatibile(&comb) {
            incompatible.push(comb);
        }
    }

    Ok(())
}
