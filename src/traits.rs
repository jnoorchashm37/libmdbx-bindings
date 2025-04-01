use rkyv::{Archive, ser::serializers::AllocSerializer};
use std::str::FromStr;

use bytes::BufMut;
use libmdbx_native::RW;
use reth_db::{DatabaseError, TableType};

use crate::implementation::LibmdbxTx;

pub trait TableSet: Send + Sync + Sized + FromStr<Err = String> {
    const NUM_TABLES: usize;

    fn create_tables(txn: &LibmdbxTx<RW>) -> Result<(), DatabaseError>;

    fn as_usize(&self) -> usize;
}

pub trait TableDet: reth_db::table::Table {
    fn table_type(&self) -> TableType;
}

pub trait WrapEncodable: rkyv::Serialize<AllocSerializer<256>> + Sized {
    fn encode_wrapped(&self, out: &mut dyn BufMut) {
        let encoded = rkyv::to_bytes(self).unwrap();

        out.put_slice(&encoded);
    }
}

pub trait WrapDecodable
where
    Self: Archive + Sized,
    <Self as Archive>::Archived: rkyv::Deserialize<Self, rkyv::Infallible>,
{
    fn decode_wrapped(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let archived = unsafe { rkyv::archived_root::<Self>(&buf[..]) };

        Ok(rkyv::Deserialize::<Self, _>::deserialize(archived, &mut rkyv::Infallible).unwrap())
    }
}

pub trait WrapCompress: WrapEncodable {
    type Compressed;

    fn compress_to_buf_wrapped<B: alloy_primitives::bytes::BufMut + AsMut<[u8]>>(
        &self,
        buf: &mut B,
    ) {
        let mut encoded = Vec::new();
        WrapEncodable::encode_wrapped(self, &mut encoded);
        let encoded_compressed = zstd::encode_all(&*encoded, 0).unwrap();

        buf.put_slice(&encoded_compressed);
    }
}

pub trait WrapDecompress: WrapDecodable
where
    Self: Archive + Sized,
    <Self as Archive>::Archived: rkyv::Deserialize<Self, rkyv::Infallible>,
{
    fn decompress_wrapped(value: &[u8]) -> Result<Self, DatabaseError> {
        let binding = value.to_vec();

        let encoded_decompressed = zstd::decode_all(&*binding).unwrap();
        let buf = &mut encoded_decompressed.as_slice();

        Self::decode_wrapped(buf).map_err(|_| DatabaseError::Decode)
    }
}

pub trait WrapEncode
where
    Self: WrapEncodable + Sized,
    <Self as Archive>::Archived: rkyv::Deserialize<Self, rkyv::Infallible>,
{
    fn encode_key_wrapped(self) -> Vec<u8> {
        let mut buf = bytes::BytesMut::new();
        WrapEncodable::encode_wrapped(&self, &mut buf);

        buf.to_vec()
    }
}

pub trait WrapDecode
where
    Self: WrapDecodable + Archive + Sized,
    <Self as Archive>::Archived: rkyv::Deserialize<Self, rkyv::Infallible>,
{
    fn decode_wrapped_key(mut value: &[u8]) -> Result<Self, DatabaseError> {
        WrapDecodable::decode_wrapped(&mut value).map_err(|_| DatabaseError::Decode)
    }
}
