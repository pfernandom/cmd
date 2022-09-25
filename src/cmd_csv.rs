use std::io::Read;
use csv::Reader;
use crate::models::cmd_record::CmdRecord;

pub fn read_cmd_file<T: Read>(rdr: &mut Reader<T>) -> Vec<CmdRecord> {
    let options = rdr
        .deserialize()
        .enumerate()
        .map(|x| x.1)
        .filter(|x| x.is_ok())
        .map(|x| {
            let rec: CmdRecord = x.unwrap();
            rec
        })
        .collect::<Vec<_>>();

    options
}