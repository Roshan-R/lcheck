use crate::languages::common::LanguageExtractor;
use crate::license::PackageLicense;
use crate::SPDXLicense;
use pyproject_toml::PyProjectToml;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::string::ToString;

#[derive(Clone, Debug)]
pub struct Python {}

impl LanguageExtractor for Python {
    async fn get_license(&self, package_name: String, client: &Client) -> PackageLicense {
        let url = format!("https://pypi.org/pypi/{}/json", package_name);

        let resp = client.get(url).send().await.unwrap();
        let metadata: PyPi = resp.json().await.unwrap();
        PackageLicense {
            name: String::from(package_name),
            license: metadata.license(),
        }
    }
    fn get_dependencies(&self) -> Vec<String> {
        println!("Python project detected (pyproject.toml found)");
        println!();
        let buf = fs::read_to_string("pyproject.toml").unwrap();
        let pyproject: PyProjectToml = toml::from_str(buf.as_str())
            .expect("Could not parse pyproject.toml, check if it's correct or not");

        return pyproject
            .project
            .expect("No project field found in pyproject.toml")
            .dependencies
            .expect("No dependencies found in pyproject.toml")
            .to_vec()
            .iter()
            .map(|t| t.name.to_string())
            .collect();
    }
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
