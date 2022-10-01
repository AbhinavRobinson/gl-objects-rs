use glfw::Context;
use glow::HasContext;
use std::{ffi::c_void, sync::mpsc::channel};
pub struct Window<WindowContext, WindowHandle> {
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub ctx: Option<WindowContext>,
    pub internal_handle: Option<WindowHandle>,
}

pub trait WindowTrait<WindowContext, WindowHandle> {
    fn new(width: u32, height: u32, title: String) -> Window<WindowContext, WindowHandle> {
        Window {
            width,
            height,
            title: format!("{}", title),
            ctx: None,
            internal_handle: None,
        }
    }
    fn create_display(&mut self);
    fn load_with(&mut self, window: &mut WindowHandle, s: &str) -> *const std::ffi::c_void;
    fn event_loop(&mut self);
}

impl WindowTrait<glfw::Glfw, glfw::Window> for Window<glfw::Glfw, glfw::Window> {
    fn create_display(&mut self) {
        let mut glfw: glfw::Glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

        glfw.window_hint(glfw::WindowHint::ContextVersionMajor(4));
        glfw.window_hint(glfw::WindowHint::ContextVersionMinor(1));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));

        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

        let (mut window, ..) = glfw
            .create_window(
                self.width,
                self.height,
                &self.title,
                glfw::WindowMode::Windowed,
            )
            .expect("Failed to create GLFW window.");

        window.set_all_polling(true);
        window.make_current();

        let gl = unsafe { glow::Context::from_loader_function(|s| self.load_with(&mut window, s)) };

        println!("{:?}", gl.version());

        self.ctx = Some(glfw);
        self.internal_handle = Some(window);
    }

    fn event_loop(&mut self) {
        let mut glfw = self.ctx.take().unwrap();
        let window = self.internal_handle.take().unwrap();

        let (sender, receiver): (
            std::sync::mpsc::Sender<(f64, glfw::WindowEvent)>,
            std::sync::mpsc::Receiver<(f64, glfw::WindowEvent)>,
        ) = channel();
        unsafe {
            glfw::ffi::glfwSetWindowUserPointer(
                window.window_ptr(),
                std::mem::transmute(Box::new(sender)),
            );
        }

        while !window.should_close() {
            glfw.poll_events();
            for (_, event) in glfw::flush_messages(&receiver) {
                println!("{:?}", event);
            }
        }
    }

    fn load_with(&mut self, window: &mut glfw::Window, s: &str) -> *const std::ffi::c_void {
        window.get_proc_address(s) as *const c_void
    }
}

impl WindowTrait<sdl2::Sdl, sdl2::video::Window> for Window<sdl2::Sdl, sdl2::video::Window> {
    fn create_display(&mut self) {
        let ctx = sdl2::init().unwrap();

        let video_subsystem = ctx.video().unwrap();

        let gl_attr = video_subsystem.gl_attr();

        gl_attr.set_context_version(4, 1);
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_flags().debug().set();

        let mut window = video_subsystem
            .window(&self.title, self.width, self.height)
            .allow_highdpi()
            .opengl()
            .position_centered()
            .resizable()
            .build()
            .unwrap();

        let gl_context = window.gl_create_context().unwrap();
        let gl = unsafe { glow::Context::from_loader_function(|s| self.load_with(&mut window, s)) };

        println!("{:?}", gl.version());

        window.gl_make_current(&gl_context).unwrap();

        window.subsystem().gl_set_swap_interval(1).unwrap();

        self.ctx = Some(ctx);
        self.internal_handle = Some(window);
    }

    fn event_loop(&mut self) {
        let mut event_pump = self
            .ctx
            .as_ref()
            .expect("Context not initialized before event loop")
            .event_pump()
            .unwrap();

        'event_loop: loop {
            for event in event_pump.poll_iter() {
                self.internal_handle
                    .as_ref()
                    .expect("REASON")
                    .gl_swap_window();
                if let sdl2::event::Event::Quit { .. } = event {
                    break 'event_loop;
                }
            }
        }
    }

    fn load_with(&mut self, window: &mut sdl2::video::Window, s: &str) -> *const std::ffi::c_void {
        window.subsystem().gl_get_proc_address(s) as _
    }
}