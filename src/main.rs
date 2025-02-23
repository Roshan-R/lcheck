use anyhow::Error;
use itertools::Itertools;
use reqwest::Client;
use std::{collections::HashMap, fs};

mod languages;
mod license;

use languages::common::LanguageExtractor;
use license::{is_compatibile, SPDXLicense};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let json_buf = include_str!("../data/osadl-matrix.json");
    let matrix: serde_json::Value = serde_json::from_str(json_buf)?;
    let client = Client::new();

    let languages_map = HashMap::from([("pyproject.toml", languages::python::Python {})]);

    let mut language = None;
    println!("Scanning project dependencies...");
    for project_file in languages_map.keys() {
        if fs::exists(project_file).unwrap() {
            language = Some(languages_map.get(project_file).unwrap());
            break;
        }
    }

    if language.is_none() {
        panic!("Could not autodetect the project language, exiting..");
    }

    let language = language.unwrap();

    let deps = language.get_dependencies();
    let num_deps = deps.len();

    let mut tasks = Vec::new();
    for dep in deps {
        let client = client.clone();
        let extractor = language.clone();
        tasks.push(tokio::spawn(async move {
            extractor.get_license(dep, &client).await
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

    let incompatible: Vec<_> = packages
        .iter()
        .combinations(2)
        .filter(|comb| {
            let package_a = &comb[0].license;
            let package_b = &comb[1].license;
            !is_compatibile(package_a, package_b, &matrix)
        })
        .collect();

    println!("Dependency Report");
    println!("--------------------------------------------");
    println!("Total dependencies identified: {}", num_deps);
    println!();

    if !incomplete_deps.is_empty() {
        println!("Missing license information for:");
        for dep in incomplete_deps {
            println!("  -{}", dep.name.to_string());
        }
    }

    if !incompatible.is_empty() {
        println!("License conflicts detected:");
        for dep in incompatible {
            println!(
                "   - {} ({}) conflicts with {} ({})",
                dep[0].name,
                dep[0].license.unwrap(),
                dep[1].name,
                dep[1].license.unwrap()
            );
        }
        println!();
    } else {
        println!();
        println!("No conflicts detected");
    }

    Ok(())
}
