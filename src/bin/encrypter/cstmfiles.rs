#[allow(dead_code)]
pub mod cstmfiles {
    use std::fs;
    use std::fs::OpenOptions;
    use std::io::{BufReader, Read, Write};
    use std::os::unix::fs::OpenOptionsExt;


    pub fn create(path: &String) -> std::io::Result<()>{
        if fs::metadata(path).is_ok() {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "File already exists"))
        }
        let f = OpenOptions::new()
                .create(true)
                .write(true)
                .mode(0o664)
                .open(&path)?;
        let perms = f.metadata()?.permissions();
        // perms.set_readonly(true);
        // f.set_permissions(perms)?;
        println!("File permissions: {:?}", perms);
        Ok(())
    }


    /**
     * .sync_all() - attempts to sync all OS-internal metadata to disk.
     * .flush()    - flush this output stream, ensuring that all intermediately
     *               buffered contents reach their destination
     */
    pub fn write(path : &String, fcontents: String) -> std::io::Result<()> {
        let fc_with_nl : String = fcontents + "\r\n";
        let mut f = OpenOptions::new()
                    .write(true)
                    .open(&path)?;
        // f.set_len(5)?;
        f.write_all(fc_with_nl.as_bytes())?;
        f.sync_all()?;
        f.flush()?;
        Ok(())
    }


    pub fn read(path : &String) -> Result<String, String> {
        let f = match OpenOptions::new().read(true).open(&path) {
            Ok(file) => file,
            Err(e) => {
                return Err(format!("Error reading file at path {}: {}", path, e));
            }
        };

        let mut buf_reader = BufReader::new(f);
        let mut fcontents : String = String::new();

        match buf_reader.read_to_string(&mut fcontents) {
            Ok(size) => {
                println!("Read {} bytes from file.", size)
            },
            Err(e) => {
                return Err(format!("Error reading file to string: {}", e));
            }
        };

        Ok(fcontents)
    }


    pub fn create_dir(path: &String) -> Result<(), String> {
        match fs::create_dir_all(path) {
            Ok(()) => {
                println!("Directory created successfuly at {}", &path);
                Ok(())
            },
            Err(e) => {
                if fs::metadata(path).is_ok() {
                    // dir exists
                    return Ok(())
                }
                let errmsg = format!("Error creating directory on path {}: {}",path, e);
                println!("{}", &errmsg);
                Err(errmsg)
            }

        }
    }


    #[allow(dead_code)]
    pub fn remove(path : &String) -> std::io::Result<()> {
        fs::remove_file(path)?;
        Ok(())
    }
    #[allow(dead_code)]
    pub fn get_f_len(path : &String) -> std::io::Result<u64> {
        let f = OpenOptions::new()
                .read(true)
                .open(&path)?;
        let len = f.metadata().unwrap().len();
        Ok(len)
    }
}