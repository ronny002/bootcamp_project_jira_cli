use crate::models::{DBState, Epic, Status, Story};
use anyhow::{anyhow, Result};
use std::{
    collections::HashMap,
    fs::{read_to_string, File},
    io::Write,
};
fn add_key_value<T>(mut map: HashMap<u32, T>, key: u32, value: T) -> HashMap<u32, T> {
    map.insert(key, value);
    map
}
fn remove_key_value<T>(mut map: HashMap<u32, T>, key: &u32) -> HashMap<u32, T> {
    map.remove(key);
    map
}
pub trait Database {
    fn read_db(&self) -> Result<DBState>;
    fn write_db(&self, db_state: &DBState) -> Result<()>;
}
pub struct JSONFileDatabase {
    pub file_path: String,
}
impl Database for JSONFileDatabase {
    fn read_db(&self) -> Result<DBState> {
        let serialized = read_to_string(&self.file_path)?;
        let deserialized: DBState = serde_json::from_str(&serialized)?;
        Ok(deserialized)
    }
    fn write_db(&self, db_state: &DBState) -> Result<()> {
        let serialized = serde_json::to_string(&db_state)?;
        let mut file = File::create(&self.file_path)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }
}
pub struct JiraDatabase {
    pub database: Box<dyn Database>,
}
impl JiraDatabase {
    pub fn new(file_path: String) -> Self {
        Self {
            database: Box::new(JSONFileDatabase { file_path }),
        }
    }
    pub fn read_db(&self) -> Result<DBState> {
        self.database.read_db()
    }

    pub fn create_epic(&self, epic: Epic) -> Result<u32> {
        let db_old = self.read_db()?;
        let db_new = DBState {
            last_item_id: db_old.last_item_id + 1,
            epics: add_key_value(db_old.epics, db_old.last_item_id + 1, epic), //generic known throu type inference
            stories: db_old.stories,
        };
        self.database.write_db(&db_new)?;
        Ok(db_new.last_item_id)
    }

    pub fn create_story(&self, story: Story, epic_id: u32) -> Result<u32> {
        let db_old = self.read_db()?;
        let mut db_new = DBState {
            last_item_id: db_old.last_item_id + 1,
            epics: db_old.epics,
            stories: add_key_value(db_old.stories, db_old.last_item_id + 1, story),
        };
        db_new
            .epics
            .get_mut(&epic_id)
            .ok_or_else(|| anyhow!("could not find epic in database for create story"))?
            .stories
            .push(db_new.last_item_id);
        self.database.write_db(&db_new)?;
        Ok(db_new.last_item_id)
    }

    pub fn delete_epic(&self, epic_id: u32) -> Result<()> {
        let mut db_old = self.read_db()?;
        for story_id in &db_old
            .epics
            .get(&epic_id)
            .ok_or_else(|| anyhow!("could not find epic in database for delete epic"))?
            .stories
        {
            db_old.stories.remove(story_id);
        }
        let db_new = DBState {
            last_item_id: db_old.last_item_id, 
            epics: remove_key_value(db_old.epics, &epic_id),
            stories: db_old.stories,
        };
        self.database.write_db(&db_new)?;
        Ok(())
    }

    pub fn delete_story(&self, epic_id: u32, story_id: u32) -> Result<()> {
        let db_old = self.read_db()?;
        let mut db_new = DBState {
            last_item_id: db_old.last_item_id, 
            epics: db_old.epics,
            stories: remove_key_value(db_old.stories, &story_id),
        };
        db_new
            .epics
            .get(&epic_id)
            .ok_or_else(|| anyhow!("could not find epic in db for delete story"))?
            .stories
            .contains(&story_id)
            .then_some(..)
            .ok_or_else(|| anyhow!("could not find story_id in epic stories for delete story"))?;
        db_new
            .epics
            .get_mut(&epic_id)
            .unwrap()
            .stories
            .retain(|x| *x != story_id); //todo!() not really clean how to do it in one combinator?
        self.database.write_db(&db_new)?;
        Ok(())
    }

    pub fn update_epic_status(&self, epic_id: u32, status: Status) -> Result<()> {
        let mut db = self.read_db()?;
        db.epics
            .get_mut(&epic_id)
            .ok_or_else(|| anyhow!("could not find epic in database for update epic status"))?
            .status = status;
        self.database.write_db(&db)?;
        Ok(())
    }

    pub fn update_story_status(&self, story_id: u32, status: Status) -> Result<()> {
        let mut db = self.read_db()?;
        db.stories
            .get_mut(&story_id)
            .ok_or_else(|| anyhow!("could not find epic in database for update story status"))?
            .status = status;
        self.database.write_db(&db)?;
        Ok(())
    }
}
pub mod test_utils {
    use std::{cell::RefCell, collections::HashMap};

    use super::*;
    pub struct MockDB {
        last_written_state: RefCell<DBState>,
    }
    #[allow(dead_code)] //used in test
    impl MockDB {
        pub fn new() -> Self {
            Self {
                last_written_state: RefCell::new(DBState {
                    last_item_id: 0,
                    epics: HashMap::new(),
                    stories: HashMap::new(),
                }),
            }
        }
    }
    impl Database for MockDB {
        fn read_db(&self) -> Result<DBState> {
            let state = self.last_written_state.borrow().clone();
            Ok(state)
        }
        fn write_db(&self, db_state: &DBState) -> Result<()> {
            let latest_state = &self.last_written_state;
            *latest_state.borrow_mut() = db_state.clone();
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use super::test_utils::MockDB;
    use super::*;

    #[test]
    fn create_epic_should_work() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic.clone());

        assert_eq!(result.is_ok(), true);

        let id = result.unwrap();
        let db_state = db.read_db().unwrap();

        let expected_id = 1;

        assert_eq!(id, expected_id);
        assert_eq!(db_state.last_item_id, expected_id);
        assert_eq!(db_state.epics.get(&id), Some(&epic));
    }

    #[test]
    fn create_story_should_error_if_invalid_epic_id() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let story = Story::new("".to_owned(), "".to_owned());

        let non_existent_epic_id = 999;

        let result = db.create_story(story, non_existent_epic_id);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn create_story_should_work() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        let result = db.create_story(story.clone(), epic_id);
        assert_eq!(result.is_ok(), true);

        let id = result.unwrap();
        let db_state = db.read_db().unwrap();

        let expected_id = 2;

        assert_eq!(id, expected_id);
        assert_eq!(db_state.last_item_id, expected_id);
        assert_eq!(
            db_state.epics.get(&epic_id).unwrap().stories.contains(&id),
            true
        );
        assert_eq!(db_state.stories.get(&id), Some(&story));
    }

    #[test]
    fn delete_epic_should_error_if_invalid_epic_id() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };

        let non_existent_epic_id = 999;

        let result = db.delete_epic(non_existent_epic_id);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn delete_epic_should_work() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);
        assert_eq!(result.is_ok(), true);

        let story_id = result.unwrap();

        let result = db.delete_epic(epic_id);
        assert_eq!(result.is_ok(), true);

        let db_state = db.read_db().unwrap();

        let expected_last_id = 2;

        assert_eq!(db_state.last_item_id, expected_last_id);
        assert_eq!(db_state.epics.get(&epic_id), None);
        assert_eq!(db_state.stories.get(&story_id), None);
    }

    #[test]
    fn delete_story_should_error_if_invalid_epic_id() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);
        assert_eq!(result.is_ok(), true);

        let story_id = result.unwrap();

        let non_existent_epic_id = 999;

        let result = db.delete_story(non_existent_epic_id, story_id);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn delete_story_should_error_if_story_not_found_in_epic() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);
        assert_eq!(result.is_ok(), true);

        let non_existent_story_id = 999;

        let result = db.delete_story(epic_id, non_existent_story_id);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn delete_story_should_work() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);
        assert_eq!(result.is_ok(), true);

        let story_id = result.unwrap();

        let result = db.delete_story(epic_id, story_id);
        assert_eq!(result.is_ok(), true);

        let db_state = db.read_db().unwrap();

        let expected_last_id = 2;

        assert_eq!(db_state.last_item_id, expected_last_id);
        assert_eq!(
            db_state
                .epics
                .get(&epic_id)
                .unwrap()
                .stories
                .contains(&story_id),
            false
        );
        assert_eq!(db_state.stories.get(&story_id), None);
    }

    #[test]
    fn update_epic_status_should_error_if_invalid_epic_id() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };

        let non_existent_epic_id = 999;

        let result = db.update_epic_status(non_existent_epic_id, Status::Closed);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn update_epic_status_should_work() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);

        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        let result = db.update_epic_status(epic_id, Status::Closed);

        assert_eq!(result.is_ok(), true);

        let db_state = db.read_db().unwrap();

        assert_eq!(db_state.epics.get(&epic_id).unwrap().status, Status::Closed);
    }

    #[test]
    fn update_story_status_should_error_if_invalid_story_id() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };

        let non_existent_story_id = 999;

        let result = db.update_story_status(non_existent_story_id, Status::Closed);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn update_story_status_should_work() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);

        let story_id = result.unwrap();

        let result = db.update_story_status(story_id, Status::Closed);

        assert_eq!(result.is_ok(), true);

        let db_state = db.read_db().unwrap();

        assert_eq!(
            db_state.stories.get(&story_id).unwrap().status,
            Status::Closed
        );
    }

    mod database {
        use super::*;
        use std::fs::remove_file;
        #[test]
        fn read_db_should_fail_with_invalid_path() {
            let db_path = JSONFileDatabase {
                file_path: "invalid_path".to_owned(),
            };
            assert_eq!(db_path.read_db().is_err(), true);
        }
        #[test]
        fn read_db_should_fail_with_invalid_json() {
            let mut tempfile = tempfile::NamedTempFile::new().unwrap();
            let content = r#"{ "last_item_id": 0 epics: {} stories {} }"#;
            write!(tempfile, "{content}").unwrap();
            let db_path = JSONFileDatabase {
                file_path: tempfile.path().to_str().unwrap().to_owned(),
            };
            println!("TempFilePath is: {}", db_path.file_path);
            let result = db_path.read_db();
            remove_file(db_path.file_path).unwrap();
            assert_eq!(result.is_err(), true);
        }
        #[test]
        fn read_db_should_parse_json_file() {
            let mut tempfile = tempfile::NamedTempFile::new().unwrap();
            let content = r#"{ "last_item_id": 0, "epics": {}, "stories": {} }"#;
            write!(tempfile, "{content}").unwrap();
            let db_path = JSONFileDatabase {
                file_path: tempfile.path().to_str().unwrap().to_owned(),
            };
            println!("TempFilePath is: {}", db_path.file_path);
            let result = db_path.read_db();
            remove_file(db_path.file_path).unwrap();
            assert_eq!(result.is_ok(), true);
        }
        #[test]
        fn write_db_should_work() {
            let mut tempfile_read = tempfile::NamedTempFile::new().unwrap();
            let content = r#"{ "last_item_id": 0, "epics": {}, "stories": {} }"#;
            write!(tempfile_read, "{content}").unwrap();
            let db_path = JSONFileDatabase {
                file_path: tempfile_read.path().to_str().unwrap().to_owned(),
            };
            println!("TempFilePath is: {}", db_path.file_path);
            let db = db_path.read_db().unwrap();
            remove_file(db_path.file_path).unwrap();

            let tempfile_write = tempfile::NamedTempFile::new().unwrap();
            let db_path = JSONFileDatabase {
                file_path: tempfile_write.path().to_str().unwrap().to_owned(),
            };
            println!("TempFilePath is: {}", db_path.file_path);
            let write_result = db_path.write_db(&db);
            remove_file(db_path.file_path).unwrap();
            assert_eq!(write_result.is_ok(), true);
        }
    }
}
