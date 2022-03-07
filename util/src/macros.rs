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

#[macro_export]
macro_rules! bitfields {
    (
        $(#[$meta:meta])* $visibility:vis
        struct $Name:ident: $InnerType:ident {
            $( [$field_start:literal$(, $field_end:literal)?] $field_get:ident, $field_set:ident: $FieldType:ident ),* $(,)?
            $( readonly = $readonly:expr  $(,)?)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Default, Copy, Clone, PartialEq, Eq, Debug)]
        $visibility struct $Name {
            pub value: $InnerType,
        }

        impl $Name {
            pub const READONLY: $InnerType = 0 $(| $readonly)?;

            pub const fn new(inner_value: $InnerType) -> Self {
                $Name { value: inner_value }
            }

            pub fn set_preserve_bits(&mut self, value: $InnerType) {
                self.value = (value & !Self::READONLY) | (self.value & Self::READONLY)
            }

            $(
                pub fn $field_get(&self) -> $FieldType {
                    let bits = $crate::extract_bits!(self.value, $field_start $(, $field_end)?);
                    $crate::from_bits!(bits, $InnerType, $FieldType)
                }

                pub fn $field_set(&mut self, value: $FieldType) {
                    let new_bits = <$InnerType>::from(value);
                    self.value = $crate::replace_bits!(self.value, new_bits, $field_start $(, $field_end)?);
                }
            )*
        }

        impl From<$InnerType> for $Name {
            fn from(inner_value: $InnerType) -> $Name {
                $Name { value: inner_value }
            }
        }

        impl From<$Name> for $InnerType {
            fn from(v: $Name) -> $InnerType {
                v.value
            }
        }
    };
}

#[macro_export]
macro_rules! extract_bits {
    ($value:expr, $start:expr) => {
        $crate::bits::Bits::bits($value, $start, $start)
    };

    ($value:expr, $start:expr, $end:expr) => {
        $crate::bits::Bits::bits($value, $start, $end)
    };
}

#[macro_export]
macro_rules! replace_bits {
    ($dst:expr, $src:expr, $start:expr) => {
        $crate::bits::Bits::replace_bits($dst, $start, $start, $src)
    };

    ($dst:expr, $src:expr, $start:expr, $end:expr) => {
        $crate::bits::Bits::replace_bits($dst, $start, $end, $src)
    };
}

#[macro_export]
macro_rules! from_bits {
    ($bits:expr, u32, u16) => {
        $bits as u16
    };

    ($bits:expr, $SrcType:ty, bool) => {
        $bits != 0
    };

    ($bits:expr, $SrcType:ty, $DstType:ty) => {
        <$DstType as From<$SrcType>>::from($bits)
    };
}
