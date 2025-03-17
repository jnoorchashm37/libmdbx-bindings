// #![allow(non_camel_case_types)]
#![allow(private_bounds)]

use std::{ffi::c_int, path::Path};

use eyre::Context;
use libmdbx_native::{RO, RW};
use reth_db::{
    ClientVersion, DatabaseError, is_database_empty,
    transaction::DbTx,
    version::{DatabaseVersionError, check_db_version_file, create_db_version_file},
};

use crate::{
    implementation::{DatabaseArguments, DatabaseEnv, DatabaseEnvKind, tx::LibmdbxTx},
    traits::TableSet,
    // tables::Tables,
};

const GIGABYTE: u64 = 1024 * 1024 * 1024;

#[derive(Debug)]
pub struct LibmdbxProvider(DatabaseEnv);

#[inline]
pub(crate) fn mdbx_result(err_code: c_int) -> eyre::Result<bool> {
    match err_code {
        reth_mdbx_sys::MDBX_SUCCESS => Ok(false),
        reth_mdbx_sys::MDBX_RESULT_TRUE => Ok(true),
        _ => Err(eyre::eyre!("shit no good")),
    }
}

impl LibmdbxProvider {
    /// Opens up an existing database or creates a new one at the specified
    /// path. Creates tables if necessary. Opens in read/write mode.
    pub fn init_db<P: AsRef<Path>, S: TableSet>(path: P) -> eyre::Result<Self> {
        let rpath = path.as_ref();
        if is_database_empty(rpath) {
            std::fs::create_dir_all(rpath).wrap_err_with(|| {
                format!("Could not create database directory {}", rpath.display())
            })?;
        } else {
            match check_db_version_file(rpath) {
                Ok(_) => (),
                Err(DatabaseVersionError::MissingFile) => create_db_version_file(rpath)?,
                Err(err) => return Err(err.into()),
            }
        }

        let db = DatabaseEnv::open(
            rpath,
            DatabaseEnvKind::RW,
            DatabaseArguments::new(ClientVersion::default()).with_log_level(None),
        )?;

        db.with_raw_env_ptr(|ptr| unsafe {
            mdbx_result(reth_mdbx_sys::mdbx_env_set_option(
                ptr,
                reth_mdbx_sys::MDBX_opt_sync_bytes,
                // 2 gb
                GIGABYTE * 2,
            ))
        })?;

        let this = Self(db);
        this.create_tables::<S>()?;

        Ok(this)
    }

    /// Creates all the defined tables, opens if already created
    fn create_tables<S: TableSet>(&self) -> Result<(), DatabaseError> {
        let tx = LibmdbxTx::new_rw_tx(&self.0)?;
        S::create_tables(&tx)?;

        tx.commit()?;

        Ok(())
    }

    /// Takes a function and passes a RW transaction
    /// makes sure it's committed at the end of execution
    pub fn write<F, R, S: TableSet>(&self, f: F) -> Result<R, DatabaseError>
    where
        F: FnOnce(&LibmdbxTx<RW, S>) -> R,
    {
        let tx = self.rw_tx()?;
        let res = f(&tx);
        tx.commit()?;

        Ok(res)
    }

    pub fn read<F, R, S: TableSet>(&self, f: F) -> Result<R, DatabaseError>
    where
        F: FnOnce(&LibmdbxTx<RO, S>) -> R,
    {
        let tx = self.ro_tx()?;
        let res = f(&tx);
        tx.commit()?;

        Ok(res)
    }

    /// returns a RO transaction
    fn ro_tx<S: TableSet>(&self) -> Result<LibmdbxTx<RO, S>, DatabaseError> {
        let tx = LibmdbxTx::new_ro_tx(&self.0)?;

        Ok(tx)
    }

    /// returns a RW transaction
    fn rw_tx<S: TableSet>(&self) -> Result<LibmdbxTx<RW, S>, DatabaseError> {
        let tx = LibmdbxTx::new_rw_tx(&self.0)?;

        Ok(tx)
    }
}
