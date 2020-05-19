use colored::*;

pub struct Logger {
    pub source: String
}

pub trait Log {
    fn log(&self, message: &str);
    fn log_color(&self, message: &str, color: &str);
}

impl Log for Logger {
    fn log(&self, message: &str) {
        self.log_color(message, "white");
    }

    fn log_color(&self, message: &str, source_color: &str) {
        // Use white as default color if the provided color is not recognized
        let color_res : Result<Color, ()> = source_color.parse();
        let color_source = self.source.color(color_res.unwrap_or(Color::White));

        // Print log statement
        println!("[{0:>14}] {1}", color_source, message);
    }
}
