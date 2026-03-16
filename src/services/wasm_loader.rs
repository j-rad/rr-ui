use anyhow::Result;
use wasmer::{Instance, Module, Store, Value};

pub struct WasmPluginLoader {
    store: Store,
}

impl WasmPluginLoader {
    pub fn new() -> Self {
        Self {
            store: Store::default(),
        }
    }

    /// Load external .wasm files as routing middleware
    pub fn load_plugin(&mut self, wasm_bytes: &[u8]) -> Result<Instance> {
        let module = Module::new(&self.store, wasm_bytes)?;
        let imports = wasmer::imports! {};
        let instance = Instance::new(&mut self.store, &module, &imports)?;
        Ok(instance)
    }

    /// Execute the loaded WASM middleware to score a route
    pub fn execute_routing_middleware(
        &mut self,
        instance: &Instance,
        latency: f64,
        packet_loss: f64,
    ) -> Result<f64> {
        let route_scorer = instance.exports.get_function("score_route")?;
        let result = route_scorer.call(
            &mut self.store,
            &[Value::F64(latency), Value::F64(packet_loss)],
        )?;

        if let Some(Value::F64(score)) = result.first() {
            Ok(*score)
        } else {
            Err(anyhow::anyhow!("Invalid return type from WASM plugin"))
        }
    }
}
