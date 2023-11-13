//! Standard library macros

/// Prints to the standard output.
///
/// Equivalent to the [`println!`] macro except that a newline is not printed at
/// the end of the message.
///
/// [`println!`]: crate::println
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::io::__print_impl(format_args!($($arg)*));
    }
}

/// Prints to the standard output, with a newline.
#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($($arg:tt)*) => {
        $crate::io::__print_impl(format_args!("{}\n", format_args!($($arg)*)));
    }
}

// u_log: 0 for all, 1 for info, 2 for dev, 3 for debug, >=4 for none

#[macro_export]
macro_rules! pinfo {
    ($($arg:tt)*) => {
        let log_level = option_env!("U_LOG").expect("0").parse::<u8>().unwrap();
        if log_level <= 1 {
            $crate::io::__print_impl(format_args!("\x1b[92m[Info]\x1b[0m {}\n", format_args!($($arg)*)));
        }
    }
}

#[macro_export]
macro_rules! pdev {
    ($($arg:tt)*) => {
        let log_level = option_env!("U_LOG").unwrap_or("0").parse::<u8>().unwrap();
        if log_level <= 2 {
            $crate::io::__print_impl(format_args!("\x1b[93m[Dev]\x1b[0m {}\n", format_args!($($arg)*)));
        }
    }
}

#[macro_export]
macro_rules! pdebug {
    ($($arg:tt)*) => {
        let log_level = option_env!("U_LOG").unwrap_or("0").parse::<u8>().unwrap();
        if log_level <= 3 {
            $crate::io::__print_impl(format_args!("\x1b[91m[Debug]\x1b[0m {}\n", format_args!($($arg)*)));
        }
    }
}
