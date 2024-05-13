use slug;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum Operation {
    Lowercase,
    Uppercase,
    NoSpaces,
    Slugify,
    Unchanged,
    Crabify,
    Csv,
}

pub fn lowercase(s: &str) -> Result<String, Box<dyn Error>> {
    Ok(s.to_lowercase())
}

pub fn uppercase(s: &str) -> Result<String, Box<dyn Error>> {
    Ok(s.to_uppercase())
}

pub fn no_spaces(s: &str) -> Result<String, Box<dyn Error>> {
    Ok(s.replace(" ", ""))
}

pub fn slugify(s: &str) -> Result<String, Box<dyn Error>> {
    Ok(slug::slugify(s))
}

pub fn unchanged(s: &str) -> Result<String, Box<dyn Error>> {
    Ok(String::from(s))
}

pub fn crabify(s: &str) -> Result<String, Box<dyn Error>> {
    Ok("🦀".repeat(s.trim().chars().count()))
}

pub fn csv(s: &str) -> Result<String, Box<dyn Error>> {
    let mut lines = s.lines();
    let header: Vec<&str> = lines.next().ok_or("Missing header!")?.split(",").collect();
    let rows: Vec<Vec<&str>> = lines.into_iter().map(|e| e.split(",").collect()).collect();
    let header_length = header.len();
    for row in &rows {
        let row_length = row.len();
        println!("{:?}", row);
        if row_length != header_length {
            return Err(From::from(format!(
                "Excepting {} columns, got {}!",
                header_length, row_length,
            )));
        }
    }
    Ok(Csv { header, rows }.to_string())
}

struct Csv<'a> {
    header: Vec<&'a str>,
    rows: Vec<Vec<&'a str>>,
}

impl fmt::Display for Csv<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut all_rows = Vec::from(self.rows.clone());
        all_rows.push(self.header.clone());
        let columns_max: Vec<usize> = (0..self.header.len())
            .map(|i| {
                all_rows
                    .iter()
                    .map(|inner| inner[i].chars().count())
                    .max()
                    .unwrap_or(0)
            })
            .collect();

        let line = columns_max.iter().fold(String::from("+"), |acc, lenght| {
            acc + &"-".repeat(*lenght) + "-+"
        });

        let head = columns_max
            .iter()
            .enumerate()
            .fold(String::from("|"), |acc, (i, length)| {
                acc + &self.header[i] + &" ".repeat(*length - self.header[i].chars().count()) + " |"
            });

        let mut rows = String::new();

        for row in self.rows.iter() {
            let table_row =
                columns_max
                    .iter()
                    .enumerate()
                    .fold(String::from("|"), |acc, (i, length)| {
                        acc + &row[i] + &" ".repeat(*length - row[i].chars().count()) + " |"
                    });
            rows.push_str(&table_row);
            rows.push_str("\n");
        }
        let output = line.clone() + "\n" + &head + "\n" + &line + "\n" + &rows + &line;

        write!(f, "{}", output)
    }
}
