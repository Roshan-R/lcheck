use crate::license::PackageLicense;
use reqwest::Client;

pub trait LanguageExtractor {
    async fn get_license(&self, package_name: String, client: &Client) -> PackageLicense;
}
