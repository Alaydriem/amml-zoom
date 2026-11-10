// what keeping the Camera lease in line with the zoom did this tick
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaseSync {
    Unchanged,
    Acquired,
    Released,
    // the manager denied the lease or the request failed
    Refused,
}
