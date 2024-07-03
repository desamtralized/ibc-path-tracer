use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Balance {
    pub denom: String,
    pub amount: String,
}

#[derive(Debug, Deserialize)]
pub struct Pagination {
    pub next_key: Option<String>,
    pub total: String,
}

#[derive(Debug, Deserialize)]
pub struct BalancesResponse {
    pub balances: Vec<Balance>,
    pub pagination: Pagination,
}
#[derive(Debug, Deserialize)]
pub struct DenomTraceResponse {
    pub denom_trace: DenomTrace,
}

#[derive(Debug, Deserialize)]
pub struct DenomTrace {
    pub path: String,
    pub base_denom: String,
}
