use std::ffi::OsStr;
use std::fs;
use std::fs::OpenOptions;
use std::io::{BufReader, Read, Write, Error, ErrorKind};
#[allow(unused_imports)]
use std::path::Path;

/**
 * create empty dir
 */
#[allow(dead_code)]
pub fn f_create_dir(path: &Path) -> std::io::Result<()>{
    if fs::metadata(path).is_ok(){
        return Err(Error::new(ErrorKind::Other, format!("cstmfiles: Directory already exists at path: {:?}", path)));
    }
    let parent_dir = Path::new(path).parent().unwrap();
    if !parent_dir.exists() {
        fs::create_dir_all(parent_dir).expect("Failed to create directory");
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
#[allow(dead_code)]
pub fn f_create(path: &Path) -> std::io::Result<()>{
    //if fs::metadata(path).is_ok() {
    //    return Err(Error::new(ErrorKind::Other, "File already exists"))
    //}
    //match f_create_dir(path) {
    //    Ok(()) => {},
    //    Err(e) => return Err(Error::new(ErrorKind::Other, format!("Error: Failed to create parent dir: {}", e)))
    //}
    let f: fs::File = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(true) 
        .open(path)?;


    println!("\n\nDEBUG: {:?}\n\n", &f);

    let perms: fs::Permissions = f.metadata()?.permissions();
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
#[allow(dead_code)]
pub fn f_write(path: &Path, fcontents: String) -> std::io::Result<()> {
    let fc_with_nl: String = fcontents + "\r\n";
    let mut f: fs::File = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(path)?;
    //f.set_len(5)?;
    f.write_all(fc_with_nl.as_bytes())?;
    f.sync_all()?;
    f.flush()?;
    Ok(())
}
#[allow(dead_code)]
pub fn f_read(path : &Path) -> std::io::Result<String> {
    let f = OpenOptions::new()
            .read(true)
            .open(&path)?;
    let mut buf_reader = BufReader::new(f);
    let mut fcontents : String = String::new();
    buf_reader.read_to_string(&mut fcontents)?;
    Ok(fcontents)
}
#[allow(dead_code)]
pub fn f_remove(path : &Path) -> std::io::Result<()> {
    fs::remove_file(path)?;
    Ok(())
}
#[allow(dead_code)]
pub fn f_get_f_len(path : &Path) -> std::io::Result<u64> {
    let f: fs::File = OpenOptions::new()
            .read(true)
            .open(&path)?;
    let len = f.metadata().unwrap().len();
    Ok(len)
}
#[allow(dead_code)]
pub fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename).extension().and_then(OsStr::to_str)
}