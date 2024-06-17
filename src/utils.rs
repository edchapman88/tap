use chrono::{Date, DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use std::io::Read;
use std::{
    fs::{File, OpenOptions},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::TAP_STORE;

pub fn now() -> (u64, DateTime<Utc>) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let now_dt = DateTime::from_timestamp(now as i64, 0).unwrap();
    (now, now_dt)
}

pub fn get_file() -> File {
    let tap_store = std::env::var(TAP_STORE).expect(
        "Set the `TAP_STORE` environment variable to point to an existing `.tap` directory.",
    );
    match OpenOptions::new()
        .create(true)
        .read(true)
        .append(true)
        .open(Path::new(&tap_store).join("log.txt"))
    {
        Ok(file) => return file,
        Err(e) => {
            panic!("{e}");
        }
    }
}

pub fn get_log() -> String {
    let mut file = get_file();
    let mut log_txt = String::new();
    file.read_to_string(&mut log_txt).unwrap();
    log_txt
}

pub fn utc_date_time(date: NaiveDate, hours: u32, mins: u32) -> DateTime<Utc> {
    DateTime::from_naive_utc_and_offset(
        NaiveDateTime::new(date, NaiveTime::from_hms_opt(hours, mins, 0).unwrap()),
        Utc,
    )
}

/// Number of days **inclusive**.
pub fn days_diff(start: NaiveDate, end: NaiveDate) -> u64 {
    let diff = (end - start).num_days();
    if diff < 0 {
        panic!("start date is after end date")
    }
    diff as u64 + 1
}

/// Hours worked between two dates **inclusive**.
pub fn hours_worked(start: NaiveDate, end: NaiveDate, log_txt: &str) -> f32 {
    let start_dt = utc_date_time(start, 0, 0);
    let end_dt = utc_date_time(end, 23, 59);
    let (relevant_ins, relevant_outs) = log_txt
        .split("\n")
        .filter_map(|chunk| {
            let parts = chunk.split(" ").collect::<Vec<_>>();
            let stamp = i64::from_str_radix(parts[1], 10).unwrap();
            if (stamp > start_dt.timestamp()) & (stamp < end_dt.timestamp()) {
                if parts[0] == Chunk::In(0).val() {
                    return Some(Chunk::In(stamp));
                }
                return Some(Chunk::Out(
                    stamp,
                    u64::from_str_radix(parts[2], 10).unwrap(),
                ));
            }
            None
        })
        .fold((Vec::new(), Vec::new()), |mut acc, chunk| {
            match chunk {
                Chunk::In(_) => acc.0.push(chunk),
                Chunk::Out(_, _) => acc.1.push(chunk),
            };
            acc
        });

    // missing tap-ins are handled at tap out time
    // if all tap-outs are present, the pre-recorded times can be summed
    if relevant_outs.len() == days_diff(start, end) as usize {
        return relevant_outs.iter().fold(0.0, |mut acc, chunk| {
            if let Chunk::Out(_, seconds) = chunk {
                acc += *seconds as f32
            } else {
                panic!("all chunks in relevant_outs will be Chunk::Out varient")
            };
            acc
        }) / (60.0 * 60.0);
    }

    todo!()
}

enum Chunk {
    In(i64),
    Out(i64, u64),
}
impl Chunk {
    pub fn val(&self) -> String {
        match self {
            Chunk::In(_) => "IN".to_string(),
            Chunk::Out(_, _) => "OUT".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EX1: &str = "IN 1718452800\nOUT 1718460000 600000\nIN 1718528400\nOUT 1718532000 3600\nIN 1718618400\nOUT 1718625600 7200";

    #[test]
    fn days_diff_test() {
        assert_eq!(
            days_diff(
                NaiveDate::from_ymd_opt(2024, 01, 3).unwrap(),
                NaiveDate::from_ymd_opt(2024, 01, 4).unwrap(),
            ),
            2
        );
        assert_eq!(
            days_diff(
                NaiveDate::from_ymd_opt(2024, 01, 3).unwrap(),
                NaiveDate::from_ymd_opt(2024, 01, 6).unwrap(),
            ),
            4
        );
    }

    #[test]
    fn hours_worked_test() {
        assert_eq!(
            hours_worked(
                NaiveDate::from_ymd_opt(2024, 6, 16).unwrap(),
                NaiveDate::from_ymd_opt(2024, 6, 17).unwrap(),
                EX1,
            ),
            3.0
        )
    }
}
