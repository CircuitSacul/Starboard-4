/// Designed for multi-line strings with args.
#[macro_export]
macro_rules! concat_format {
    ($($string: literal $(<- $($arg: expr),*)?;)+) => {{
        use std::fmt::Write;
        let mut ret = String::new();
        $(
            write!(
                ret,
                $string,
                $($(
                    $arg,
                )*)*
            ).unwrap();
        )*
        ret
    }};
}
