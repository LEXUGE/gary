pub mod runtime;

use runtime::*;

use libloading::{Library, Symbol};
use std::env;
use std::ffi::OsStr;

pub struct RuntimePluginManager {
    //TODO: could be a Map<Name,Plugin>, might be useful to be able to access them by name
    plugins: Vec<Box<RuntimePlugin>>,
    loaded_libraries: Vec<Library>,
}

impl RuntimePluginManager {
    pub fn new() -> RuntimePluginManager {
        RuntimePluginManager {
            plugins: Vec::new(),
            loaded_libraries: Vec::new(),
        }
    }

    pub fn get_plugin_name(&self, id: String) -> Result<String, RuntimeError> {
        for plugin in &self.plugins {
            if plugin.name() == id {
                return Ok(plugin.name());
            }
        }
        Err(RuntimeError::new(RuntimeErrorType::Unknown))
    }

    pub unsafe fn load_plugin<P: AsRef<OsStr>>(&mut self, filename: P) -> Result<(), ()> {
        type PluginCreate = unsafe fn() -> *mut RuntimePlugin;

        let mut path = env::current_dir().unwrap();
        path.push("plugins/");
        path.push(filename.as_ref());

        let lib = Library::new(path.into_os_string()).unwrap();

        // We need to keep the library around otherwise our plugin's vtable will
        // point to garbage. We do this little dance to make sure the library
        // doesn't end up getting moved.
        self.loaded_libraries.push(lib);

        let lib = self.loaded_libraries.last().unwrap();

        let constructor: Symbol<PluginCreate> = lib.get(b"_plugin_create").unwrap();
        let boxed_raw = constructor();

        let plugin = Box::from_raw(boxed_raw);

        // Call `on_plugin_load` to let the plugin know we have connected
        plugin.on_plugin_load();
        self.plugins.push(plugin);

        Ok(())
    }
}

#[macro_export]
macro_rules! declare_plugin {
    ($plugin_type:ty, $constructor:path) => {
        #[no_mangle]
        pub extern "C" fn _plugin_create() -> *mut $crate::plugins::runtime::RuntimePlugin {
            // make sure the constructor is the correct type.
            let constructor: fn() -> $plugin_type = $constructor;

            let object = constructor();
            let boxed: Box<$crate::plugins::runtime::RuntimePlugin> = Box::new(object);
            Box::into_raw(boxed)
        }
    };
}
