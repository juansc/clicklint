use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::fmt::Error;
use nom::branch::alt;
use nom::bytes::complete::{is_a, is_not, tag, tag_no_case, take_until};
use nom::combinator::opt;
use nom::IResult;
use nom::multi::separated_list0;

fn main() -> std::result::Result<(), BoxError> {
    // First parse the table to make sure that it has the correct syntax
    let (_, table) = parse_table("CREATE TABLE table (my_date Date, my_string String)")?;
    // Run linters to catch any errors
    let linters: Vec<fn(&Table) -> Option<String>> = vec![check_duplicate_col_names, check_table_name_is_not_short];
    let mut did_find_issue = false;
    for linter in &linters {
        match linter(&table) {
            Some(err) => {
                did_find_issue = true;
                println!("encountered error:\n\n{}", err)
            }
            None => {}
        }
    }
    if !did_find_issue {
        println!("Congrats! Your table looks fine")
    }
    Ok(())
}

/// Type-erased errors.
pub type BoxError = std::boxed::Box<
    dyn std::error::Error // must implement Error to satisfy ?
        + std::marker::Send // needed for threads
        + std::marker::Sync, // needed for threads
>;

#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    name: String,
    columns: Vec<Col>,
    if_not_exists: bool,
}

fn parse_table(input: &str) -> IResult<&str, Table> {
    let (input, _) = tag_no_case("create table ")(input)?;
    let (input, if_not_exists_str) = opt(tag_no_case("if not exists"))(input)?;
    let if_not_exists = match if_not_exists_str {
        Some(_) => true,
        _ => false,
    };
    let (input, name) = take_until(" ")(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, _) = tag("(")(input)?;
    let (input, cols) = separated_list0(tag(", "), parse_col)(input)?;
    let (input, _) = tag(")")(input)?;
    Ok((
        input,
        Table {
            name: name.to_string(),
            columns: cols,
            if_not_exists,
        },
    ))
}

#[derive(Debug, Clone, PartialEq)]
pub struct Col {
    name: String,
    col_type: String,
}

fn parse_col(input: &str) -> IResult<&str, Col> {
    let (input, name) = take_until(" ")(input)?;
    let (input, _) = is_a(" \t\r\n")(input)?;
    let (input, col_type) = alt((tag("Date"), tag("String")))(input)?;
    Ok((
        input,
        Col {
            name: name.to_string(),
            col_type: col_type.to_string(),
        },
    ))
}

fn check_duplicate_col_names(t: &Table) -> Option<String> {
    let mut errors = "".to_string();
    let mut col_names = HashMap::new();
    for col in t.columns.iter() {
        *col_names.entry(col.name.clone()).or_insert(0) += 1;
    }
    for entry in &col_names {
        if *entry.1 > 1 {
            errors += &format!("Duplicated column {} was encountered {} times.\n", entry.0, entry.1).to_owned()
        }
    }
    if errors.len() == 0 {
        return None
    }
    Some(errors.to_string())
}

fn check_table_name_is_not_short(t: &Table) -> Option<String> {
    const MIN_LENGTH: usize = 5;
    if t.name.len() < MIN_LENGTH  {
        return Some(format!("Your table name '{}' is too short. We recommend at least {} characters." ,t.name, MIN_LENGTH));
    }
    None
}


#[cfg(test)]
mod test {
    use crate::{parse_col, parse_table, Col, Table};

    #[test]
    fn test_parse_col() {
        assert_eq!(
            parse_col("name Date"),
            Ok((
                "",
                Col {
                    name: "name".to_string(),
                    col_type: "Date".to_string()
                }
            ))
        )
    }

    #[test]
    fn test_parse_table() {
        assert_eq!(
            parse_table("CREATE TABLE table (my_date Date, my_string String)"),
            Ok((
                "",
                Table {
                    name: "table".to_string(),
                    columns: vec!(
                        Col {
                            name: "my_date".to_string(),
                            col_type: "Date".to_string(),
                        },
                        Col {
                            name: "my_string".to_string(),
                            col_type: "String".to_string(),
                        },
                    ),
                    if_not_exists: false
                }
            ))
        )
    }
}
