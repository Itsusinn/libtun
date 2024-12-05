use libtun::TunSystem;
use libtun::DeviceID;

#[tokio::main]
async fn main() -> eyre::Result<()> {
  let mut tun_system = TunSystem::new(DeviceID::Dev("CLASHRS".into()), true);
  tun_system.create_device()?;
  let (tun_system,_,_) = tun_system.create_netstack();
  let tun_system = tun_system.create_route();
  tokio::signal::ctrl_c().await?;
  println!("Hello, world!");
  Ok(())
}
