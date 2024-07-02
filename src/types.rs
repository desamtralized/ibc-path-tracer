use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Balance {
    denom: String,
    amount: String,
}

#[derive(Debug, Deserialize)]
pub struct Pagination {
    next_key: Option<String>,
    total: String,
}

#[derive(Debug, Deserialize)]
pub struct BalancesResponse {
    balances: Vec<Balance>,
    pagination: Pagination,
}
