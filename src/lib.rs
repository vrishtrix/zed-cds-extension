use zed_extension_api::{register_extension, Extension};

struct CdsExtension {
    // ...state
}

impl Extension for CdsExtension {
    fn new() -> Self {
        Self {
			// ...
		}
    }
}

register_extension!(CdsExtension);
