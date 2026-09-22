mod budget_guard;
mod infrai_client;

use budget_guard::{decide, Decision};
use infrai_client::InfraiClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let request_usd = std::env::var("REQUEST_USD").ok().and_then(|v| v.parse().ok()).unwrap_or(0.25);
    let spent_usd = std::env::var("SPENT_USD").ok().and_then(|v| v.parse().ok()).unwrap_or(0.0);
    let cap_usd = std::env::var("HARD_CAP_USD").ok().and_then(|v| v.parse().ok()).unwrap_or(10.0);
    if decide(spent_usd, request_usd, cap_usd) == Decision::Deny { println!("request denied before inference: cap would be exceeded"); return Ok(()); }
    let api = InfraiClient::from_env()?;
    api.set_budget(cap_usd, "monthly", cap_usd * 0.8).await?;
    let usage = api.usage().await?;
    let result = api.infer("auto", "Summarize the latest build event and suggest the next release check.").await?;
    println!("usage={usage}\n inference={result}");
    Ok(())
}
