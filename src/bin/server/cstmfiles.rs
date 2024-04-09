use std::fs;
use std::fs::OpenOptions;
use std::io::{BufReader, Read, Write, Error, ErrorKind};
use std::os::unix::fs::OpenOptionsExt;
use crate::server::cstmconfig::AssetsConfig;
#[allow(unused_imports)]
use std::path::Path;

/**
 * create empty dir
 */
pub fn f_create_dir(path: &String) -> std::io::Result<()>{
    if fs::metadata(path).is_ok(){
        return Err(Error::new(ErrorKind::Other, format!("cstmfiles: Directory already exists at path: {}", path)));
    }
    match fs::create_dir(path){
        Ok(()) => {},
        Err(e) => {
            return Err(Error::new(ErrorKind::Other, format!("cstmfiles: Error creating dir: {}", e)))
        }
    }
    Ok(())
}

/**
 * Since many things can go wrong when doing file I/O,
 * all the File methods return the io::Result<T> type,
 * which is an alias for Result<T, io::Error>
 * 
 * '?' operator is shorthand for e.g: .expect("Unable to open file")
 * 'Result<()>' is shorthand for e.g: Result<T,io::Error>
 */
pub fn f_create(path: &String) -> std::io::Result<()>{
    if fs::metadata(path).is_ok() {
        return Err(Error::new(ErrorKind::Other, "File already exists"))
    }

    let assets_config : AssetsConfig = AssetsConfig::new_cfg();

    match f_create_dir(&assets_config.log_dir) {
        Ok(()) => {
            println!("cstmfiles: Parent dir created: {}", assets_config.log_dir);
        },
        Err(e) => {
            println!("cstmfiles: Parent dir already exists: {}", e);
        }
    }

    let f = OpenOptions::new()
            .create(true)
            .write(true)
            .mode(0o644)
            .open(&path)?;
    let perms = f.metadata()?.permissions();
    //perms.set_readonly(true);
    //f.set_permissions(perms)?;
    println!("cstmfiles: File permissions: {:?}", perms);
    Ok(())
}
/**
 * .sync_all() - attempts to sync all OS-internal metadata to disk.
 * .flush()    - flush this output stream, ensuring that all intermediately
 *               buffered contents reach their destination
 */
pub fn f_write(path : &String, fcontents: String) -> std::io::Result<()> {
    let fc_with_nl : String = fcontents + "\r\n";
    let mut f = OpenOptions::new()
                .write(true)
                .append(true)
                .open(&path)?;
    //f.set_len(5)?;
    f.write_all(fc_with_nl.as_bytes())?;
    f.sync_all()?;
    f.flush()?;
    Ok(())
}
#[allow(dead_code)]
pub fn f_read(path : &String) -> std::io::Result<String> {
    let f = OpenOptions::new()
            .read(true)
            .open(&path)?;
    let mut buf_reader = BufReader::new(f);
    let mut fcontents : String = String::new();
    buf_reader.read_to_string(&mut fcontents)?;
    Ok(fcontents)
}
#[allow(dead_code)]
pub fn f_remove(path : &String) -> std::io::Result<()> {
    fs::remove_file(path)?;
    Ok(())
}
#[allow(dead_code)]
pub fn f_get_f_len(path : &String) -> std::io::Result<u64> {
    let f = OpenOptions::new()
            .read(true)
            .open(&path)?;
    let len = f.metadata().unwrap().len();
    Ok(len)
}
