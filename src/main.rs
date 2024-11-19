use std::collections::HashMap;
use std::fmt::format;
use std::fs;
use std::path::Path;
use reqwest;
use serde_json::Value;
use csv;
use clap::{App, Arg};
use anyhow::{Result, anyhow};
use lazy_static::lazy_static;
use tokio;

lazy_static! {
    static ref CHAINS: HashMap<&'static str, u32> = {
        let mut m = HashMap::new();
        m.insert("eth", 1);
        m.insert("bsc", 56);
        m.insert("ftm", 250);
        m.insert("pg", 137);
        m.insert("avax", 43114);
        m.insert("arb", 42161);
        m.insert("op", 10);
        m.insert("sepolia", 11155111);
        m.insert("base", 8453);
        m.insert("moonbeam", 1284);
        m.insert("moonriver", 1285);
        m.insert("cro", 25);
        m.insert("merlin", 4200);
        m.insert("bitlayer", 200901);
        m.insert("mode", 34443);
        m.insert("scroll", 534352);
        m.insert("core", 1116);
        m.insert("linea", 59144);
        m.insert("ailayer", 2649);
        m
    };
}


/// 从 ailayer explorer 获取智能合约源代码
async fn get_code_from_ailayer(basepath: &str, address: &str) -> Result<()> {
    let api_url = format!("https://mainnet-explorer.ailayer.xyz/api/v2/smart-contracts/{}", address);
    let resp: Value = reqwest::get(&api_url).await?.json().await?;

    let main_sol = resp["source_code"].as_str()
        .ok_or_else(|| anyhow!("Source code is missing or not a string"))?;
    let main_path = format!("{}/{}", address,
                            resp["file_path"].as_str().unwrap_or("Error: No main file path"));
    save_code(basepath, "ailayer", &main_path, main_sol)?;
    if let Some(additional_sources) = resp["additional_sources"].as_array() {
        for source in additional_sources {
            let code = source["source_code"].as_str()
                .unwrap_or("Error: No additional_sources code");
            let path = format!("{}/{}", address, source["file_path"].as_str()
                .unwrap_or("Error: No additional_sources file path"));
            save_code(basepath, "ailayer", &path, code)?;
        }
    }
    Ok(())
}


/// 从 linea scan 获取智能合约源代码
async fn get_code_from_linea(basepath: &str, address: &str) -> Result<()> {
    let api_url =
        format!("https://api.lineascan.build/api?module=contract&action=getsourcecode&address={}&apikey={yourApiKey}",
                address);
    let resp: Value = reqwest::get(&api_url).await?.json().await?;
    if resp["status"] == "1" {
        let result = resp["result"].as_array().ok_or(anyhow!("Result is not an array"))?;
        for item in result {
            let formatted_code = item["SourceCode"].as_str()
                .unwrap_or("")
                .strip_prefix("{")
                .unwrap_or("")
                .strip_suffix("}")
                .unwrap_or("");
            let outer_dict: Value = serde_json::from_str(&formatted_code)?;
            if let Some(sources) = outer_dict["sources"].as_object() {
                for (p, c) in sources {
                    let path = format!("{}/{}", address, p);
                    let code = c["content"].as_str().unwrap_or("Error: No content");
                    save_code(basepath, "linea", &path, code)?;
                }
            }
        }
    } else {
        println!("linea scan status error {}", address);
    }
    Ok(())
}


/// 从 core scan 获取智能合约源代码
async fn get_code_from_core(basepath: &str, address: &str) -> Result<()> {
    let api_url =
        format!("https://openapi.coredao.org/api?module=contract&action=getsourcecode&address={}&apikey={yourApiKey}",
                address);
    let resp: Value = reqwest::get(&api_url).await?.json().await?;
    if resp["status"] == "1" {
        let result = resp["result"].as_array().ok_or(anyhow!("Result is not an array"))?;
        for item in result {
            let formatted_code = item["SourceCode"].as_str()
                .unwrap_or("")
                .strip_prefix("{")
                .unwrap_or("")
                .strip_suffix("}")
                .unwrap_or("");
            let outer_dict: Value = serde_json::from_str(&formatted_code)?;
            if let Some(sources) = outer_dict["sources"].as_object() {
                for (p, c) in sources {
                    let path = format!("{}/{}", address, p);
                    let code = c["content"].as_str().unwrap_or("Error: No content");
                    save_code(basepath, "core", &path, code)?;
                }
            }
        }
    } else {
        println!("core scan status error {}", address);
    }
    Ok(())
}


/// 从 scrollscan 获取智能合约源代码
async fn get_code_from_scroll(basepath: &str, address: &str) -> Result<()> {
    let api_url =
        format!("https://api.scrollscan.com/api?module=contract&action=getsourcecode&address={}&apikey={yourApiKey}",
                address);
    let resp: Value = reqwest::get(&api_url).await?.json().await?;
    if resp["status"] == "1" {
        let result = resp["result"].as_array().ok_or(anyhow!("Result is not an array"))?;
        for item in result {
            let formatted_code = item["SourceCode"].as_str()
                .unwrap_or("")
                .replace("{{", "{")
                .replace("}}", "}");
            let outer_dict: Value = serde_json::from_str(&formatted_code)?;
            if let Some(sources) = outer_dict["sources"].as_object() {
                for (p, c) in sources {
                    let path = format!("{}/{}", address, p);
                    let code = c["content"].as_str().unwrap_or("Error: No content");
                    save_code(basepath, "scroll", &path, code)?;
                }
            }
        }
    } else {
        println!("scroll scan status error {}", address);
    }
    Ok(())
}


/// 从 merlinchain 获取智能合约源代码
async fn get_code_from_merlin(basepath: &str, address: &str) -> Result<()> {
    let api_url =
        format!("https://scan.merlinchain.io/api/?module=contract&action=getsourcecode&address={}&api_key={yourApiKey}",
                address);
    let resp: Value = reqwest::get(&api_url).await?.json().await?;

    if resp["status"] == "1" {
        let result = resp["result"].as_array().ok_or(anyhow!("Result is not an array"))?;
        for item in result {
            let outer_dict: Value = serde_json::from_str(item["SourceCode"].as_str()
                .unwrap_or("Error serde_json"))?;
            if let Some(sources) = outer_dict["sources"].as_object() {
                for (p, c) in sources {
                    let path = format!("{}/{}", address, p);
                    let code = c["content"].as_str().unwrap_or("Error: No content");
                    save_code(basepath, "merlin", &path, code)?;
                }
            }
        }
    } else {
        println!("merlin scan status error {}", address);
    }
    Ok(())
}


/// 从 btrscan 获取智能合约源代码
async fn get_code_from_bitlayer(basepath: &str, address: &str) -> Result<()> {
    let api_url =
        format!("https://api.btrscan.com/scan/api?module=contract&action=getsourcecode&address={}",
                address);
    let resp: Value = reqwest::get(&api_url).await?.json().await?;

    if resp["status"] == 1 {
        let result = resp["result"].as_array().ok_or(anyhow!("Result is not an array"))?;
        for item in result {
            let outer_dict: Value = serde_json::from_str(item["SourceCode"].as_str()
                .unwrap_or("Error serde_json"))?;
            if let Some(sources) = outer_dict["sources"].as_object() {
                for (p, c) in sources {
                    let path = format!("{}/{}", address, p);
                    let code = c["content"].as_str().unwrap_or("Error: No content");
                    save_code(basepath, "bitlayer", &path, code)?;
                }
            }
        }
    } else {
        println!("bitlayer scan status error {}", address);
    }
    Ok(())
}


/// 从 Mode explorer 获取智能合约源代码
async fn get_code_from_mode(basepath: &str, address: &str) -> Result<()> {
    let api_url = format!("https://explorer.mode.network/api/v2/smart-contracts/{}", address);
    let resp: Value = reqwest::get(&api_url).await?.json().await?;

    let main_sol = resp["source_code"].as_str()
        .ok_or_else(|| anyhow!("Source code is missing or not a string"))?;
    let main_path = format!("{}/{}", address,
                            resp["file_path"].as_str().unwrap_or("Error: No main file path"));
    save_code(basepath, "mode", &main_path, main_sol)?;
    if let Some(additional_sources) = resp["additional_sources"].as_array() {
        for source in additional_sources {
            let code = source["source_code"].as_str()
                .unwrap_or("Error: No additional_sources code");
            let path = format!("{}/{}", address, source["file_path"].as_str()
                .unwrap_or("Error: No additional_sources file path"));
            save_code(basepath, "mode", &path, code)?;
        }
    }
    Ok(())
}

/// 从 Tenderly 获取智能合约源代码
///
/// NOTE: 已废弃，因为 Tenderly 不再提供免费的 API
// async fn get_code_from_tenderly(basepath: &str, address: &str, chain: &str) -> Result<()> {
//     let chain_id = CHAINS.get(chain).ok_or(anyhow!("Invalid chain"))?;
//     let tdl_url = format!("https://api.tenderly.co/api/v1/public-contracts/{}/{}", chain_id, address);
//     let resp: Value = reqwest::get(&tdl_url).await?.json().await?;
//     let contract_info = resp["data"]["contract_info"]
//         .as_array()
//         .ok_or(anyhow!("Failed to extract contract_info array from API response"))?;
//     for info in contract_info {
//         let path = format!("{}/{}", address, info["path"].as_str().unwrap());
//         let code_content = info["source"].as_str().unwrap();
//         // println!("{}\n{}", path, code_content);
//         save_code(basepath, chain, &path, code_content)?;
//     }
//     Ok(())
// }


/// 从 snowtrace 获取智能合约源代码
async fn get_code_from_snowtrace(basepath: &str, address: &str) -> Result<()> {
    let api_url = format!("https://api.routescan.io/v2/network/mainnet/evm/43114/etherscan/api?module=contract&action=getsourcecode&address={}", address);
    let resp: Value = reqwest::get(&api_url).await?.json().await?;
    if resp["status"] == "1" {
        let result = resp["result"].as_array().ok_or(anyhow!("Result is not an array"))?;
        for item in result {
            let code = item["SourceCode"].as_str()
                .unwrap_or("Error: No source code");
            if code.is_empty() {
                println!("Empty source code for {}", address);
                continue;
            }
            let trimmed = code.trim();
            if trimmed.starts_with("{{") && trimmed.ends_with("}}") {
                let formatted_code = item["SourceCode"].as_str()
                    .unwrap_or("")
                    .strip_prefix("{")
                    .unwrap_or("")
                    .strip_suffix("}")
                    .unwrap_or("");
                let outer_dict: Value = serde_json::from_str(&formatted_code)?;
                if let Some(sources) = outer_dict["sources"].as_object() {
                    for (p, c) in sources {
                        let path = format!("{}/{}", address, p);
                        let code = c["content"].as_str().unwrap_or("Error: No content");
                        save_code(basepath, "avax", &path, code)?;
                    }
                }
            } else {
                let contract_name = item["ContractName"].as_str().unwrap_or("Error: No contract name");
                let path = format!("{}/{}.sol", address, contract_name);
                save_code(basepath, "avax", &path, code)?;
            }
        }
    } else {
        println!("core scan status error {}", address);
    }

    Ok(())
}


/// 从 Etherscan 获取智能合约源代码
async fn get_code_from_etherscan(basepath: &str, address: &str, chain: &str) -> Result<()> {
    let chain_id = CHAINS.get(chain).ok_or(anyhow!("Invalid chain"))?;
    let api_url = format!(
        "https://api.etherscan.io/v2/api?chainid={}&module=contract&action=getsourcecode&address={}&apikey={yourApiKey}",
        chain_id,
        address
    );

    let resp: Value = reqwest::get(&api_url).await?.json().await?;
    if resp["status"] == "1" {
        let result = resp["result"].as_array().ok_or(anyhow!("Result is not an array"))?;
        for item in result {
            let code = item["SourceCode"].as_str()
                .unwrap_or("Error: No source code");
            if code.is_empty() {
                println!("Empty source code for {}", address);
                continue;
            }
            let trimmed = code.trim();
            if trimmed.starts_with("{{") && trimmed.ends_with("}}") {
                let formatted_code = item["SourceCode"].as_str()
                    .unwrap_or("")
                    .strip_prefix("{")
                    .unwrap_or("")
                    .strip_suffix("}")
                    .unwrap_or("");
                let outer_dict: Value = serde_json::from_str(&formatted_code)?;
                if let Some(sources) = outer_dict["sources"].as_object() {
                    for (p, c) in sources {
                        let path = format!("{}/{}", address, p);
                        let code = c["content"].as_str().unwrap_or("Error: No content");
                        save_code(basepath, chain, &path, code)?;
                    }
                }
            } else {
                let contract_name = item["ContractName"].as_str().unwrap_or("Error: No contract name");
                let path = format!("{}/{}.sol", address, contract_name);
                save_code(basepath, chain, &path, code)?;
            }
        }
    } else {
        println!("core scan status error {}", address);
    }

    Ok(())
}


/// 从 Tenderly 或者其他特定的链获取智能合约源代码
async fn get_code(basepath: &str, address: &str, chain: &str) -> Result<()> {
    let chain_id = CHAINS.get(chain).ok_or(anyhow!("Invalid chain"))?;
    // tenderly 不支持的链，使用不同的获取方式
    match chain_id {
        4200 => get_code_from_merlin(basepath, address).await?,
        200901 => get_code_from_bitlayer(basepath, address).await?,
        1116 => get_code_from_core(basepath, address).await?,
        43114 => get_code_from_snowtrace(basepath, address).await?,
        2649 => get_code_from_ailayer(basepath, address).await?,
        34443 => get_code_from_mode(basepath, address).await?,
        _ => {
            get_code_from_etherscan(basepath, address, chain).await?;
        }
    }
    Ok(())
}


fn save_code(basepath: &str, chain: &str, path: &str, data: &str) -> Result<()> {
    let output_dir = Path::new(basepath).join(chain).join(path);
    fs::create_dir_all(output_dir.parent().unwrap())?;
    fs::write(&output_dir, data)?;
    println!("Saved: {}", output_dir.display());
    Ok(())
}


#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("Smart Contract Source Code Retriever")
        .version("0.4.0")
        .author("Kong")
        .about("Retrieve smart contract source code from various chains")
        .arg(Arg::with_name("address")
            .short('d')
            .long("address")
            .value_name("ADDRESS")
            .help("Specify the address (required in single mode)")
            .takes_value(true))
        .arg(Arg::with_name("chain")
            .short('c')
            .long("chain")
            .value_name("CHAIN")
            .help("Specify the chain (required in single mode)")
            .takes_value(true))
        .arg(Arg::with_name("file")
            .short('f')
            .long("file")
            .value_name("FILE")
            .help("Specify the file (required in batch mode). e.g. 0x0,eth")
            .takes_value(true))
        .arg(Arg::with_name("output")
            .short('o')
            .long("output")
            .value_name("OUTPUT")
            .help("Specify the output directory")
            .takes_value(true)
            .default_value("./output"))
        .arg(Arg::with_name("list")
            .short('l')
            .long("list")
            .help("List all supported chains"))
        .get_matches();

    let output = matches.value_of("output").unwrap();

    // 如果指定了 --list 参数，列出所有可用的链
    if matches.is_present("list") {
        println!("Available chains:");
        for (chain, id) in CHAINS.iter() {
            println!("{}: {}", chain, id);
        }
        return Ok(());
    }

    // 处理输入：从文件读取或使用单个地址和链
    if let Some(file) = matches.value_of("file") {
        let mut rdr = csv::Reader::from_path(file)?;
        for result in rdr.records() {
            let record = result?;
            let address = record.get(0).ok_or(anyhow!("Invalid CSV file"))?;
            let chain = record.get(1).ok_or(anyhow!("Invalid CSV file"))?;
            // let chain_id = CHAINS.get(chain).ok_or(anyhow!("Invalid chain"))?;
            get_code(output, address, chain).await?;
            // 在每次请求之间等待200毫秒
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }
    } else if let (Some(address), Some(chain)) = (matches.value_of("address"), matches.value_of("chain")) {
        // let chain_id = CHAINS.get(chain).ok_or(anyhow!("Invalid chain"))?;
        get_code(output, address, chain).await?;
    } else {
        println!("Invalid arguments. Use --help for usage instructions.");
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tokio;

    #[tokio::test]
    async fn test_get_code() {
        // 使用临时目录作为基础路径
        let basepath = "./tmp";

        // 使用一个已知的、公开的智能合约地址
        // 多合约
        // let address = "0x43506849D7C04F9138D1A2050bbF3A0c054402dd";
        // 单合约
        // let address = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
        // let address = "0xf0F161fDA2712DB8b566946122a5af183995e2eD"; // mode USDT
        // let address = "0xd64023c388541dca887a9193afa2c23d03d1b0ff"; // BSC
        // let address = "0x770ef9f4fe897e59daCc474EF11238303F9552b6"; // FTM
        // let address = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174"; // polygon
        let address = "0xde3A24028580884448a5397872046a019649b084"; // avax
        // let address = "0xaf88d065e77c8cC2239327C5EDb3A432268e5831"; // arb
        // let address = "0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85"; // op
        // let address = "0xd964B8cC21f5A28FEB4B4eD524E20b9165123553"; // sepolia
        // let address = "0xd9aAEc86B65D86f6A7B5B1b0c42FFA531710b6CA"; // base
        // let address = "0x098d6eE48341D6a0a0A72dE5baaF80A10E0F6082"; // moonbeam
        // let address = "0xE3F5a90F9cb311505cd691a46596599aA1A0AD7D"; // moonriver
        // let address = "0xc21223249CA28397B4B6541dfFaEcC539BfF0c59"; // cro
        // let address = "0x06eFdBFf2a14a7c8E15944D1F4A48F9F95F663A4"; // scroll
        // let address = "0x176211869cA2b568f2A7D4EE941E073a821EE1ff"; // linea

        // 使用以太坊主网
        let chain = "avax";

        // 调用函数
        let result = get_code(&basepath, address, chain).await;

        // 检查函数是否成功执行
        assert!(result.is_ok(), "Function should return Ok");

        // 检查是否创建了输出目录
        // let output_dir = Path::new(&basepath).join(chain).join(address);
        // assert!(output_dir.exists(), "Output directory should exist");

        // 检查是否至少保存了一个文件
        // let files: Vec<_> = fs::read_dir(&output_dir)
        //     .unwrap()
        //     .filter_map(|entry| entry.ok())
        //     .collect();
        // assert!(!files.is_empty(), "At least one file should be saved");

        // 清理：删除临时目录
        // fs::remove_dir_all(basepath).unwrap();
    }
}
