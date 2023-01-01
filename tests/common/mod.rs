use std::{fs::File, io::Write, time::Duration};

pub fn sleep(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}

pub struct TestOutput {
    expected: String,
    file: File,
}

impl TestOutput {
    pub fn new(expected_lines: &[&str]) -> Self {
        let expected = expected_lines.join("\n") + "\n";
        let file = File::create("tmp.txt").unwrap();
        Self { expected, file }
    }

    pub fn write_line(&mut self, line: impl AsRef<str>) {
        let line = line.as_ref();
        self.file.write_all(line.as_bytes()).unwrap();
        self.file.write_all(&[b'\n']).unwrap();
        self.file.flush().unwrap();
    }

    pub fn check(&self) {
        let output = std::fs::read_to_string("tmp.txt").unwrap();
        assert_eq!(output, self.expected);
        eprintln!("output matches ({} bytes)", output.len());
    }
}

impl Drop for TestOutput {
    fn drop(&mut self) {
        self.check();
    }
}
