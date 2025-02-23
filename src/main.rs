use anyhow::Error;
use itertools::Itertools;
use reqwest::Client;
use std::{collections::HashMap, fs};

mod languages;
mod license;

use languages::common::LanguageExtractor;
use license::{is_compatibile, SPDXLicense};

use colored::Colorize;

use clap::Parser;

/// cli tool to check license compatibility across your project dependencies
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();

    let json_buf = include_str!("../data/osadl-matrix.json");
    let matrix: serde_json::Value = serde_json::from_str(json_buf)?;
    let client = Client::new();

    let languages_map = HashMap::from([("pyproject.toml", languages::python::Python {})]);

    let mut language = None;
    println!("{}", "Scanning project dependencies...".blue());
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

    println!("{}", "Dependency Scan Summary".green());
    println!("--------------------------------------------");
    println!("{} {}", "Total dependencies:".yellow(), num_deps);
    println!();

    if args.verbose {
        println!("All dependencies: ");
        let mut all_deps = packages.clone();
        let incomplete_deps = incomplete_deps.clone();
        all_deps.extend(incomplete_deps);

        for dep in all_deps {
            let license = match dep.license {
                Some(l) => l.to_string(),
                None => "Unknown".to_string(),
            };
            println!("  - {:<20} ({})", dep.name, license);
        }
        println!();
    }

    if !incomplete_deps.is_empty() {
        println!("{}", "Missing license information for:".red());
        for dep in incomplete_deps {
            println!("  -{}", dep.name.to_string());
        }
    }

    if !incompatible.is_empty() {
        println!("{}", "License conflicts detected.".red());
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
        println!("{}", "No license conflicts detected".green());
    }

    Ok(())
}
