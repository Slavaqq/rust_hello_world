use slug;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;

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

impl FromStr for Operation {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "lowercase" => Ok(Operation::Lowercase),
            "uppercase" => Ok(Operation::Uppercase),
            "no-spaces" => Ok(Operation::NoSpaces),
            "slugify" => Ok(Operation::Slugify),
            "unchanged" => Ok(Operation::Unchanged),
            "crabify" => Ok(Operation::Crabify),
            "csv" => Ok(Operation::Csv),
            _ => Err(From::from(format!("Unknown argument: {s}!"))),
        }
    }
}

pub fn lowercase(s: &str) -> Result<String, Box<dyn Error>> {
    Ok(s.trim().to_lowercase())
}

pub fn uppercase(s: &str) -> Result<String, Box<dyn Error>> {
    Ok(s.trim().to_uppercase())
}

pub fn no_spaces(s: &str) -> Result<String, Box<dyn Error>> {
    Ok(s.trim().replace(" ", ""))
}

pub fn slugify(s: &str) -> Result<String, Box<dyn Error>> {
    Ok(slug::slugify(s.trim()))
}

pub fn unchanged(s: &str) -> Result<String, Box<dyn Error>> {
    Ok(String::from(s.trim()))
}

pub fn crabify(s: &str) -> Result<String, Box<dyn Error>> {
    Ok("🦀".repeat(s.trim().chars().count()))
}

pub fn csv(s: &str) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(s.trim())?;
    let mut input = String::new();
    file.read_to_string(&mut input)?;
    let mut lines = input.lines();
    let header: Vec<&str> = lines.next().ok_or("Missing header!")?.split(",").collect();
    let rows: Vec<Vec<&str>> = lines.into_iter().map(|e| e.split(",").collect()).collect();
    let header_length = header.len();
    for row in &rows {
        let row_length = row.len();
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
