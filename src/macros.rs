macro_rules! define_enum {
    (
        #[$($attr:tt)*]
        enum $name:ident {
            Other( $inner_ty:ty )
            $(,$variant:ident = $variant_value:expr)*
        }
    ) => {
        #[$($attr)*]
        pub enum $name {
            $($variant,)*
            Other( $inner_ty )
        }

        impl $name {
            pub fn raw( &self ) -> $inner_ty {
                match *self {
                    $($name::$variant => $variant_value,)*
                    $name::Other( value ) => value,
                }
            }

            #[inline(never)]
            pub fn try_from_str( string: &str ) -> Option< Self > {
                match string {
                    $(stringify!( $variant ) => Some( $name::$variant ),)*
                    _ => None
                }
            }

            pub const LIST: &'static [(&'static str, $name)] = &[
                $((stringify!( $variant ), $name::$variant),)*
            ];
        }

        impl From< $inner_ty > for $name {
            fn from( value: $inner_ty ) -> Self {
                match value {
                    $($variant_value => $name::$variant,)*
                    _ => $name::Other( value )
                }
            }
        }

        impl From< $name > for $inner_ty {
            fn from( value: $name ) -> Self {
                match value {
                    $($name::$variant => $variant_value,)*
                    $name::Other( value ) => value
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt( &self, fmt: &mut std::fmt::Formatter ) -> std::fmt::Result {
                match *self {
                    $name::Other( value ) => write!( fmt, "0x{:03X}", value ),
                    $(
                        $name::$variant => write!( fmt, stringify!( $variant ) ),
                    )*
                }
            }
        }
    }
}
