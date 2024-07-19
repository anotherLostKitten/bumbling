use std::collections::HashMap;
use std::sync::{Mutex, Arc, RwLock};

use once_cell::sync::Lazy;

pub enum ValueStoreEnum {
    StoredUsize(usize),
    StoredU64(u64),
    StoredU32(u32),
    StoredU8(u8),
    StoredIsize(isize),
    StoredI64(i64),
    StoredI32(i32),
    StoredChar(char),
    StoredString(Arc<String>),
}

macro_rules! value_store_impl_into {
    ( $t: path, $e: ident ) => {
        impl From<$t> for ValueStoreEnum {
            fn from(v: $t) -> Self {
                ValueStoreEnum::$e(v)
            }
        }
    };
}

macro_rules! value_store_impl_from {
    ( $t: path, $e: ident ) => {
        impl From<&ValueStoreEnum> for $t {
            fn from(store_enum: &ValueStoreEnum) -> Self {
                if let ValueStoreEnum::$e(v) = store_enum {
                    *v
                } else {
                    panic!("Could not convert value store to {} as it was not a {}", stringify!($t), stringify!($e));
                }
            }
        }
    };
}

macro_rules! value_store_impl_both {
    ( $t: path, $e: ident ) => {
        value_store_impl_into!($t, $e);
        value_store_impl_from!($t, $e);
    };
}

value_store_impl_both!(usize, StoredUsize);
value_store_impl_both!(u64, StoredU64);
value_store_impl_both!(u32, StoredU32);
value_store_impl_both!(u8, StoredU8);
value_store_impl_both!(isize, StoredIsize);
value_store_impl_both!(i64, StoredI64);
value_store_impl_both!(i32, StoredI32);
value_store_impl_both!(char, StoredChar);

impl From<Arc<String>> for ValueStoreEnum {
    fn from(v: Arc<String>) -> Self {
        ValueStoreEnum::StoredString(v)
    }
}

impl From<&ValueStoreEnum> for Arc<String> {
    fn from(store_enum: &ValueStoreEnum) -> Self {
        if let ValueStoreEnum::StoredString(v) = store_enum {
            v.clone()
        } else {
            panic!("Could not convert value store to {} as it was not a {}", stringify!(Arc<String>), stringify!(StoredString));
        }
    }
}


static VALUE_STORE: Lazy<RwLock<HashMap<u128, ValueStoreEnum>>> = Lazy::new(|| {
    RwLock::new(HashMap::new())
});

pub fn check_find_live_value<T: Into<ValueStoreEnum> + for <'a> From<&'a ValueStoreEnum> + Copy>(id: u128, val: T, _file: &'static str, _line: u32, _column: u32) -> T {
    let value_store = VALUE_STORE.read().unwrap();

    if let Some(v) = value_store.get(&id) {
        return T::from(v);
    }

    std::mem::drop(value_store);
    let mut value_store = VALUE_STORE.write().unwrap();

    match value_store.get(&id) {
        Some(v) =>
            T::from(v),
        None => {
            value_store.insert(id, val.into());
            val
        },
    }
}

pub fn check_find_live_value_arc<T: Into<ValueStoreEnum> + for <'a> From<&'a ValueStoreEnum> + Clone>(id: u128, val: T, _file: &'static str, _line: u32, _column: u32) -> T {
    let value_store = VALUE_STORE.read().unwrap();

    if let Some(v) = value_store.get(&id) {
        return T::from(v);
    }

    std::mem::drop(value_store);
    let mut value_store = VALUE_STORE.write().unwrap();

    match value_store.get(&id) {
        Some(v) =>
            T::from(v),
        None => {
            value_store.insert(id, val.clone().into());
            val
        },
    }
}

#[macro_export]
macro_rules! live_value {
    ( $x: expr ) => {
        match $x {
            tmp => {
                #[cfg(debug_assertions)]
                let val = $crate::live_value::check_find_live_value($crate::dev_macros::print_unique!(), tmp, file!(), line!(), column!());
                #[cfg(not(debug_assertions))]
                let val = tmp;
                val
            }
        }
    };
}

#[macro_export]
macro_rules! live_value_arc {
    ( $x: expr ) => {
        match Arc::new($x) {
            tmp => {
                #[cfg(debug_assertions)]
                let val = $crate::live_value::check_find_live_value_arc($crate::dev_macros::print_unique!(), tmp.clone(), file!(), line!(), column!());
                #[cfg(not(debug_assertions))]
                let val = tmp;
                val
            }
        }
    };
}
