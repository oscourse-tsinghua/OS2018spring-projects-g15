
/// Returns true if the passed expression matches the pattern
#[macro_export]
macro_rules! is
{
	($val:expr, $p:pat) => ( match $val { $p => true, _ => false } );
}

#[doc(hidden)]
#[macro_export]
macro_rules! _count
{
	() => {0};
	($a:expr) => {1};
	($a:expr, $($b:expr),+) => {1+_count!($($b),+)};
}

#[macro_export]
macro_rules! impl_fmt
{
	( $( <$($g:ident),+> $tr:ident ($s:ident, $f:ident) for $ty:ty { $code:expr } )+ ) => { $(
		impl<$($g),+> ::core::fmt::$tr for $ty {
			fn fmt(&$s, $f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
				$code
			}
		}
		)+
		};
	
	( $( $tr:ident ($s:ident, $f:ident) for $ty:ty { $code:expr } )+ ) => { $(
		impl ::core::fmt::$tr for $ty {
			fn fmt(&$s, $f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
				$code
			}
		}
		)+
		};
}

/// Implements the From trait for the provided type, avoiding boilerplate
#[macro_export]
macro_rules! impl_from {
	(@as_item $($i:item)*) => {$($i)*};

	($( $(<($($params:tt)+)>)* From<$src:ty>($v:ident) for $t:ty { $($code:stmt)*} )+) => {
		$(impl_from!{ @as_item 
			impl$(<$($params)+>)* ::core::convert::From<$src> for $t {
				fn from($v: $src) -> $t {
					$($code)*
				}
			}
		})+
	};
}

/// Override libcore's `try!` macro with one that backs onto `From`
#[macro_export]
macro_rules! try {
	($e:expr) => (
		match $e {
			Ok(v) => v,
			Err(e) => return Err(From::from(e)),
		}
	);
}

/// Provides a short and noticable "TODO: " message
#[macro_export]
macro_rules! todo
{
	( $s:expr ) => ( panic!( concat!("TODO: ",$s) ) );
	( $s:expr, $($v:tt)* ) => ( panic!( concat!("TODO: ",$s), $($v)* ) );
}
