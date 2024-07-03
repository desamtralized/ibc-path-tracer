use bech32_addr_converter::converter::any_addr_to_prefix_addr;
use config::{Config, File};
use ibc_tokens_path_tracer::types::{BalancesResponse, DenomTraceResponse};
use std::collections::HashMap;
use std::error::Error;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn Error>> {
    let config = &get_config()?;
    // Prompt the user to enter the Neutron address
    print!("Please enter Neutron address: ");
    io::stdout().flush().unwrap();
    let mut neutron_address = String::new();
    io::stdin().read_line(&mut neutron_address).unwrap();
    let neutron_address = neutron_address.trim().to_string();

    // Create a map to store the chain ID and respective addresses
    let mut address_map: HashMap<String, String> = HashMap::new();

    // Iterate over the chains and derive addresses
    let chains = &config.get_table("chains")?;
    chains.into_iter().for_each(|(chain_key, chain_config)| {
        let chain_key = chain_key.clone();
        let chain_config = chain_config.clone().into_table().unwrap();
        let chain_name = chain_config.get("name").unwrap().clone().into_string().unwrap();
        let chain_prefix = chain_config.get("prefix").unwrap().clone().into_string().unwrap();
        if chain_prefix == "neutron" {
            address_map.insert(chain_key, neutron_address.clone());
        } else {
            let derived_address = any_addr_to_prefix_addr(neutron_address.clone(), &chain_prefix).unwrap();
            print!("Please enter the address for chain {} or hit enter to use the derived address ({}): ", chain_name, derived_address);
            io::stdout().flush().unwrap();
            let mut chain_address = String::new();
            io::stdin().read_line(&mut chain_address).unwrap();
            let chain_address = chain_address.trim().to_string();
            if chain_address.is_empty() {
                address_map.insert(chain_key, derived_address);
            } else {
                let chain_prefix = chain_prefix.to_string();
                let new_address = any_addr_to_prefix_addr(chain_address.clone(), &chain_prefix).unwrap();
                address_map.insert(chain_key, new_address);
            }
        }
    });

    println!("");

    // Load the balances for each chain based on the address_map
    for (chain_name, address) in address_map.into_iter() {
        let chain_config: HashMap<String, config::Value> =
            config.get_table(&format!("chains.{}", chain_name)).unwrap();
        let result = load_chain_balances(&chain_config, &address)?;
        if result.balances.len() == 0 {
            println!("No balances found for chain {}", chain_name);
        }
    }

    Ok(())
}

// Load the balances for a given address on a chain
fn load_chain_balances(
    chain_config: &HashMap<String, config::Value>,
    address: &str,
) -> Result<BalancesResponse, Box<dyn Error>> {
    let balances_path = "cosmos/bank/v1beta1/balances/";
    let lcd_url = chain_config
        .get("lcd")
        .unwrap()
        .clone()
        .into_string()
        .unwrap();
    let url = format!("{}/{}/{}", lcd_url, balances_path, address);
    let response = reqwest::blocking::get(&url)?.json::<BalancesResponse>()?;
    let mut ibc_denoms: Vec<String> = Vec::new();
    response.balances.iter().for_each(|balance| {
        if balance.denom.starts_with("ibc/") {
            ibc_denoms.push(balance.denom.clone());
        }
    });
    let _ = trace_denoms_path(ibc_denoms, chain_config);
    Ok(response)
}

fn get_config() -> Result<Config, Box<dyn Error>> {
    let config = Config::builder()
        .add_source(File::with_name("config"))
        .build()
        .unwrap();
    Ok(config)
}

fn trace_denoms_path(
    denoms: Vec<String>,
    chain_config: &HashMap<String, config::Value>,
) -> Result<(), Box<dyn Error>> {
    let trace_path = "ibc/apps/transfer/v1/denom_traces/";
    let chain_name = chain_config
        .get("name")
        .unwrap()
        .clone()
        .into_string()
        .unwrap();
    let lcd_url = chain_config
        .get("lcd")
        .unwrap()
        .clone()
        .into_string()
        .unwrap();
    let config = &get_config()?;
    let allowed_denoms: Vec<String> = config
        .get_array("denoms")
        .unwrap()
        .iter()
        .map(|value| value.clone().into_string().unwrap())
        .collect();

    let mut chain_name_printed = false;
    denoms.iter().for_each(|denom| {
        let ibc_hash = denom.split("/").last().unwrap();
        let url = format!("{}/{}/{}", lcd_url, trace_path, ibc_hash);
        let response = reqwest::blocking::get(&url)
            .unwrap()
            .json::<DenomTraceResponse>()
            .unwrap();
        let base_denom = &response.denom_trace.base_denom;
        if allowed_denoms.contains(base_denom) {
            let path = get_route_array_by_path(&response.denom_trace.path, chain_config);
            if chain_name_printed == false {
                println!("{}", chain_name);
                chain_name_printed = true;
            }
            println!("{}, {}, {:?}", denom, base_denom, path);
        }
    });
    Ok(())
}

/// Path is a string with format transfer/channel-25/transfer/channel-1/transfer/channel-874,
/// we should extract the channels in the reverse order and match with the value from the config file to return the
/// chain name
fn get_route_array_by_path(
    path: &str,
    chain_config: &HashMap<String, config::Value>,
) -> Vec<String> {
    let mut route_array: Vec<String> = Vec::new();
    let channels: Vec<&str> = path.split("/").collect();
    let channels = channels
        .iter()
        .filter(|&channel| channel.starts_with("channel-"))
        .collect::<Vec<_>>();
    let channels: Vec<String> = channels
        .iter()
        .map(|&channel| channel.to_string())
        .collect();
    let config = &get_config().unwrap();
    let mut chain_id = chain_config
        .get("chain_id")
        .unwrap()
        .clone()
        .into_string()
        .unwrap();
    let paths = config.get_table("paths").unwrap();
    let source_chain_id = config.get_string("denoms_source").unwrap();
    route_array.push(source_chain_id.clone());

    // Iterate over the paths.chain-id and search for the channel-id that matches the chain-id
    for channel in channels.iter().rev() {
        for (path_chain_id, path) in paths.iter() {
            let path_table = path.clone().into_table().unwrap();
            for (key, value) in path_table.iter() {
                let value = value.clone().into_string().unwrap();
                if channel.eq(&value) && key.eq(&chain_id) {
                    chain_id = path_chain_id.clone();
                    route_array.push(path_chain_id.clone());
                }
            }
        }
    }

    route_array
}
