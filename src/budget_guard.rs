#[derive(Debug, PartialEq)]
pub enum Decision { Allow, Deny }

pub fn decide(spent_usd: f64, request_usd: f64, hard_cap_usd: f64) -> Decision {
    if spent_usd + request_usd <= hard_cap_usd { Decision::Allow } else { Decision::Deny }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn denies_the_request_that_would_cross_the_cap() {
        assert_eq!(decide(9.50, 0.60, 10.00), Decision::Deny);
        assert_eq!(decide(9.50, 0.50, 10.00), Decision::Allow);
    }
}
