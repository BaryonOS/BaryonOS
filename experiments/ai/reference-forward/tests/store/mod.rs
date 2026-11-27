//! In-memory artifact store shared by the opt-in oracle tests.

use artifact_store_core::{
    ArtifactDescriptor, ArtifactRef, ArtifactStore, Error as StoreError, StoreCapacity,
};
use nieuw_os_model_store_contract::Cancellation;

pub struct Never;
impl Cancellation for Never {
    fn is_cancelled(&self) -> bool {
        false
    }
}

pub struct Memory(pub Vec<u8>, pub ArtifactRef);
impl ArtifactStore for Memory {
    type Handle = ArtifactRef;
    fn open(
        &mut self,
        d: &ArtifactDescriptor,
    ) -> Result<(Self::Handle, ArtifactDescriptor), StoreError> {
        Ok((
            self.1,
            ArtifactDescriptor {
                length: self.0.len() as u64,
                ..*d
            },
        ))
    }
    fn read_at(
        &mut self,
        _: &ArtifactRef,
        offset: u64,
        out: &mut [u8],
    ) -> Result<usize, StoreError> {
        let start = offset as usize;
        let n = out.len().min(self.0.len() - start);
        out[..n].copy_from_slice(&self.0[start..start + n]);
        Ok(n)
    }
    fn capacity(&mut self) -> Result<StoreCapacity, StoreError> {
        Err(StoreError::Io)
    }
}
