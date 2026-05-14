import("./pkg").then(module => {
    const startApp = module.run_app || module.main || (module.default && (module.default.run_app || module.default.main));
    if (typeof startApp === 'function') {
        startApp();
    } else {
        console.log("WASM Loaded! App auto-started by wasm-bindgen.");
    }
}).catch(e => console.error("Error loading WebAssembly:", e));