use colored::*;

pub struct Banner {
    pub lines: Vec<BannerLine>,
    pub width: usize
}

impl Banner {
    pub fn print(self: &Banner) {
        for line in self.lines.iter() {
            line.print(self.width);
        }
    }
}

pub struct BannerLine {
    parts: Vec<BannerPart>
}

impl BannerLine {
    pub fn build_key_value(
        key_text: &str, 
        key_color: &str, 
        value_text: &str, 
        value_color: &str
    ) -> BannerLine {
        return BannerLine { 
            parts: vec![
                BannerPart { text: String::from(key_text), color: String::from(key_color)},
                BannerPart { text: String::from(value_text), color: String::from(value_color) }
            ]
        }
    }

    pub fn print(self: &BannerLine, panel_width: usize) {
        let edge = "│".green();
        let mut col: usize;
    
        // Print left edge plus one space (col: 2)
        print!("{} ", edge);
        col = 1;
    
        // Print parts
        for part in self.parts.iter() {
            print!("{}", part.text.color(&part.color[..]));
            col = col + part.text.len();
        }
    
        // Print remaining space
        print!("{}", (col..panel_width).map(|_| " ").collect::<String>());
        println!("{}", edge);
    }
}

pub struct BannerPart {
    text: String,
    color: String
}
