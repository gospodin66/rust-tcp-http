use std::fmt;
use std::error;

#[derive(Debug, Clone)]
pub struct CoreErr {
    pub errmsg: String,
    pub errno: u8
}

impl fmt::Display for CoreErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = format!(
            "CoreErr: Errno: {errno} | Errmsg: {errmsg}",
            errno=self.errno,
            errmsg=self.errmsg
        );
        write!(f, "{}", err)
    }
}

impl error::Error for CoreErr {}
