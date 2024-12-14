use rusqlite::Connection;

use crate::config::get_db_path;

pub struct Storage {
    connection: Connection,
}

#[derive(Debug, Clone)]
pub struct BrowserInfo {
    pub name: String,
    pub path: String,
    pub icon_data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct MatchItem {
    pub browser_path: String,
    pub profile: Option<String>,
    pub match_type: String,
    pub match_value: String,
}

#[derive(Debug, Clone)]
pub struct BrowserProfile {
    pub browser_path: String,
    pub profile: String,
    pub description: Option<String>,
}

impl Storage {
    pub fn new() -> Self {
        let connection = Connection::open(get_db_path()).unwrap();
        connection.execute_batch("
        BEGIN;
         CREATE TABLE IF NOT EXISTS browsers (path text primary key, name text not null, icon_data blob not null);
         CREATE TABLE IF NOT EXISTS browser_profiles (browser_path text not null, profile text not null, description text, primary key(browser_path, profile));
         CREATE TABLE IF NOT EXISTS matches (browser_path text not null, profile text, match_type text not null, match_value text primary key);
         COMMIT;
         ").unwrap();
        Storage { connection }
    }
    pub fn batch_insert_browsers(&mut self, browsers: Vec<BrowserInfo>) {
        let tx = self.connection.transaction().unwrap();
        {
            let mut stmt = tx
                .prepare("INSERT INTO browsers (name, path, icon_data) VALUES (?, ?, ?) ON CONFLICT(path) DO NOTHING")
                .unwrap();

            for browser in browsers {
                stmt.execute((&browser.name, &browser.path, &browser.icon_data))
                    .unwrap();
            }
        }
        tx.commit().unwrap();
    }
    pub fn get_browsers(&mut self) -> Vec<BrowserInfo> {
        let mut stmt = self
            .connection
            .prepare("SELECT name, path, icon_data FROM browsers")
            .unwrap();
        stmt.query_map([], |row| {
            Ok(BrowserInfo {
                name: row.get(0)?,
                path: row.get(1)?,
                icon_data: row.get(2)?,
            })
        })
        .unwrap()
        .map(|r| r.unwrap())
        .collect()
    }
    pub fn insert_match(&mut self, match_item: MatchItem) {
        self.connection
            .execute(
                "INSERT INTO matches (browser_path, profile, match_type, match_value) VALUES (?, ?, ?, ?) ON CONFLICT(match_value) DO NOTHING",
                (match_item.browser_path, match_item.profile, match_item.match_type, match_item.match_value),
            )
            .unwrap();
    }
    pub fn find_equal_matches_by_url(&mut self, url: String) -> Option<MatchItem> {
        let mut stmt = self
            .connection
            .prepare("SELECT * FROM matches WHERE match_type = ? AND match_value = ?")
            .unwrap();
        let result: Vec<MatchItem> = stmt
            .query_map(["Equal", &url], |row| {
                Ok(MatchItem {
                    browser_path: row.get(0)?,
                    profile: row.get(1)?,
                    match_type: row.get(2)?,
                    match_value: row.get(3)?,
                })
            })
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        if result.is_empty() {
            None
        } else {
            Some(result[0].clone())
        }
    }
    pub fn find_contain_matches_by_url(&mut self, url: String) -> Option<MatchItem> {
        let mut stmt = self
            .connection
            .prepare("SELECT * FROM matches WHERE match_type = ?")
            .unwrap();
        let result: Vec<MatchItem> = stmt
            .query_map(["Contain"], |row| {
                Ok(MatchItem {
                    browser_path: row.get(0)?,
                    profile: row.get(1)?,
                    match_type: row.get(2)?,
                    match_value: row.get(3)?,
                })
            })
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        let matched = result
            .iter()
            .filter(|item| url.contains(&item.match_value))
            .next();
        matched.cloned()
    }
    pub fn insert_browser_profile(&mut self, browser_profile: BrowserProfile) {
        self.connection
            .execute(
                "INSERT INTO browser_profiles (browser_path, profile, description) VALUES (?, ?, ?) ON CONFLICT(browser_path, profile) DO NOTHING",
                (browser_profile.browser_path, browser_profile.profile, browser_profile.description),
            )
            .unwrap();
    }
    pub fn get_browser_profiles(&mut self, browser_path: String) -> Vec<BrowserProfile> {
        let mut stmt = self
            .connection
            .prepare("SELECT * FROM browser_profiles WHERE browser_path = ?")
            .unwrap();
        stmt.query_map([browser_path], |row| {
            Ok(BrowserProfile {
                browser_path: row.get(0)?,
                profile: row.get(1)?,
                description: row.get(2)?,
            })
        })
        .unwrap()
        .map(|r| r.unwrap())
        .collect()
    }
    pub fn delete_browser_profile(&mut self, browser_path: String, profile: String) {
        self.connection
            .execute(
                "DELETE FROM browser_profiles WHERE browser_path = ? AND profile = ?",
                (browser_path, profile),
            )
            .unwrap();
    }
    pub fn delete_match_by_profile_and_browser_path(
        &mut self,
        browser_path: String,
        profile: String,
    ) {
        self.connection
            .execute(
                "DELETE FROM matches WHERE browser_path = ? AND profile = ?",
                (browser_path, profile),
            )
            .unwrap();
    }
    pub fn delete_match_by_match_value(&mut self, match_value: String) {
        self.connection
            .execute("DELETE FROM matches WHERE match_value = ?", (match_value,))
            .unwrap();
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::fs;
    use std::path::Path;

    fn cleanup_database() {
        let db_path = get_db_path();
        if Path::new(&db_path).exists() {
            fs::remove_file(&db_path).unwrap();
        }
    }

    #[test]
    fn test_browser_crud() {
        cleanup_database();
        let mut storage = Storage::new();

        // 测试插入多个浏览器
        let test_browsers = vec![
            BrowserInfo {
                name: "Browser 1".to_string(),
                path: "/test/path1".to_string(),
                icon_data: vec![1, 2, 3],
            },
            BrowserInfo {
                name: "Browser 2".to_string(),
                path: "/test/path2".to_string(),
                icon_data: vec![4, 5, 6],
            },
        ];
        storage.batch_insert_browsers(test_browsers);

        let browsers = storage.get_browsers();
        assert_eq!(browsers.len(), 2);
        assert_eq!(browsers[1].name, "Browser 2");
        assert_eq!(browsers[1].icon_data, vec![4, 5, 6]);
    }

    #[test]
    fn test_profile_operations() {
        cleanup_database();
        let mut storage = Storage::new();

        // 测试插入多个配置文件
        let profiles = vec![
            BrowserProfile {
                browser_path: "/test/path1".to_string(),
                profile: "Default".to_string(),
                description: Some("Default Profile".to_string()),
            },
            BrowserProfile {
                browser_path: "/test/path1".to_string(),
                profile: "Work".to_string(),
                description: None,
            },
        ];

        for profile in profiles {
            storage.insert_browser_profile(profile);
        }
        let saved_profiles = storage.get_browser_profiles("Test Browser".to_string());
        assert_eq!(saved_profiles.len(), 2);
        assert!(saved_profiles
            .iter()
            .any(|p| p.profile == "Work" && p.description.is_none()));

        storage.delete_browser_profile("Test Browser".to_string(), "Work".to_string());
        let remaining_profiles = storage.get_browser_profiles("Test Browser".to_string());
        assert_eq!(remaining_profiles.len(), 1);
        assert_eq!(remaining_profiles[0].profile, "Default");
    }

    #[test]
    fn test_match_rules() {
        cleanup_database();
        let mut storage = Storage::new();

        let matches = vec![
            MatchItem {
                browser_path: "/test/path1".to_string(),
                profile: Some("Default".to_string()),
                match_type: "Equal".to_string(),
                match_value: "https://example.com".to_string(),
            },
            MatchItem {
                browser_path: "/test/path2".to_string(),
                profile: None,
                match_type: "Contain".to_string(),
                match_value: "github.com".to_string(),
            },
        ];

        for match_item in matches {
            storage.insert_match(match_item);
        }

        let exact_match = storage.find_equal_matches_by_url("https://example.com".to_string());
        assert!(exact_match.is_some());
        let matched = exact_match.unwrap();
        assert_eq!(matched.browser_path, "/test/path1");
        assert_eq!(matched.profile, Some("Default".to_string()));
        let contain_match = storage.find_contain_matches_by_url("test.github.com".to_string());
        assert!(contain_match.is_some());
        let matched = contain_match.unwrap();
        assert_eq!(matched.browser_path, "/test/path2");
        assert_eq!(matched.profile, None);

        assert!(storage
            .find_equal_matches_by_url("https://other.com".to_string())
            .is_none());
        assert!(storage
            .find_contain_matches_by_url("example.org".to_string())
            .is_none());
    }

    #[test]
    fn test_empty_database() {
        cleanup_database();
        let mut storage = Storage::new();

        // 测试空数据库的查询
        assert!(storage.get_browsers().is_empty());
        assert!(storage
            .get_browser_profiles("NonExistent".to_string())
            .is_empty());
        assert!(storage
            .find_equal_matches_by_url("any".to_string())
            .is_none());
        assert!(storage
            .find_contain_matches_by_url("any".to_string())
            .is_none());
    }
}
