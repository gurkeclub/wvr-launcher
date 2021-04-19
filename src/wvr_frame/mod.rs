use core::ffi::c_void;

use std::path::Path;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::Mutex;
use std::thread;

use anyhow::{Context, Result};

use gtk::prelude::*;

use gtk::GLArea;

use glium::Surface;
use glium::SwapBuffersError;

use wvr_com::data::{Message, SetInfo};
use wvr_com::server::OrderServer;
use wvr_data::config::project_config::ProjectConfig;

use wvr::Wvr;

struct GtkWvrBackend {
    glarea: gtk::GLArea,
}

unsafe impl glium::backend::Backend for GtkWvrBackend {
    fn swap_buffers(&self) -> Result<(), SwapBuffersError> {
        Ok(())
    }

    unsafe fn get_proc_address(&self, symbol: &str) -> *const c_void {
        gl_loader::get_proc_address(symbol) as *const _
    }

    fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        let allocation = self.glarea.get_allocation();
        (allocation.width as u32, allocation.height as u32)
    }

    fn is_current(&self) -> bool {
        true
    }

    unsafe fn make_current(&self) {
        self.glarea.make_current();
    }
}

impl GtkWvrBackend {
    fn new(glarea: gtk::GLArea) -> Self {
        Self { glarea }
    }
}

pub fn build_wvr_frame(
    glarea: &GLArea,
    project_path: &Path,
    project_config: &ProjectConfig,
) -> Result<Sender<Message>> {
    gl_loader::init_gl();

    let context = unsafe {
        glium::backend::Context::new(
            GtkWvrBackend::new(glarea.clone()),
            true,
            glium::debug::DebugCallbackBehavior::DebugMessageOnError,
        )?
    };

    let (order_sender, order_receiver) = channel();

    {
        let glarea = glarea.clone();

        {
            let order_sender = order_sender.clone();
            if project_config.server.enable {
                let mut order_server = OrderServer::new(&project_config.server);
                thread::spawn(move || loop {
                    order_sender
                        .send(order_server.next_order(None).unwrap())
                        .unwrap();
                });
            }
        }

        let app = Mutex::new({
            let mut app = Wvr::new(&project_path, project_config.clone(), &context)
                .context("Failed creating Wvr app")
                .unwrap();

            app.handle_message(&context, &Message::Set(SetInfo::DynamicResolution(false)))
                .unwrap();
            app
        });

        glarea.connect_render(move |glarea, _glcontext| {
            if let Ok(mut app) = app.lock() {
                let resolution = context.get_framebuffer_dimensions();

                for message in order_receiver.try_iter() {
                    if let Message::Set(SetInfo::DynamicResolution(_)) = &message {
                    } else {
                        app.handle_message(&context, &message).unwrap();
                    }

                    if let Message::Set(set_info) = &message {
                        match &set_info {
                            SetInfo::Width(_) => {
                                glarea.set_size_request(
                                    app.config.view.width as i32 / 4,
                                    app.config.view.height as i32 / 4,
                                );
                            }
                            SetInfo::Height(_) => {
                                glarea.set_size_request(
                                    app.config.view.width as i32 / 4,
                                    app.config.view.height as i32 / 4,
                                );
                            }
                            SetInfo::VSync(_vsync) => (),
                            SetInfo::Fullscreen(_fullscreen) => (),
                            SetInfo::LockedSpeed(_fullscreen) => (),
                            SetInfo::Screenshot(_screenshot) => (),
                            _ => (),
                        }
                    }
                }

                if let Err(error) =
                    app.update(&context, (resolution.0 as usize, resolution.1 as usize))
                {
                    eprintln!("Failed to update app: {:?}", error);
                }

                let mut frame = glium::Frame::new(context.clone(), resolution);
                frame.clear_color(0.0, 1.0, 0.0, 1.0);
                if let Err(error) = app.render_final_stage(&context, &mut frame) {
                    eprintln!("Failed to render to windowc: {:?}", error);
                }

                if let Err(error) = app.render_stages(&context) {
                    eprintln!("Failed to render app: {:?}", error);
                }

                frame
                    .finish()
                    .context("Failed to finalize rendering")
                    .unwrap();
            }

            glarea.queue_draw();

            Inhibit(true)
        });
    }

    glarea.queue_draw();

    Ok(order_sender)
}
