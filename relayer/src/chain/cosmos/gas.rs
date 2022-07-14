use core::cmp::min;
use core::fmt;
use ibc_proto::cosmos::base::v1beta1::Coin;
use ibc_proto::cosmos::tx::v1beta1::Fee;
use num_bigint::BigInt;
use num_rational::BigRational;
use crate::chain::cosmos::types::gas::GasConfig;
use crate::config::GasPrice;
pub struct PrettyFee<'a>(pub &'a Fee);
#[prusti_contracts::trusted]
pub fn gas_amount_to_fee(config: &GasConfig, gas_amount: u64) -> Fee {
    let adjusted_gas_limit = adjust_estimated_gas(AdjustGas {
        gas_multiplier: config.gas_multiplier,
        max_gas: config.max_gas,
        gas_amount,
    });
    let amount = calculate_fee(adjusted_gas_limit, &config.gas_price);
    Fee {
        amount: vec![amount],
        gas_limit: adjusted_gas_limit,
        payer: "".to_string(),
        granter: config.fee_granter.clone(),
    }
}
#[prusti_contracts::trusted]
pub fn calculate_fee(adjusted_gas_amount: u64, gas_price: &GasPrice) -> Coin {
    let fee_amount = mul_ceil(adjusted_gas_amount, gas_price.price);
    Coin {
        denom: gas_price.denom.to_string(),
        amount: fee_amount.to_string(),
    }
}
/// Multiply `a` with `f` and round the result up to the nearest integer.
#[prusti_contracts::trusted]
pub fn mul_ceil(a: u64, f: f64) -> BigInt {
    assert!(f.is_finite());
    let a = BigInt::from(a);
    let f = BigRational::from_float(f).expect("f is finite");
    (f * a).ceil().to_integer()
}
/// Multiply `a` with `f` and round the result down to the nearest integer.
#[prusti_contracts::trusted]
pub fn mul_floor(a: u64, f: f64) -> BigInt {
    assert!(f.is_finite());
    let a = BigInt::from(a);
    let f = BigRational::from_float(f).expect("f is finite");
    (f * a).floor().to_integer()
}
struct AdjustGas {
    gas_multiplier: f64,
    max_gas: u64,
    gas_amount: u64,
}
/// Adjusts the fee based on the configured `gas_multiplier` to prevent out of gas errors.
/// The actual gas cost, when a transaction is executed, may be slightly higher than the
/// one returned by the simulation.
#[prusti_contracts::trusted]
fn adjust_estimated_gas(
    AdjustGas { gas_multiplier, max_gas, gas_amount }: AdjustGas,
) -> u64 {
    assert!(gas_multiplier >= 1.0);
    if gas_amount == 0 {
        return 0;
    }
    if gas_multiplier == 1.0 {
        return min(gas_amount, max_gas);
    }
    let (_sign, digits) = mul_floor(gas_amount, gas_multiplier).to_u64_digits();
    let gas = match digits.as_slice() {
        [] => 0,
        [gas] => *gas,
        _ => u64::MAX,
    };
    min(gas, max_gas)
}
impl fmt::Display for PrettyFee<'_> {
    #[prusti_contracts::trusted]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let amount = match self.0.amount.get(0) {
            Some(coin) => format!("{}{}", coin.amount, coin.denom),
            None => "<no amount specified>".to_string(),
        };
        f.debug_struct("Fee")
            .field("amount", &amount)
            .field("gas_limit", &self.0.gas_limit)
            .finish()
    }
}
#[cfg(test)]
mod tests {
    use super::{adjust_estimated_gas, AdjustGas};
    #[test]
    #[prusti_contracts::trusted]
    fn adjust_zero_gas() {
        let adjusted_gas = adjust_estimated_gas(AdjustGas {
            gas_multiplier: 1.1,
            max_gas: 1_000_000,
            gas_amount: 0,
        });
        assert_eq!(adjusted_gas, 0);
    }
    #[test]
    #[prusti_contracts::trusted]
    fn adjust_gas_one() {
        let adjusted_gas = adjust_estimated_gas(AdjustGas {
            gas_multiplier: 1.0,
            max_gas: 1_000_000,
            gas_amount: 400_000,
        });
        assert_eq!(adjusted_gas, 400_000);
    }
    #[test]
    #[prusti_contracts::trusted]
    fn adjust_gas_small() {
        let adjusted_gas = adjust_estimated_gas(AdjustGas {
            gas_multiplier: 1.1,
            max_gas: 1_000_000,
            gas_amount: 400_000,
        });
        assert_eq!(adjusted_gas, 440_000);
    }
    #[test]
    #[prusti_contracts::trusted]
    fn adjust_gas_over_max() {
        let adjusted_gas = adjust_estimated_gas(AdjustGas {
            gas_multiplier: 3.0,
            max_gas: 1_000_000,
            gas_amount: 400_000,
        });
        assert_eq!(adjusted_gas, 1_000_000);
    }
    #[test]
    #[prusti_contracts::trusted]
    fn adjust_gas_overflow() {
        let adjusted_gas = adjust_estimated_gas(AdjustGas {
            gas_multiplier: 3.0,
            max_gas: u64::MAX,
            gas_amount: u64::MAX / 2,
        });
        assert_eq!(adjusted_gas, u64::MAX);
    }
}

