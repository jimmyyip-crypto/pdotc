use parity_scale_codec::Encode;
use sp_core::crypto::AccountId32;

use crate::client::{Api, ClientError, Result, Signer};
use crate::network::SubstrateNetwork;
use crate::rpc::RpcClient;
use crate::{public_into_account, Era, GenericExtra, SignedPayload, UncheckedExtrinsic};

pub mod balances;
pub mod proxy;
pub mod staking;
pub mod storage;
pub mod timestamp;

pub(crate) type CallIndex = [u8; 2];

impl<S: Signer, Client: RpcClient, N: SubstrateNetwork> Api<'_, S, Client, N> {
    /// Creates and signs an extrinsic that can be submitted to a node
    pub fn create_xt<C: Encode + Clone>(&self, call: C) -> Result<UncheckedExtrinsic<C>> {
        let signature = if let Some(signer) = &self.signer {
            let gen_hash = self.genesis_hash;
            let runtime_version = self.runtime_version;
            let extra = GenericExtra::new(Era::Immortal, self.nonce().unwrap_or_default());
            let s_extra = (
                runtime_version.spec_version,
                runtime_version.transaction_version,
                gen_hash,
                gen_hash,
                (),
                (),
                (),
            );
            let raw_payload = SignedPayload::new(call.clone(), extra, s_extra);

            let from = signer.public()?.into();
            let sig = raw_payload.encoded(|payload| signer.sign(payload))?;
            Some((from, sig, extra))
        } else {
            None
        };

        Ok(UncheckedExtrinsic {
            signature,
            function: call,
        })
    }

    pub fn signer_account(&self) -> Result<AccountId32> {
        match &self.signer {
            Some(signer) => Ok(public_into_account(signer.public()?)),
            None => Err(ClientError::NoSigner),
        }
    }

    pub fn nonce(&self) -> Result<u32> {
        let acct = self.signer_account()?;
        let info = self
            .account_info(acct, None)?
            .ok_or(ClientError::SignerAccountDoesNotExist)?;
        Ok(info.nonce)
    }
}
