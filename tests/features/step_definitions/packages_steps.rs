use cucumber::{given, when, then};
use thirtyfour::prelude::*;
use mainlib::packages::{Package, PackageVersion};
use mainlib::test::DB_POOL;

use super::super::world::TestWorld;

#[given("there is a package in the system")]
async fn package_in_system(_world: &mut TestWorld) {
    let uid = Package::create_test_package(&"test-package".to_string(), &"https://github.com/Elements-Studio/starswap-core".to_string(), &"package_description".to_string(), &"first_version".to_string(), &"first_readme_content".to_string(), &DB_POOL).await.unwrap();
    PackageVersion::create(uid, "second_version".to_string(), "second_readme_content".to_string(), &DB_POOL).await.unwrap();
}

#[when("I access the package details page")]
async fn visit_home_page(world: &mut TestWorld) {
    world.driver.get("http://localhost:17002/packages/test-package").await.unwrap();
}

#[then("I should see information of that package")]
async fn see_home_page(world: &mut TestWorld) {
  let package_name_element = world.driver.find_element(By::ClassName("package-name")).await.unwrap();
  let package_name = package_name_element.text().await.unwrap();
  assert_eq!(package_name, "test-package");

  let package_description_element = world.driver.find_element(By::ClassName("package-description")).await.unwrap();
  let package_description = package_description_element.text().await.unwrap();
  assert_eq!(package_description, "package_description");
}

#[when("I click on versions of that package")]
async fn click_on_versions_tab(world: &mut TestWorld) {
    let versions_tab_element = world.driver.find_element(By::ClassName("tab-versions")).await.unwrap();
    versions_tab_element.click().await.unwrap();
}

#[then("I should see the versions of that package by latest")]
async fn see_latest_versions(world: &mut TestWorld) {
    let version_item_elements = world.driver.find_elements(By::ClassName("package-version-number")).await.unwrap();

    let first_version_item_element = version_item_elements.first().unwrap();
    let first_version_text = first_version_item_element.text().await.unwrap();
    assert_eq!(first_version_text, "second_version");

    let second_version_item_element = version_item_elements.last().unwrap();
    let second_version_text = second_version_item_element.text().await.unwrap();
    assert_eq!(second_version_text, "first_version");
}

#[when("I sort the package versions by oldest")]
async fn sort_versions_by_oldest(world: &mut TestWorld) {
    let select_element = world.driver.find_element(By::ClassName("select2-container")).await.unwrap();
    select_element.click().await.unwrap();

    let dropdown_element = world.driver.find_element(By::ClassName("select2-dropdown")).await.unwrap();
    let option_elements = dropdown_element.find_elements(By::ClassName("select2-results__option")).await.unwrap();
    option_elements[1].click().await.unwrap();
}

#[then("I should see the versions of that package by oldest")]
async fn see_oldest_versions(world: &mut TestWorld) {
    let version_item_elements = world.driver.find_elements(By::ClassName("package-version-number")).await.unwrap();

    let first_version_item_element = version_item_elements.first().unwrap();
    let first_version_text = first_version_item_element.text().await.unwrap();
    assert_eq!(first_version_text, "first_version");

    let second_version_item_element = version_item_elements.last().unwrap();
    let second_version_text = second_version_item_element.text().await.unwrap();
    assert_eq!(second_version_text, "second_version");
}
