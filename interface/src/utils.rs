use crate::error::OrcaError;

pub fn to_timestamp_u64(t: i64) -> Result<u64, OrcaError> {
    u64::try_from(t).or(Err(OrcaError::InvalidTimestampConversion.into()))
}
