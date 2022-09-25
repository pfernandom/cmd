use std::{ borrow::Borrow, collections::HashMap };
use serde::{ Serialize, Deserialize };
use rusqlite::Row;
use std::hash::Hash;

#[derive(Debug, Deserialize, Serialize, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct CmdRecord {
    #[serde(rename = "id")]
    pub id: usize,
    #[serde(rename = "command")]
    pub command: String,
    #[serde(rename = "used_times")]
    pub used_times: usize,
}

pub trait CmdRecordIterable {
    fn sum_count(self) -> usize;

    fn group_and_sum_count<K: Hash + Eq + Clone>(
        self,
        get_key: impl FnMut(&CmdRecord) -> K
    ) -> Vec<CmdRecord>;

    fn group_by<K: Hash + Eq + Clone>(
        self,
        key: impl FnMut(&CmdRecord) -> K
    ) -> HashMap<K, Vec<CmdRecord>>;
}

impl<I> CmdRecordIterable for I where I: Iterator, I::Item: Borrow<CmdRecord> {
    fn sum_count(self) -> usize {
        self.fold(0, |acc, el| {
            let b: &CmdRecord = el.borrow();
            acc + b.used_times
        })
    }

    fn group_and_sum_count<K: Hash + Eq + Clone>(
        self,
        get_key: impl FnMut(&CmdRecord) -> K
    ) -> Vec<CmdRecord> {
        let call_key = move || { get_key };
        self.group_by(call_key())
            .values()
            .map(|vec_cmd| {
                vec_cmd
                    .iter()
                    .map(|cmd| cmd.clone())
                    .reduce(|mut acc, cur| {
                        acc.used_times += cur.used_times;
                        acc
                    })
                    .unwrap()
            })
            .collect::<Vec<CmdRecord>>()
    }

    fn group_by<K: Hash + Eq + Clone>(
        self,
        mut get_key: impl FnMut(&CmdRecord) -> K
    ) -> HashMap<K, Vec<CmdRecord>> {
        let map = HashMap::<K, Vec<CmdRecord>>::new();

        self.fold(map, |mut acc: HashMap<K, Vec<CmdRecord>>, item| {
            let b: &CmdRecord = item.borrow();

            let key_val = &get_key(&b.clone());
            match acc.get(&key_val) {
                Some(record) => {
                    let mut existing = record.clone();
                    existing.append(&mut vec![b.clone()]);

                    acc.insert(key_val.clone(), existing);
                }
                None => {
                    acc.insert(key_val.clone(), vec![b.clone()]);
                }
            }
            acc
        })
    }
}

impl Hash for CmdRecord {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.command.hash(state);
        self.used_times.hash(state);
    }
}

impl PartialEq for CmdRecord {
    fn eq(&self, other: &Self) -> bool {
        self.command == other.command
    }
}

impl CmdRecord {
    pub fn update_command(mut self: &mut Self, command: &str) {
        self.command = command.to_string();
    }

    pub fn increase_usage(mut self: &mut Self) {
        self.used_times += 1;
    }
}

impl From<&Row<'_>> for CmdRecord {
    fn from(row: &Row<'_>) -> Self {
        CmdRecord {
            id: row.get("id").expect("Could not parse ID"),
            command: row.get("command").expect("Could not parse command"),
            used_times: row.get("used_times").expect("Could not parse used_time"),
        }
    }
}