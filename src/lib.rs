use chrono::DateTime;
use chrono::NaiveTime;
use chrono::Utc;
use std::io::Write;
use utils::get_file;
use utils::get_log;
use utils::now;

pub mod utils;

pub const TAP_STORE: &str = "TAP_STORE";

pub fn tap_in() {
    let mut file = get_file();
    let (now, now_dt) = now();
    let log_txt = get_log();
    let (tapped_in_already, time) = last_in_today(&log_txt, now_dt);
    if tapped_in_already {
        panic!("already tapped in today at {}", time.to_string())
    }

    if let Err(e) = writeln!(file, "IN {}", now) {
        println!("Couldn't write to file: {}", e);
    }
}

pub fn tap_out() {
    let (now, now_dt) = now();
    let log_txt = get_log();
    let (tapped_out_already, time) = last_out_today(&log_txt, now_dt);
    if tapped_out_already {
        panic!("already tapped out today at {}", time.to_string())
    }

    let in_time = last_in_today(&log_txt, now_dt).1;
    let work = now_dt - in_time;

    if let Err(e) = writeln!(get_file(), "OUT {} {}", now, work.num_seconds()) {
        println!("Couldn't write to file: {}", e);
    }
}

pub fn last_in(log_txt: &str) -> DateTime<Utc> {
    let mut last_in_str = log_txt.split("IN ").last().unwrap();
    if let Some((time, _)) = last_in_str.split_once("\n") {
        last_in_str = time;
    }
    DateTime::from_timestamp(i64::from_str_radix(last_in_str, 10).unwrap(), 0).unwrap()
}

pub fn last_out(log_txt: &str) -> DateTime<Utc> {
    let last_out_str = log_txt
        .split("OUT ")
        .last()
        .unwrap()
        .split_once(" ")
        .unwrap()
        .0;
    DateTime::from_timestamp(i64::from_str_radix(last_out_str, 10).unwrap(), 0).unwrap()
}

pub fn last_in_today(log_txt: &str, now_dt: DateTime<Utc>) -> (bool, DateTime<Utc>) {
    let mut last_in_dt = last_in(log_txt);
    // last 'IN' not recorded today
    if last_in_dt.date_naive() != now_dt.date_naive() {
        last_in_dt = now_dt
            .with_time(NaiveTime::from_hms_opt(9, 0, 0).unwrap())
            .unwrap();
        return (false, last_in_dt);
    }
    (true, last_in_dt)
}

pub fn last_out_today(log_txt: &str, now_dt: DateTime<Utc>) -> (bool, DateTime<Utc>) {
    let mut last_out_dt = last_out(log_txt);
    // last 'OUT' not recorded today
    if last_out_dt.date_naive() != now_dt.date_naive() {
        last_out_dt = now_dt
            .with_time(NaiveTime::from_hms_opt(17, 0, 0).unwrap())
            .unwrap();
        return (false, last_out_dt);
    }
    (true, last_out_dt)
}

#[cfg(test)]
mod tests {
    use chrono::Days;

    use super::*;

    #[test]
    fn find_last_in_from_str() {
        let t = [
            "OUT 12222\nIN 123456\n",
            "OUT 12222\nIN 123456",
            "OUT 1222\nIN 123456\nOUT 1222",
            "OUT 1222\nIN 123333\nOUT 1222\nIN 123456",
        ];
        t.map(|t| assert_eq!(DateTime::from_timestamp(123456, 0).unwrap(), last_in(t)));
    }

    #[test]
    fn infer_last_in_when_missing() {
        let t = "OUT 12222\nIN 1718627587\n";
        assert_eq!(
            DateTime::from_timestamp(1718627587, 0).unwrap(),
            last_in_today(&t, DateTime::from_timestamp(1718629359, 0).unwrap()).1
        );
    }

    #[test]
    fn scenarios() {
        let (_, now) = now();
        let today_10am = now
            .with_time(NaiveTime::from_hms_opt(10, 0, 0).unwrap())
            .unwrap();
        let today_5pm = now
            .with_time(NaiveTime::from_hms_opt(17, 0, 0).unwrap())
            .unwrap();

        // last in was 10am this morning
        assert_eq!(
            true,
            last_in_today(
                &format!("OUT 12222\nIN {}\n", today_10am.timestamp()),
                today_5pm,
            )
            .0
        );
        assert_eq!(
            today_10am,
            last_in_today(
                &format!("OUT 12222\nIN {}\n", today_10am.timestamp()),
                today_5pm,
            )
            .1
        );

        let tomorrow_5pm = today_5pm.checked_add_days(Days::new(1)).unwrap();
        let tomorrow_9am = tomorrow_5pm
            .with_time(NaiveTime::from_hms_opt(9, 0, 0).unwrap())
            .unwrap();

        // last in is missing, so 9am assumed
        assert_eq!(
            false,
            last_in_today(
                &format!("OUT 12222\nIN {}\n", today_10am.timestamp()),
                tomorrow_5pm,
            )
            .0
        );
        assert_eq!(
            tomorrow_9am,
            last_in_today(
                &format!("OUT 12222\nIN {}\n", today_10am.timestamp()),
                tomorrow_5pm,
            )
            .1
        );
    }
}
