use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlatformFee {
    pub amount: u64,
    pub fee_bps: u8,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SwapInfo {
    amm_key: String,
    label: String,
    input_mint: String,
    output_mint: String,
    in_amount: String,
    out_amount: String,
    fee_amount: String,
    fee_mint: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RoutePlan {
    swap_info: SwapInfo,
    percent: u8,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    input_mint: String,
    in_amount: String,
    output_mint: String,
    out_amount: String,
    other_amount_threshold: String,
    swap_mode: String,
    slippage_bps: u64,
    platform_fee: Option<PlatformFee>,
    price_impact_pct: String,
    route_plan: Vec<RoutePlan>,
    context_slot: u64,
    time_taken: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PriorityLevelWithMaxLamports {
    max_lamports: u64,
    global: bool,
    priority_level: String,
}

impl Default for PriorityLevelWithMaxLamports {
    fn default() -> Self {
        Self {
            max_lamports: 4000000,
            priority_level: "medium".to_string(),
            global: false,
        }
    }
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PrioritizationFeeLamports {
    priority_level_with_max_lamports: PriorityLevelWithMaxLamports,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicSlippage {
    max_bps: u8,
}

impl Default for DynamicSlippage {
    fn default() -> Self {
        Self { max_bps: 50 }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapBody {
    user_public_key: String,
    wrap_and_unwrap_sol: bool,
    prioritization_fee_lamports: PrioritizationFeeLamports,
    as_legacy_transaction: bool,
    dynamic_compute_unit_limit: bool,
    allow_optimized_wrapped_sol_token_account: bool,
    quote_response: QuoteResponse,
    dynamic_slippage: DynamicSlippage,
    correct_last_valid_block_height: bool,
}

impl SwapBody {
    pub fn new(user_public_key: &Pubkey, quote_response: QuoteResponse) -> Self {
        Self {
            user_public_key: user_public_key.to_string(),
            wrap_and_unwrap_sol: true,
            prioritization_fee_lamports: Default::default(),
            as_legacy_transaction: Default::default(),
            dynamic_compute_unit_limit: true,
            allow_optimized_wrapped_sol_token_account: true,
            quote_response,
            dynamic_slippage: Default::default(),
            correct_last_valid_block_height: true,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapResponse {
    pub swap_transaction: String,
}
