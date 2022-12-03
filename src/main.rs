use color_eyre::eyre::Result;
use input_window::InputWindow;
use manifest_dir_macros::directory_relative_path;
use manifold::Manifold;
use stardust_xr_molecules::fusion::client::Client;
use std::thread;
use tokio::{runtime::Handle, sync::oneshot};
use winit::{event_loop::EventLoopBuilder, platform::unix::EventLoopBuilderExtUnix};

pub mod emitter;
pub mod input_window;
pub mod keyboard;
pub mod manifold;
pub mod mouse;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
	color_eyre::install()?;
	let (client, stardust_event_loop) = Client::connect_with_async_loop().await?;
	client.set_base_prefixes(&[directory_relative_path!("res")]);

	let tokio_handle = Handle::current();
	let manifold = client.wrap_root(Manifold::new(&client));
	let (winit_stop_tx, mut winit_stop_rx) = oneshot::channel::<()>();
	let winit_thread = thread::Builder::new().name("winit".to_owned()).spawn({
		let client = client.clone();
		let keyboard = manifold.lock().keyboard();
		let mouse = manifold.lock().mouse();
		move || -> Result<()> {
			let _tokio_guard = tokio_handle.enter();
			let event_loop = EventLoopBuilder::new()
				.with_any_thread(true)
				.with_x11()
				.build();
			let mut input_window = InputWindow::new(&event_loop, client, keyboard, mouse)?;

			event_loop.run(move |event, _, control_flow| {
				match winit_stop_rx.try_recv() {
					Ok(_) => {
						control_flow.set_exit();
						return;
					}
					Err(ref e) if *e == oneshot::error::TryRecvError::Closed => {
						return;
					}
					_ => (),
				}

				input_window.handle_event(event);
				control_flow.set_wait();
			});
		}
	})?;

	let result = stardust_event_loop.await?;

	winit_stop_tx
		.send(())
		.expect("Failed to send stop signal to winit thread");
	winit_thread.join().expect("Couldn't rejoin winit thread")?;
	result?;
	Ok(())
}
