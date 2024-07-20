use sdl2::*;


pub struct Sdlstate {
    pub ctx: Sdl,
    pub vid: VideoSubsystem,
    pub pump: Mutex<EventPump>,
    pub canv: RwLock<Vec<Windowstate>>,
}

pub struct Windowstate {
    priv window: video::Window,
    pub canv: Mutex<render::Canvas<video::Window>>,
    pub render: impl Render
}

pub trait Update {

}
