use std::error::Error;
use std::collections::HashMap;
use std::io::{self, Write};
use bech32_addr_converter::converter::any_addr_to_prefix_addr;
use config::{Config, File};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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
    let chains = config.get_table("chains")?;
    chains.into_iter().for_each(|(_, chain_config)| {
        let chain_config = chain_config.into_table().unwrap();
        let chain_id = chain_config.get("chain_id").unwrap().clone().into_string().unwrap();
        let chain_name = chain_config.get("name").unwrap().clone().into_string().unwrap();
        let chain_prefix = chain_config.get("prefix").unwrap().clone().into_string().unwrap();
        if chain_prefix == "neutron" {
            address_map.insert(chain_id, neutron_address.clone());
        } else {
            let derived_address = any_addr_to_prefix_addr(neutron_address.clone(), &chain_prefix).unwrap();
            print!("Please enter the address for chain {} or hit enter to use the derived address ({}): ", chain_name, derived_address);
            io::stdout().flush().unwrap();
            let mut chain_address = String::new();
            io::stdin().read_line(&mut chain_address).unwrap();
            let chain_address = chain_address.trim().to_string();
            if chain_address.is_empty() {
                address_map.insert(chain_id, derived_address);
            } else {
                let chain_prefix = chain_prefix.to_string();
                let new_address = any_addr_to_prefix_addr(chain_address.clone(), &chain_prefix).unwrap();
                address_map.insert(chain_id, new_address);
            }
        }
    });

    address_map.into_iter().for_each(|(chain_id, address)| {
        println!("{}: {}", chain_id, address);
    });

    Ok(())
}
