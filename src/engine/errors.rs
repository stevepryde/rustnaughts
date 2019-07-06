use std::fmt;

#[derive(Default, Debug, Clone)]
pub struct StringError {
    msg: String,
}

impl StringError {
    pub fn new(err: &str) -> Self {
        StringError {
            msg: err.to_owned(),
        }
    }

}

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}
