use eyre::Error;
use eyre::Result;
use futures_util::{SinkExt, StreamExt};
use netstack_smoltcp::StackBuilder;
use tun::AbstractDevice;

pub use netstack_smoltcp::{TcpStream, UdpSocket};

pub struct TunSystem {
  state: State, // TODO add name, ip-range, etc
}

enum State {
  Init(tun::Configuration),
  DeviceCreated {
    device: tun::AsyncDevice,
    device_name: String
  },
  NetstackCreated{
    tasks: Vec<tokio::task::JoinHandle<()>>,
    device_name: String,
    adapter_index: u32
  },
  AllReady(),
  Destoried,
  // TODO include thiserror
  Failed,
}

pub enum DeviceID {
  Dev(String),
  Fd(i32),
}
impl Default for DeviceID {
  #[cfg(target_os = "macos")]
  fn default() -> Self {
    Self::Dev("utun233".into())
  }
  #[cfg(not(target_os = "macos"))]
  fn default() -> Self {
    Self::Dev("utun233".into())
  }
}

impl TunSystem {
  // TODO dont expose tun2's API
  pub fn new(device_id: DeviceID, auto_up: bool) -> Self {
    let mut cfg = tun::Configuration::default();
    match device_id {
      DeviceID::Dev(id) => cfg.tun_name(id),
      DeviceID::Fd(id) => cfg.raw_fd(id),
    };
    if auto_up {
      cfg.up();
    }
    TunSystem {
      state: State::Init(cfg),
    }
  }
  pub fn device_name(&mut self) -> &str {
    match &self.state {
      State::DeviceCreated { device: _, device_name }  => {
        return &device_name;
      },
      State::NetstackCreated { tasks: _, device_name, adapter_index: _ } => {
        return &device_name;
      },
      _ => todo!("TODO"),
    }
  }
  /// STEP 1
  pub fn create_device(&mut self) -> Result<&Self> {
    if let State::Init(cfg) = &self.state {
      match tun::create_as_async(cfg) {
        Ok(device) => self.state = State::DeviceCreated {
          device_name: device.as_ref().tun_name().expect("TODO"),
          device,
        },
        // TODO include thiserror
        Err(e) => return Err(eyre::eyre!(e)),
      }
    } else {
      panic!("TODO")
    }
    return Ok(self);
  }

  /// SETP2
  /// TODO add function callbacks
  pub fn create_netstack(
    mut self,
  ) -> (
    Self,
    netstack_smoltcp::tcp::TcpListener,
    netstack_smoltcp::udp::UdpSocket,
  ) {
    if let State::DeviceCreated { device, device_name } = self.state {

      let mut tasks = Vec::new();
      let adapter_index = device.as_ref().adapter_index;
      let framed = device.into_framed();
      let builder = StackBuilder::default();

      let (netstack_task, udp_socket, tcp_listener, stack) = builder.build();
      tasks.push(tokio::spawn(netstack_task));

      let (mut stack_sink, mut stack_stream) = stack.split();
      let (mut tun_sink, mut tun_stream) = framed.split();
      // stack -> tun
      let stack2tun_task = async move {
        while let Some(pkt) = stack_stream.next().await {
          if let Ok(pkt) = pkt {
            tun_sink.send(pkt).await.expect("TODO");
          } else {
            panic!("TODO")
          }
        }
      };
      // tun -> stack
      let tun2stack_task = async move {
        while let Some(pkt) = tun_stream.next().await {
          if let Ok(pkt) = pkt {
            stack_sink.send(pkt).await.expect("TODO");
          } else {
            panic!("TODO")
          }
        }
      };
      tasks.push(tokio::spawn(stack2tun_task));
      tasks.push(tokio::spawn(tun2stack_task));

      self.state = State::NetstackCreated { tasks, device_name, adapter_index };
      return (self, tcp_listener, udp_socket);
    } else {

      panic!("TODO")
    }
  }
  pub fn create_route(
    mut self,
  ) -> Self {
    if let State::NetstackCreated { tasks, device_name, adapter_index } = self.state {
      #[cfg(target_os = "windows")]
      {
        println!("adapter index {}", adapter_index);
      }
      self.state = State::AllReady();
      return self;
    } else {
      panic!("TODO")
    }
  }

}
