use rpassword::read_password;
use std::{
    fmt,
    fs::File,
    io::{self, BufRead, BufReader, Write},
    ptr,
};

#[derive(Clone, PartialEq, Eq)]
pub struct Password(String);

impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Password(******)")
    }
}

impl Password {
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

// Custom drop impl to zero out memory.
impl Drop for Password {
    fn drop(&mut self) {
        unsafe {
            for byte_ref in self.0.as_mut_vec() {
                ptr::write_volatile(byte_ref, 0)
            }
        }
    }
}

impl From<String> for Password {
    fn from(s: String) -> Password {
        Password(s)
    }
}

impl<'a> From<&'a str> for Password {
    fn from(s: &'a str) -> Password {
        Password::from(String::from(s))
    }
}

const PASSWORD_STDIN_ERROR: &str = "Unable to ask for password on non-interactive terminal.";

/// Flush output buffer.
pub fn flush_stdout() {
    io::stdout().flush().expect("stdout is flushable; qed");
}

/// Prompts user asking for password.
pub fn password_prompt() -> Result<Password, String> {
    println!("Please note that password is NOT RECOVERABLE.");
    print!("Type password: ");
    flush_stdout();

    let password = read_password()
        .map_err(|_| PASSWORD_STDIN_ERROR.to_owned())?
        .into();

    print!("Repeat password: ");
    flush_stdout();

    let password_repeat = read_password()
        .map_err(|_| PASSWORD_STDIN_ERROR.to_owned())?
        .into();

    if password != password_repeat {
        return Err("Passwords do not match!".into());
    }

    Ok(password)
}

pub fn input_password() -> Result<Password, String> {
    print!("Type password: ");
    flush_stdout();

    let password = read_password()
        .map_err(|_| PASSWORD_STDIN_ERROR.to_owned())?
        .into();

    Ok(password)
}

/// Read a password from password file.
pub fn password_from_file(path: String) -> Result<Password, String> {
    let passwords = passwords_from_files(&[path])?;
    // use only first password from the file
    passwords
        .get(0)
        .map(Password::clone)
        .ok_or_else(|| "Password file seems to be empty.".to_owned())
}

/// Reads passwords from files. Treats each line as a separate password.
pub fn passwords_from_files(files: &[String]) -> Result<Vec<Password>, String> {
    let passwords = files.iter().map(|filename| {
		let file = File::open(filename).map_err(|_| format!("{} Unable to read password file. Ensure it exists and permissions are correct.", filename))?;
		let reader = BufReader::new(&file);
		let lines = reader.lines()
			.filter_map(|l| l.ok())
			.map(|pwd| pwd.trim().to_owned().into())
			.collect::<Vec<Password>>();
		Ok(lines)
	}).collect::<Result<Vec<Vec<Password>>, String>>();
    Ok(passwords?.into_iter().flatten().collect())
}
