use std::error::Error;
use std::collections::HashMap;
use std::io::{self, Write};
use bech32_addr_converter::converter::any_addr_to_prefix_addr;
use config::{Config, File};
use ibc_tokens_path_tracer::types::BalancesResponse;

fn main() -> Result<(), Box<dyn Error>> {
    // Read the config file
    let config = Config::builder()
        .add_source(File::with_name("config"))
        .build()
        .unwrap();

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

    // Load the balances for each chain based on the address_map 
    for (chain_name, address) in address_map.into_iter() {
        let chain_config = config.get_table(&format!("chains.{}", chain_name)).unwrap();
        let lcd_url = chain_config.get("lcd").unwrap().clone().into_string().unwrap();
        let _ = load_chain_balances(&lcd_url, &address)?;
    }

    Ok(())
}

fn load_chain_balances(lcd_url: &str, address: &str) -> Result<BalancesResponse, Box<dyn Error>> {
    let balances_path = "cosmos/bank/v1beta1/balances/";
    let url = format!("{}/{}/{}", lcd_url, balances_path, address);
    let response = reqwest::blocking::get(&url)?.json::<BalancesResponse>()?;
    println!("{:?}", response);
    Ok(response)
}
