use diesel::prelude::*;
use diesel::sql_types::Text;
use diesel::{sql_query, AsChangeset, Identifiable, Insertable, Queryable};

use jelly::chrono::{DateTime, Utc};
use jelly::error::Error;
use jelly::serde::{Deserialize, Serialize};
use jelly::DieselPgPool;

use mockall_double::double;

// use super::forms::{LoginForm, NewAccountForm};
#[double]
use crate::github_service::GithubService;
use crate::schema::package_versions;
use crate::schema::package_versions::dsl::*;
use crate::schema::packages;
use crate::schema::packages::dsl::*;

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, AsChangeset, QueryableByName)]
#[table_name = "packages"]
pub struct Package {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub repository_url: String,
    pub total_downloads_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, QueryableByName)]
#[table_name = "packages"]
pub struct PackageSearchResult {
    pub name: String,
}

#[derive(Insertable)]
#[table_name = "packages"]
pub struct NewPackage {
    pub name: String,
    pub description: String,
    pub repository_url: String,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, AsChangeset, Clone)]
pub struct PackageVersion {
    pub id: i32,
    pub package_id: i32,
    pub version: String,
    pub readme_content: Option<String>,
    pub license: Option<String>,
    pub downloads_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub rev: Option<String>,
    pub total_files: Option<i32>,
    pub total_size: Option<i32>
}

#[derive(Insertable)]
#[table_name="package_versions"]
pub struct NewPackageVersion {
    pub package_id: i32,
    pub version: String,
    pub readme_content: String,
    pub rev: String,
    pub total_files: i32,
    pub total_size: i32
}

#[derive(Serialize, Deserialize)]
pub enum PackageVersionSort {
    Latest,
    Oldest,
    MostDownloads
}

impl Package {
    pub async fn create(repo_url: &String, package_description: &String, version_rev: &String, version_files: i32, version_size: i32, service: &GithubService, pool: &DieselPgPool) -> Result<i32, Error> {
        let connection = pool.get()?;

        let github_data = service.fetch_repo_data(&repo_url).unwrap();

        let record = match Package::get_by_name(&github_data.name, &pool).await {
            Ok(package) => {
                package
            }
            Err(_) => {
                let new_package = NewPackage {
                    name: github_data.name,
                    description: package_description.to_string(),
                    repository_url: repo_url.to_string()
                };

                let record = diesel::insert_into(packages::table)
                    .values(new_package)
                    .get_result::<Package>(&connection)?;

                record
            }
        };

        if let Err(_) = record.get_version(&github_data.version, &pool).await {
            PackageVersion::create(record.id, github_data.version, github_data.readme_content, version_rev.to_string(), version_files, version_size, pool).await.unwrap();
        }

        Ok(record.id)
    }

    pub async fn get(uid: i32, pool: &DieselPgPool) -> Result<Self, Error> {
        let connection = pool.get()?;
        let result = packages
            .find(uid)
            .first::<Package>(&connection)?;

        Ok(result)
    }

    pub async fn get_by_name(package_name: &String, pool: &DieselPgPool) -> Result<Self, Error> {
        let connection = pool.get()?;
        let result = packages
            .filter(name.eq(package_name))
            .first::<Package>(&connection)?;

        Ok(result)
    }

    pub async fn get_version(&self, version_name: &String, pool: &DieselPgPool) -> Result<PackageVersion, Error> {
        let connection = pool.get()?;
        let result = package_versions
            .filter(package_id.eq(self.id).and(version.eq(version_name)))
            .first::<PackageVersion>(&connection)?;

        Ok(result)
    }

    pub async fn search(search_query: &str, pool: &DieselPgPool) -> Result<Vec<String>, Error> {
        let connection = pool.get()?;
        let search_query = &search_query.split(" ").collect::<Vec<&str>>().join(" & ");
        let matched_packages: Vec<PackageSearchResult> = sql_query(
            "SELECT name \
                           FROM packages \
                           WHERE tsv @@ to_tsquery($1)\
                           ORDER BY ts_rank(tsv, to_tsquery($1), 2) DESC",
        )
        .bind::<Text, _>(search_query)
        .load(&connection)
        .unwrap();
        return Ok(matched_packages
            .into_iter()
            .map(|package| package.name)
            .collect());
    }
}

impl PackageVersion {
    pub async fn create(
        version_package_id: i32,
        version_name: String,
        version_readme_content: String,
        version_rev: String,
        version_files: i32,
        version_size: i32,
        pool: &DieselPgPool,
    ) -> Result<PackageVersion, Error> {
        let connection = pool.get()?;

        let new_package_version = NewPackageVersion {
            package_id: version_package_id,
            version: version_name,
            readme_content: version_readme_content,
            rev: version_rev,
            total_files: version_files,
            total_size: version_size
        };

        let record = diesel::insert_into(package_versions::table)
            .values(new_package_version)
            .get_result::<PackageVersion>(&connection)?;

        Ok(record)
    }

    pub async fn from_package_id(uid: i32, sort_type: &PackageVersionSort, pool: &DieselPgPool) -> Result<Vec<PackageVersion>, Error> {
        let connection = pool.get()?;
        let versions = package_versions.filter(package_id.eq(uid));

        let records = match sort_type {
            PackageVersionSort::Latest => {
                versions.order_by(package_versions::dsl::id.desc()).load::<PackageVersion>(&connection)?
            }
            PackageVersionSort::Oldest => {
                versions.order_by(package_versions::dsl::id.asc()).load::<PackageVersion>(&connection)?
            }
            PackageVersionSort::MostDownloads => {
                versions.order_by(package_versions::dsl::downloads_count.desc()).load::<PackageVersion>(&connection)?
                //versions.load::<PackageVersion>(&connection)?
            }
        };

        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::{DatabaseTestContext, DB_POOL};
    use mockall::predicate::*;

    use crate::github_service::GithubRepoData;

    async fn setup() -> Result<(), Error> {
        let pool = &DB_POOL;
        let connection = pool.get()?;
        let new_package = NewPackage {
            name: "The first package 1".to_string(),
            description: "description 1".to_string(),
            repository_url: "".to_string(),
        };
        diesel::insert_into(packages::table)
            .values(new_package)
            .get_result::<Package>(&connection)?;

        let new_package = NewPackage {
            name: "The first Diva".to_string(),
            description: "randomly picked, and changes some".to_string(),
            repository_url: "".to_string(),
        };
        diesel::insert_into(packages::table)
            .values(new_package)
            .get_result::<Package>(&connection)?;

        let new_package = NewPackage {
            name: "Charles Diya".to_string(),
            description: "randomly picked, and changes some".to_string(),
            repository_url: "".to_string(),
        };
        diesel::insert_into(packages::table)
            .values(new_package)
            .get_result::<Package>(&connection)?;
        Ok(())
    }

    #[actix_rt::test]
    async fn search_by_single_word_works() {
        crate::test::init();
        let _ctx = DatabaseTestContext::new();
        setup().await;
        let pool = &DB_POOL;
        let search_query = "package";
        let search_result = Package::search(search_query, pool).await.unwrap();
        assert_eq!(search_result.len(), 1);
        let result = search_result.iter();
        let mut is_found = false;
        for package_name in result {
            if package_name == "The first package 1" {
                is_found = true;
            }
            if package_name == "Charles Diya" {
                panic!()
            }
        }
        assert!(is_found)
    }

    #[actix_rt::test]
    async fn search_by_multiple_words_works() {
        crate::test::init();
        let _ctx = DatabaseTestContext::new();
        setup().await;
        let pool = &DB_POOL;
        let search_query = "the package";
        let search_result = Package::search(search_query, pool).await.unwrap();
        assert_eq!(search_result.len(), 1);
        let result = search_result.iter();
        let mut is_found = false;
        for package_name in result {
            if package_name == "The first package 1" {
                is_found = true;
            }
            if package_name == "Charles Diya" {
                panic!()
            }
        }
        assert!(is_found)
    }

    #[actix_rt::test]
    async fn search_return_multiple_result() {
        crate::test::init();
        let _ctx = DatabaseTestContext::new();
        setup().await;
        let pool = &DB_POOL;
        let search_query = "first";
        let search_result = Package::search(search_query, pool).await.unwrap();
        assert_eq!(search_result.len(), 2);
        let result = search_result.iter();
        for package_name in result {
            if package_name != "The first package 1" && package_name != "The first Diva" {
                panic!()
            }
            if package_name == "Charles Diya" {
                panic!()
            }
        }
    }

    #[actix_rt::test]
    async fn create_package_works() {
        crate::test::init();
        let _ctx = DatabaseTestContext::new();

        let mut mock_github_service = GithubService::new();
        mock_github_service.expect_fetch_repo_data()
            .with(eq("repo_url".to_string()))
            .returning(|_| Ok(GithubRepoData {
                name: "name".to_string(),
                version: "version".to_string(),
                readme_content: "readme_content".to_string(),
            }));

        let uid = Package::create(&"repo_url".to_string(), &"package_description".to_string(), &"1".to_string(), 2, 100,&mock_github_service, &DB_POOL).await.unwrap();

        let package = Package::get(uid, &DB_POOL).await.unwrap();
        assert_eq!(package.name, "name");
        assert_eq!(package.description, "package_description");

        let package_version = &PackageVersion::from_package_id(uid, &PackageVersionSort::Latest, &DB_POOL).await.unwrap()[0];
        assert_eq!(package_version.version, "version");
        match &package_version.readme_content {
            Some(content) => {
                assert_eq!(content, "readme_content");
            },
            None => { panic!("readme content is wrong") }
        }
    }

    #[actix_rt::test]
    async fn get_versions_by_latest() {
        crate::test::init();
        let _ctx = DatabaseTestContext::new();

        let mut mock_github_service = GithubService::new();
        mock_github_service.expect_fetch_repo_data()
            .returning(|_| Ok(GithubRepoData {
                name: "name".to_string(),
                version: "first_version".to_string(),
                readme_content: "first_readme_content".to_string(),
            }));

        let uid = Package::create(&"repo_url".to_string(), &"package_description".to_string(), &"1".to_string(), 2, 100,&mock_github_service, &DB_POOL).await.unwrap();

        PackageVersion::create(uid, "second_version".to_string(), "second_readme_content".to_string(), "1".to_string(), 2, 100, &DB_POOL).await.unwrap();

        let versions = PackageVersion::from_package_id(uid, &PackageVersionSort::Latest, &DB_POOL).await.unwrap();

        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].version, "second_version");
        assert_eq!(versions[1].version, "first_version");
    }

    #[actix_rt::test]
    async fn get_versions_by_oldest() {
        crate::test::init();
        let _ctx = DatabaseTestContext::new();
        let mut mock_github_service = GithubService::new();
        mock_github_service.expect_fetch_repo_data()
            .returning(|_| Ok(GithubRepoData {
                name: "name".to_string(),
                version: "first_version".to_string(),
                readme_content: "first_readme_content".to_string(),
            }));

        let uid = Package::create(&"repo_url".to_string(), &"package_description".to_string(), &"1".to_string(), 2, 3,&mock_github_service, &DB_POOL).await.unwrap();

        PackageVersion::create(uid, "second_version".to_string(), "second_readme_content".to_string(), "1".to_string(), 2, 3, &DB_POOL).await.unwrap();

        let versions = PackageVersion::from_package_id(uid, &PackageVersionSort::Oldest, &DB_POOL).await.unwrap();

        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].version, "first_version");
        assert_eq!(versions[1].version, "second_version");
    }

    #[actix_rt::test]
    async fn get_versions_by_most_downloads() {
        crate::test::init();
        let _ctx = DatabaseTestContext::new();
        let mut mock_github_service = GithubService::new();
        mock_github_service.expect_fetch_repo_data()
            .returning(|_| Ok(GithubRepoData {
                name: "name".to_string(),
                version: "first_version".to_string(),
                readme_content: "first_readme_content".to_string(),
            }));

        let uid = Package::create(&"repo_url".to_string(), &"package_description".to_string(), &"1".to_string(), 2, 3, &mock_github_service, &DB_POOL).await.unwrap();

        let mut version_2 = PackageVersion::create(uid, "second_version".to_string(), "second_readme_content".to_string(), "5".to_string(), 2, 3, &DB_POOL).await.unwrap();
        version_2.downloads_count = 5;
        _ = &version_2.save_changes::<PackageVersion>(&*(DB_POOL.get().unwrap())).unwrap();

        let versions = PackageVersion::from_package_id(uid, &PackageVersionSort::MostDownloads, &DB_POOL).await.unwrap();

        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].version, "second_version");
        assert_eq!(versions[1].version, "first_version");
    }
}

// Helpers for integration tests only. Wondering why cfg(test) below doesn't work... (commented out for now)
#[cfg(any(test, feature = "test"))]
impl Package {
    pub async fn create_test_package(package_name: &String, repo_url: &String, package_description: &String, package_version: &String, package_readme_content: &String, version_rev: &String, version_files: i32, version_size: i32,pool: &DieselPgPool) -> Result<i32, Error> {
        let connection = pool.get()?;

        let new_package = NewPackage {
            name: package_name.to_string(),
            description: package_description.to_string(),
            repository_url: repo_url.to_string()
        };

        let record = diesel::insert_into(packages::table)
            .values(new_package)
            .get_result::<Package>(&connection)?;

        PackageVersion::create(record.id, package_version.to_string(), package_readme_content.to_string(), version_rev.to_string(), version_files, version_size, pool).await.unwrap();

        Ok(record.id)
    }
}
