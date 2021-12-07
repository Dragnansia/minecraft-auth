use crate::instance::Instance;

pub struct Client {
    pub ram_min: i32,
    pub ram_max: i32,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            ram_min: 512,
            ram_max: 1024,
        }
    }
}

impl Client {
    pub fn new(ram_min: i32, ram_max: i32) -> Self {
        Self { ram_min, ram_max }
    }

    /// Run instance on other thread
    ///
    /// # Example
    /// ```
    /// use minecraft_auth::{instance::Instance, client::Client};
    ///
    /// let client = Client::default();
    /// let instance = Instance::default();
    /// client.start_instance(&instance);
    /// ```
    pub fn start_instance(&self, _instance: &Instance) {}
}
