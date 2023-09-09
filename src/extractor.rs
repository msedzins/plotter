use chrono::{DateTime, FixedOffset, ParseResult};
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct RestProxyLog {
    files: Vec<String>,
    filters: Vec<String>,

    date_index: usize,
    time_index: usize,
    exec_time_index: usize,
}

#[derive(Debug, PartialEq)]
pub struct Response {
    pub date_as_string: String,
    pub date_as_timestamp: i64,
    pub exec_time: i32,

    pub date_as_date: DateTime<FixedOffset>,
}

impl RestProxyLog {
    pub fn new(
        files: Vec<String>,
        filters: Vec<String>,
        date_index: usize,
        time_index: usize,
        exec_time_index: usize,
    ) -> Self {
        RestProxyLog {
            files,
            filters,
            date_index,
            time_index,
            exec_time_index,
        }
    }

    pub fn get(&self) -> Vec<Response> {
       return self.process_files();
    }

    fn parse_exec_time(time: &str) -> i32 {
        match time.find("ms") {
            Some(i) => {
                let time: String = time.to_string().drain(..i).collect();

                time.parse::<f32>().unwrap_or_else(|_| {
                    eprint!("This time can't be parsed: {}\n", time);
                    0f32
                }) as i32
            }
            None => {
                let offset = time.find("s").unwrap_or_else(|| {
                    eprint!("This time can't be parsed: {}\n", time);
                    0
                });
                if offset == 0 {
                    return 0;
                };

                let time: String = time.to_string().drain(..offset).collect();

                (time.parse::<f32>().unwrap_or_else(|_| {
                    eprint!("This time can't be parsed: {}\n", time);
                    0f32
                }) * 1000.0) as i32
            }
        }
    }

    fn extract_date(date: &str) -> String {
        let mut s: String = date.to_string();
        let offset = s.find("m20").unwrap_or(0);
        if offset == 0 {
            return s;
        };

        s.drain(..offset + 1);

        return s;
    }

    fn extract_time(time: &str) -> String {
        let offset = time.find(".").unwrap_or(time.len());
        time.to_string().drain(..offset).collect()
    }

    fn parse_date_time(date: &str, time: &str) -> ParseResult<DateTime<FixedOffset>> {
        let date = RestProxyLog::extract_date(date);
        let time = RestProxyLog::extract_time(time);

        let str = date + " " + &time + " +00:00";

        DateTime::parse_from_str(&str, "%Y-%m-%d %H:%M:%S %z")
    }

    fn process_files(&self) -> Vec<Response> {
        let mut buffer = Vec::new();

        for file in self.files.iter() { 
            let file = File::open(file).expect("Error reading a file");
            let mut reader = BufReader::new(file);

            let mut resp = self.process(&mut reader);
            buffer.append(&mut resp);
        }

        buffer.sort_by(|a,b| a.date_as_timestamp.partial_cmp(&b.date_as_timestamp).unwrap());

        return buffer;
    }

    fn process<R: std::io::Read>(&self, reader: &mut BufReader<R>) -> Vec<Response> {
        let mut ret: Vec<Response> = Vec::new();

        loop {
            let mut buf = String::new();
            if reader.read_line(&mut buf).expect("Reading failed") == 0 {
                break;
            }

            if self.include(&buf) == true {
                let words: Vec<&str> = buf.split_whitespace().collect();

                let datetime =
                    RestProxyLog::parse_date_time(words[self.date_index], words[self.time_index]);
                if let Some(_) = datetime.err() {
                    eprint!(
                        "DateTime can't be parsed: [{}] [{}]\n",
                        words[self.date_index], words[self.time_index]
                    );
                    continue;
                }

                let exec_time: i32 = RestProxyLog::parse_exec_time(words[self.exec_time_index]);

                ret.push(Response {
                    date_as_string: datetime.unwrap().to_string(),
                    date_as_timestamp: datetime.unwrap().timestamp(),
                    exec_time,

                    date_as_date: datetime.unwrap(),
                });
            }
        }

        return ret;
    }

    fn include(&self, line: &str) -> bool {
        for val in self.filters.iter() {
            if !line.contains(val) {
                return false;
            }
        }
        return true;
    }
}

#[cfg(test)]
mod tests {

    use chrono::format::ParseErrorKind;

    use super::*;

    #[test]
    fn parse_exec_time_ms() {
        let time = RestProxyLog::parse_exec_time("12345.99ms");
        assert_eq!(12345, time)
    }

    #[test]
    fn parse_exec_time_s() {
        let time = RestProxyLog::parse_exec_time("1.123456789s");
        assert_eq!(1123, time)
    }

    #[test]
    fn parse_exec_time_wrong() {
        let time = RestProxyLog::parse_exec_time("1.123456789");
        assert_eq!(0, time);

        let time = RestProxyLog::parse_exec_time("1.A123456789ms");
        assert_eq!(0, time);

        let time = RestProxyLog::parse_exec_time("1.A123456789s");
        assert_eq!(0, time);
    }

    #[test]
    fn extract_date() {
        //OK
        let date = RestProxyLog::extract_date("34m2023-08-28");
        assert_eq!("2023-08-28", date);

        //Bad format - no prefix
        let date = RestProxyLog::extract_date("2023-08-28");
        assert_eq!("2023-08-28", date);
    }

    #[test]
    fn extract_time() {
        //OK
        let time = RestProxyLog::extract_time("07:02:54.235");
        assert_eq!("07:02:54", time);

        //Bad format - no suffix
        let time = RestProxyLog::extract_time("07:02:54");
        assert_eq!("07:02:54", time);
    }

    #[test]
    fn parse_date_time() {
        //OK
        let time = RestProxyLog::parse_date_time("34m2023-08-28", "07:02:54.235");
        assert_eq!(
            DateTime::parse_from_str("2023-08-28 07:02:54 +00:00", "%Y-%m-%d %H:%M:%S %z").unwrap(),
            time.unwrap()
        );

        //Wrong format
        let time = RestProxyLog::parse_date_time("34m2023-08-28", "A07:02:54.235");
        assert_eq!(ParseErrorKind::Invalid, time.unwrap_err().kind());
    }

    #[test]
    fn include() {
        let r = RestProxyLog::new(Vec::new(), Vec::new(), 0, 0, 0);
        assert_eq!(true, r.include("TEST LINE"));

        let r = RestProxyLog::new(Vec::new(), vec![String::from("TEST")], 0, 0, 0);
        assert_eq!(true, r.include("TEST LINE"));

        let r = RestProxyLog::new(
            Vec::new(),
            vec![String::from("TEST"), String::from("LINE")],
            0,
            0,
            0,
        );
        assert_eq!(true, r.include("TEST LINE"));

        let r = RestProxyLog::new(
            Vec::new(),
            vec![String::from("TEST"), String::from("NOT_LINE")],
            0,
            0,
            0,
        );
        assert_eq!(false, r.include("TEST LINE"));
    }

    #[test]
    fn process() {
        let r = RestProxyLog::new(Vec::new(), Vec::new(), 0, 1, 2);

        let lines = "34m2023-08-28 07:02:54.235 12345.99ms\n\
                                34m2023-08-28 07:02:54.235 1.123s\n\
                                34m2023-08-28 07:02:54.235 1.As"
            .as_bytes();

        let response = r.process(&mut BufReader::new(lines));

        let expected = vec![
            Response {
                date_as_string: String::from("2023-08-28 07:02:54 +00:00"),
                date_as_timestamp: 1693206174,
                exec_time: 12345,

                date_as_date: DateTime::parse_from_str("2023-08-28 07:02:54 +00:00", "%Y-%m-%d %H:%M:%S %z").unwrap(),
            },
            Response {
                date_as_string: String::from("2023-08-28 07:02:54 +00:00"),
                date_as_timestamp: 1693206174,
                exec_time: 1123,

                date_as_date: DateTime::parse_from_str("2023-08-28 07:02:54 +00:00", "%Y-%m-%d %H:%M:%S %z").unwrap(),
            },
            Response {
                date_as_string: String::from("2023-08-28 07:02:54 +00:00"),
                date_as_timestamp: 1693206174,
                exec_time: 0, //not parsed correctly (on purpose)

                date_as_date: DateTime::parse_from_str("2023-08-28 07:02:54 +00:00", "%Y-%m-%d %H:%M:%S %z").unwrap(),
            },
        ];
        assert_eq!(expected, response);
        //println!("RESPONSE: {:?}", response)
    }
}
