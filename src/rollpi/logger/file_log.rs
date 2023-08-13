use std::fs::File;

pub struct FileLogger
{
    file: File,
}

// #[cfg(windows)]
// const LINE_ENDING: &str = "\r\n";
// #[cfg(not(windows))]
// const LINE_ENDING: &str = "\n";

impl FileLogger
{
    pub fn new(id: String) -> Self
    {
        FileLogger {
            file: File::create(format!("logs/{}.log", id)).unwrap()
        }
    }

    pub fn log_s(self: &mut Self, msg: &str)
    {
        use std::io::Write;

        // let m = format!("{}{}", msg, LINE_ENDING);
        self.file.write_all(msg.as_bytes()).unwrap()
    }

    pub fn log(self: &mut Self, msg: String)
    {
        use std::io::Write;

        self.file.write_all(msg.as_bytes()).unwrap();
    }
}