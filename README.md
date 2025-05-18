
## Safety and Robustness
> Are you doing something dangerous? Tell us why you chose to do it this way. How are you handling errors?


I was going to write some tests like:

```rust
#[cfg(test)]
pub mod tests {
	use super::*;

	#[test]
	fn can_parse_decimals() {
		assert!(Decimal::from_f64(-1.00000000000000000001).is_some());
		assert!(Decimal::from_f64(0.99999999999999).is_some());
		assert!(Decimal::from_f64(f64::EPSILON).is_some());
		assert!(Decimal::from_f64(f64::NAN).is_none());
	}

	#[test]
	fn can_round_decimals() {
		assert_eq!(dec!(1.1000), Decimal::from_f64(1.1).unwrap().round_dp(4));
		assert_eq!(dec!(-1), Decimal::from_f64(-1.00000000000000000099).unwrap().round_dp(4));
		assert_eq!(dec!(1), Decimal::from_f64(0.99999999999999).unwrap().round_dp(4));
	}
}
```

But like. If you care about that - pull rust-csv as a dependency, and run it's tests.
There's no need to rewrite their tests but worse.
