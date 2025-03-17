use std::{marker::PhantomData, str::FromStr, sync::Arc};

use libmdbx_native::{DatabaseFlags, RO, RW, Transaction, TransactionKind, WriteFlags, ffi::DBI};
use parking_lot::RwLock;
use reth_db::{
    DatabaseError, DatabaseWriteOperation, TableType, Tables,
    table::{Compress, DupSort, Encode, Key, Table},
    transaction::{DbTx, DbTxMut},
};
use reth_storage_errors::db::DatabaseWriteError;

use super::{cursor::LibmdbxCursor, utils::decode_one};
use crate::{
    // tables::{NUM_TABLES, Tables},
    implementation::DatabaseEnv,
    traits::{TableDet, TableSet},
};

pub struct LibmdbxTx<K: TransactionKind, S: TableSet> {
    /// Libmdbx-sys transaction.
    inner: Transaction<K>,
    /// Database table handle cache.
    db_handles: Arc<RwLock<Vec<Option<DBI>>>>,
    // db_handles_len: usize,
    _phantom: PhantomData<S>,
}

impl<S: TableSet> LibmdbxTx<RO, S> {
    pub(crate) fn new_ro_tx(env: &DatabaseEnv) -> eyre::Result<LibmdbxTx<RO, S>, DatabaseError> {
        Ok(Self {
            inner: env
                .begin_ro_txn()
                .map_err(|e| DatabaseError::InitTx(e.into()))?,
            db_handles: Arc::new(RwLock::new(vec![None; S::NUM_TABLES])),
            // db_handles_len: S::NUM_TABLES,
            _phantom: PhantomData,
        })
    }
}

impl<S: TableSet> LibmdbxTx<RW, S> {
    pub fn create_table<T: TableDet>(&self, table: &T) -> Result<(), DatabaseError> {
        let flags = match table.table_type() {
            TableType::Table => DatabaseFlags::default(),
            TableType::DupSort => DatabaseFlags::DUP_SORT,
        };

        self.inner
            .create_db(Some(T::NAME), flags)
            .map_err(|e| DatabaseError::CreateTable(e.into()))?;

        Ok(())
    }

    pub(crate) fn new_rw_tx(env: &DatabaseEnv) -> Result<LibmdbxTx<RW, S>, DatabaseError> {
        Ok(Self {
            inner: env
                .begin_rw_txn()
                .map_err(|e| DatabaseError::InitTx(e.into()))?,
            db_handles: Arc::new(RwLock::new(vec![None; S::NUM_TABLES])),
            _phantom: PhantomData,
        })
    }
}

impl<K: TransactionKind, S: TableSet> LibmdbxTx<K, S> {
    /// Gets a table database handle if it exists, otherwise creates it.
    pub(crate) fn get_dbi<T: Table>(&self) -> Result<DBI, DatabaseError> {
        let mut handles = self.db_handles.write();

        let table = S::from_str(T::NAME).expect("Requested table should be part of `Tables`.");

        let dbi_handle = handles.get_mut(table.as_usize()).expect("should exist");
        if dbi_handle.is_none() {
            *dbi_handle = Some(
                self.inner
                    .open_db(Some(T::NAME))
                    .map_err(|e| DatabaseError::InitCursor(e.into()))?
                    .dbi(),
            );
        }

        Ok(dbi_handle.expect("is some; qed"))
    }

    /// Create db Cursor
    pub(crate) fn new_cursor<T: Table>(&self) -> Result<LibmdbxCursor<T, K>, DatabaseError> {
        let inner = self
            .inner
            .cursor_with_dbi(self.get_dbi::<T>()?)
            .map_err(|e| DatabaseError::InitCursor(e.into()))?;

        Ok(LibmdbxCursor::new(inner))
    }
}

impl<K: TransactionKind, S: TableSet> DbTx for LibmdbxTx<K, S> {
    type Cursor<T: Table> = LibmdbxCursor<T, K>;
    type DupCursor<T: DupSort> = LibmdbxCursor<T, K>;

    fn get<T: Table>(&self, key: T::Key) -> Result<Option<<T as Table>::Value>, DatabaseError> {
        self.get_by_encoded_key::<T>(&key.encode())
    }

    fn get_by_encoded_key<T: Table>(
        &self,
        key: &<T::Key as Encode>::Encoded,
    ) -> Result<Option<T::Value>, DatabaseError> {
        self.inner
            .get(self.get_dbi::<T>()?, key.as_ref())
            .map_err(|e| DatabaseError::Read(e.into()))?
            .map(decode_one::<T>)
            .transpose()
    }

    fn commit(self) -> Result<bool, DatabaseError> {
        self.inner
            .commit()
            .map(|(res, _latency)| res)
            .map_err(|e| DatabaseError::Commit(e.into()))
    }

    fn disable_long_read_transaction_safety(&mut self) {
        self.inner.disable_timeout();
    }

    fn abort(self) {
        drop(self.inner)
    }

    // Iterate over read only values in database.
    fn cursor_read<T: Table>(&self) -> Result<Self::Cursor<T>, DatabaseError> {
        self.new_cursor::<T>()
    }

    /// Iterate over read only values in database.
    fn cursor_dup_read<T: DupSort>(&self) -> Result<Self::DupCursor<T>, DatabaseError> {
        self.new_cursor::<T>()
    }

    /// Returns number of entries in the table using cheap DB stats invocation.
    fn entries<T: Table>(&self) -> Result<usize, DatabaseError> {
        Ok(self
            .inner
            .db_stat_with_dbi(self.get_dbi::<T>()?)
            .map_err(|e| DatabaseError::Stats(e.into()))?
            .entries())
    }
}

impl<S: TableSet> DbTxMut for LibmdbxTx<RW, S> {
    type CursorMut<T: Table> = LibmdbxCursor<T, RW>;
    type DupCursorMut<T: DupSort> = LibmdbxCursor<T, RW>;

    fn put<T: Table>(&self, key: T::Key, value: T::Value) -> Result<(), DatabaseError> {
        let key = key.encode();
        let value = value.compress();
        self.inner
            .put(
                self.get_dbi::<T>()?,
                key.as_ref(),
                value,
                WriteFlags::UPSERT,
            )
            .map_err(|e| {
                DatabaseWriteError {
                    info: e.into(),
                    operation: DatabaseWriteOperation::Put,
                    table_name: T::NAME,
                    key: key.into(),
                }
                .into()
            })
    }

    fn delete<T: Table>(
        &self,
        key: T::Key,
        value: Option<T::Value>,
    ) -> Result<bool, DatabaseError> {
        let mut data = None;

        let value = value.map(Compress::compress);
        if let Some(value) = &value {
            data = Some(value.as_ref());
        };

        self.inner
            .del(self.get_dbi::<T>()?, key.encode(), data)
            .map_err(|e| DatabaseError::Delete(e.into()))
    }

    fn clear<T: Table>(&self) -> Result<(), DatabaseError> {
        self.inner
            .clear_db(self.get_dbi::<T>()?)
            .map_err(|e| DatabaseError::Delete(e.into()))?;

        Ok(())
    }

    fn cursor_write<T: Table>(&self) -> Result<Self::CursorMut<T>, DatabaseError> {
        self.new_cursor::<T>()
    }

    fn cursor_dup_write<T: DupSort>(&self) -> Result<Self::DupCursorMut<T>, DatabaseError> {
        self.new_cursor::<T>()
    }
}
