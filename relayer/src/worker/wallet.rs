use std::{ops::Div, time::Duration};
use tracing::{error_span, trace};
use ibc::bigint::U256;
use crate::{
    chain::handle::ChainHandle, telemetry,
    util::task::{spawn_background_task, Next, TaskError, TaskHandle},
};
#[prusti_contracts::trusted]
pub fn spawn_wallet_worker<Chain: ChainHandle>(chain: Chain) -> TaskHandle {
    let span = error_span!("wallet", chain = % chain.id());
    spawn_background_task(
        span,
        Some(Duration::from_secs(5)),
        move || {
            let key = chain
                .get_key()
                .map_err(|e| {
                    TaskError::Fatal(
                        format!("failed to get key in use by the relayer: {e}"),
                    )
                })?;
            let balance = chain
                .query_balance(None)
                .map_err(|e| {
                    TaskError::Ignore(
                        format!("failed to query balance for the account: {e}"),
                    )
                })?;
            let amount: U256 = U256::from_dec_str(&balance.amount)
                .map_err(|_| {
                    TaskError::Ignore(
                        format!("failed to parse amount into U256: {}", balance.amount),
                    )
                })?;
            trace!(
                % amount, denom = % balance.denom, account = % key.account,
                "wallet balance"
            );
            if let Some(_scaled_amount) = scale_down(amount) {
                telemetry!(
                    wallet_balance, & chain.id(), & key.account, _scaled_amount, &
                    balance.denom,
                );
            } else {
                trace!(
                    % amount, denom = % balance.denom, account = % key.account,
                    "amount cannot be scaled down to fit into u64 and therefore won't be reported to telemetry"
                );
            }
            Ok(Next::Continue)
        },
    )
}
/// Scale down the given amount by a factor of 10^6,
/// and return it as a `u64` if it fits.
#[prusti_contracts::trusted]
fn scale_down(amount: U256) -> Option<u64> {
    amount.div(10_u64.pow(6)).try_into().ok()
}
#[cfg(test)]
mod tests {
    use super::scale_down;
    use ibc::bigint::U256;
    #[test]
    #[prusti_contracts::trusted]
    fn example_input() {
        let u: U256 = U256::from_dec_str("349999631379421794336")
            .expect("failed to parse into U256");
        let s = scale_down(u);
        assert_eq!(s, Some(349999631379421_u64));
    }
}

