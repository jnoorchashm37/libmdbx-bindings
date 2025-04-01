#[macro_export]
macro_rules! tables {
    ([$($derives:path),*] => $set_name:ident, $num_tables:expr, [$($table:ident),*]) => {
        #[derive(Debug, PartialEq, Copy, Clone, Eq, Hash, $($derives),*)]
        #[repr(u8)]
        /// Default tables that should be present inside database.
        pub enum $set_name {
            $(
                #[doc = concat!("Represents a ", stringify!($table), " table")]
                $table,
            )*
        }
        tables!(PRIVATE | $set_name, $num_tables, [$($table),*]);
    };

    ($set_name:ident, $num_tables:expr, [$($table:ident),*]) => {
        #[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
        #[repr(u8)]
        /// Default tables that should be present inside database.
        pub enum $set_name {
            $(
                #[doc = concat!("Represents a ", stringify!($table), " table")]
                $table,
            )*
        }
        tables!(PRIVATE | $set_name, $num_tables, [$($table),*]);
    };

    (PRIVATE | $set_name:ident, $num_tables:expr, [$($table:ident),*]) => {
        impl $set_name {
            /// Array of all tables in database
            pub const ALL: [$set_name; $num_tables] = [$($set_name::$table,)*];

            /// The name of the given table in database
            pub const fn name(&self) -> &str {
                match self {
                    $($set_name::$table => {
                        <$table as libmdbx_bindings::Table>::NAME
                    },)*
                }
            }

            /// The type of the given table in database
            pub const fn table_type(&self) -> libmdbx_bindings::TableType {
                match self {
                    $($set_name::$table => {
                        libmdbx_bindings::TableType::Table
                    },)*
                }
            }

            fn create_table(
                &self,
                txn: &libmdbx_bindings::LibmdbxTx<libmdbx_bindings::RW>
            ) -> Result<(), libmdbx_bindings::DatabaseError> {

                match self {
                    $(
                        Self::$table => txn.create_table(&$table),
                    )*
                }
            }
        }

        impl std::fmt::Display for $set_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.name())
            }
        }

        impl std::str::FromStr for $set_name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(<$table as libmdbx_bindings::Table>::NAME => {
                        Ok($set_name::$table)
                    },)*
                    _ => {
                        Err("Unknown table".to_string())
                    }
                }
            }
        }

        impl libmdbx_bindings::TableSet for $set_name {
            const NUM_TABLES: usize = $num_tables;

            fn create_tables(
                txn: &libmdbx_bindings::LibmdbxTx<libmdbx_bindings::RW>
            ) -> Result<(), libmdbx_bindings::DatabaseError> {

                for table in Self::ALL {
                    table.create_table(txn)?;
                }
                Ok(())
            }

            fn as_usize(&self) -> usize {
                *self as usize
            }
        }
    };
}

#[macro_export]
macro_rules! db_table {
    ( ( $table:ident ) | $key:ty, $value:ty) => {
        #[doc = concat!("Takes [`", stringify!($key), "`] as a key and returns [`", stringify!($value), "`].")]
        #[derive(Clone, Copy, Debug, Default)]
        pub struct $table;

        impl libmdbx_bindings::Table for $table {
            type Key = $key;
            type Value = $value;

            const NAME: &'static str = stringify!($table);
            const DUPSORT: bool = false;
        }

        impl std::fmt::Display for $table {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", stringify!($table))
            }
        }

        impl libmdbx_bindings::TableDet for $table {
            fn table_type(&self) -> libmdbx_bindings::TableType {
                libmdbx_bindings::TableType::Table
            }
        }
    };
}
