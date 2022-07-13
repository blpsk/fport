use std::io::Error;
use std::net::{IpAddr, UdpSocket};
use std::time::Duration;

const PORT: u16 = 5351;
const TIMEOUT: u64 = 200;

#[derive(Debug)]
pub enum MappingType {
  Tcp,
  Udp,
}

#[derive(Debug)]
pub struct AddressResponse {
  pub version: u8,
  pub op_code: u8,
  pub result_code: u16,
  pub time_since_init: u32,
  pub ip_address: String,
}

#[derive(Debug)]
pub struct MappingResponse {
  pub version: u8,
  pub op_code: u8,
  pub result_code: u16,
  pub time_since_init: u32,
  pub private_port: u16,
  pub public_port: u16,
  pub lifetime: u32,
  pub mapping_type: MappingType,
}

pub trait PMPResultCode {
  fn get_result_code(&self) -> u16;
}

pub fn send_address_request(gateway: IpAddr) -> Result<AddressResponse, &'static str> {
  return match send_receive(&[0, 0], gateway) {
    Ok(response_data) => AddressResponse::new(&response_data.0[..response_data.1]),
    Err(_) => Err("Networking error."),
  };
}

pub fn send_mapping_request(
  mapping_type: MappingType,
  public_port: u16,
  private_port: u16,
  lifetime: u32,
  gateway: IpAddr,
) -> Result<MappingResponse, &'static str> {
  let payload = gen_mapping_request(mapping_type, public_port, private_port, lifetime);

  return match send_receive(&payload, gateway) {
    Ok(response_data) => MappingResponse::new(&response_data.0[..response_data.1]),
    Err(_) => Err("Networking error."),
  };
}

pub fn get_result<T: PMPResultCode>(response: &T) -> Result<&'static str, &'static str> {
  match response.get_result_code() {
    0 => Ok("Success"),
    1 => Err("Unsupported version"),
    2 => Err("Not authorized / Refused"),
    3 => Err("Network failure"),
    4 => Err("Out of resources"),
    5 => Err("Unsupported operation code"),
    _ => Err("Unknown"),
  }
}

impl AddressResponse {
  fn new(data: &[u8]) -> Result<Self, &'static str> {
    if data.len() != 12 {
      return Err("Invalid response length.");
    }

    if data[0] != 0 {
      return Err("Unsupported protocol version.");
    }

    if data[1] != 128 {
      return Err("Invalid operation code received.");
    }

    Ok(AddressResponse {
      version: data[0],
      op_code: data[1],
      result_code: u16::from_be_bytes([data[2], data[3]]),
      time_since_init: u32::from_be_bytes([data[4], data[5], data[6], data[7]]),
      ip_address: format!("{}.{}.{}.{}", data[8], data[9], data[10], data[11]),
    })
  }
}

impl PMPResultCode for AddressResponse {
  fn get_result_code(&self) -> u16 {
    self.result_code
  }
}

impl MappingResponse {
  fn new(data: &[u8]) -> Result<Self, &'static str> {
    if data.len() != 16 {
      return Err("Invalid response length.");
    }

    if data[0] != 0 {
      return Err("Unsupported protocol version.");
    }

    let mapping_type = match data[1] {
      129 => MappingType::Udp,
      130 => MappingType::Tcp,
      _ => return Err("Invalid operation code received."),
    };

    Ok(MappingResponse {
      version: data[0],
      op_code: data[1],
      result_code: u16::from_be_bytes([data[2], data[3]]),
      time_since_init: u32::from_be_bytes([data[4], data[5], data[6], data[7]]),
      private_port: u16::from_be_bytes([data[8], data[9]]),
      public_port: u16::from_be_bytes([data[10], data[11]]),
      lifetime: u32::from_be_bytes([data[12], data[13], data[14], data[15]]),
      mapping_type,
    })
  }
}

impl PMPResultCode for MappingResponse {
  fn get_result_code(&self) -> u16 {
    self.result_code
  }
}

fn gen_mapping_request(mapping_type: MappingType, public_port: u16, private_port: u16, lifetime: u32) -> [u8; 12] {
  let mut data = [0; 12];

  data[1] = match mapping_type {
    MappingType::Udp => 1,
    MappingType::Tcp => 2,
  };

  let private_port_data = private_port.to_be_bytes();
  data[4] = private_port_data[0];
  data[5] = private_port_data[1];

  let public_port_data = public_port.to_be_bytes();
  data[6] = public_port_data[0];
  data[7] = public_port_data[1];

  let lifetime_data = lifetime.to_be_bytes();
  data[8] = lifetime_data[0];
  data[9] = lifetime_data[1];
  data[10] = lifetime_data[2];
  data[11] = lifetime_data[3];

  data
}

fn send_receive(packet: &[u8], gateway: IpAddr) -> Result<([u8; 16], usize), Error> {
  let socket = UdpSocket::bind("0.0.0.0:0")?;

  socket.set_read_timeout(Some(Duration::from_millis(TIMEOUT)))?;
  socket.connect((gateway, PORT))?;

  let mut buffer = [0; 16];

  socket.send(packet)?;
  let read = socket.recv(&mut buffer)?;

  Ok((buffer, read))
}
