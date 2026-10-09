use std::fs;
use std::path::Path;

#[derive(Clone)]
pub struct SpeechRecord {
    pub id: i32,
    pub text: String,
}

pub struct HistoryManager {
    records: Vec<SpeechRecord>,
    next_id: i32,
}

impl HistoryManager {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            next_id: 0,
        }
    }

    pub fn push(&mut self, text: String) -> i32 {
        let id = self.next_id;
        self.records.push(SpeechRecord { id, text });
        self.next_id += 1;
        id
    }

    pub fn remove(&mut self, id: i32) {
        self.records.retain(|record| record.id != id);
    }

    pub fn export_subtitle(&self, text: &str, path: &Path) -> std::io::Result<()> {
        fs::write(path, text)
    }

    pub fn all_records(&self) -> &[SpeechRecord] {
        &self.records
    }

    pub fn find_text(&self, id: i32) -> Option<String> {
        self.records
            .iter()
            .find(|r| r.id == id)
            .map(|r| r.text.clone())
    }
}
