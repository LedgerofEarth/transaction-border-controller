use crate::signing::tbc_signer::TbcSigner;
use crate::validation::{merchant::verify_merchant, policy_hash::verify_policy_hash};

use tbc_core::{
    policy::compute_policy_hash,
    enforcement::EnforcementContext,
    state::TbcStorage,
    types::{TbcResponse, TbcDecision, tbc_rejection::TbcRejectionReason},
};

use chrono::Utc;
use tgp::messages::TgpQuery;

pub struct TgpQueryHandler<'a> {
    pub storage: &'a mut TbcStorage,
    pub signer: &'a TbcSigner,
}

impl<'a> TgpQueryHandler<'a> {
    pub fn handle(&mut self, query: TgpQuery) -> TbcResponse {

        // 1. Merchant signature validation
        let merchant_ok = verify_merchant(
            &self.storage.merchants,
            &query.merchant_id,
            &query.merchant_pubkey,
            &query.merchant_signature,
            &query.payload_json,
        );

        if !merchant_ok {
            return TbcResponse::rejected(TbcRejectionReason::VendorMismatch);
        }

        // 2. Load Policy
        let Some(policy) = self.storage.sessions.get(&query.session_key_id) else {
            return TbcResponse::rejected(TbcRejectionReason::SessionKeyExpired);
        };

        // 3. Policy hash match
        if !verify_policy_hash(
            &compute_policy_hash(policy),
            &query.policy_hash_from_coreprove,
        ) {
            return TbcResponse::rejected(TbcRejectionReason::PolicyHashMismatch);
        }

        // 4. Enforcement engine
        let mut ctx = EnforcementContext {
            policy,
            reservations: &mut self.storage.reservations,
            idempotency: &mut self.storage.idempotency,
            now: Utc::now(),
        };

        let decision = ctx.evaluate(
            query.amount,
            query.chain,
            query.function_selector.clone(),
            Some(query.policy_hash_from_coreprove.clone()),
            query.merchant_policy_hash.clone(),
            &query.idempotency_key,
        );

        // 5. Sign response
        match decision {
            TbcDecision::Approve => {
                let hash = compute_policy_hash(policy);
                let sig = self.signer.sign(&hash);
                let anomaly_score = 0;

                TbcResponse::approved(hash, sig, anomaly_score)
            }
            TbcDecision::Reject(reason) => {
                TbcResponse::rejected(reason)
            }
        }
    }
}