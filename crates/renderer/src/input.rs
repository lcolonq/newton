use device_query::DeviceQuery;

pub enum Command {
    None,
    Drawing,
    EraseAll,
}
pub struct Input {
    pub device: device_query::DeviceState,
}
impl Input {
    pub fn new() -> Self {
        Self {
            device: device_query::DeviceState::new(),
        }
    }
    pub fn get_mouse(&self) -> (i32, i32) {
        self.device.get_mouse().coords
    }
    pub fn get_command(&mut self) -> Command {
        let keys = self.device.get_keys();
        if keys.contains(&device_query::Keycode::LMeta) {
            Command::Drawing
        } else if keys.contains(&device_query::Keycode::RMeta) {
            Command::EraseAll
        } else {
            Command::None
        }
    }
}
