
#[macro_export]
macro_rules! debug {
	($fm:expr, $($arg:expr),*) => ({
		println!("[DEBUG]: {}", format!($fm, $($arg,)* )
		)
	});
}

#[macro_export]
macro_rules! info {
	($fm:expr, $($arg:expr)*) => ({
		println!("[INFO]: {}", format!($fm, $($arg,)* )
		)
	});
}

#[macro_export]
macro_rules! warn {
	($fm:expr, $($arg:expr),*) => ({
		println!("[WARN]: {}", format!($fm, $($arg,)* )
		)
	});
}

#[macro_export]
macro_rules! error {
	($fm:expr, $($arg:expr)*) => ({
		panic!("[ERROR]: {}", format!($fm, $($arg,)* )
		)
	});
}
