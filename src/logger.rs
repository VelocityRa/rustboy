
#[macro_export]
macro_rules! debug {
	($fm:expr) => ({
		println!("[DEBUG]: {}", format!($fm))
	});
	($fm:expr, $($arg:expr),*) => ({
		println!("[DEBUG]: {}", format!($fm, $($arg,)* ))
	});
}

#[macro_export]
macro_rules! info {
	($fm:expr) => ({
		println!("[INFO]: {}", format!($fm))
	});
	($fm:expr, $($arg:expr)*) => ({
		println!("[INFO]: {}", format!($fm, $($arg,)* ))
	});
}

#[macro_export]
macro_rules! warn {
	($fm:expr) => ({
		println!("[WARN]: {}", format!($fm))
	});
	($fm:expr, $($arg:expr),*) => ({
		println!("[WARN]: {}", format!($fm, $($arg,)* ))
	});
}

#[macro_export]
macro_rules! error {
	($fm:expr) => ({
		println!("[ERROR]: {}", format!($fm))
	});
	($fm:expr, $($arg:expr)*) => ({
		panic!("[ERROR]: {}", format!($fm, $($arg,)* ))
	});
}
