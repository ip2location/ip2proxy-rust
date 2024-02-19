# IP2Proxy Rust Library

*Note: This repo is no longer maintained. Please switch to this repo (https://github.com/marirs/rust-ip2location) to query for the proxy information in Rust.*


This library allows user to query an IP address if it was being used as VPN servers, open proxies, web proxies, Tor exit nodes, search engine robots, data center ranges, residential proxies, consumer privacy networks, and enterprise private networks. It lookup the proxy IP address from **IP2Proxy BIN Data** file. This data file can be downloaded at

- Free IP2Proxy BIN Data: [https://lite.ip2location.com](https://lite.ip2location.com/)
- Commercial IP2Proxy BIN Data: https://www.ip2location.com/proxy-database

For more details, please visit: https://www.ip2location.com/ip2proxy/developers/rust

## Methods

Below are the methods supported in this library.

| Method Name    | Description                                                  |
| -------------- | ------------------------------------------------------------ |
| from_file      | Open the IP2Proxy BIN data with **File I/O** mode for lookup. |
| from_file_mmap | Open the IP2Proxy BIN data with **Mmap** mode for lookup.    |
| print_db_info  | Print the package and database version of IP2Proxy BIN database. |
| ip_lookup      | Return the proxy information in array.                       |

## Installation

Add the following line to your Cargo.toml file under `[dependencies]` section:

`ip2proxy = "0.1.0"`

## Usage

```rust
use ip2proxy::DB;

fn main() {
    let ip_address = "8.8.8.8";
    let db_path = "IP2PROXY-IP-PROXYTYPE-COUNTRY-REGION-CITY-ISP-DOMAIN-USAGETYPE-ASN-LASTSEEN-THREAT-RESIDENTIAL.BIN";
    let mut db = DB::from_file(db_path).unwrap();

    // print the db information
    db.print_db_info();
    println!();

    let result = db.ip_lookup(ip_address).unwrap();
    println!("{:#?}", result);
}
```



## Proxy Type

| Proxy Type | Description                    |
| ---------- | ------------------------------ |
| VPN        | Anonymizing VPN services.      |
| TOR        | Tor Exit Nodes.                |
| PUB        | Public Proxies.                |
| WEB        | Web Proxies.                   |
| DCH        | Hosting Providers/Data Center. |
| SES        | Search Engine Robots.          |
| RES        | Residential Proxies [PX10+]    |
| CPN        | Consumer Privacy Networks. [PX11+] |
| EPN        | Enterprise Private Networks. [PX11+] |

## Usage Type

| Usage Type | Description                     |
| ---------- | ------------------------------- |
| COM        | Commercial                      |
| ORG        | Organization                    |
| GOV        | Government                      |
| MIL        | Military                        |
| EDU        | University/College/School       |
| LIB        | Library                         |
| CDN        | Content Delivery Network        |
| ISP        | Fixed Line ISP                  |
| MOB        | Mobile ISP                      |
| DCH        | Data Center/Web Hosting/Transit |
| SES        | Search Engine Spider            |
| RSV        | Reserved                        |

## Threat Type

| Threat Type | Description                |
| ----------- | -------------------------- |
| SPAM        | Spammer                    |
| SCANNER     | Security Scanner or Attack |
| BOTNET      | Spyware or Malware         |

## Support

Email: [support@ip2location.com](mailto:support@ip2location.com). URL: [https://www.ip2location.com](https://www.ip2location.com/)
