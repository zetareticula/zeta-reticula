import init, { infer_wasm, quantize_wasm, get_available_models_wasm } from './zeta-reticula-api.js';

async function run() {
  await init();
  const models = get_available_models_wasm();
  console.log(models);
  const result = await infer_wasm("sample text", "CustomModel", "8");
  console.log(result);
}
run();