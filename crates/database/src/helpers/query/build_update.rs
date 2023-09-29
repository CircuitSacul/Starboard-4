macro_rules! build_update {
    ($builder_struct: expr, $builder_query: expr, $($arg: ident),*) => {
        {
            let mut is_first = true;
            $(
                if !is_first {
                    $builder_query.push(", ");
                } else {
                    is_first = false;
                }

                $builder_query
                    .push(stringify!($arg))
                    .push("=")
                    .push_bind($builder_struct.$arg);
            )+

            is_first
        }
    };
}

pub(crate) use build_update;
