use super::*;
use crate::accounts::forms::NewAccountForm;
use crate::test::{DatabaseTestContext, DB_POOL};
use jelly::forms::{EmailField, PasswordField};

use crate::github_service::GithubRepoData;

async fn setup() -> Result<(), Error> {
    let pool = &DB_POOL;
    Package::create_test_package(
        &"The first package".to_string(),
        &"".to_string(),
        &"description 1".to_string(),
        &"1.0.0".to_string(),
        &"".to_string(),
        &"".to_string(),
        0,
        0,
        None,
        &pool,
    )
    .await?;
    Package::create_test_package(
        &"The first Diva".to_string(),
        &"".to_string(),
        &"randomly picked, and changes some".to_string(),
        &"1.0.0".to_string(),
        &"".to_string(),
        &"".to_string(),
        0,
        0,
        None,
        &pool,
    )
    .await?;
    Package::create_test_package(
        &"Charles Diya".to_string(),
        &"".to_string(),
        &"randomly picked, and changes some".to_string(),
        &"1.0.0".to_string(),
        &"".to_string(),
        &"".to_string(),
        0,
        0,
        None,
        &pool,
    )
    .await?;
    Ok(())
}

#[actix_rt::test]
async fn search_by_single_word_works() {
    crate::test::init();
    let _ctx = DatabaseTestContext::new();
    setup().await.unwrap();
    let pool = &DB_POOL;
    let search_query = "package";
    let (search_result, total_count, total_pages) = Package::search(
        search_query,
        &PackageSortField::Name,
        &PackageSortOrder::Desc,
        Some(1),
        None,
        pool,
    )
    .await
    .unwrap();
    assert_eq!(total_count, 1);
    assert_eq!(total_pages, 1);
    assert_eq!(search_result[0].name, "The first package");
}

#[actix_rt::test]
async fn search_by_multiple_words_works() {
    crate::test::init();
    let _ctx = DatabaseTestContext::new();
    setup().await.unwrap();
    let pool = &DB_POOL;
    let search_query = "the package";
    let (search_result, total_count, total_pages) = Package::search(
        search_query,
        &PackageSortField::Name,
        &PackageSortOrder::Desc,
        Some(1),
        None,
        pool,
    )
    .await
    .unwrap();
    assert_eq!(total_count, 1);
    assert_eq!(total_pages, 1);
    assert_eq!(search_result[0].name, "The first package");
}

#[actix_rt::test]
async fn search_return_multiple_result() {
    crate::test::init();
    let _ctx = DatabaseTestContext::new();
    setup().await.unwrap();
    let pool = &DB_POOL;
    let search_query = "first";
    let (search_result, total_count, total_pages) = Package::search(
        search_query,
        &PackageSortField::Name,
        &PackageSortOrder::Desc,
        Some(1),
        Some(1),
        pool,
    )
    .await
    .unwrap();
    assert_eq!(total_count, 2);
    assert_eq!(total_pages, 2);

    assert_eq!(search_result.len(), 1);
    assert_eq!(search_result[0].name, "The first package");

    let (search_result, _total_count, _total_pages) = Package::search(
        search_query,
        &PackageSortField::Name,
        &PackageSortOrder::Desc,
        Some(2),
        Some(1),
        pool,
    )
    .await
    .unwrap();
    assert_eq!(search_result.len(), 1);
    assert_eq!(search_result[0].name, "The first Diva");
}

#[actix_rt::test]
async fn all_packages_with_pagination() {
    crate::test::init();
    let _ctx = DatabaseTestContext::new();
    setup().await.unwrap();
    let pool = &DB_POOL;
    let (search_result, total_count, total_pages) = Package::all_packages(
        &PackageSortField::Name,
        &PackageSortOrder::Desc,
        Some(1),
        Some(2),
        pool,
    )
    .await
    .unwrap();
    assert_eq!(total_count, 3);
    assert_eq!(total_pages, 2);

    assert_eq!(search_result.len(), 2);
    assert_eq!(search_result[0].name, "The first package");
    assert_eq!(search_result[1].name, "The first Diva");

    let (search_result, _total_count, _total_pages) = Package::all_packages(
        &PackageSortField::Name,
        &PackageSortOrder::Desc,
        Some(2),
        Some(2),
        pool,
    )
    .await
    .unwrap();
    assert_eq!(search_result.len(), 1);
    assert_eq!(search_result[0].name, "Charles Diya");
}

#[actix_rt::test]
async fn create_package_works() {
    crate::test::init();
    let _ctx = DatabaseTestContext::new();

    let form = NewAccountForm {
        email: EmailField {
            value: "email@host.com".to_string(),
            errors: vec![],
        },
        password: PasswordField {
            value: "So$trongpas0word!".to_string(),
            errors: vec![],
            hints: vec![],
        },
    };
    let uid = Account::register(&form, &DB_POOL).await.unwrap();

    let mut mock_github_service = GithubService::new();
    mock_github_service
        .expect_fetch_repo_data()
        .withf(|x: &String, y: &Option<String>, z: &Option<String>| {
            x == &"repo_url".to_string() && y.is_none() && z.is_none()
        })
        .returning(|_, _, _| {
            Ok(GithubRepoData {
                name: "name".to_string(),
                version: "version".to_string(),
                readme_content: "readme_content".to_string(),
                description: "".to_string(),
                size: 0,
                url: "".to_string(),
                rev: "".to_string(),
            })
        });

    let uid = Package::create(
        &"repo_url".to_string(),
        &"package_description".to_string(),
        &"1".to_string(),
        2,
        100,
        Some(uid),
        &mock_github_service,
        None,
        &DB_POOL,
    )
    .await
    .unwrap();

    let package = Package::get(uid, &DB_POOL).await.unwrap();
    assert_eq!(package.name, "name");
    assert_eq!(package.description, "package_description");

    let package_version =
        &PackageVersion::from_package_id(uid, &PackageVersionSort::Latest, &DB_POOL)
            .await
            .unwrap()[0];
    assert_eq!(package_version.version, "version");
    match &package_version.readme_content {
        Some(content) => {
            assert_eq!(content, "readme_content");
        }
        None => {
            panic!("readme content is wrong")
        }
    }

    // Asserts that no new version is created with different account id
    let mut mock_github_service_2 = GithubService::new();
    mock_github_service_2
        .expect_fetch_repo_data()
        .withf(|x: &String, y: &Option<String>, z: &Option<String>| {
            x == &"repo_url".to_string() && y.is_none() && z.is_none()
        })
        .returning(|_, _, _| {
            Ok(GithubRepoData {
                name: "name".to_string(),
                version: "version_2".to_string(),
                readme_content: "readme_content".to_string(),
                description: "".to_string(),
                size: 0,
                url: "".to_string(),
                rev: "".to_string(),
            })
        });

    let uid = Package::create(
        &"repo_url".to_string(),
        &"package_description".to_string(),
        &"1".to_string(),
        2,
        100,
        None,
        &mock_github_service_2,
        None,
        &DB_POOL,
    )
    .await
    .unwrap();

    assert_eq!(package.id, uid);
    let versions = PackageVersion::from_package_id(uid, &PackageVersionSort::Latest, &DB_POOL)
        .await
        .unwrap();

    assert_eq!(versions.len(), 1);
}

#[actix_rt::test]
async fn get_versions_by_latest() {
    crate::test::init();
    let _ctx = DatabaseTestContext::new();

    let mut mock_github_service = GithubService::new();
    mock_github_service
        .expect_fetch_repo_data()
        .returning(|_, _, _| {
            Ok(GithubRepoData {
                name: "name".to_string(),
                version: "first_version".to_string(),
                readme_content: "first_readme_content".to_string(),
                description: "".to_string(),
                size: 0,
                url: "".to_string(),
                rev: "".to_string(),
            })
        });

    let uid = Package::create(
        &"repo_url".to_string(),
        &"package_description".to_string(),
        &"1".to_string(),
        2,
        100,
        None,
        &mock_github_service,
        None,
        &DB_POOL,
    )
    .await
    .unwrap();

    PackageVersion::create(
        uid,
        "second_version".to_string(),
        "second_readme_content".to_string(),
        "1".to_string(),
        2,
        100,
        &DB_POOL,
    )
    .await
    .unwrap();

    let versions = PackageVersion::from_package_id(uid, &PackageVersionSort::Latest, &DB_POOL)
        .await
        .unwrap();

    assert_eq!(versions.len(), 2);
    assert_eq!(versions[0].version, "second_version");
    assert_eq!(versions[1].version, "first_version");
}

#[actix_rt::test]
async fn get_versions_by_oldest() {
    crate::test::init();
    let _ctx = DatabaseTestContext::new();
    let mut mock_github_service = GithubService::new();
    mock_github_service
        .expect_fetch_repo_data()
        .returning(|_, _, _| {
            Ok(GithubRepoData {
                name: "name".to_string(),
                version: "first_version".to_string(),
                readme_content: "first_readme_content".to_string(),
                description: "".to_string(),
                size: 0,
                url: "".to_string(),
                rev: "".to_string(),
            })
        });

    let uid = Package::create(
        &"repo_url".to_string(),
        &"package_description".to_string(),
        &"1".to_string(),
        2,
        3,
        None,
        &mock_github_service,
        None,
        &DB_POOL,
    )
    .await
    .unwrap();

    PackageVersion::create(
        uid,
        "second_version".to_string(),
        "second_readme_content".to_string(),
        "1".to_string(),
        2,
        3,
        &DB_POOL,
    )
    .await
    .unwrap();

    let versions = PackageVersion::from_package_id(uid, &PackageVersionSort::Oldest, &DB_POOL)
        .await
        .unwrap();

    assert_eq!(versions.len(), 2);
    assert_eq!(versions[0].version, "first_version");
    assert_eq!(versions[1].version, "second_version");
}

#[actix_rt::test]
async fn get_versions_by_most_downloads() {
    crate::test::init();
    let _ctx = DatabaseTestContext::new();
    let mut mock_github_service = GithubService::new();
    mock_github_service
        .expect_fetch_repo_data()
        .returning(|_, _, _| {
            Ok(GithubRepoData {
                name: "name".to_string(),
                version: "first_version".to_string(),
                readme_content: "first_readme_content".to_string(),
                description: "".to_string(),
                size: 0,
                url: "".to_string(),
                rev: "".to_string(),
            })
        });

    let uid = Package::create(
        &"repo_url".to_string(),
        &"package_description".to_string(),
        &"1".to_string(),
        2,
        3,
        None,
        &mock_github_service,
        None,
        &DB_POOL,
    )
    .await
    .unwrap();

    let mut version_2 = PackageVersion::create(
        uid,
        "second_version".to_string(),
        "second_readme_content".to_string(),
        "5".to_string(),
        2,
        3,
        &DB_POOL,
    )
    .await
    .unwrap();
    version_2.downloads_count = 5;
    _ = &version_2
        .save_changes::<PackageVersion>(&*(DB_POOL.get().unwrap()))
        .unwrap();

    let versions =
        PackageVersion::from_package_id(uid, &PackageVersionSort::MostDownloads, &DB_POOL)
            .await
            .unwrap();

    assert_eq!(versions.len(), 2);
    assert_eq!(versions[0].version, "second_version");
    assert_eq!(versions[1].version, "first_version");
}

#[actix_rt::test]
async fn count_package_works() {
    crate::test::init();
    let _ctx = DatabaseTestContext::new();

    assert_eq!(Package::count(&DB_POOL).await.unwrap(), 0);
    assert_eq!(PackageVersion::count(&DB_POOL).await.unwrap(), 0);
    setup().await.unwrap();

    assert_eq!(Package::count(&DB_POOL).await.unwrap(), 3);
    assert_eq!(PackageVersion::count(&DB_POOL).await.unwrap(), 3);

    PackageVersion::create(
        1,
        "second_version".to_string(),
        "second_readme_content".to_string(),
        "rev_2".to_string(),
        2,
        100,
        &DB_POOL,
    )
    .await
    .unwrap();
    assert_eq!(PackageVersion::count(&DB_POOL).await.unwrap(), 4);
}

#[actix_rt::test]
async fn increase_download_count_works() {
    crate::test::init();
    let _ctx = DatabaseTestContext::new();

    let url = &"https://github.com/eadungn/taohe".to_string();
    let rev_ = &"30d4792b29330cf701af04b493a38a82102ed4fd".to_string();
    let package_id_ = Package::create_test_package(
        &"Test package".to_string(),
        url,
        &"".to_string(),
        &"1.0.0".to_string(),
        &"".to_string(),
        rev_,
        20,
        100,
        None,
        &DB_POOL,
    )
    .await
    .unwrap();

    let package_versions_before =
        PackageVersion::from_package_id(package_id_, &PackageVersionSort::Latest, &DB_POOL)
            .await
            .unwrap();
    let package_version_before = package_versions_before.first().unwrap();
    assert_eq!(package_version_before.downloads_count, 0);

    let mut mock_github_service = GithubService::new();

    mock_github_service
        .expect_fetch_repo_data()
        .returning(|_, _, _| {
            Ok(GithubRepoData {
                name: "name".to_string(),
                version: "1.0.0".to_string(),
                readme_content: "first_readme_content".to_string(),
                description: "".to_string(),
                size: 0,
                url: "".to_string(),
                rev: "".to_string(),
            })
        });

    Package::increase_download_count(url, rev_, &String::new(), &mock_github_service, &DB_POOL)
        .await
        .unwrap();
    Package::increase_download_count(url, rev_, &String::new(), &mock_github_service, &DB_POOL)
        .await
        .unwrap();
    let package_versions_after =
        PackageVersion::from_package_id(package_id_, &PackageVersionSort::Latest, &DB_POOL)
            .await
            .unwrap();
    let package_version_after = package_versions_after.first().unwrap();
    assert_eq!(package_version_after.downloads_count, 2);

    let _ = Package::increase_download_count(
        &"git@github.com:eadungn/taohe.git".to_string(),
        rev_,
        &String::new(),
        &mock_github_service,
        &DB_POOL,
    )
    .await;
    let package_versions_after =
        PackageVersion::from_package_id(package_id_, &PackageVersionSort::Latest, &DB_POOL)
            .await
            .unwrap();
    let package_version_after = package_versions_after.first().unwrap();
    assert_eq!(package_version_after.downloads_count, 3);
}

#[actix_rt::test]
async fn increase_download_count_for_nonexistent_package() {
    crate::test::init();
    let _ctx = DatabaseTestContext::new();

    let url = &"https://github.com/eadungn/taohe".to_string();
    let rev_ = &"30d4792b29330cf701af04b493a38a82102ed4fd".to_string();

    let mut mock_github_service = GithubService::new();
    mock_github_service
        .expect_fetch_repo_data()
        .returning(|_, _, _| {
            Ok(GithubRepoData {
                name: "name".to_string(),
                version: "first_version".to_string(),
                readme_content: "first_readme_content".to_string(),
                description: "".to_string(),
                size: 0,
                url: "".to_string(),
                rev: "".to_string(),
            })
        });

    let rev_not_existed = package_versions
        .filter(rev.eq(rev_))
        .count()
        .get_result::<i64>(&DB_POOL.get().unwrap())
        .unwrap();
    assert_eq!(rev_not_existed, 0);

    let package_before = packages
        .select(diesel::dsl::count(packages::id))
        .first::<i64>(&DB_POOL.get().unwrap())
        .unwrap();
    let package_version_before = package_versions
        .select(diesel::dsl::count(package_versions::id))
        .first::<i64>(&DB_POOL.get().unwrap())
        .unwrap();
    assert_eq!(package_before, 0);
    assert_eq!(package_version_before, 0);

    Package::increase_download_count(url, rev_, &String::new(), &mock_github_service, &DB_POOL)
        .await
        .unwrap();
    Package::increase_download_count(url, rev_, &String::new(), &mock_github_service, &DB_POOL)
        .await
        .unwrap();

    let package_after = packages
        .select(diesel::dsl::count(packages::id))
        .first::<i64>(&DB_POOL.get().unwrap())
        .unwrap();
    let package_version_after = package_versions
        .select(diesel::dsl::count(package_versions::id))
        .first::<i64>(&DB_POOL.get().unwrap())
        .unwrap();
    assert_eq!(package_after, 1);
    assert_eq!(package_version_after, 1);

    let rev_existed = package_versions
        .filter(rev.eq(rev_))
        .count()
        .execute(&DB_POOL.get().unwrap())
        .unwrap();
    assert_eq!(rev_existed, 1);
}

#[actix_rt::test]
async fn increase_download_count_for_multiple_versions() {
    crate::test::init();
    let _ctx = DatabaseTestContext::new();

    let url = "https://github.com/eadungn/taohe".to_string();
    let rev1 = "30d4792b29330cf701af04b493a38a82102ed4fd".to_string();
    let rev2 = "fe66d6c60a3765c322edbcfa9b63650593971a28".to_string();
    let package_id_ = Package::create_test_package(
        &"Test package".to_string(),
        &url,
        &"".to_string(),
        &"1.0.0".to_string(),
        &"".to_string(),
        &rev1,
        20,
        100,
        None,
        &DB_POOL,
    )
    .await
    .unwrap();
    PackageVersion::create(
        package_id_,
        String::from("1.0.0"),
        String::from(""),
        rev2.clone(),
        40,
        200,
        &DB_POOL,
    )
    .await
    .unwrap();

    let mut mock_github_service = GithubService::new();

    mock_github_service
        .expect_fetch_repo_data()
        .returning(|_, _, _| {
            Ok(GithubRepoData {
                name: "name".to_string(),
                version: "1.0.0".to_string(),
                readme_content: "first_readme_content".to_string(),
                description: "".to_string(),
                size: 0,
                url: "".to_string(),
                rev: "".to_string(),
            })
        });

    let package_versions_before =
        PackageVersion::from_package_id(package_id_, &PackageVersionSort::Latest, &DB_POOL)
            .await
            .unwrap();
    for package_version_before in package_versions_before {
        assert_eq!(package_version_before.downloads_count, 0);
    }
    Package::increase_download_count(&url, &rev1, &String::new(), &mock_github_service, &DB_POOL)
        .await
        .unwrap();
    Package::increase_download_count(&url, &rev2, &String::new(), &mock_github_service, &DB_POOL)
        .await
        .unwrap();
    let package_versions_after =
        PackageVersion::from_package_id(package_id_, &PackageVersionSort::Latest, &DB_POOL)
            .await
            .unwrap();
    for package_version_after in package_versions_after {
        assert_eq!(package_version_after.downloads_count, 1);
    }
    let package_total_downloads = Package::get(package_id_, &DB_POOL)
        .await
        .unwrap()
        .total_downloads_count;
    assert_eq!(package_total_downloads, 2);

    Package::increase_download_count(
        &"git@github.com:eadungn/taohe.git".to_string(),
        &rev2,
        &String::new(),
        &mock_github_service,
        &DB_POOL,
    )
    .await
    .unwrap();
    let package_versions_after =
        PackageVersion::from_package_id(package_id_, &PackageVersionSort::Latest, &DB_POOL)
            .await
            .unwrap();
    let first_package_version_after = package_versions_after.first().unwrap();
    assert_eq!(first_package_version_after.downloads_count, 2);
    let second_package_version_after = package_versions_after.last().unwrap();
    assert_eq!(second_package_version_after.downloads_count, 1);
    let package_total_downloads = Package::get(1, &DB_POOL)
        .await
        .unwrap()
        .total_downloads_count;
    assert_eq!(package_total_downloads, 3);
}