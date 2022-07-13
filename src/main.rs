use clap::{Parser, Subcommand};
use std::net::IpAddr;

mod pmp;

use pmp::*;

#[derive(Debug, Parser)]
#[clap(name = "fport")]
#[clap(version, about = "Simple port-forwarding utility using the NAT-PMP protocol", long_about = None)]
struct Cli {
  #[clap(subcommand)]
  command: Commands,
  #[clap(
    short,
    long,
    value_name = "IP",
    help = "The IP address of the default gateway",
    default_value = "192.168.1.1"
  )]
  gateway: IpAddr,
  #[clap(short, long, help = "Print verbose messages, useful for debugging")]
  verbose: bool,
}

#[derive(Debug, Subcommand)]
enum Commands {
  #[clap(about = "Get the public IP address from the router")]
  Ip,
  #[clap(about = "Create a new port mapping")]
  Map {
    #[clap(value_name = "PORT", help = "The public port")]
    public_port: u16,
    #[clap(
      short = 'r',
      long,
      value_name = "PORT",
      help = "The private port [default: same as public]"
    )]
    private_port: Option<u16>,
    #[clap(short, long, help = "Create an UDP mapping instead")]
    udp: bool,
    #[clap(
      short,
      long,
      value_name = "SECONDS",
      help = "The lifetime of the mapping in seconds",
      default_value_t = 86400
    )]
    lifetime: u32,
  },
}

fn main() {
  let args = Cli::parse();

  match args.command {
    Commands::Ip => match send_address_request(args.gateway) {
      Ok(address_response) => match get_result(&address_response) {
        Ok(status_message) => {
          println!("Status: {}", status_message);
          println!("IP address: {}", address_response.ip_address);

          if args.verbose {
            println!("{:?}", address_response);
          }
        }
        Err(error_status) => eprintln!("Status: {}", error_status),
      },
      Err(e) => eprintln!("Error: {}", e),
    },

    Commands::Map {
      public_port,
      private_port,
      udp,
      lifetime,
    } => {
      let mapping_type = match udp {
        false => MappingType::Tcp,
        true => MappingType::Udp,
      };

      match send_mapping_request(
        mapping_type,
        public_port,
        private_port.unwrap_or(public_port),
        lifetime,
        args.gateway,
      ) {
        Ok(mapping_response) => match get_result(&mapping_response) {
          Ok(status_message) => {
            println!("Status: {}", status_message);
            println!(
              "Mapped {} -> {}, protocol {:?}, with a lifetime of {}s.",
              mapping_response.public_port,
              mapping_response.private_port,
              mapping_response.mapping_type,
              mapping_response.lifetime
            );

            if args.verbose {
              println!("{:?}", mapping_response);
            }
          }
          Err(error_status) => eprintln!("Status: {}", error_status),
        },
        Err(e) => eprintln!("Error: {}", e),
      }
    }
  }
}
