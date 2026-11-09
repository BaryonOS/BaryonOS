//! Pure terminal transition used by the NIC broker on every device-wide stop.
//! A reset can abandon a published TX, but cannot leave its ticket pending.
pub(crate) type Binding = (u64, u64, u64); // PID, object generation key, ticket
pub(crate) type Completion = (Binding, Result<usize, i64>);

pub(crate) fn terminalize(
    pending: &mut Option<Binding>,
    completed: &mut Option<Completion>,
    code: i64,
) {
    if let Some(binding) = pending.take() {
        *completed = Some((binding, Err(code)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rx_failure_during_published_tx_ends_the_bound_ticket_without_replay() {
        let binding = (17, 0x10000002, 9);
        let mut pending = Some(binding);
        let mut completed = None;
        terminalize(&mut pending, &mut completed, -15);
        assert_eq!(pending, None);
        assert_eq!(completed, Some((binding, Err(-15))));
        // A later offline/reset indication neither resurrects the request nor
        // replaces its first terminal cause with an unrelated caller result.
        terminalize(&mut pending, &mut completed, -14);
        assert_eq!(completed, Some((binding, Err(-15))));
    }
    #[test]
    fn later_rx_failure_preserves_an_already_observed_tx_completion() {
        let binding = (17, 0x10000002, 9);
        let mut pending = None;
        let mut completed = Some((binding, Ok(60)));
        terminalize(&mut pending, &mut completed, -15);
        assert_eq!(completed, Some((binding, Ok(60))));
        assert_eq!(pending, None);
    }
}
