#[macro_export]
macro_rules! unlikely {
    ($e:expr) => {
        $e
    };
}

#[macro_export]
macro_rules! likely {
    ($e:expr) => {
        $e
    };
}

#[macro_export]
macro_rules! primitive_enum {
    (
        $(#[$meta:meta])* $visibility:vis
        enum $Name:ident: $PrimitiveType:ident {
            $($Variant:ident $(= $value:expr)?),*
            $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Copy, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
        #[repr($PrimitiveType)]
        $visibility enum $Name {
            $($Variant $(= $value)?),*
        }

        impl From<$PrimitiveType> for $Name {
            fn from(primitive: $PrimitiveType) -> $Name {
                unsafe { std::mem::transmute(primitive) }
            }
        }

        impl From<$Name> for $PrimitiveType {
            fn from(v: $Name) -> $PrimitiveType {
                v as $PrimitiveType
            }
        }
    };
}
