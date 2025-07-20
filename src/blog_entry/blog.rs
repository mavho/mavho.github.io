use chrono::NaiveDate;
use std::collections::HashMap;

#[derive(Clone)]
#[derive(PartialEq, Eq)]
pub struct Blog{
    pub html_file:String,
    pub date_string:NaiveDate,
    pub md_metadata: Option<HashMap<String,String>>
}

impl Ord for Blog{
    // Sort in descending order
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.date_string.cmp(&self.date_string)
    }
}

impl PartialOrd for Blog{
    // Sort in descending order
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.date_string.partial_cmp(&self.date_string)
    }

}
