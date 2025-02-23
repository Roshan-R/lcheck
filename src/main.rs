use anyhow::Error;
use itertools::Itertools;
use pyproject_toml::PyProjectToml;
use reqwest::Client;
use std::fs;

mod languages;
mod license;

use languages::common::LanguageExtractor;
use license::{is_compatibile, SPDXLicense};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let json_buf = include_str!("../data/osadl-matrix.json");
    let matrix: serde_json::Value = serde_json::from_str(json_buf)?;

    let client = Client::new();
    let buf = fs::read_to_string("pyproject.toml")?;
    let pyproject: PyProjectToml = toml::from_str(buf.as_str())?;

    let extractor = languages::python::Python {};

    let mut tasks = Vec::new();
    for dep in pyproject.project.unwrap().dependencies.unwrap() {
        let name = dep.name.clone().as_ref().to_string();

        let client = client.clone();
        let extractor = extractor.clone();
        tasks.push(tokio::spawn(async move {
            extractor.get_license(name, &client).await
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
