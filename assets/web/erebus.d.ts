/* tslint:disable */
/* eslint-disable */

/**
 * WebAssembly entry point. Resolves the canvas element by id, hands off to
 * `eframe::WebRunner` with the same `ErebusApp` the native binary uses.
 */
export function start(canvas_id: string): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly start: (a: number, b: number, c: number) => void;
    readonly __wasm_bindgen_func_elem_3808: (a: number, b: number, c: number, d: number) => void;
    readonly __wasm_bindgen_func_elem_1866: (a: number, b: number, c: number) => void;
    readonly __wasm_bindgen_func_elem_668: (a: number, b: number, c: number) => void;
    readonly __wasm_bindgen_func_elem_668_3: (a: number, b: number, c: number) => void;
    readonly __wasm_bindgen_func_elem_1866_4: (a: number, b: number, c: number) => void;
    readonly __wasm_bindgen_func_elem_671: (a: number, b: number, c: number) => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number) => void;
    readonly __wbindgen_export4: (a: number, b: number, c: number) => void;
    readonly __wbindgen_export5: (a: number, b: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
