#[macro_export(local_inner_macros)]
macro_rules! etf {
    ($($json:tt)+) => {
        etf_internal!($($json)+)
    };
}

#[macro_export(local_inner_macros)]
macro_rules! etf_internal {
    // Done.
    (@array [$($elems:expr),*]) => {
        etf_internal_vec![$($elems),*]
    };

    // Next element is `null`.
    (@array [$($elems:expr,)*] null $($rest:tt)*) => {
        etf_internal!(@array [$($elems,)* etf_internal!(null)] $($rest)*)
    };

    // Next element is `true`.
    (@array [$($elems:expr,)*] true $($rest:tt)*) => {
        etf_internal!(@array [$($elems,)* etf_internal!(true)] $($rest)*)
    };

    // Next element is `false`.
    (@array [$($elems:expr,)*] false $($rest:tt)*) => {
        etf_internal!(@array [$($elems,)* etf_internal!(false)] $($rest)*)
    };

    // Next element is an array.
    (@array [$($elems:expr,)*] [$($array:tt)*] $($rest:tt)*) => {
        etf_internal!(@array [$($elems,)* etf_internal!([$($array)*])] $($rest)*)
    };

    // Next element is a map.
    (@array [$($elems:expr,)*] {$($map:tt)*} $($rest:tt)*) => {
        etf_internal!(@array [$($elems,)* etf_internal!({$($map)*})] $($rest)*)
    };

    // Next element is an expression followed by comma.
    (@array [$($elems:expr,)*] $next:expr, $($rest:tt)*) => {
        etf_internal!(@array [$($elems,)* etf_internal!($next),] $($rest)*)
    };

    // Last element is an expression with no trailing comma.
    (@array [$($elems:expr,)*] $last:expr) => {
        etf_internal!(@array [$($elems,)* etf_internal!($last)])
    };

    // Comma after the most recent element.
    (@array [$($elems:expr),*] , $($rest:tt)*) => {
        etf_internal!(@array [$($elems,)*] $($rest)*)
    };

    // Unexpected token after most recent element.
    (@array [$($elems:expr),*] $unexpected:tt $($rest:tt)*) => {
        json_unexpected!($unexpected)
    };

    // Done.
    (@object $object:ident () () ()) => {};

    // Insert the current entry followed by trailing comma.
    (@object $object:ident [$($key:tt)+] ($value:expr) , $($rest:tt)*) => {
        let _ = $object.push(($crate::convert::to_term(&$($key)+), $value));
        etf_internal!(@object $object () ($($rest)*) ($($rest)*));
    };

    // Current entry followed by unexpected token.
    (@object $object:ident [$($key:tt)+] ($value:expr) $unexpected:tt $($rest:tt)*) => {
        json_unexpected!($unexpected);
    };

    // Insert the last entry without trailing comma.
    (@object $object:ident [$($key:tt)+] ($value:expr)) => {
        let _ = $object.push(($crate::convert::to_term(&$($key)+), $value));
    };

    // Next value is `null`.
    (@object $object:ident ($($key:tt)+) (: null $($rest:tt)*) $copy:tt) => {
        etf_internal!(@object $object [$($key)+] (etf_internal!(null)) $($rest)*);
    };

    // Next value is `true`.
    (@object $object:ident ($($key:tt)+) (: true $($rest:tt)*) $copy:tt) => {
        etf_internal!(@object $object [$($key)+] (etf_internal!(true)) $($rest)*);
    };

    // Next value is `false`.
    (@object $object:ident ($($key:tt)+) (: false $($rest:tt)*) $copy:tt) => {
        etf_internal!(@object $object [$($key)+] (etf_internal!(false)) $($rest)*);
    };

    // Next value is an array.
    (@object $object:ident ($($key:tt)+) (: [$($array:tt)*] $($rest:tt)*) $copy:tt) => {
        etf_internal!(@object $object [$($key)+] (etf_internal!([$($array)*])) $($rest)*);
    };

    // Next value is a map.
    (@object $object:ident ($($key:tt)+) (: {$($map:tt)*} $($rest:tt)*) $copy:tt) => {
        etf_internal!(@object $object [$($key)+] (etf_internal!({$($map)*})) $($rest)*);
    };

    // Next value is an expression followed by comma.
    (@object $object:ident ($($key:tt)+) (: $value:expr , $($rest:tt)*) $copy:tt) => {
        etf_internal!(@object $object [$($key)+] (etf_internal!($value)) , $($rest)*);
    };

    // Last value is an expression with no trailing comma.
    (@object $object:ident ($($key:tt)+) (: $value:expr) $copy:tt) => {
        etf_internal!(@object $object [$($key)+] (etf_internal!($value)));
    };

    // Missing value for last entry. Trigger a reasonable error message.
    (@object $object:ident ($($key:tt)+) (:) $copy:tt) => {
        etf_internal!();
    };

    // Missing colon and value for last entry. Trigger a reasonable error message.
    (@object $object:ident ($($key:tt)+) () $copy:tt) => {
        etf_internal!();
    };

    // Misplaced colon. Trigger a reasonable error message.
    (@object $object:ident () (: $($rest:tt)*) ($colon:tt $($copy:tt)*)) => {
        json_unexpected!($colon);
    };

    // Found a comma inside a key. Trigger a reasonable error message.
    (@object $object:ident ($($key:tt)*) (, $($rest:tt)*) ($comma:tt $($copy:tt)*)) => {
        json_unexpected!($comma);
    };

    // Key is fully parenthesized. This avoids clippy double_parens false
    // positives because the parenthesization may be necessary here.
    (@object $object:ident () (($key:expr) : $($rest:tt)*) $copy:tt) => {
        etf_internal!(@object $object ($key) (: $($rest)*) (: $($rest)*));
    };

    // Refuse to absorb colon token into key expression.
    (@object $object:ident ($($key:tt)*) (: $($unexpected:tt)+) $copy:tt) => {
        json_expect_expr_comma!($($unexpected)+);
    };

    // Munch a token into the current key.
    (@object $object:ident ($($key:tt)*) ($tt:tt $($rest:tt)*) $copy:tt) => {
        etf_internal!(@object $object ($($key)* $tt) ($($rest)*) ($($rest)*));
    };

    (null) => {
        eetf::Term::Atom(eetf::Atom{ name: "nil".to_string() })
    };

    (true) => {
        eetf::Term::Atom(eetf::Atom{ name: "true".to_string() })
    };

    (false) => {
        eetf::Term::Atom(eetf::Atom{ name: "false".to_string() })
    };

    ([]) => {
        eetf::Term::List(eetf::List{ elements: Vec::new() })
    };

    ([ $($tt:tt)+ ]) => {
        eetf::Term::List(etf_internal!(@array [] $($tt)+))
    };

    ({}) => {
        eetf::Term::Map(eetf::Map{ entries: Vec::new() })
    };

    ({ $($tt:tt)+ }) => {
        eetf::Term::Map({
            let mut object = Vec::<(eetf::Term, eetf::Term)>::new();
            etf_internal!(@object object () ($($tt)+) ($($tt)+));
            eetf::Map{ entries: object }
        })
    };
    
    ($other:expr) => {
      $crate::convert::to_term(&$other)
    };
}

#[macro_export]
macro_rules! etf_internal_vec {
    ($($content:tt)*) => {
      eetf::List{ elements: vec![$($content)*] }
    };
}

#[macro_export]
macro_rules! json_unexpected {
    () => {};
}

#[macro_export]
macro_rules! json_expect_expr_comma {
    ($e:expr , $($tt:tt)*) => {};
}