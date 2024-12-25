use crate::packet::seder::deserializer::Deserialize;
use crate::packet::seder::serializer::Serialize;

pub mod deserializer;
pub mod serializer;

pub trait TryFromBytes: Sized {
    type Error;
    fn try_from_bytes(decoder: &mut Deserialize) -> Result<Self, Self::Error>;
}

pub trait ToBytes {
    fn to_bytes(&self, encoder: &mut Serialize);
}