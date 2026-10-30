//! Native acceptance predicate, also exercised by actual smoltcp model
//! conversations. HTTP bytes and EOF alone do not prove the final ACK.
pub struct HttpEvidence {
    pub established: bool,
    pub eof: bool,
    pub sent: usize,
    pub received: usize,
    pub elapsed_ticks: u64,
}

pub fn http_complete(
    evidence: HttpEvidence,
    request_len: usize,
    response_len: usize,
    timeout: u64,
) -> bool {
    evidence.established
        && evidence.eof
        && evidence.sent == request_len
        && evidence.received == response_len
        && evidence.elapsed_ticks < timeout
}
