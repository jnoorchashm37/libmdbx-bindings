use std::{any::Any, fmt::Debug};

use alloy_rlp::{Decodable, Encodable};
use bytes::BufMut;
use reth_db::{
    DatabaseError,
    table::{Compress, Decode, Decompress, DupSort, Encode, Key, Table, Value},
};
use rkyv::{AlignedVec, Archive, ser::serializers::AllocSerializer};

use crate::{WrapCompress, WrapDecodable, WrapDecode, WrapDecompress, WrapEncodable, WrapEncode};

pub struct Wrapper<T>(T);

impl<T: Debug> Debug for Wrapper<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Wrapper<T> {
    pub fn wrap(val: T) -> Self {
        Self(val)
    }
}

impl<T: serde::Serialize> serde::Serialize for Wrapper<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for Wrapper<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Wrapper(T::deserialize(deserializer)?))
    }
}

impl<T> alloy_rlp::Encodable for Wrapper<T>
where
    T: WrapEncodable + rkyv::Serialize<AllocSerializer<256>> + Sized,
{
    fn encode(&self, out: &mut dyn BufMut) {
        self.0.encode_wrapped(out);
    }
}

impl<T> alloy_rlp::Decodable for Wrapper<T>
where
    T: WrapDecodable + Archive + Sized,
    <T as Archive>::Archived: rkyv::Deserialize<T, rkyv::Infallible>,
{
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        Ok(Wrapper(T::decode_wrapped(buf)?))
    }
}

impl<T> reth_db::table::Compress for Wrapper<T>
where
    T: WrapCompress + Sized + Send + Sync + Debug,
    <T as Archive>::Archived: rkyv::Deserialize<T, rkyv::Infallible>,
{
    type Compressed = Vec<u8>;

    fn compress_to_buf<B: bytes::BufMut + AsMut<[u8]>>(&self, buf: &mut B) {
        self.0.compress_to_buf_wrapped(buf)
    }
}

impl<T> reth_db::table::Decompress for Wrapper<T>
where
    T: WrapDecompress + WrapDecodable + Archive + Sized + Send + Sync + Debug,
    <T as Archive>::Archived: rkyv::Deserialize<T, rkyv::Infallible>,
{
    fn decompress(value: &[u8]) -> Result<Self, DatabaseError> {
        Ok(Wrapper(T::decompress_wrapped(value)?))
    }
}

#[macro_export]
macro_rules! table_value_codecs_with_zc {
    ($table_value:ident) => {
        impl libmdbx_bindings::WrapEncodable for $table_value {}
        // impl libmdbx_bindings::Encodable for $table_value {
        //     fn encode(&self, out: &mut dyn libmdbx_bindings::BufMut) {
        //         <$table_value as libmdbx_bindings::WrapEncodable>::encode_wrapped(self, out)
        //     }
        // }

        impl libmdbx_bindings::WrapDecodable for $table_value {}
        // impl libmdbx_bindings::Decodable for $table_value {
        //     fn decode(buf: &mut &[u8]) -> libmdbx_bindings::RlpResult<Self> {
        //         <$table_value as libmdbx_bindings::WrapDecodable>::decode_wrapped(buf)
        //     }
        // }

        impl libmdbx_bindings::WrapCompress for $table_value {
            type Compressed = Vec<u8>;
        }

        // impl libmdbx_bindings::Compress for $table_value {
        //     type Compressed = Vec<u8>;

        //     fn compress_to_buf<B: libmdbx_bindings::AlloyBytesMut + AsMut<[u8]>>(
        //         &self,
        //         buf: &mut B,
        //     ) {
        //         let mut encoded = Vec::new();
        //         libmdbx_bindings::Encodable::encode(&self, &mut encoded);
        //         let encoded_compressed = libmdbx_bindings::encode_all(&*encoded, 0).unwrap();

        //         buf.put_slice(&encoded_compressed);
        //     }
        // }

        impl libmdbx_bindings::WrapDecompress for $table_value {}
        // impl libmdbx_bindings::Decompress for $table_value {
        //     fn decompress(value: &[u8]) -> Result<Self, libmdbx_bindings::DatabaseError> {
        //         let binding = value.to_vec();

        //         let encoded_decompressed = libmdbx_bindings::decode_all(&*binding).unwrap();
        //         let buf = &mut encoded_decompressed.as_slice();

        //         libmdbx_bindings::Decodable::decode(buf)
        //             .map_err(|_| libmdbx_bindings::DatabaseError::Decode)
        //     }
        // }
    };

    ($table_value:ident, $wrapper_table_value:ident) => {
        table_value_codecs_with_zc!($wrapper_table_value);

        impl libmdbx_bindings::WrapEncodable for $table_value {}
        impl libmdbx_bindings::WrapDecodable for $table_value {}
        impl libmdbx_bindings::WrapCompress for $table_value {
            type Compressed = Vec<u8>;
        }
        impl libmdbx_bindings::WrapDecompress for $table_value {}
        // impl libmdbx_bindings::Encodable for $table_value {
        //     fn encode(&self, out: &mut dyn libmdbx_bindings::BufMut) {
        //         let this: $wrapper_table_value = self.clone().into();
        //         libmdbx_bindings::Encodable::encode(&this, out)
        //     }
        // }

        // impl libmdbx_bindings::Decodable for $table_value {
        //     fn decode(buf: &mut &[u8]) -> libmdbx_bindings::RlpResult<Self> {
        //         libmdbx_bindings::Decodable::decode(buf).map(|v: $wrapper_table_value| v.into())
        //     }
        // }

        // impl libmdbx_bindings::Compress for $table_value {
        //     type Compressed = Vec<u8>;

        //     fn compress_to_buf<B: libmdbx_bindings::AlloyBytesMut + AsMut<[u8]>>(
        //         self,
        //         buf: &mut B,
        //     ) {
        //         let this: $wrapper_table_value = self.into();
        //         libmdbx_bindings::Compress::compress_to_buf(this, buf)
        //     }
        // }

        // impl libmdbx_bindings::Decompress for $table_value {
        //     fn decompress(value: &[u8]) -> Result<Self, libmdbx_bindings::DatabaseError> {
        //         libmdbx_bindings::Decompress::decompress(value)
        //             .map(|v: $wrapper_table_value| v.into())
        //     }
        // }
    };
}

#[macro_export]
macro_rules! table_value_codecs_with_zc2 {
    ($table_value:ident) => {
        impl libmdbx_bindings::Encodable for $table_value {
            fn encode(&self, out: &mut dyn libmdbx_bindings::BufMut) {
                let encoded = rkyv::to_bytes::<_, 256>(self).unwrap();

                out.put_slice(&encoded)
            }
        }

        impl libmdbx_bindings::Decodable for $table_value {
            fn decode(buf: &mut &[u8]) -> libmdbx_bindings::RlpResult<Self> {
                let archived: &$crate::paste!([<Archived $table_value>]) =
                unsafe { libmdbx_bindings::archived_root::<Self>(&buf[..]) };


                let this = rkyv::Deserialize::deserialize(archived, &mut rkyv::Infallible).unwrap();

                Ok(this)
            }
        }

        impl libmdbx_bindings::Compress for $table_value {
            type Compressed = Vec<u8>;

            fn compress_to_buf<B: libmdbx_bindings::AlloyBytesMut + AsMut<[u8]>>(&self, buf: &mut B) {
                let mut encoded = Vec::new();
                libmdbx_bindings::Encodable::encode(&self, &mut encoded);
                let encoded_compressed = libmdbx_bindings::encode_all(&*encoded, 0).unwrap();

                buf.put_slice(&encoded_compressed);
            }
        }

        impl libmdbx_bindings::Decompress for $table_value {
            fn decompress(value: &[u8]) -> Result<Self, libmdbx_bindings::DatabaseError> {
                let binding = value.to_vec();

                let encoded_decompressed = libmdbx_bindings::decode_all(&*binding).unwrap();
                let buf = &mut encoded_decompressed.as_slice();

                libmdbx_bindings::Decodable::decode(buf).map_err(|_| libmdbx_bindings::DatabaseError::Decode)
            }
        }
    };


    ($table_value:ident, $wrapper_table_value:ident) => {
        table_value_codecs_with_zc!($wrapper_table_value);

        impl libmdbx_bindings::Encodable for $table_value {
            fn encode(&self, out: &mut dyn libmdbx_bindings::BufMut) {
                let this: $wrapper_table_value = self.clone().into();
                libmdbx_bindings::Encodable::encode(&this, out)
            }
        }

        impl libmdbx_bindings::Decodable for $table_value {
            fn decode(buf: &mut &[u8]) -> libmdbx_bindings::RlpResult<Self> {
                libmdbx_bindings::Decodable::decode(buf).map(|v: $wrapper_table_value| v.into())
            }
        }

        impl libmdbx_bindings::Compress for $table_value {
            type Compressed = Vec<u8>;

            fn compress_to_buf<B: libmdbx_bindings::AlloyBytesMut + AsMut<[u8]>>(self, buf: &mut B) {
                let this: $wrapper_table_value = self.into();
                libmdbx_bindings::Compress::compress_to_buf(this, buf)
            }
        }

        impl libmdbx_bindings::Decompress for $table_value {
            fn decompress(value: &[u8]) -> Result<Self, libmdbx_bindings::DatabaseError> {
                libmdbx_bindings::Decompress::decompress(value).map(|v: $wrapper_table_value| v.into())
            }
        }
    };

}

impl<T> Encode for Wrapper<T>
where
    T: WrapEncode + Sized + Debug + Send + Sync,
    <T as Archive>::Archived: rkyv::Deserialize<T, rkyv::Infallible>,
{
    type Encoded = Vec<u8>;
    fn encode(self) -> Self::Encoded {
        T::encode_key_wrapped(self.0)
    }
}

impl<T> Decode for Wrapper<T>
where
    T: WrapDecode + Sized + Debug + Send + Sync,
    <T as Archive>::Archived: rkyv::Deserialize<T, rkyv::Infallible>,
{
    fn decode(value: &[u8]) -> Result<Self, DatabaseError> {
        Ok(Wrapper(T::decode_wrapped_key(value)?))
    }
}

#[macro_export]
macro_rules! table_key_codecs_with_zc {
    ($table_value:ident) => {
        crate::table_value_codecs_with_zc!($table_value);

        impl libmdbx_bindings::WrapEncode for $table_value {}
        impl libmdbx_bindings::WrapDecode for $table_value {}

        // impl libmdbx_bindings::Encode for $table_value {
        //     type Encoded = Vec<u8>;

        //     fn encode(self) -> Self::Encoded {
        //         let mut buf = bytes::BytesMut::new();
        //         libmdbx_bindings::Encodable::encode(&self, &mut buf);
        //         buf.to_vec()
        //     }
        // }

        // impl libmdbx_bindings::Decode for $table_value {
        //     fn decode(mut value: &[u8]) -> Result<Self, libmdbx_bindings::DatabaseError> {
        //         libmdbx_bindings::Decodable::decode(&mut value)
        //             .map_err(|_| libmdbx_bindings::DatabaseError::Decode)
        //     }
        // }
    };
}

impl<T> Table for Wrapper<T>
where
    T: Table,
    Wrapper<<T as Table>::Key>: Key,
    Wrapper<<T as Table>::Value>: Value,
{
    const NAME: &'static str = T::NAME;

    const DUPSORT: bool = T::DUPSORT;

    type Key = Wrapper<T::Key>;

    type Value = Wrapper<T::Value>;
}

impl<T> DupSort for Wrapper<T>
where
    T: DupSort,
    Wrapper<T>: Table,
    Wrapper<<T as DupSort>::SubKey>: Key,
    Wrapper<<T as Table>::Key>: Key,
    Wrapper<<T as Table>::Value>: Value,
{
    type SubKey = Wrapper<T::SubKey>;
}
